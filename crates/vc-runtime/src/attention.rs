//! Attention Engine: Evaluates salience, curiosity, urgency, and cooldown penalties.
//!
//! Governs autonomous character life, preventing chatter spurt and ensuring non-intrusive awareness.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use vc_core::personality::Personality;
use vc_core::state::world::WorldState;

/// Environmental or interactive events that can capture the character's attention.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttentionEvent {
    /// Significant screen change detected by Level 1 Sensing.
    ScreenVisualDiff {
        diff_percentage: f32,
        window_title: String,
        process_name: String,
    },
    /// The user switched active window or application.
    ActiveWindowChanged {
        old_title: String,
        new_title: String,
        new_process: String,
    },
    /// User addressed the character directly via voice or text.
    UserSpeechInput {
        text: String,
        is_direct_mention: bool,
    },
    /// Operating system notification, error, or completion alert.
    SystemAlert { message: String, is_error: bool },
    /// Idle periodic heartbeat check.
    IdleTick { idle_duration_secs: u64 },
}

/// Configuration parameters for the Attention Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionConfig {
    /// Score threshold required to trigger an autonomous verbal comment (default: 0.65).
    pub speech_threshold: f32,
    /// Score threshold required to trigger On-Demand Level 3 Vision (default: 0.45).
    pub vlm_threshold: f32,
    /// Minimum time in seconds between unprompted autonomous speeches (default: 30s).
    pub min_speech_cooldown_secs: u64,
    /// Penalty applied when user is engaged in high-focus activity (default: 0.35).
    pub busy_user_penalty: f32,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            speech_threshold: 0.65,
            vlm_threshold: 0.45,
            min_speech_cooldown_secs: 30,
            busy_user_penalty: 0.35,
        }
    }
}

/// Multi-factor score computed by the Attention Engine.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttentionScore {
    pub salience: f32,
    pub curiosity: f32,
    pub urgency: f32,
    pub cooldown_penalty: f32,
    pub busy_penalty: f32,
    pub total: f32,
}

/// Final evaluation output deciding whether to trigger speech or vision understanding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttentionEvaluation {
    pub should_speak: bool,
    pub should_capture_vlm: bool,
    pub score: AttentionScore,
    pub reason: String,
}

/// Core Attention Engine implementing the cognitive filtering loop.
pub struct AttentionEngine {
    config: AttentionConfig,
}

impl Default for AttentionEngine {
    fn default() -> Self {
        Self::new(AttentionConfig::default())
    }
}

impl AttentionEngine {
    pub fn new(config: AttentionConfig) -> Self {
        Self { config }
    }

