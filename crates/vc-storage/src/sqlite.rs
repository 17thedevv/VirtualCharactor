use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

use vc_core::character::{Character, CharacterId};
use vc_core::error::{CoreError, Result};
use vc_core::memory::{Memory, MemoryId, MemoryQuery};
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;

use crate::repository::{
    CharacterRepository, MemoryRepository, RelationshipRepository, StateRepository,
};

/// Record of an executed web search and its retrieved snippets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRecord {
    pub id: String,
    pub character_id: CharacterId,
    pub query: String,
    pub source: String,
    pub snippets: Vec<String>,
    pub summary: Option<String>,
    pub created_at: u64,
}

/// Record of an interaction dialogue turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueRecord {
    pub id: String,
    pub character_id: CharacterId,
    pub actor_id: String,
    pub sender: String,
    pub text: String,
    pub created_at: u64,
}

/// Production SQLite implementation for Local-First persistence.
///
/// Thread-safe: Access is protected by an internal mutex to serialize writes
/// and ensure single-writer database integrity under SQLite concurrency constraints.
#[derive(Clone)]
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    /// Open or create an SQLite database at the specified file system path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CoreError::StorageError(format!("Failed to create storage directory: {}", e))
                })?;
            }
        }

        let conn = Connection::open(p).map_err(|e| {
            CoreError::StorageError(format!("Failed to open SQLite database at {:?}: {}", p, e))
        })?;

        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory SQLite database instance (ideal for fast, isolated integration tests).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| {
            CoreError::StorageError(format!("Failed to open in-memory SQLite: {}", e))
        })?;

        Self::init_tables(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Initialize SQLite schema with required tables and indexes.
    fn init_tables(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS characters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                personality_json TEXT,
                state_json TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS relationships (
                character_id TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                stage TEXT NOT NULL,
                closeness REAL NOT NULL,
                trust REAL NOT NULL,
                familiarity REAL NOT NULL,
                affection REAL NOT NULL,
                tension REAL NOT NULL,
                state_json TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (character_id, actor_id)
            );

            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                actor_id TEXT,
                memory_type TEXT NOT NULL,
                importance TEXT NOT NULL,
                content TEXT NOT NULL,
                memory_json TEXT NOT NULL,
                embedding_json TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS search_history (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                query TEXT NOT NULL,
                source TEXT NOT NULL,
                snippets_json TEXT NOT NULL,
                summary TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS dialogue_history (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                sender TEXT NOT NULL,
                text TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_relationships_char_actor 
            ON relationships (character_id, actor_id);

            CREATE INDEX IF NOT EXISTS idx_memories_char_actor 
            ON memories (character_id, actor_id);

            CREATE INDEX IF NOT EXISTS idx_memories_char 
            ON memories (character_id);

            CREATE INDEX IF NOT EXISTS idx_search_history_char 
            ON search_history (character_id, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_dialogue_history_char 
            ON dialogue_history (character_id, created_at ASC);
            "#,
        )
        .map_err(|e| {
            CoreError::StorageError(format!("Failed to initialize database schema: {}", e))
        })?;

        // Backward compatibility migration: add embedding_json column if missing in older databases
        let _ = conn.execute("ALTER TABLE memories ADD COLUMN embedding_json TEXT", []);

        // Initialize RAG knowledge tables
        Self::init_rag_tables(conn)?;

        Ok(())
    }

    pub(crate) fn conn_lock(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Persist or update a web search record.
    pub fn save_search_record(&self, record: &SearchRecord) -> Result<()> {
        let conn = self.conn.lock();
        let char_id_str = record.character_id.0.to_string();
        let snippets_json = serde_json::to_string(&record.snippets).unwrap_or_else(|_| "[]".into());
        conn.execute(
            r#"
            INSERT INTO search_history (id, character_id, query, source, snippets_json, summary, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                query = excluded.query,
                source = excluded.source,
                snippets_json = excluded.snippets_json,
                summary = excluded.summary
            "#,
            params![
                record.id,
                char_id_str,
                record.query,
                record.source,
                snippets_json,
                record.summary,
                record.created_at as i64
            ],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save search record: {}", e)))?;
        Ok(())
    }

    /// List recent web searches for a character.
    pub fn list_search_records(
        &self,
        character_id: CharacterId,
        limit: usize,
    ) -> Result<Vec<SearchRecord>> {
        let conn = self.conn.lock();
        let char_id_str = character_id.0.to_string();
        let mut stmt = conn
            .prepare(
                "SELECT id, character_id, query, source, snippets_json, summary, created_at 
                 FROM search_history 
                 WHERE character_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT ?2",
            )
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![char_id_str, limit as i64], |row| {
                let id: String = row.get(0)?;
                let cid_str: String = row.get(1)?;
                let query: String = row.get(2)?;
                let source: String = row.get(3)?;
                let snippets_raw: String = row.get(4)?;
                let summary: Option<String> = row.get(5)?;
                let created_at: i64 = row.get(6)?;

                let cid = CharacterId(Uuid::parse_str(&cid_str).unwrap_or_default());
                let snippets: Vec<String> = serde_json::from_str(&snippets_raw).unwrap_or_default();
                Ok(SearchRecord {
                    id,
                    character_id: cid,
                    query,
                    source,
                    snippets,
                    summary,
                    created_at: created_at as u64,
                })
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                list.push(item);
            }
        }
        Ok(list)
    }

    /// Clear all search records for a character.
    pub fn clear_search_records(&self, character_id: CharacterId) -> Result<()> {
        let conn = self.conn.lock();
        let char_id_str = character_id.0.to_string();
        conn.execute(
            "DELETE FROM search_history WHERE character_id = ?1",
            params![char_id_str],
        )
        .map_err(|e| CoreError::StorageError(e.to_string()))?;
        Ok(())
    }

    /// Persist or update an interaction dialogue turn.
    pub fn save_dialogue_record(&self, record: &DialogueRecord) -> Result<()> {
        let conn = self.conn.lock();
        let char_id_str = record.character_id.0.to_string();
        conn.execute(
            r#"
            INSERT INTO dialogue_history (id, character_id, actor_id, sender, text, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                text = excluded.text
            "#,
            params![
                record.id,
                char_id_str,
                record.actor_id,
                record.sender,
                record.text,
                record.created_at as i64
            ],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save dialogue record: {}", e)))?;
        Ok(())
    }

    /// List recent dialogue history for a character in chronological order (oldest to newest among the most recent).
    pub fn list_dialogue_records(
        &self,
        character_id: CharacterId,
        limit: usize,
    ) -> Result<Vec<DialogueRecord>> {
        let conn = self.conn.lock();
        let char_id_str = character_id.0.to_string();
        let mut stmt = conn
            .prepare(
                "SELECT id, character_id, actor_id, sender, text, created_at 
                 FROM dialogue_history 
                 WHERE character_id = ?1 
                 ORDER BY created_at DESC 
                 LIMIT ?2",
            )
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![char_id_str, limit as i64], |row| {
                let id: String = row.get(0)?;
                let cid_str: String = row.get(1)?;
                let actor_id: String = row.get(2)?;
                let sender: String = row.get(3)?;
                let text: String = row.get(4)?;
                let created_at: i64 = row.get(5)?;

                let cid = CharacterId(Uuid::parse_str(&cid_str).unwrap_or_default());
                Ok(DialogueRecord {
                    id,
                    character_id: cid,
                    actor_id,
                    sender,
                    text,
                    created_at: created_at as u64,
                })
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                list.push(item);
            }
        }
        // Reverse so that the latest messages are ordered chronologically (oldest -> newest)
        list.reverse();
        Ok(list)
    }

    /// Clear dialogue history for a character.
    pub fn clear_dialogue_records(&self, character_id: CharacterId) -> Result<()> {
        let conn = self.conn.lock();
        let char_id_str = character_id.0.to_string();
        conn.execute(
            "DELETE FROM dialogue_history WHERE character_id = ?1",
            params![char_id_str],
        )
        .map_err(|e| CoreError::StorageError(e.to_string()))?;
        Ok(())
    }
}

impl CharacterRepository for SqliteStorage {
    fn get_character(&self, id: CharacterId) -> Result<Character> {
        let conn = self.conn.lock();
        let id_str = id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT id, name FROM characters WHERE id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let char_opt = stmt
            .query_row(params![id_str], |row| {
                let id_raw: String = row.get(0)?;
                let name: String = row.get(1)?;
                let parsed_uuid = Uuid::parse_str(&id_raw).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "invalid uuid",
                        )),
                    )
                })?;
                Ok(Character {
                    id: CharacterId(parsed_uuid),
                    name,
                })
            })
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        char_opt
            .ok_or_else(|| CoreError::CharacterNotFound(format!("Character {} not found", id.0)))
    }

    fn save_character(&self, character: &Character) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = character.id.0.to_string();
        let now = Self::now_secs() as i64;

        conn.execute(
            r#"
            INSERT INTO characters (id, name, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?3)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                updated_at = excluded.updated_at
            "#,
            params![id_str, character.name, now],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save character: {}", e)))?;

        Ok(())
    }

    fn get_personality(&self, character_id: CharacterId) -> Result<Option<Personality>> {
        let conn = self.conn.lock();
        let id_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT personality_json FROM characters WHERE id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let json_opt: Option<Option<String>> = stmt
            .query_row(params![id_str], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        match json_opt.flatten() {
            Some(json_str) => {
                let personality: Personality = serde_json::from_str(&json_str).map_err(|e| {
                    CoreError::StorageError(format!(
                        "Failed to deserialize personality JSON: {}",
                        e
                    ))
                })?;
                Ok(Some(personality))
            }
            None => Ok(None),
        }
    }

    fn save_personality(&self, character_id: CharacterId, personality: &Personality) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = character_id.0.to_string();
        let json_str = serde_json::to_string(personality).map_err(|e| {
            CoreError::StorageError(format!("Failed to serialize personality: {}", e))
        })?;
        let now = Self::now_secs() as i64;

        conn.execute(
            r#"
            INSERT INTO characters (id, name, personality_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?4)
            ON CONFLICT(id) DO UPDATE SET
                personality_json = excluded.personality_json,
                updated_at = excluded.updated_at
            "#,
            params![id_str, personality.identity.core_identity, json_str, now],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save personality: {}", e)))?;

        Ok(())
    }

    fn list_characters(&self) -> Result<Vec<Character>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, name FROM characters")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id_raw: String = row.get(0)?;
                let name: String = row.get(1)?;
                let parsed_uuid = Uuid::parse_str(&id_raw).unwrap_or_default();
                Ok(Character {
                    id: CharacterId(parsed_uuid),
                    name,
                })
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut characters = Vec::new();
        for r in rows {
            if let Ok(c) = r {
                characters.push(c);
            }
        }
        Ok(characters)
    }
}

