//! SQLite KnowledgeRepository implementation for RAG (Step 1.8.3 - RAG-03).
//!
//! Provides Hybrid Search (Dense Cosine Similarity + Sparse Keyword Matching),
//! Reciprocal Rank Fusion (RRF), and Multi-Factor Memory Scoring.

use crate::sqlite::SqliteStorage;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use vc_core::character::CharacterId;
use vc_core::error::{CoreError, Result};
use vc_core::memory::MemoryImportance;
use vc_core::rag::scoring::{calculate_composite_score, reciprocal_rank_fusion};
use vc_core::rag::traits::KnowledgeRepository;
use vc_core::rag::types::{
    DocumentChunk, DocumentChunkId, EmbeddingVector, RagQuery, RagQueryResult, RagSourceType,
};

impl SqliteStorage {
    /// Initialize RAG tables (chunks and optional FTS5 virtual table).
    pub(crate) fn init_rag_tables(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS rag_chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT,
                character_id TEXT,
                actor_id TEXT,
                source_type TEXT NOT NULL,
                content TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                importance TEXT NOT NULL,
                embedding_blob BLOB,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_rag_chunks_char_actor 
            ON rag_chunks (character_id, actor_id);

            CREATE INDEX IF NOT EXISTS idx_rag_chunks_doc 
            ON rag_chunks (document_id);
            "#,
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to initialize RAG schema: {}", e)))?;

        // Try initializing FTS5 virtual table for keyword search
        let _ = conn.execute_batch(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS rag_chunks_fts USING fts5(
                chunk_id UNINDEXED,
                content,
                tokenize = 'unicode61'
            );
            "#,
        );

        Ok(())
    }

    /// Encode float vector to little-endian bytes.
    pub fn vector_to_bytes(vec: &EmbeddingVector) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(vec.0.len() * 4);
        for f in &vec.0 {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        bytes
    }

    /// Decode little-endian bytes to float vector.
    pub fn bytes_to_vector(bytes: &[u8]) -> Option<EmbeddingVector> {
        if bytes.is_empty() || bytes.len() % 4 != 0 {
            return None;
        }
        let floats: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        Some(EmbeddingVector::new(floats))
    }
}