    /// Evaluate an event against current world state and personality baseline.
    pub fn evaluate(
        &self,
        event: &AttentionEvent,
        world: &WorldState,
        personality: &Personality,
        current_time_secs: u64,
    ) -> AttentionEvaluation {
        let (salience, urgency, is_direct, is_visual_event, reason_context) = match event {
            AttentionEvent::ScreenVisualDiff {
                diff_percentage,
                window_title,
                ..
            } => {
                if *diff_percentage >= 0.15 {
                    (
                        (diff_percentage * 1.5).min(0.9),
                        0.0,
                        false,
                        true,
                        format!(
                            "Màn hình thay đổi {:.1}% ở '{}'",
                            diff_percentage * 100.0,
                            window_title
                        ),
                    )
                } else {
                    (
                        0.05,
                        0.0,
                        false,
                        true,
                        "Thay đổi màn hình không đáng kể (<15%)".to_string(),
                    )
                }
            }
            AttentionEvent::ActiveWindowChanged {
                new_title,
                new_process,
                ..
            } => (
                0.55,
                0.0,
                false,
                true,
                format!("Chuyển cửa sổ sang '{}' ({})", new_title, new_process),
            ),
            AttentionEvent::UserSpeechInput {
                is_direct_mention,
                text,
            } => {
                let direct = *is_direct_mention;
                (
                    if direct { 1.0 } else { 0.75 },
                    if direct { 1.0 } else { 0.6 },
                    direct,
                    false,
                    format!("Người dùng nói: '{}'", text),
                )
            }
            AttentionEvent::SystemAlert { message, is_error } => (
                if *is_error { 0.95 } else { 0.6 },
                if *is_error { 0.9 } else { 0.4 },
                false,
                false,
                format!("Thông báo hệ thống: {}", message),
            ),
            AttentionEvent::IdleTick { idle_duration_secs } => {
                let s = if *idle_duration_secs > 300 {
                    ((*idle_duration_secs - 300) as f32 / 600.0).min(0.45)
                } else {
                    0.05
                };
                let r = if *idle_duration_secs > 300 {
                    format!("Nhàn rỗi trong {} giây", idle_duration_secs)
                } else {
                    "Nhịp đập nhàn rỗi bình thường".to_string()
                };
                (s, 0.0, false, false, r)
            }
        };

        // Curiosity contribution from personality traits
        let curiosity_weight = personality.traits.curiosity.value() * 0.3;

        // Cooldown calculation:
        // Direct mentions bypass cooldown completely.
        let mut cooldown_penalty = 0.0;
        if !is_direct && world.last_speech_timestamp > 0 {
            let elapsed = current_time_secs.saturating_sub(world.last_speech_timestamp);
            if elapsed < self.config.min_speech_cooldown_secs {
                // Within cooldown window: maximum penalty to prevent chatter spurt
                cooldown_penalty = 1.0;
            } else if elapsed < self.config.min_speech_cooldown_secs * 2 {
                // Partial decaying penalty
                cooldown_penalty = 0.35;
            }
        }

        // Busy user penalty:
        // Do not interrupt user if user is coding/gaming, unless it is a direct mention or critical error.
        let mut busy_penalty = 0.0;
        if !is_direct && urgency < 0.8 && world.is_user_busy() {
            busy_penalty = self.config.busy_user_penalty;
        }

        // Quiet hours penalty (late night):
        if !is_direct && world.is_quiet_hours() {
            busy_penalty += 0.25;
        }

        let raw_total =
            salience + curiosity_weight + (urgency * 0.4) - cooldown_penalty - busy_penalty;
        let total = raw_total.clamp(0.0, 1.0);

        let score = AttentionScore {
            salience,
            curiosity: curiosity_weight,
            urgency,
            cooldown_penalty,
            busy_penalty,
            total,
        };

        // Decision thresholds
        let should_speak =
            is_direct || (total >= self.config.speech_threshold && cooldown_penalty < 0.9);
        let should_capture_vlm = is_visual_event && (total >= self.config.vlm_threshold);

        let reason = if should_speak {
            format!("Chú ý cao ({:.2}): {}", total, reason_context)
        } else if should_capture_vlm {
            format!("Quan sát VLM ({:.2}): {}", total, reason_context)
        } else {
            format!("Im lặng quan sát ({:.2}): {}", total, reason_context)
        };

        AttentionEvaluation {
            should_speak,
            should_capture_vlm,
            score,
            reason,
        }
    }
}

/// Configuration for the Temporal Attention Gate (P0-F).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalGateConfig {
    /// Window duration in milliseconds (default: 2000ms = 2s).
    pub window_duration_ms: u64,

    /// Minimum number of accumulated screen change events in the window to trigger gate (default: 3).
    pub minimum_events: usize,

    /// Aggregate salience threshold required to wake up On-Demand Vision (default: 0.65).
    pub salience_threshold: f32,

    /// Minimum cooldown in milliseconds between Vision VLM triggers (default: 30000ms = 30s).
    pub cooldown_ms: u64,
}

impl Default for TemporalGateConfig {
    fn default() -> Self {
        Self {
            window_duration_ms: 2000,
            minimum_events: 3,
            salience_threshold: 0.65,
            cooldown_ms: 30000,
        }
    }
}

/// A recorded frame diff event in the temporal sliding window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualTemporalEvent {
    pub timestamp_ms: u64,
    pub diff_percentage: f32,
    pub window_title: String,
    pub process_name: String,
    pub is_app_switched: bool,
}

/// Decision output of the Temporal Attention Gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemporalGateDecision {
    /// Jitter or isolated frame: still accumulating events.
    Accumulating {
        current_events: usize,
        required_events: usize,
    },

    /// Enough events accumulated, but in cooldown period: no Vision.
    InCooldown { remaining_cooldown_ms: u64 },

    /// Enough events, but aggregate salience score did not meet threshold.
    InsufficientSalience {
        salience_score: f32,
        required_threshold: f32,
    },

    /// Gate open! Significant, continuous, salient visual change detected -> Wake up On-Demand Vision.
    TriggerVision {
        salience_score: f32,
        event_count: usize,
        summary: String,
    },
}

