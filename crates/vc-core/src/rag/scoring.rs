//! Multi-Factor Scoring & Ranking for RAG Retrieval (Skill 12: Memory Engineering).
//!
//! Blends Semantic Relevance, Discretized Importance, and Exponential Recency Decay:
//! FinalScore = (Relevance * 0.50) + (Importance * 0.30) + (Recency * 0.20)

use crate::memory::MemoryImportance;
use crate::rag::types::DocumentChunkId;
use std::collections::HashMap;

/// Compute exponential recency retention factor based on half-life in days.
///
/// Decay formula: R(t) = 2^(-delta_t / half_life)
pub fn calculate_recency_retention(created_at: u64, now: u64, half_life_days: f32) -> f32 {
    if now <= created_at {
        return 1.0;
    }
    let elapsed_seconds = (now - created_at) as f32;
    let elapsed_days = elapsed_seconds / 86400.0;
    let half_life = half_life_days.max(0.1);
    2.0_f32.powf(-elapsed_days / half_life).clamp(0.01, 1.0)
}

/// Calculate composite memory / RAG score.
pub fn calculate_composite_score(
    relevance_similarity: f32,
    importance: MemoryImportance,
    created_at: u64,
    now: u64,
    half_life_days: f32,
) -> f32 {
    let sim = relevance_similarity.clamp(0.0, 1.0);
    let imp = importance.weight();
    let rec = calculate_recency_retention(created_at, now, half_life_days);

    // Weights: 50% relevance, 30% importance, 20% recency
    let score = (sim * 0.50) + (imp * 0.30) + (rec * 0.20);
    score.clamp(0.0, 1.0)
}

/// Reciprocal Rank Fusion (RRF) for combining multiple ranking lists (Dense Vector & Sparse Keyword).
///
/// Score formula: RRF(doc) = sum_{list in Lists} 1 / (k + rank(doc, list))
/// Default standard k = 60.
pub fn reciprocal_rank_fusion(
    dense_ranks: &[(DocumentChunkId, usize)],
    sparse_ranks: &[(DocumentChunkId, usize)],
    k: usize,
) -> Vec<(DocumentChunkId, f32)> {
    let mut scores: HashMap<DocumentChunkId, f32> = HashMap::new();
    let k_f32 = k as f32;

    for (id, rank) in dense_ranks {
        let rrf = 1.0 / (k_f32 + *rank as f32);
        *scores.entry(*id).or_insert(0.0) += rrf;
    }

    for (id, rank) in sparse_ranks {
        let rrf = 1.0 / (k_f32 + *rank as f32);
        *scores.entry(*id).or_insert(0.0) += rrf;
    }

    let mut result: Vec<(DocumentChunkId, f32)> = scores.into_iter().collect();
    // Sort descending by RRF score
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recency_retention_decay() {
        let now = 2_000_000;
        // 0 days elapsed
        let r0 = calculate_recency_retention(now, now, 7.0);
        assert_eq!(r0, 1.0);

        // Exactly 7 days elapsed (half life) -> retention ≈ 0.50
        let seven_days = (7.0 * 86400.0) as u64;
        let r_half = calculate_recency_retention(now - seven_days, now, 7.0);
        assert!((r_half - 0.50).abs() < 1e-3);

        // 14 days elapsed (two half lives) -> retention ≈ 0.25
        let fourteen_days = (14.0 * 86400.0) as u64;
        let r_quarter = calculate_recency_retention(now - fourteen_days, now, 7.0);
        assert!((r_quarter - 0.25).abs() < 1e-3);
    }

    #[test]
    fn test_composite_score() {
        let now = 100_000;
        let score = calculate_composite_score(0.9, MemoryImportance::Critical, now, now, 7.0);
        // (0.9 * 0.5) + (1.0 * 0.3) + (1.0 * 0.2) = 0.45 + 0.30 + 0.20 = 0.95
        assert!((score - 0.95).abs() < 1e-3);
    }

    #[test]
    fn test_rrf_merging() {
        let id1 = DocumentChunkId::new();
        let id2 = DocumentChunkId::new();
        let id3 = DocumentChunkId::new();

        // Dense ranking: id1 (1), id2 (2)
        let dense = vec![(id1, 1), (id2, 2)];
        // Sparse ranking: id2 (1), id3 (2)
        let sparse = vec![(id2, 1), (id3, 2)];

        let merged = reciprocal_rank_fusion(&dense, &sparse, 60);

        // id2 appeared in both top ranks, so it must rank #1 in RRF!
        assert_eq!(merged[0].0, id2);
        assert!(merged[0].1 > merged[1].1);
    }
}