impl StateRepository for SqliteStorage {
    fn get_state(&self, character_id: CharacterId) -> Result<Option<CharacterState>> {
        let conn = self.conn.lock();
        let id_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT state_json FROM characters WHERE id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let json_opt: Option<Option<String>> = stmt
            .query_row(params![id_str], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        match json_opt.flatten() {
            Some(json_str) => {
                let state: CharacterState = serde_json::from_str(&json_str).map_err(|e| {
                    CoreError::StorageError(format!("Failed to deserialize state JSON: {}", e))
                })?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    fn save_state(&self, character_id: CharacterId, state: &CharacterState) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = character_id.0.to_string();
        let json_str = serde_json::to_string(state)
            .map_err(|e| CoreError::StorageError(format!("Failed to serialize state: {}", e)))?;
        let now = Self::now_secs() as i64;

        conn.execute(
            r#"
            INSERT INTO characters (id, name, state_json, created_at, updated_at)
            VALUES (?1, 'Unknown', ?2, ?3, ?3)
            ON CONFLICT(id) DO UPDATE SET
                state_json = excluded.state_json,
                updated_at = excluded.updated_at
            "#,
            params![id_str, json_str, now],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save state: {}", e)))?;

        Ok(())
    }
}

impl RelationshipRepository for SqliteStorage {
    fn get_relationship(
        &self,
        character_id: CharacterId,
        actor_id: &str,
    ) -> Result<Option<Relationship>> {
        let conn = self.conn.lock();
        let char_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare(
                "SELECT state_json FROM relationships WHERE character_id = ?1 AND actor_id = ?2",
            )
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let json_opt: Option<String> = stmt
            .query_row(params![char_str, actor_id], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        match json_opt {
            Some(json_str) => {
                let rel: Relationship = serde_json::from_str(&json_str).map_err(|e| {
                    CoreError::StorageError(format!(
                        "Failed to deserialize relationship JSON: {}",
                        e
                    ))
                })?;
                Ok(Some(rel))
            }
            None => Ok(None),
        }
    }

    fn save_relationship(&self, relationship: &Relationship) -> Result<()> {
        let conn = self.conn.lock();
        let char_str = relationship.character_id.0.to_string();
        let actor_str = &relationship.target_id;
        let stage_str = relationship.state.stage.as_str();
        let json_str = serde_json::to_string(relationship).map_err(|e| {
            CoreError::StorageError(format!("Failed to serialize relationship: {}", e))
        })?;
        let now = Self::now_secs() as i64;

        conn.execute(
            r#"
            INSERT INTO relationships (
                character_id, actor_id, stage, closeness, trust, 
                familiarity, affection, tension, state_json, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(character_id, actor_id) DO UPDATE SET
                stage = excluded.stage,
                closeness = excluded.closeness,
                trust = excluded.trust,
                familiarity = excluded.familiarity,
                affection = excluded.affection,
                tension = excluded.tension,
                state_json = excluded.state_json,
                updated_at = excluded.updated_at
            "#,
            params![
                char_str,
                actor_str,
                stage_str,
                relationship.state.closeness as f64,
                relationship.state.trust as f64,
                relationship.state.familiarity as f64,
                relationship.state.affection as f64,
                relationship.state.tension as f64,
                json_str,
                now,
            ],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save relationship: {}", e)))?;

        Ok(())
    }

    fn list_relationships(&self, character_id: CharacterId) -> Result<Vec<Relationship>> {
        let conn = self.conn.lock();
        let char_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT state_json FROM relationships WHERE character_id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![char_str], |row| {
                let json_str: String = row.get(0)?;
                Ok(json_str)
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut relationships = Vec::new();
        for r in rows {
            if let Ok(json_str) = r {
                if let Ok(rel) = serde_json::from_str::<Relationship>(&json_str) {
                    relationships.push(rel);
                }
            }
        }
        Ok(relationships)
    }
}

impl MemoryRepository for SqliteStorage {
    fn get_memory(&self, id: MemoryId) -> Result<Memory> {
        let conn = self.conn.lock();
        let id_str = id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT memory_json FROM memories WHERE id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let json_opt: Option<String> = stmt
            .query_row(params![id_str], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        match json_opt {
            Some(json_str) => {
                let memory: Memory = serde_json::from_str(&json_str).map_err(|e| {
                    CoreError::StorageError(format!("Failed to deserialize memory JSON: {}", e))
                })?;
                Ok(memory)
            }
            None => Err(CoreError::MemoryNotFound(format!(
                "Memory {} not found",
                id.0
            ))),
        }
    }

    fn save_memory(&self, memory: &Memory) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = memory.id.0.to_string();
        let char_str = memory
            .character_id
            .map(|c| c.0.to_string())
            .unwrap_or_else(|| Uuid::nil().to_string());
        let actor_str = memory.metadata.source_actor_id.as_deref();
        let type_str = format!("{:?}", memory.metadata.memory_type);
        let importance_str = format!("{:?}", memory.metadata.importance);
        let json_str = serde_json::to_string(memory)
            .map_err(|e| CoreError::StorageError(format!("Failed to serialize memory: {}", e)))?;
        let now = Self::now_secs() as i64;
        let created_at = memory.metadata.timestamp as i64;

        conn.execute(
            r#"
            INSERT INTO memories (
                id, character_id, actor_id, memory_type, importance, 
                content, memory_json, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                importance = excluded.importance,
                memory_json = excluded.memory_json,
                updated_at = excluded.updated_at
            "#,
            params![
                id_str,
                char_str,
                actor_str,
                type_str,
                importance_str,
                memory.content,
                json_str,
                created_at,
                now,
            ],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to save memory: {}", e)))?;

        Ok(())
    }

    fn list_memories(&self, character_id: CharacterId) -> Result<Vec<Memory>> {
        let conn = self.conn.lock();
        let char_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare(
                "SELECT memory_json FROM memories WHERE character_id = ?1 ORDER BY created_at ASC",
            )
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![char_str], |row| {
                let json_str: String = row.get(0)?;
                Ok(json_str)
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut memories = Vec::new();
        for r in rows {
            if let Ok(json_str) = r {
                if let Ok(m) = serde_json::from_str::<Memory>(&json_str) {
                    memories.push(m);
                }
            }
        }
        Ok(memories)
    }

    fn query_memories(
        &self,
        character_id: CharacterId,
        query: &MemoryQuery,
        now: u64,
    ) -> Result<Vec<Memory>> {
        let mut all_memories = self.list_memories(character_id)?;
        let mut scored: Vec<(f32, usize)> = Vec::new();

        for (idx, mem) in all_memories.iter().enumerate() {
            // 1. Strict Actor Privacy Isolation (Skill 12 Rule)
            if !mem.can_be_retrieved_by(query.actor_id.as_deref()) {
                continue;
            }

            // 2. Filter by Memory Type if specified
            if let Some(ref required_type) = query.memory_type {
                if &mem.metadata.memory_type != required_type {
                    continue;
                }
            }

            // 3. Filter by minimum importance threshold
            let eff_imp = mem.effective_importance();
            if let Some(min_imp) = query.min_importance {
                if eff_imp < min_imp {
                    continue;
                }
            }

            // 4. Keyword / Semantic match score [0.0, 1.0]
            let (sim_score, has_text_filter) = match &query.query_text {
                Some(text) if !text.trim().is_empty() => {
                    let query_tokens: Vec<&str> = text.split_whitespace().collect();
                    let content_lower = mem.content.to_lowercase();
                    let mut matched = 0;
                    for token in &query_tokens {
                        let tok_lower = token.to_lowercase();
                        if content_lower.contains(&tok_lower)
                            || mem.metadata.tags.iter().any(|t| t.contains(&tok_lower))
                        {
                            matched += 1;
                        }
                    }
                    if query_tokens.is_empty() {
                        (0.5, false)
                    } else {
                        ((matched as f32) / (query_tokens.len() as f32), true)
                    }
                }
                _ => (0.5, false),
            };

            // If a specific query was provided, skip non-pinned memories with zero keyword relevance
            if !mem.lifecycle.is_pinned && has_text_filter && sim_score == 0.0 {
                continue;
            }

            // 5. Recency score [0.0, 1.0]
            let age_secs = now.saturating_sub(mem.metadata.timestamp);
            let age_days = (age_secs as f32) / 86400.0;
            let recency_score = (-0.05 * age_days).exp();

            // 6. Multidimensional composite score
            let composite_score = (sim_score * query.semantic_weight)
                + (recency_score * query.recency_weight)
                + (eff_imp * query.importance_weight);

            scored.push((composite_score, idx));
        }

        // Sort descending by composite score
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let limit = query.limit.max(1);
        let selected_indices: Vec<usize> =
            scored.into_iter().take(limit).map(|(_, idx)| idx).collect();

        let mut results = Vec::with_capacity(selected_indices.len());
        for idx in selected_indices {
            all_memories[idx].touch(now);
            // Save updated access stats back to SQLite
            let _ = self.save_memory(&all_memories[idx]);
            results.push(all_memories[idx].clone());
        }

        Ok(results)
    }

    fn delete_memory(&self, id: MemoryId) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = id.0.to_string();

        conn.execute("DELETE FROM memories WHERE id = ?1", params![id_str])
            .map_err(|e| CoreError::StorageError(format!("Failed to delete memory: {}", e)))?;

        Ok(())
    }

    fn clear_memories(&self, character_id: CharacterId) -> Result<()> {
        let conn = self.conn.lock();
        let char_str = character_id.0.to_string();

        conn.execute(
            "DELETE FROM memories WHERE character_id = ?1",
            params![char_str],
        )
        .map_err(|e| CoreError::StorageError(format!("Failed to clear memories: {}", e)))?;

        Ok(())
    }

    fn save_memory_with_embedding(&self, memory: &Memory, embedding: &[f32]) -> Result<()> {
        let conn = self.conn.lock();
        let id_str = memory.id.0.to_string();
        let char_str = memory
            .character_id
            .map(|c| c.0.to_string())
            .unwrap_or_else(|| Uuid::nil().to_string());
        let actor_str = memory.metadata.source_actor_id.as_deref();
        let type_str = format!("{:?}", memory.metadata.memory_type);
        let importance_str = format!("{:?}", memory.metadata.importance);
        let json_str = serde_json::to_string(memory)
            .map_err(|e| CoreError::StorageError(format!("Failed to serialize memory: {}", e)))?;
        let emb_json = serde_json::to_string(embedding).map_err(|e| {
            CoreError::StorageError(format!("Failed to serialize embedding: {}", e))
        })?;
        let now = Self::now_secs() as i64;
        let created_at = memory.metadata.timestamp as i64;

        conn.execute(
            r#"
            INSERT INTO memories (
                id, character_id, actor_id, memory_type, importance, 
                content, memory_json, embedding_json, created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                importance = excluded.importance,
                memory_json = excluded.memory_json,
                embedding_json = excluded.embedding_json,
                updated_at = excluded.updated_at
            "#,
            params![
                id_str,
                char_str,
                actor_str,
                type_str,
                importance_str,
                memory.content,
                json_str,
                emb_json,
                created_at,
                now,
            ],
        )
        .map_err(|e| {
            CoreError::StorageError(format!("Failed to save memory with embedding: {}", e))
        })?;

        Ok(())
    }

    fn get_memory_embedding(&self, id: MemoryId) -> Result<Option<Vec<f32>>> {
        let conn = self.conn.lock();
        let id_str = id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT embedding_json FROM memories WHERE id = ?1")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let row_opt: Option<Option<String>> = stmt
            .query_row(params![id_str], |row| row.get(0))
            .optional()
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        match row_opt.flatten() {
            Some(json_str) => {
                let vec: Vec<f32> = serde_json::from_str(&json_str).map_err(|e| {
                    CoreError::StorageError(format!("Failed to deserialize embedding JSON: {}", e))
                })?;
                Ok(Some(vec))
            }
            None => Ok(None),
        }
    }

    fn search_similar_memories(
        &self,
        character_id: CharacterId,
        query: &crate::embedding::VectorMemoryQuery,
    ) -> Result<Vec<(Memory, f32)>> {
        let conn = self.conn.lock();
        let char_str = character_id.0.to_string();

        let mut stmt = conn
            .prepare("SELECT memory_json, embedding_json FROM memories WHERE character_id = ?1 AND embedding_json IS NOT NULL")
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![char_str], |row| {
                let mem_json: String = row.get(0)?;
                let emb_json: Option<String> = row.get(1)?;
                Ok((mem_json, emb_json))
            })
            .map_err(|e| CoreError::StorageError(e.to_string()))?;

        let mut candidates = Vec::new();
        for r in rows {
            if let Ok((mem_str, Some(emb_str))) = r {
                if let (Ok(mem), Ok(emb)) = (
                    serde_json::from_str::<Memory>(&mem_str),
                    serde_json::from_str::<Vec<f32>>(&emb_str),
                ) {
                    // Strict Actor Privacy Isolation (Skill 12 / 25)
                    if !mem.can_be_retrieved_by(query.actor_id.as_deref()) {
                        continue;
                    }

                    let sim = crate::embedding::cosine_similarity(&query.query_vector, &emb);
                    if sim >= query.min_similarity {
                        candidates.push((mem, sim));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_k = query.top_k.max(1);
        candidates.truncate(top_k);

        Ok(candidates)
    }
}