/// Sliding window temporal gate preventing accidental VLM wakeups on single-frame noise.
pub struct TemporalAttentionGate {
    config: TemporalGateConfig,
    events_buffer: VecDeque<VisualTemporalEvent>,
    last_triggered_at_ms: Option<u64>,
}

impl Default for TemporalAttentionGate {
    fn default() -> Self {
        Self::new(TemporalGateConfig::default())
    }
}

impl TemporalAttentionGate {
    pub fn new(config: TemporalGateConfig) -> Self {
        Self {
            config,
            events_buffer: VecDeque::new(),
            last_triggered_at_ms: None,
        }
    }

    /// Feed a newly sensed frame change into the temporal window.
    pub fn push_event(
        &mut self,
        now_ms: u64,
        diff_percentage: f32,
        window_title: impl Into<String>,
        process_name: impl Into<String>,
        is_app_switched: bool,
    ) -> TemporalGateDecision {
        // 1. Purge expired events outside the sliding window
        let cutoff = now_ms.saturating_sub(self.config.window_duration_ms);
        while let Some(front) = self.events_buffer.front() {
            if front.timestamp_ms < cutoff {
                self.events_buffer.pop_front();
            } else {
                break;
            }
        }

        // 2. Add current event
        self.events_buffer.push_back(VisualTemporalEvent {
            timestamp_ms: now_ms,
            diff_percentage,
            window_title: window_title.into(),
            process_name: process_name.into(),
            is_app_switched,
        });

        // 3. Check event accumulation count
        if self.events_buffer.len() < self.config.minimum_events {
            return TemporalGateDecision::Accumulating {
                current_events: self.events_buffer.len(),
                required_events: self.config.minimum_events,
            };
        }

        // 4. Check cooldown if a previous trigger occurred
        if let Some(last_trig) = self.last_triggered_at_ms {
            let elapsed_cooldown = now_ms.saturating_sub(last_trig);
            if elapsed_cooldown < self.config.cooldown_ms {
                return TemporalGateDecision::InCooldown {
                    remaining_cooldown_ms: self.config.cooldown_ms - elapsed_cooldown,
                };
            }
        }

        // 5. Calculate aggregated salience score
        // Salience = (AvgDiff * 0.40) + (AppSwitchedBonus * 0.35) + (EventDensity * 0.25)
        let count = self.events_buffer.len() as f32;
        let avg_diff: f32 = self
            .events_buffer
            .iter()
            .map(|e| e.diff_percentage)
            .sum::<f32>()
            / count;
        let has_app_switch = self.events_buffer.iter().any(|e| e.is_app_switched);
        let app_switch_score = if has_app_switch { 1.0 } else { 0.0 };
        let density_score = (count / (self.config.minimum_events as f32 * 2.0)).min(1.0);

        let salience_score = (avg_diff * 0.40) + (app_switch_score * 0.35) + (density_score * 0.25);

        if salience_score < self.config.salience_threshold {
            return TemporalGateDecision::InsufficientSalience {
                salience_score,
                required_threshold: self.config.salience_threshold,
            };
        }

        // 6. Gate opened!
        self.last_triggered_at_ms = Some(now_ms);
        let latest_title = self
            .events_buffer
            .back()
            .map(|e| e.window_title.clone())
            .unwrap_or_default();
        self.events_buffer.clear();

        TemporalGateDecision::TriggerVision {
            salience_score,
            event_count: count as usize,
            summary: format!(
                "Temporal salience triggered ({:.2}) on '{}'",
                salience_score, latest_title
            ),
        }
    }