impl KnowledgeRepository for SqliteStorage {
    fn save_chunk(&self, chunk: &DocumentChunk, embedding: Option<&EmbeddingVector>) -> Result<()> {
        let conn = self.conn_lock();
        let chunk_id_str = chunk.id.to_string();
        let char_id_str = chunk.character_id.map(|c| c.0.to_string());
        let importance_str = match chunk.importance {
            MemoryImportance::Low => "Low",
            MemoryImportance::Medium => "Medium",
            MemoryImportance::High => "High",
            MemoryImportance::Critical => "Critical",
        };
        let source_type_str = chunk.source_type.as_str();
        let emb_bytes = embedding.map(Self::vector_to_bytes);

        conn.execute(
            r#"
            INSERT INTO rag_chunks (
                id, document_id, character_id, actor_id, source_type,
                content, chunk_index, importance, embedding_blob, created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                document_id = excluded.document_id,
                character_id = excluded.character_id,
                actor_id = excluded.actor_id,
                source_type = excluded.source_type,
                content = excluded.content,
                chunk_index = excluded.chunk_index,
                importance = excluded.importance,
                embedding_blob = excluded.embedding_blob,
                created_at = excluded.created_at
            "#,
            params![
                chunk_id_str,
                chunk.document_id,
                char_id_str,
                chunk.actor_id,
                source_type_str,
                chunk.content,
                chunk.chunk_index as i64,
                importance_str,
                emb_bytes,
                chunk.created_at as i64,
            ],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save RAG chunk: {}", e)))?;

        // Update FTS index if available
        let _ = conn.execute(
            "DELETE FROM rag_chunks_fts WHERE chunk_id = ?1",
            params![chunk_id_str],
        );
        let _ = conn.execute(
            "INSERT INTO rag_chunks_fts (chunk_id, content) VALUES (?1, ?2)",
            params![chunk_id_str, chunk.content],
        );

        Ok(())
    }

    fn search_hybrid(&self, query: &RagQuery) -> Result<Vec<RagQueryResult>> {
        let conn = self.conn_lock();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Fetch matching chunks respecting Character and Actor Isolation (Skill 25)
        let mut sql = "SELECT id, document_id, character_id, actor_id, source_type, content, chunk_index, importance, embedding_blob, created_at FROM rag_chunks WHERE 1=1".to_string();
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();

        if let Some(char_id) = query.character_id {
            sql.push_str(" AND (character_id = ? OR character_id IS NULL)");
            params_vec.push(char_id.0.to_string().into());
        }

        if let Some(ref actor_id) = query.actor_id {
            // Actor Isolation: can only see shared chunks (actor_id IS NULL) OR own chunks
            sql.push_str(" AND (actor_id = ? OR actor_id IS NULL)");
            params_vec.push(actor_id.clone().into());
        } else {
            // Anonymous caller: can ONLY see shared chunks
            sql.push_str(" AND actor_id IS NULL");
        }

        let mut stmt = conn.prepare(&sql).map_err(|e| {
            CoreError::StorageError(format!("Failed to prepare RAG search SQL: {}", e))
        })?;

        let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect();

        let mut rows = stmt
            .query(params_slice.as_slice())
            .map_err(|e| CoreError::StorageError(format!("Failed to execute RAG query: {}", e)))?;

        let mut chunks_map: HashMap<DocumentChunkId, DocumentChunk> = HashMap::new();
        let mut dense_scores: HashMap<DocumentChunkId, f32> = HashMap::new();

        while let Some(row) = rows
            .next()
            .map_err(|e| CoreError::StorageError(e.to_string()))?
        {
            let id_str: String = row
                .get(0)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let doc_id: Option<String> = row
                .get(1)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let char_id_str: Option<String> = row
                .get(2)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let actor_id: Option<String> = row
                .get(3)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let source_str: String = row
                .get(4)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let content: String = row
                .get(5)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let chunk_idx: i64 = row
                .get(6)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let imp_str: String = row
                .get(7)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let emb_blob: Option<Vec<u8>> = row
                .get(8)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;
            let created_at: i64 = row
                .get(9)
                .map_err(|e| CoreError::StorageError(e.to_string()))?;

            let chunk_id = DocumentChunkId(
                uuid::Uuid::parse_str(&id_str)
                    .map_err(|e| CoreError::StorageError(e.to_string()))?,
            );
            let character_id = char_id_str
                .and_then(|s| uuid::Uuid::parse_str(&s).ok())
                .map(CharacterId);
            let source_type = match source_str.as_str() {
                "EpisodicConversation" => RagSourceType::EpisodicConversation,
                "SemanticFact" => RagSourceType::SemanticFact,
                "UserTeaching" => RagSourceType::UserTeaching,
                _ => RagSourceType::LoreDocument,
            };
            let importance = match imp_str.as_str() {
                "Low" => MemoryImportance::Low,
                "High" => MemoryImportance::High,
                "Critical" => MemoryImportance::Critical,
                _ => MemoryImportance::Medium,
            };

            let chunk = DocumentChunk {
                id: chunk_id,
                document_id: doc_id,
                character_id,
                actor_id,
                source_type,
                content,
                chunk_index: chunk_idx as usize,
                importance,
                created_at: created_at as u64,
            };

            // Calculate dense similarity if query vector provided
            if let Some(ref q_vec) = query.query_vector {
                if let Some(ref blob) = emb_blob {
                    if let Some(c_vec) = Self::bytes_to_vector(blob) {
                        let sim = q_vec.cosine_similarity(&c_vec);
                        if let Some(min_sim) = query.min_similarity {
                            if sim < min_sim {
                                continue;
                            }
                        }
                        dense_scores.insert(chunk_id, sim);
                    }
                }
            }

            chunks_map.insert(chunk_id, chunk);
        }

        // Rank dense list
        let mut dense_ranks: Vec<(DocumentChunkId, usize)> = Vec::new();
        let mut sorted_dense: Vec<_> = dense_scores.iter().collect();
        sorted_dense.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, (id, _)) in sorted_dense.iter().enumerate() {
            dense_ranks.push((**id, idx + 1));
        }

        // 2. Sparse Keyword Matching via FTS5 or LIKE
        let mut sparse_scores: HashMap<DocumentChunkId, f32> = HashMap::new();
        let mut sparse_ranks: Vec<(DocumentChunkId, usize)> = Vec::new();

        // Prepare query keywords
        let keywords: Vec<&str> = query
            .text
            .split_whitespace()
            .filter(|w| w.len() > 1)
            .collect();
        if !keywords.is_empty() {
            let fts_query = keywords.join(" OR ");
            if let Ok(mut fts_stmt) = conn.prepare("SELECT chunk_id, bm25(rag_chunks_fts) as rank FROM rag_chunks_fts WHERE rag_chunks_fts MATCH ?1 ORDER BY rank LIMIT 50") {
                if let Ok(mut fts_rows) = fts_stmt.query(params![fts_query]) {
                    let mut rank_idx = 1;
                    while let Ok(Some(row)) = fts_rows.next() {
                        if let Ok(c_id_str) = row.get::<_, String>(0) {
                            if let Ok(u) = uuid::Uuid::parse_str(&c_id_str) {
                                let c_id = DocumentChunkId(u);
                                if chunks_map.contains_key(&c_id) {
                                    let bm25_val: f64 = row.get(1).unwrap_or(0.0);
                                    sparse_scores.insert(c_id, -bm25_val as f32); // lower bm25 is better match in sqlite
                                    sparse_ranks.push((c_id, rank_idx));
                                    rank_idx += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Reciprocal Rank Fusion (RRF)
        let rrf_results = reciprocal_rank_fusion(&dense_ranks, &sparse_ranks, 60);
        let rrf_map: HashMap<DocumentChunkId, f32> = rrf_results.into_iter().collect();

        // 4. Assemble Results with Multi-Factor Scoring
        let mut final_results = Vec::new();

        for (id, chunk) in chunks_map {
            let d_score = dense_scores.get(&id).copied();
            let s_score = sparse_scores.get(&id).copied();
            let rrf = rrf_map.get(&id).copied();

            let base_similarity = d_score.unwrap_or(0.5);
            let final_score = calculate_composite_score(
                base_similarity,
                chunk.importance,
                chunk.created_at,
                now,
                7.0, // 7-day half life
            );

            final_results.push(RagQueryResult {
                chunk,
                dense_score: d_score,
                sparse_score: s_score,
                rrf_score: rrf,
                final_score,
            });
        }

        // Sort descending by final score
        final_results.sort_by(|a, b| {
            b.final_score
                .partial_cmp(&a.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        final_results.truncate(query.top_k);
        Ok(final_results)
    }

    fn delete_document_chunks(&self, document_id: &str) -> Result<usize> {
        let conn = self.conn_lock();
        let deleted = conn
            .execute(
                "DELETE FROM rag_chunks WHERE document_id = ?1",
                params![document_id],
            )
            .map_err(|e| {
                CoreError::StorageError(format!("Failed to delete RAG document chunks: {}", e))
            })?;
        Ok(deleted)
    }

    fn count_chunks(&self, character_id: Option<CharacterId>) -> Result<usize> {
        let conn = self.conn_lock();
        let count: i64 = if let Some(char_id) = character_id {
            conn.query_row(
                "SELECT COUNT(*) FROM rag_chunks WHERE character_id = ?1 OR character_id IS NULL",
                params![char_id.0.to_string()],
                |r| r.get(0),
            )
        } else {
            conn.query_row("SELECT COUNT(*) FROM rag_chunks", [], |r| r.get(0))
        }
        .map_err(|e| CoreError::StorageError(format!("Failed to count RAG chunks: {}", e)))?;

        Ok(count as usize)
    }
}
