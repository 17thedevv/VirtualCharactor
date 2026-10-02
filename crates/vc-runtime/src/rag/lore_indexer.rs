//! Lore & Document Ingestion Indexer for RAG Knowledge Bases (Step 1.8.4).
//!
//! Scans knowledge directories (e.g. `data/knowledge/`), chunks files,
//! computes dense vector embeddings, and stores them in the KnowledgeRepository.

use crate::rag::chunker::{ChunkerOptions, SemanticTextChunker};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::error::{CoreError, Result};
use vc_core::memory::MemoryImportance;
use vc_core::rag::traits::{EmbeddingProvider, KnowledgeRepository};
use vc_core::rag::types::RagSourceType;

/// Result summary of an indexing run.
#[derive(Debug, Clone, Default)]
pub struct IndexingReport {
    pub files_scanned: usize,
    pub chunks_created: usize,
    pub errors: Vec<String>,
}

/// Automated document indexer for local knowledge files.
pub struct LoreIndexer {
    chunker: SemanticTextChunker,
    embedder: Arc<dyn EmbeddingProvider>,
    repository: Arc<dyn KnowledgeRepository>,
}

impl LoreIndexer {
    pub fn new(
        embedder: Arc<dyn EmbeddingProvider>,
        repository: Arc<dyn KnowledgeRepository>,
    ) -> Self {
        Self {
            chunker: SemanticTextChunker::with_defaults(),
            embedder,
            repository,
        }
    }

    pub fn with_options(
        embedder: Arc<dyn EmbeddingProvider>,
        repository: Arc<dyn KnowledgeRepository>,
        options: ChunkerOptions,
    ) -> Self {
        Self {
            chunker: SemanticTextChunker::new(options),
            embedder,
            repository,
        }
    }

    /// Index a single file from the filesystem.
    pub fn index_file(
        &self,
        file_path: &Path,
        character_id: Option<CharacterId>,
        importance: MemoryImportance,
    ) -> Result<usize> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            CoreError::StorageError(format!(
                "Failed to read knowledge file {:?}: {}",
                file_path, e
            ))
        })?;

        let doc_id = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_doc");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // 1. Delete prior chunks for this document
        let _ = self.repository.delete_document_chunks(doc_id);

        // 2. Chunk text
        let chunks = self.chunker.chunk_document(
            doc_id,
            &content,
            RagSourceType::LoreDocument,
            character_id,
            None, // Shared public lore has actor_id: None
            importance,
            now,
        );

        let count = chunks.len();

        // 3. Embed chunks and save to repository
        for chunk in chunks {
            let emb = self.embedder.embed_text(&chunk.content).ok();
            self.repository.save_chunk(&chunk, emb.as_ref())?;
        }

        Ok(count)
    }

    /// Recursively scan and index an entire directory.
    pub fn index_directory(
        &self,
        dir_path: &Path,
        character_id: Option<CharacterId>,
    ) -> IndexingReport {
        let mut report = IndexingReport::default();

        if !dir_path.exists() || !dir_path.is_dir() {
            report
                .errors
                .push(format!("Knowledge directory {:?} does not exist", dir_path));
            return report;
        }

        let entries = match fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(err) => {
                report
                    .errors
                    .push(format!("Cannot read directory {:?}: {}", dir_path, err));
                return report;
            }
        };

        for entry_res in entries {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    report.errors.push(format!("Directory entry error: {}", e));
                    continue;
                }
            };

            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "md" || ext == "txt" || ext == "json" {
                    report.files_scanned += 1;
                    match self.index_file(&path, character_id, MemoryImportance::High) {
                        Ok(chunk_count) => {
                            report.chunks_created += chunk_count;
                        }
                        Err(e) => {
                            report
                                .errors
                                .push(format!("Failed to index {:?}: {}", path, e));
                        }
                    }
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vc_core::rag::types::EmbeddingVector;
    use vc_storage::SqliteStorage;

    struct DummyEmbedder;
    impl EmbeddingProvider for DummyEmbedder {
        fn embed_text(&self, text: &str) -> Result<EmbeddingVector> {
            Ok(EmbeddingVector::new(vec![text.len() as f32, 1.0, 0.0]).normalize())
        }
        fn dimension(&self) -> usize {
            3
        }
    }

    #[test]
    fn test_lore_indexer_indexes_file() {
        let storage = Arc::new(SqliteStorage::in_memory().unwrap());
        let embedder = Arc::new(DummyEmbedder);
        let indexer = LoreIndexer::new(embedder, storage.clone());

        // Create temporary test file
        let temp_dir = std::env::temp_dir().join(format!("test_lore_{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test_lore.md");
        fs::write(&test_file, "# World Setting\n\nVirtualCharacter exists in Neo-Tokyo.\n\nShe loves systems programming.").unwrap();

        let count = indexer
            .index_file(&test_file, None, MemoryImportance::High)
            .expect("index ok");
        assert!(count >= 1);

        assert_eq!(storage.count_chunks(None).unwrap(), count);

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