    pub fn reset(&mut self) {
        self.events_buffer.clear();
        self.last_triggered_at_ms = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_diff_is_ignored() {
        let engine = AttentionEngine::default();
        let world = WorldState::default();
        let personality = Personality::baseline_aria();

        // 5% visual diff -> small irrelevant change
        let event = AttentionEvent::ScreenVisualDiff {
            diff_percentage: 0.05,
            window_title: "VS Code".into(),
            process_name: "Code.exe".into(),
        };

        let eval = engine.evaluate(&event, &world, &personality, 100);
        assert!(!eval.should_speak);
        assert!(!eval.should_capture_vlm);
        assert!(eval.score.salience < 0.1);
    }

    #[test]
    fn test_significant_diff_triggers_vlm_capture() {
        let engine = AttentionEngine::default();
        let world = WorldState::default();
        let personality = Personality::baseline_aria();

        // 35% visual diff -> game state change or new dialog
        let event = AttentionEvent::ScreenVisualDiff {
            diff_percentage: 0.35,
            window_title: "Minecraft".into(),
            process_name: "javaw.exe".into(),
        };

        let eval = engine.evaluate(&event, &world, &personality, 100);
        assert!(eval.should_capture_vlm);
        assert!(eval.score.salience > 0.5);
    }

    #[test]
    fn test_cooldown_penalty_prevents_chatter_spurt() {
        let engine = AttentionEngine::default();
        let mut world = WorldState::default();
        let personality = Personality::baseline_aria();

        // Spoke at timestamp 100
        world.record_speech(100);

        // Another event arrives at 110s (only 10s elapsed, within 30s cooldown)
        let event = AttentionEvent::ScreenVisualDiff {
            diff_percentage: 0.45,
            window_title: "Browser".into(),
            process_name: "chrome.exe".into(),
        };

        let eval = engine.evaluate(&event, &world, &personality, 110);
        assert!(!eval.should_speak, "Must not speak during cooldown!");
        assert_eq!(eval.score.cooldown_penalty, 1.0);
    }

    #[test]
    fn test_direct_mention_bypasses_cooldown_and_busy_state() {
        let engine = AttentionEngine::default();
        let mut world = WorldState::default();
        world.update_window("Visual Studio Code", "Code.exe", 100);
        world.record_speech(100); // just spoke
        let personality = Personality::baseline_aria();

        // User directly asks Aria something at 105s
        let event = AttentionEvent::UserSpeechInput {
            text: "Aria ơi, em thấy hàm này ổn không?".into(),
            is_direct_mention: true,
        };

        let eval = engine.evaluate(&event, &world, &personality, 105);
        assert!(eval.should_speak, "Direct mention MUST trigger response!");
        assert_eq!(eval.score.cooldown_penalty, 0.0);
    }

    #[test]
    fn test_temporal_gate_single_frame_is_filtered_no_vision() {
        let mut gate = TemporalAttentionGate::new(TemporalGateConfig {
            window_duration_ms: 2000,
            minimum_events: 3,
            salience_threshold: 0.65,
            cooldown_ms: 30000,
        });

        // Frame 1: Even with a huge 80% diff, a single frame must NOT trigger vision!
        let d1 = gate.push_event(1000, 0.80, "Game", "game.exe", false);
        assert!(matches!(
            d1,
            TemporalGateDecision::Accumulating {
                current_events: 1,
                required_events: 3
            }
        ));

        // Frame 2: Still only 2 events
        let d2 = gate.push_event(1200, 0.75, "Game", "game.exe", false);
        assert!(matches!(
            d2,
            TemporalGateDecision::Accumulating {
                current_events: 2,
                required_events: 3
            }
        ));
    }

    #[test]
    fn test_temporal_gate_accumulated_salient_events_triggers_vision() {
        let mut gate = TemporalAttentionGate::new(TemporalGateConfig {
            window_duration_ms: 2000,
            minimum_events: 3,
            salience_threshold: 0.60,
            cooldown_ms: 30000,
        });

        // 3 consecutive frames with significant diff & app switch within 2s window
        gate.push_event(1000, 0.50, "Desktop", "explorer.exe", false);
        gate.push_event(1300, 0.70, "VS Code", "Code.exe", true); // app switch
        let d3 = gate.push_event(1600, 0.60, "VS Code", "Code.exe", false);

        assert!(
            matches!(d3, TemporalGateDecision::TriggerVision { .. }),
            "Accumulated salient events MUST trigger vision!"
        );
    }

    #[test]
    fn test_temporal_gate_cooldown_prevents_repeated_vision() {
        let mut gate = TemporalAttentionGate::new(TemporalGateConfig {
            window_duration_ms: 2000,
            minimum_events: 2,
            salience_threshold: 0.50,
            cooldown_ms: 10000, // 10s cooldown
        });

        // Trigger 1 at t=1000ms
        gate.push_event(500, 0.8, "App", "app.exe", true);
        let d1 = gate.push_event(1000, 0.8, "App", "app.exe", false);
        assert!(matches!(d1, TemporalGateDecision::TriggerVision { .. }));

        // Only 3 seconds later at t=4000ms, another burst of changes arrives
        gate.push_event(3500, 0.9, "App", "app.exe", true);
        let d2 = gate.push_event(4000, 0.9, "App", "app.exe", false);
        assert!(
            matches!(d2, TemporalGateDecision::InCooldown { .. }),
            "Must be in cooldown, NO repeated vision!"
        );
    }
}
