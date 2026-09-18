use serde::{Deserialize, Serialize};
use super::metrics::RelationshipMetrics;

/// Developmental stages of a Character ↔ Actor relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipStage {
    /// Initial interaction: No history, cautious politeness.
    Stranger,
    /// Known entity: Basic familiarity established, polite rapport.
    Acquaintance,
    /// Regular positive interactions: Mutual comfort and shared context.
    CasualCompanion,
    /// High trust and emotional closeness: Deeper vulnerability and personal sharing.
    CloseFriend,
    /// Highest bond: Enduring mutual trust, emotional safety and loyalty.
    Confidant,
}

impl RelationshipStage {
    /// Return a human-readable English label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stranger => "Stranger",
            Self::Acquaintance => "Acquaintance",
            Self::CasualCompanion => "Casual Companion",
            Self::CloseFriend => "Close Friend",
            Self::Confidant => "Confidant",
        }
    }

    /// Return a localized Vietnamese label for display.
    pub fn display_vi(&self) -> &'static str {
        match self {
            Self::Stranger => "Người lạ",
            Self::Acquaintance => "Người quen",
            Self::CasualCompanion => "Bạn đồng hành thân quen",
            Self::CloseFriend => "Bạn thân thiết",
            Self::Confidant => "Tri kỷ gắn kết",
        }
    }

    /// Dynamically determine the relationship stage based on current metrics.
    pub fn from_metrics(metrics: &RelationshipMetrics) -> Self {
        let trust = metrics.trust.value();
        let closeness = metrics.closeness.value();
        let familiarity = metrics.familiarity.value();
        let tension = metrics.tension.value();

        // Confidant: Requires high trust, high closeness, high familiarity, low tension
        if trust >= 0.80 && closeness >= 0.75 && familiarity >= 0.70 && tension <= 0.25 {
            Self::Confidant
        }
        // Close Friend: Substantial trust and closeness
        else if trust >= 0.60 && closeness >= 0.55 && familiarity >= 0.45 && tension <= 0.35 {
            Self::CloseFriend
        }
        // Casual Companion: Established familiarity and comfortable trust
        else if familiarity >= 0.35 && trust >= 0.35 {
            Self::CasualCompanion
        }
        // Acquaintance: Past the stranger threshold
        else if familiarity >= 0.10 || trust >= 0.25 {
            Self::Acquaintance
        }
        // Stranger: Baseline
        else {
            Self::Stranger
        }
    }
}

impl Default for RelationshipStage {
    fn default() -> Self {
        Self::Stranger
    }
}

impl std::fmt::Display for RelationshipStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relationship::metrics::RelationshipScore;

    #[test]
    fn test_stage_from_metrics() {
        let mut m = RelationshipMetrics::stranger();
        assert_eq!(RelationshipStage::from_metrics(&m), RelationshipStage::Stranger);

        m.familiarity = RelationshipScore::clamped(0.2);
        assert_eq!(RelationshipStage::from_metrics(&m), RelationshipStage::Acquaintance);

        m.familiarity = RelationshipScore::clamped(0.4);
        m.trust = RelationshipScore::clamped(0.4);
        assert_eq!(RelationshipStage::from_metrics(&m), RelationshipStage::CasualCompanion);

        m.familiarity = RelationshipScore::clamped(0.6);
        m.closeness = RelationshipScore::clamped(0.6);
        m.trust = RelationshipScore::clamped(0.7);
        assert_eq!(RelationshipStage::from_metrics(&m), RelationshipStage::CloseFriend);

        m.familiarity = RelationshipScore::clamped(0.8);
        m.closeness = RelationshipScore::clamped(0.85);
        m.trust = RelationshipScore::clamped(0.9);
        m.tension = RelationshipScore::clamped(0.1);
        assert_eq!(RelationshipStage::from_metrics(&m), RelationshipStage::Confidant);
    }
}
