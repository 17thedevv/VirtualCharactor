use crate::events::PlatformChatMessage;
use crate::stream::priority::ChatPriorityEngine;
use serde::{Deserialize, Serialize};

/// Decision emitted by the StreamDirector indicating what the VTuber should do next.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DirectorDecision {
    RespondToChat {
        message: PlatformChatMessage,
        priority_score: f32,
    },
    ReactToScreen {
        description: String,
        urgency: f32,
    },
    SelfInitiatedBanter {
        topic: String,
    },
    WaitIdle,
}

/// Configuration for Livestream Director.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDirectorConfig {
    pub silence_threshold_secs: u64,
    pub min_speech_interval_secs: u64,
    pub max_chat_per_minute: u32,
}

impl Default for StreamDirectorConfig {
    fn default() -> Self {
        Self {
            silence_threshold_secs: 15,
            min_speech_interval_secs: 4,
            max_chat_per_minute: 12,
        }
    }
}

/// Orchestrates live streaming pacing, chat engagement, and visual commentary.
pub struct StreamDirector {
    config: StreamDirectorConfig,
    last_speech_ts: u64,
    speech_count_current_window: u32,
    window_start_ts: u64,
}

impl StreamDirector {
    pub fn new(config: StreamDirectorConfig) -> Self {
        Self {
            config,
            last_speech_ts: 0,
            speech_count_current_window: 0,
            window_start_ts: 0,
        }
    }

    pub fn default_director() -> Self {
        Self::new(StreamDirectorConfig::default())
    }

    /// Evaluate current context and decide next stream action.
    ///
    /// Pacing rules:
    /// 1. Cooldown enforcement: VTuber cannot speak immediately after speaking.
    /// 2. Rate limit: Max chat replies per minute to preserve voice naturalness.
    /// 3. Chat Priority: High-scoring chat items (SuperChats, mentions) take precedence.
    /// 4. Visual Salience: Significant game/screen events trigger reactions.
    /// 5. Silence detection: If idle for > silence_threshold_secs, trigger self-banter.
    pub fn evaluate_turn(
        &mut self,
        priority_engine: &mut ChatPriorityEngine,
        screen_salience: Option<(String, f32)>, // (event_description, score)
        now: u64,
    ) -> DirectorDecision {
        // Reset minute window if passed
        if now.saturating_sub(self.window_start_ts) >= 60 {
            self.window_start_ts = now;
            self.speech_count_current_window = 0;
        }

        // Rule 1: Speech cooldown
        if self.last_speech_ts > 0
            && now.saturating_sub(self.last_speech_ts) < self.config.min_speech_interval_secs
        {
            return DirectorDecision::WaitIdle;
        }

        // Rule 2: Rate limit
        if self.speech_count_current_window >= self.config.max_chat_per_minute {
            return DirectorDecision::WaitIdle;
        }

        // Rule 3: Check highest priority chat message
        if let Some(top_item) = priority_engine.pop_highest_priority() {
            if top_item.priority_score >= 15.0
                || top_item.message.is_superchat
                || top_item.message.is_mention
            {
                self.record_speech(now);
                return DirectorDecision::RespondToChat {
                    message: top_item.message,
                    priority_score: top_item.priority_score,
                };
            }
        }

        // Rule 4: Visual salience event from screen/gameplay
        if let Some((desc, urgency)) = screen_salience {
            if urgency >= 0.65 {
                self.record_speech(now);
                return DirectorDecision::ReactToScreen {
                    description: desc,
                    urgency,
                };
            }
        }

        // Rule 5: Silence break (Self-initiated banter)
        let silence_duration = now.saturating_sub(self.last_speech_ts);
        if silence_duration >= self.config.silence_threshold_secs {
            self.record_speech(now);
            return DirectorDecision::SelfInitiatedBanter {
                topic: "Tâm sự nhẹ cùng phòng stream về không khí hiện tại".to_string(),
            };
        }

        DirectorDecision::WaitIdle
    }

    fn record_speech(&mut self, now: u64) {
        self.last_speech_ts = now;
        self.speech_count_current_window += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{PlatformChatMessage, PlatformType};

    #[test]
    fn test_stream_director_cooldown_and_silence_trigger() {
        let mut director = StreamDirector::default_director();
        let mut priority_engine = ChatPriorityEngine::default_engine();

        // 1. Initial turn at now = 100 with a superchat -> should respond
        let sc = PlatformChatMessage::new(
            PlatformType::YouTube,
            "chan",
            "whale",
            "WhaleFan",
            "Chào Aria!",
            100,
        )
        .with_superchat(20.0, "USD");
        priority_engine.evaluate_and_ingest(sc, 100);

        let dec1 = director.evaluate_turn(&mut priority_engine, None, 100);
        assert!(matches!(dec1, DirectorDecision::RespondToChat { .. }));

        // 2. Immediately at now = 101 (within 4s cooldown) -> should WaitIdle
        let dec2 = director.evaluate_turn(&mut priority_engine, None, 101);
        assert_eq!(dec2, DirectorDecision::WaitIdle);

        // 3. At now = 105 (cooldown passed, but no chat, no screen event) -> WaitIdle
        let dec3 = director.evaluate_turn(&mut priority_engine, None, 105);
        assert_eq!(dec3, DirectorDecision::WaitIdle);

        // 4. At now = 120 (silence exceeds 15s) -> should trigger SelfInitiatedBanter
        let dec4 = director.evaluate_turn(&mut priority_engine, None, 120);
        assert!(matches!(dec4, DirectorDecision::SelfInitiatedBanter { .. }));
    }
}
