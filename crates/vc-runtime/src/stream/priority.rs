use crate::events::PlatformChatMessage;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Priority score breakdown for an incoming livestream chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPriorityItem {
    pub message: PlatformChatMessage,
    pub priority_score: f32,
    pub is_duplicate: bool,
    pub reason: String,
}

/// Configuration for Livestream Chat Priority and Anti-Spam engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPriorityConfig {
    pub dedup_window_secs: u64,
    pub superchat_weight: f32,
    pub mention_bonus: f32,
    pub question_bonus: f32,
    pub min_score_threshold: f32,
}

impl Default for ChatPriorityConfig {
    fn default() -> Self {
        Self {
            dedup_window_secs: 10,
            superchat_weight: 100.0,
            mention_bonus: 35.0,
            question_bonus: 25.0,
            min_score_threshold: 5.0,
        }
    }
}

/// Anti-spam and Priority Engine for Livestream environments.
///
/// Under Skill 15 & 16: Filters spam storms, prioritizes SuperChats and engaging questions.
pub struct ChatPriorityEngine {
    config: ChatPriorityConfig,
    recent_hashes: VecDeque<(u64, u64)>, // (hash, timestamp)
    message_buffer: Vec<ChatPriorityItem>,
}

impl ChatPriorityEngine {
    pub fn new(config: ChatPriorityConfig) -> Self {
        Self {
            config,
            recent_hashes: VecDeque::new(),
            message_buffer: Vec::new(),
        }
    }

    pub fn default_engine() -> Self {
        Self::new(ChatPriorityConfig::default())
    }

    /// Calculate hash for text deduplication.
    fn compute_hash(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.trim().to_lowercase().hash(&mut hasher);
        hasher.finish()
    }

    /// Clean up expired hash records outside the sliding deduplication window.
    fn prune_old_hashes(&mut self, now: u64) {
        let cutoff = now.saturating_sub(self.config.dedup_window_secs);
        while let Some(&(h_ts, _)) = self.recent_hashes.front() {
            if h_ts < cutoff {
                self.recent_hashes.pop_front();
            } else {
                break;
            }
        }
    }

    /// Ingest a chat message and compute its priority score.
    pub fn evaluate_and_ingest(&mut self, msg: PlatformChatMessage, now: u64) -> ChatPriorityItem {
        self.prune_old_hashes(now);

        let h = Self::compute_hash(&msg.content);
        let is_dup = self.recent_hashes.iter().any(|&(_, hash)| hash == h);
        self.recent_hashes.push_back((now, h));

        let mut score: f32 = 10.0; // Base score
        let mut reasons = Vec::new();

        // Superchat / Donation bonus
        if msg.is_superchat {
            let amount = msg.donation_amount.unwrap_or(1.0);
            let sc_bonus = self.config.superchat_weight + (amount as f32 * 0.5).min(200.0);
            score += sc_bonus;
            reasons.push(format!("SuperChat(+{:.0})", sc_bonus));
        }

        // Direct mention bonus
        if msg.is_mention {
            score += self.config.mention_bonus;
            reasons.push("Mention(+35)".into());
        }

        // Question mark or inquiring intent
        let lower = msg.content.to_lowercase();
        if lower.contains('?')
            || lower.contains("sao")
            || lower.contains("thế nào")
            || lower.contains("tại sao")
            || lower.contains("gì vậy")
        {
            score += self.config.question_bonus;
            reasons.push("Question(+25)".into());
        }

        // Content length nuance: very short emotes (1-3 chars) penalised slightly, rich questions rewarded
        let char_count = msg.content.chars().count();
        if char_count <= 3 {
            score = (score - 5.0).max(1.0);
            reasons.push("ShortText(-5)".into());
        } else if char_count >= 15 && char_count <= 120 {
            score += 10.0;
            reasons.push("GoodLength(+10)".into());
        }

        // Duplicate penalty
        if is_dup {
            score = (score * 0.2).min(5.0);
            reasons.push("SpamDuplicate(-80%)".into());
        }

        let item = ChatPriorityItem {
            message: msg,
            priority_score: score,
            is_duplicate: is_dup,
            reason: reasons.join(", "),
        };

        self.message_buffer.push(item.clone());
        item
    }

    /// Select the highest priority message from the buffer and clear it.
    pub fn pop_highest_priority(&mut self) -> Option<ChatPriorityItem> {
        if self.message_buffer.is_empty() {
            return None;
        }

        // Sort descending by priority_score
        self.message_buffer.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Some(self.message_buffer.remove(0))
    }

    /// Aggregate questions in the buffer to group similar topics.
    pub fn aggregate_topics(&self) -> HashMap<String, usize> {
        let mut topic_counts = HashMap::new();
        for item in &self.message_buffer {
            let lower = item.message.content.to_lowercase();
            if lower.contains("game") || lower.contains("chơi") {
                *topic_counts.entry("gaming".to_string()).or_insert(0) += 1;
            } else if lower.contains("ăn") || lower.contains("cơm") || lower.contains("uống") {
                *topic_counts.entry("food_drink".to_string()).or_insert(0) += 1;
            } else if lower.contains("hát") || lower.contains("nhạc") {
                *topic_counts.entry("music_singing".to_string()).or_insert(0) += 1;
            } else if lower.contains("yêu") || lower.contains("thích") || lower.contains("bạn gái")
            {
                *topic_counts.entry("romance_flirt".to_string()).or_insert(0) += 1;
            }
        }
        topic_counts
    }

    /// Current count of pending messages in buffer.
    pub fn pending_count(&self) -> usize {
        self.message_buffer.len()
    }

    /// Clear all pending messages in buffer.
    pub fn clear(&mut self) {
        self.message_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::PlatformType;

    #[test]
    fn test_chat_priority_superchat_ranks_highest() {
        let mut engine = ChatPriorityEngine::default_engine();

        let normal_msg = PlatformChatMessage::new(
            PlatformType::YouTube,
            "stream-1",
            "actor-1",
            "Viewer1",
            "hello",
            1000,
        );
        let sc_msg = PlatformChatMessage::new(
            PlatformType::YouTube,
            "stream-1",
            "actor-2",
            "WhaleFan",
            "Em ăn tối chưa Aria ơi?",
            1000,
        )
        .with_mention(true)
        .with_superchat(50.0, "USD");

        let evaluated_norm = engine.evaluate_and_ingest(normal_msg, 1000);
        let evaluated_sc = engine.evaluate_and_ingest(sc_msg, 1000);

        assert!(evaluated_sc.priority_score > evaluated_norm.priority_score);

        let top = engine
            .pop_highest_priority()
            .expect("should have top message");
        assert_eq!(top.message.author_name, "WhaleFan");
        assert!(top.message.is_superchat);
    }

    #[test]
    fn test_anti_spam_duplicate_penalized() {
        let mut engine = ChatPriorityEngine::default_engine();

        let msg1 = PlatformChatMessage::new(
            PlatformType::Twitch,
            "chan",
            "spammer",
            "SpamUser",
            "Aria nhìn em đi!",
            1000,
        )
        .with_mention(true);

        let msg2 = PlatformChatMessage::new(
            PlatformType::Twitch,
            "chan",
            "spammer",
            "SpamUser",
            "Aria nhìn em đi!",
            1001,
        )
        .with_mention(true);

        let eval1 = engine.evaluate_and_ingest(msg1, 1000);
        let eval2 = engine.evaluate_and_ingest(msg2, 1001);

        assert!(!eval1.is_duplicate);
        assert!(eval2.is_duplicate);
        assert!(eval2.priority_score < eval1.priority_score);
    }
}
