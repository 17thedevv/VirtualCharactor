//! Ollama Embedding Client implementation of `vc_core::rag::EmbeddingProvider`.
//!
//! Provides dense vector generation via Ollama's `/api/embeddings` endpoint.
//! Includes graceful fallback to deterministic vectors when Ollama embeddings are unavailable.

use serde::{Deserialize, Serialize};
use vc_core::error::{CoreError, Result};
use vc_core::rag::traits::EmbeddingProvider;
use vc_core::rag::types::EmbeddingVector;

/// Request DTO for Ollama `/api/embeddings`.
#[derive(Debug, Serialize)]
struct OllamaEmbeddingsRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

/// Response DTO from Ollama `/api/embeddings`.
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingsResponse {
    embedding: Option<Vec<f32>>,
    error: Option<String>,
}

/// Request DTO for Ollama batched `/api/embed`.
#[derive(Debug, Serialize)]
struct OllamaEmbedBatchRequest<'a> {
    model: &'a str,
    input: &'a [String],
}

/// Response DTO from Ollama batched `/api/embed`.
#[derive(Debug, Deserialize)]
struct OllamaEmbedBatchResponse {
    embeddings: Option<Vec<Vec<f32>>>,
    #[allow(dead_code)]
    error: Option<String>,
}

/// Production Ollama Embedding Provider.
pub struct OllamaEmbeddingProvider {
    base_url: String,
    model: String,
    dimension: usize,
    agent: ureq::Agent,
    fallback_on_error: bool,
}

impl OllamaEmbeddingProvider {
    /// Create provider with target model (e.g. "nomic-embed-text", "bge-m3").
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, dimension: usize) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(std::time::Duration::from_secs(3))
            .timeout_read(std::time::Duration::from_secs(10))
            .build();

        Self {
            base_url: url,
            model: model.into(),
            dimension,
            agent,
            fallback_on_error: true,
        }
    }

    /// Default configuration for `nomic-embed-text` (768-dim) on localhost.
    pub fn default_nomic() -> Self {
        Self::new("http://127.0.0.1:11434", "nomic-embed-text", 768)
    }

    /// Configuration for `bge-m3` (1024-dim, multilingual / Vietnamese) on localhost.
    pub fn default_bge_m3() -> Self {
        Self::new("http://127.0.0.1:11434", "bge-m3", 1024)
    }

    /// Enable or disable deterministic fallback when Ollama is unavailable.
    pub fn with_fallback(mut self, fallback: bool) -> Self {
        self.fallback_on_error = fallback;
        self
    }

    /// Deterministic pseudo-embedding generator used for testing or fallback.
    fn compute_deterministic_vector(&self, text: &str) -> EmbeddingVector {
        let mut vec = vec![0.0f32; self.dimension];
        let words: Vec<&str> = text.split_whitespace().collect();

        for (word_idx, word) in words.iter().enumerate() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&word.to_lowercase(), &mut hasher);
            let h = std::hash::Hasher::finish(&hasher);

            let pos = (h as usize) % self.dimension;
            let weight = 1.0 / (1.0 + (word_idx as f32 * 0.1));
            vec[pos] += weight;

            // Bigram hashing for phrase context
            if word_idx > 0 {
                let mut bi_hasher = std::collections::hash_map::DefaultHasher::new();
                std::hash::Hash::hash(&(words[word_idx - 1], word), &mut bi_hasher);
                let bi_h = std::hash::Hasher::finish(&bi_hasher);
                let bi_pos = (bi_h as usize) % self.dimension;
                vec[bi_pos] += weight * 0.5;
            }
        }

        EmbeddingVector::new(vec).normalize()
    }
}

impl EmbeddingProvider for OllamaEmbeddingProvider {
    fn embed_text(&self, text: &str) -> Result<EmbeddingVector> {
        let url = format!("{}/api/embeddings", self.base_url);
        let req = OllamaEmbeddingsRequest {
            model: &self.model,
            prompt: text,
        };

        match self.agent.post(&url).send_json(&req) {
            Ok(resp) => {
                let parsed: OllamaEmbeddingsResponse = resp.into_json().map_err(|e| {
                    CoreError::ProviderError(format!(
                        "Failed to parse Ollama embedding response: {}",
                        e
                    ))
                })?;

                if let Some(err) = parsed.error {
                    if self.fallback_on_error {
                        return Ok(self.compute_deterministic_vector(text));
                    }
                    return Err(CoreError::ProviderError(format!(
                        "Ollama embedding error: {}",
                        err
                    )));
                }

                if let Some(emb) = parsed.embedding {
                    return Ok(EmbeddingVector::new(emb).normalize());
                }

                if self.fallback_on_error {
                    Ok(self.compute_deterministic_vector(text))
                } else {
                    Err(CoreError::ProviderError(
                        "Empty embedding in Ollama response".into(),
                    ))
                }
            }
            Err(e) => {
                if self.fallback_on_error {
                    Ok(self.compute_deterministic_vector(text))
                } else {
                    Err(CoreError::ProviderError(format!(
                        "Ollama embedding HTTP error connecting to '{}': {}",
                        url, e
                    )))
                }
            }
        }
    }

    fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbeddingVector>> {
        // Try batched `/api/embed` first
        let url = format!("{}/api/embed", self.base_url);
        let req = OllamaEmbedBatchRequest {
            model: &self.model,
            input: texts,
        };

        if let Ok(resp) = self.agent.post(&url).send_json(&req) {
            if let Ok(parsed) = resp.into_json::<OllamaEmbedBatchResponse>() {
                if let Some(embeddings) = parsed.embeddings {
                    if embeddings.len() == texts.len() {
                        return Ok(embeddings
                            .into_iter()
                            .map(|v| EmbeddingVector::new(v).normalize())
                            .collect());
                    }
                }
            }
        }

        // Fallback to sequential individual requests
        texts.iter().map(|t| self.embed_text(t)).collect()
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &'static str {
        "OllamaEmbeddingProvider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_provider_deterministic_fallback() {
        let provider = OllamaEmbeddingProvider::new("http://127.0.0.1:9999", "test-model", 384);
        let emb1 = provider.embed_text("Rust programming language").unwrap();
        let emb2 = provider.embed_text("Rust programming language").unwrap();
        let emb3 = provider.embed_text("Cooking Italian pasta").unwrap();

        assert_eq!(emb1.dimension(), 384);
        // Same text produces identical embeddings
        assert_eq!(emb1, emb2);

        // Similar topic produces higher similarity than completely unrelated topic
        let sim_same = emb1.cosine_similarity(&emb2);
        let sim_diff = emb1.cosine_similarity(&emb3);

        assert!((sim_same - 1.0).abs() < 1e-4);
        assert!(sim_same > sim_diff);
    }
}
