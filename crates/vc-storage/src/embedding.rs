use vc_core::error::Result;

/// Calculate cosine similarity between two float vectors.
///
/// Returns a value between -1.0 and 1.0 (or 0.0 to 1.0 for normalized positive embeddings).
/// Returns 0.0 if vectors are empty, have mismatched lengths, or have zero magnitude.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-8 {
        0.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

/// Abstract contract for generating dense vector representations of text.
pub trait EmbeddingProvider: Send + Sync {
    /// Compute vector embedding for a single text input.
    fn embed_text(&self, text: &str) -> Result<Vec<f32>>;

    /// Compute embeddings for a batch of text inputs.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        texts.iter().map(|t| self.embed_text(t)).collect()
    }

    /// Dimensionality of the output embedding vector (e.g. 384 for bge-small).
    fn dimension(&self) -> usize;

    /// Provider or model name identifier.
    fn model_name(&self) -> &str;
}

/// Query parameters for searching memories using vector similarity.
#[derive(Debug, Clone)]
pub struct VectorMemoryQuery {
    /// Optional querying actor ID to enforce strict Actor Privacy Isolation (Skill 12 / 25).
    pub actor_id: Option<String>,
    /// Dense query embedding vector.
    pub query_vector: Vec<f32>,
    /// Maximum number of memory items to return.
    pub top_k: usize,
    /// Minimum cosine similarity threshold (0.0 to 1.0).
    pub min_similarity: f32,
}

impl VectorMemoryQuery {
    /// Create a new query with query vector and top_k limit.
    pub fn new(query_vector: Vec<f32>, top_k: usize) -> Self {
        Self {
            actor_id: None,
            query_vector,
            top_k,
            min_similarity: 0.0,
        }
    }

    /// Set caller actor identity for privacy isolation.
    pub fn with_actor_id(mut self, actor_id: impl Into<String>) -> Self {
        self.actor_id = Some(actor_id.into());
        self
    }

    /// Set minimum similarity threshold.
    pub fn with_min_similarity(mut self, min_sim: f32) -> Self {
        self.min_similarity = min_sim.clamp(0.0, 1.0);
        self
    }
}

/// Deterministic, zero-dependency mock embedding provider for tests and offline environments.
pub struct MockEmbeddingProvider {
    dimension: usize,
    model_name: String,
}

impl MockEmbeddingProvider {
    /// Create a mock provider with specified dimensionality (default: 384).
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            model_name: "mock-embedding-384".to_string(),
        }
    }

    /// Default 384-dimensional mock provider (matches BGE-small dimension).
    pub fn default_384() -> Self {
        Self::new(384)
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::default_384()
    }
}

impl EmbeddingProvider for MockEmbeddingProvider {
    fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let mut vector = vec![0.0f32; self.dimension];
        if text.trim().is_empty() {
            return Ok(vector);
        }

        let tokens: Vec<&str> = text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .collect();

        for token in &tokens {
            let tok_lower = token.to_lowercase();
            let mut h: usize = 5381;
            for &b in tok_lower.as_bytes() {
                h = h.wrapping_mul(33).wrapping_add(b as usize);
            }
            let slot = h % self.dimension;
            vector[slot] += 1.0;
        }

        // Add bigrams for phrase awareness
        for window in tokens.windows(2) {
            let bigram = format!("{}_{}", window[0].to_lowercase(), window[1].to_lowercase());
            let mut h: usize = 5381;
            for &b in bigram.as_bytes() {
                h = h.wrapping_mul(33).wrapping_add(b as usize);
            }
            let slot = h % self.dimension;
            vector[slot] += 0.5;
        }

        // L2-normalize vector
        let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-8 {
            for x in vector.iter_mut() {
                *x /= norm;
            }
        }

        Ok(vector)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}

#[cfg(feature = "local-fastembed")]
mod fastembed_impl {
    use super::*;
    use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
    use parking_lot::Mutex;

