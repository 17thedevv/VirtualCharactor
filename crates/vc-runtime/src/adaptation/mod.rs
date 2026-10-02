use serde::{Deserialize, Serialize};
use uuid::Uuid;
use vc_core::character::CharacterId;
use vc_core::personality::Personality;

use crate::interaction::InteractionId;

/// User rating given to an interaction response.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeedbackRating {
    Upvote,
    Downvote,
    Custom(f32),
}

impl FeedbackRating {
    pub fn score(&self) -> f32 {
        match self {
            Self::Upvote => 1.0,
            Self::Downvote => -1.0,
            Self::Custom(val) => val.clamp(-1.0, 1.0),
        }
    }
}

/// A structured feedback record captured from user interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionFeedback {
    pub id: String,
    pub interaction_id: InteractionId,
    pub character_id: CharacterId,
    pub actor_id: String,
    pub prompt: String,
    pub response: String,
    pub rating: FeedbackRating,
    pub tags: Vec<String>,
    pub timestamp: u64,
}

impl InteractionFeedback {
    pub fn new(
        interaction_id: InteractionId,
        character_id: CharacterId,
        actor_id: impl Into<String>,
        prompt: impl Into<String>,
        response: impl Into<String>,
        rating: FeedbackRating,
        now: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            interaction_id,
            character_id,
            actor_id: actor_id.into(),
            prompt: prompt.into(),
            response: response.into(),
            rating,
            tags: Vec::new(),
            timestamp: now,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Offline preference dataset container.
///
/// Under Skill 26 (Learning Engineering): Collects offline alignment data without running costly training in-memory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreferenceDataset {
    pub records: Vec<InteractionFeedback>,
}

impl PreferenceDataset {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn add_record(&mut self, record: InteractionFeedback) {
        self.records.push(record);
    }

    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    pub fn satisfaction_ratio(&self) -> f32 {
        if self.records.is_empty() {
            return 1.0;
        }
        let positive = self
            .records
            .iter()
            .filter(|r| r.rating.score() > 0.0)
            .count();
        positive as f32 / self.records.len() as f32
    }

    /// Export records formatted as JSON Lines for offline fine-tuning or analysis.
    pub fn export_jsonl(&self) -> String {
        self.records
            .iter()
            .filter_map(|r| serde_json::to_string(r).ok())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

/// Summary report of suggested adjustments from offline interaction analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationReport {
    pub sample_size: usize,
    pub satisfaction_score: f32,
    pub suggested_verbosity_delta: f32,
    pub suggested_initiative_delta: f32,
    pub top_complaints: Vec<String>,
}

/// Engine that analyzes offline interaction history to suggest personality parameter shifts.
pub struct OfflineAdaptationEngine;

impl OfflineAdaptationEngine {
    /// Analyze a preference dataset and produce personality tuning recommendations.
    pub fn analyze(dataset: &PreferenceDataset) -> AdaptationReport {
        let sample_size = dataset.total_count();
        if sample_size == 0 {
            return AdaptationReport {
                sample_size: 0,
                satisfaction_score: 1.0,
                suggested_verbosity_delta: 0.0,
                suggested_initiative_delta: 0.0,
                top_complaints: Vec::new(),
            };
        }

        let satisfaction = dataset.satisfaction_ratio();
        let mut too_long_count = 0;
        let mut too_short_count = 0;
        let mut cold_count = 0;

        for r in &dataset.records {
            for tag in &r.tags {
                match tag.as_str() {
                    "too_long" | "verbose" => too_long_count += 1,
                    "too_short" | "brief" => too_short_count += 1,
                    "cold" | "distant" => cold_count += 1,
                    _ => {}
                }
            }
        }

        let mut verbosity_delta: f32 = 0.0;
        if too_long_count > too_short_count {
            verbosity_delta = -0.1;
        } else if too_short_count > too_long_count {
            verbosity_delta = 0.1;
        }

        let mut initiative_delta: f32 = 0.0;
        if satisfaction >= 0.8 {
            initiative_delta = 0.05; // User enjoys character, can be slightly more proactive
        } else if satisfaction < 0.5 {
            initiative_delta = -0.05; // User less satisfied, be more restrained
        }

        let mut complaints = Vec::new();
        if too_long_count > 0 {
            complaints.push(format!("Phản hồi quá dài ({})", too_long_count));
        }
        if cold_count > 0 {
            complaints.push(format!("Thiếu ấm áp ({})", cold_count));
        }

        AdaptationReport {
            sample_size,
            satisfaction_score: satisfaction,
            suggested_verbosity_delta: verbosity_delta,
            suggested_initiative_delta: initiative_delta,
            top_complaints: complaints,
        }
    }

    /// Apply adaptation recommendations safely to a personality's behavioral baseline.
    pub fn apply_to_personality(personality: &mut Personality, report: &AdaptationReport) {
        if report.suggested_initiative_delta > 0.0 {
            let current = personality.traits.playfulness.value();
            personality.traits.playfulness = vc_core::personality::TraitScore::clamped(
                current + report.suggested_initiative_delta,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_dataset_and_adaptation_analysis() {
        let mut dataset = PreferenceDataset::new();
        let char_id = CharacterId::new();

        // 3 positive interactions
        for i in 0..3 {
            let fb = InteractionFeedback::new(
                InteractionId::new(),
                char_id,
                "user-1",
                format!("Prompt {}", i),
                format!("Response {}", i),
                FeedbackRating::Upvote,
                1000 + i,
            );
            dataset.add_record(fb);
        }

        // 1 negative interaction with "too_long"
        let neg_fb = InteractionFeedback::new(
            InteractionId::new(),
            char_id,
            "user-1",
            "Prompt 4",
            "Response 4 is way too long",
            FeedbackRating::Downvote,
            1005,
        )
        .with_tag("too_long");
        dataset.add_record(neg_fb);

        assert_eq!(dataset.total_count(), 4);
        assert_eq!(dataset.satisfaction_ratio(), 0.75);

        let report = OfflineAdaptationEngine::analyze(&dataset);
        assert_eq!(report.sample_size, 4);
        assert_eq!(report.satisfaction_score, 0.75);
        assert_eq!(report.suggested_verbosity_delta, -0.1);
        assert_eq!(report.top_complaints.len(), 1);

        // JSONL export
        let jsonl = dataset.export_jsonl();
        assert_eq!(jsonl.lines().count(), 4);
    }
}
