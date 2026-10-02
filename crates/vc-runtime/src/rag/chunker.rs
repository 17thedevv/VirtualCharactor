//! Semantic Document Chunker for RAG Knowledge Ingestion (Skill 16: Context Engineering).
//!
//! Splits documents along natural boundaries (paragraphs, sentences)
//! with configurable chunk size and overlap to preserve semantic context.

use vc_core::character::CharacterId;
use vc_core::memory::MemoryImportance;
use vc_core::rag::types::{DocumentChunk, RagSourceType};

/// Configuration options for text chunking.
#[derive(Debug, Clone)]
pub struct ChunkerOptions {
    /// Target maximum characters per chunk. Default: 500.
    pub max_chars: usize,
    /// Number of overlap characters carried over to subsequent chunk. Default: 50.
    pub overlap_chars: usize,
}

impl Default for ChunkerOptions {
    fn default() -> Self {
        Self {
            max_chars: 500,
            overlap_chars: 50,
        }
    }
}

/// Chunker capable of splitting plain text or markdown into semantic document chunks.
pub struct SemanticTextChunker {
    options: ChunkerOptions,
}

impl SemanticTextChunker {
    pub fn new(options: ChunkerOptions) -> Self {
        Self { options }
    }

    pub fn with_defaults() -> Self {
        Self::new(ChunkerOptions::default())
    }

    /// Split raw text into raw text slice chunks respecting paragraph and sentence boundaries.
    pub fn split_text(&self, text: &str) -> Vec<String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        if trimmed.chars().count() <= self.options.max_chars {
            return vec![trimmed.to_string()];
        }

        let mut chunks = Vec::new();
        let paragraphs: Vec<&str> = trimmed.split("\n\n").collect();
        let mut current_chunk = String::new();

        for p in paragraphs {
            let p_trimmed = p.trim();
            if p_trimmed.is_empty() {
                continue;
            }

            // If adding paragraph exceeds max_chars, flush current chunk
            if !current_chunk.is_empty()
                && (current_chunk.chars().count() + p_trimmed.chars().count() + 2)
                    > self.options.max_chars
            {
                chunks.push(current_chunk.trim().to_string());

                // Carry over overlap tail
                let current_chars: Vec<char> = current_chunk.chars().collect();
                if current_chars.len() > self.options.overlap_chars {
                    let overlap_start = current_chars.len() - self.options.overlap_chars;
                    current_chunk = current_chars[overlap_start..].iter().collect();
                    current_chunk.push('\n');
                } else {
                    current_chunk.clear();
                }
            }

            // If a single paragraph is larger than max_chars, split by sentences
            if p_trimmed.chars().count() > self.options.max_chars {
                let sentences = self.split_sentences(p_trimmed);
                for s in sentences {
                    if !current_chunk.is_empty()
                        && (current_chunk.chars().count() + s.chars().count() + 1)
                            > self.options.max_chars
                    {
                        chunks.push(current_chunk.trim().to_string());
                        current_chunk.clear();
                    }
                    if !current_chunk.is_empty() {
                        current_chunk.push(' ');
                    }
                    current_chunk.push_str(&s);
                }
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(p_trimmed);
            }
        }

        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }

        chunks
    }

    /// Split a paragraph into sentence segments.
    fn split_sentences<'a>(&self, text: &'a str) -> Vec<String> {
        let mut sentences = Vec::new();
        let mut current = String::new();

        for c in text.chars() {
            current.push(c);
            if c == '.' || c == '!' || c == '?' || c == '\n' {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
                current.clear();
            }
        }

        let trailing = current.trim();
        if !trailing.is_empty() {
            sentences.push(trailing.to_string());
        }

        sentences
    }

    /// Chunk text and convert into structured `DocumentChunk` instances.
    pub fn chunk_document(
        &self,
        document_id: &str,
        content: &str,
        source_type: RagSourceType,
        character_id: Option<CharacterId>,
        actor_id: Option<String>,
        importance: MemoryImportance,
        created_at: u64,
    ) -> Vec<DocumentChunk> {
        let raw_chunks = self.split_text(content);
        raw_chunks
            .into_iter()
            .enumerate()
            .map(|(index, chunk_text)| {
                let mut chunk = DocumentChunk::new(chunk_text, source_type, created_at)
                    .with_document_id(document_id)
                    .with_chunk_index(index)
                    .with_importance(importance);

                if let Some(char_id) = character_id {
                    chunk = chunk.with_character_id(char_id);
                }
                if let Some(ref a_id) = actor_id {
                    chunk = chunk.with_actor_id(a_id.clone());
                }
                chunk
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_text_single_chunk() {
        let chunker = SemanticTextChunker::with_defaults();
        let text = "Aria is a kind virtual companion.";
        let chunks = chunker.split_text(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }

    #[test]
    fn test_paragraph_splitting_with_overlap() {
        let chunker = SemanticTextChunker::new(ChunkerOptions {
            max_chars: 80,
            overlap_chars: 20,
        });

        let text = "Paragraph one with some interesting background story about the world.\n\nParagraph two continuing the description of the magical kingdom.";
        let chunks = chunker.split_text(text);

        assert!(chunks.len() >= 2);
        assert!(chunks[0].contains("Paragraph one"));
        assert!(chunks[1].contains("Paragraph two"));
    }

    #[test]
    fn test_chunk_document_metadata() {
        let chunker = SemanticTextChunker::with_defaults();
        let doc = "Line 1.\n\nLine 2.";
        let chunks = chunker.chunk_document(
            "lore.md",
            doc,
            RagSourceType::LoreDocument,
            None,
            None,
            MemoryImportance::High,
            1000,
        );

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].document_id.as_deref(), Some("lore.md"));
        assert_eq!(chunks[0].importance, MemoryImportance::High);
    }
}