    /// Local CPU-based dense vector embedding provider powered by FastEmbed and ONNX Runtime.
    ///
    /// Consumes 0 MB GPU VRAM; runs strictly on host CPU (12 cores / 16 threads).
    pub struct FastembedProvider {
        model: Mutex<TextEmbedding>,
        dimension: usize,
        model_name: String,
    }

    impl FastembedProvider {
        /// Initialize Fastembed with default `BGESmallENV15` (384 dimensions, ~130MB on CPU).
        pub fn new() -> Result<Self> {
            Self::with_model(EmbeddingModel::BGESmallENV15)
        }

        /// Initialize with a specific Fastembed embedding model.
        pub fn with_model(model: EmbeddingModel) -> Result<Self> {
            let init_options = InitOptions::new(model).map_err(|e| {
                CoreError::StorageError(format!("Failed to initialize FastEmbed options: {}", e))
            })?;

            let text_embedding = TextEmbedding::try_new(init_options).map_err(|e| {
                CoreError::StorageError(format!("Failed to load FastEmbed model: {}", e))
            })?;

            Ok(Self {
                model: Mutex::new(text_embedding),
                dimension: 384,
                model_name: "fastembed-bge-small-en-v1.5".to_string(),
            })
        }
    }

    impl EmbeddingProvider for FastembedProvider {
        fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
            let mut model = self.model.lock();
            let embeddings = model.embed(vec![text], None).map_err(|e| {
                CoreError::StorageError(format!("FastEmbed inference failed: {}", e))
            })?;

            embeddings.into_iter().next().ok_or_else(|| {
                CoreError::StorageError("FastEmbed returned empty embeddings".into())
            })
        }

        fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            let mut model = self.model.lock();
            let embeddings = model.embed(texts.to_vec(), None).map_err(|e| {
                CoreError::StorageError(format!("FastEmbed batch inference failed: {}", e))
            })?;
            Ok(embeddings)
        }

        fn dimension(&self) -> usize {
            self.dimension
        }

        fn model_name(&self) -> &str {
            &self.model_name
        }
    }
}

#[cfg(feature = "local-fastembed")]
pub use fastembed_impl::FastembedProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_properties() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];
        let v4 = vec![-1.0, 0.0, 0.0];

        // Identical vectors -> 1.0
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 1e-6);

        // Orthogonal vectors -> 0.0
        assert!(cosine_similarity(&v1, &v3).abs() < 1e-6);

        // Opposite vectors -> -1.0
        assert!((cosine_similarity(&v1, &v4) - (-1.0)).abs() < 1e-6);

        // Empty or mismatched
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
        assert_eq!(cosine_similarity(&v1, &[]), 0.0);
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 2.0]), 0.0);
    }

    #[test]
    fn test_mock_embedding_provider() {
        let provider = MockEmbeddingProvider::default_384();
        assert_eq!(provider.dimension(), 384);
        assert_eq!(provider.model_name(), "mock-embedding-384");

        let emb1 = provider.embed_text("Xin chao").unwrap();
        let emb2 = provider.embed_text("Xin chao").unwrap();
        let emb3 = provider.embed_text("Tam biet").unwrap();

        assert_eq!(emb1.len(), 384);
        // Deterministic: identical text produces identical embedding
        assert_eq!(emb1, emb2);

        // Different text produces different embedding
        assert_ne!(emb1, emb3);

        // Similarity of identical text is 1.0
        let sim_same = cosine_similarity(&emb1, &emb2);
        assert!((sim_same - 1.0).abs() < 1e-5);

        // Batch embedding matches individual embedding
        let batch = provider.embed_batch(&["Xin chao", "Tam biet"]).unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0], emb1);
        assert_eq!(batch[1], emb3);
    }

    #[test]
    fn test_vector_memory_query_builder() {
        let query_vec = vec![0.1; 384];
        let query = VectorMemoryQuery::new(query_vec.clone(), 5)
            .with_actor_id("user-123")
            .with_min_similarity(0.75);

        assert_eq!(query.actor_id.as_deref(), Some("user-123"));
        assert_eq!(query.top_k, 5);
        assert_eq!(query.min_similarity, 0.75);
        assert_eq!(query.query_vector.len(), 384);
    }
}
