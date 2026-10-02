//! World Model: External environment, time of day, active applications, and user presence.
//!
//! Maintains an environmental awareness layer parallel to the character's internal `CharacterState`.

use serde::{Deserialize, Serialize};

/// Type of activity the user is currently engaged in based on the active window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityType {
    Coding,
    Gaming,
    Browsing,
    Media,
    Document,
    System,
    Idle,
    Unknown,
}

impl ActivityType {
    /// Infer activity type from process name or window title.
    pub fn infer_from(title: &str, process: &str) -> Self {
        let lower_t = title.to_lowercase();
        let lower_p = process.to_lowercase();

        if lower_p.contains("code")
            || lower_p.contains("cargo")
            || lower_p.contains("rust")
            || lower_p.contains("idea")
            || lower_p.contains("devenv")
            || lower_t.contains(".rs")
            || lower_t.contains(".tsx")
            || lower_t.contains("visual studio")
        {
            ActivityType::Coding
        } else if lower_p.contains("steam")
            || lower_p.contains("game")
            || lower_p.contains("genshin")
            || lower_p.contains("valorant")
            || lower_p.contains("league")
            || lower_p.contains("minecraft")
            || lower_t.contains("game")
        {
            ActivityType::Gaming
        } else if lower_p.contains("chrome")
            || lower_p.contains("firefox")
            || lower_p.contains("msedge")
            || lower_p.contains("brave")
        {
            if lower_t.contains("youtube")
                || lower_t.contains("netflix")
                || lower_t.contains("spotify")
                || lower_t.contains("twitch")
            {
                ActivityType::Media
            } else {
                ActivityType::Browsing
            }
        } else if lower_p.contains("vlc")
            || lower_p.contains("spotify")
            || lower_p.contains("foobar")
        {
            ActivityType::Media
        } else if lower_p.contains("word")
            || lower_p.contains("excel")
            || lower_p.contains("powerpnt")
            || lower_t.contains(".pdf")
            || lower_t.contains(".docx")
        {
            ActivityType::Document
        } else if lower_p.contains("explorer")
            || lower_p.contains("cmd")
            || lower_p.contains("powershell")
            || lower_p.contains("taskmgr")
        {
            ActivityType::System
        } else if title.is_empty() && process.is_empty() {
            ActivityType::Idle
        } else {
            ActivityType::Unknown
        }
    }
}

/// Metadata describing the currently active application or foreground window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowContext {
    pub title: String,
    pub process_name: String,
    pub activity: ActivityType,
}

impl WindowContext {
    pub fn new(title: impl Into<String>, process_name: impl Into<String>) -> Self {
        let t = title.into();
        let p = process_name.into();
        let act = ActivityType::infer_from(&t, &p);
        Self {
            title: t,
            process_name: p,
            activity: act,
        }
    }
}

/// Broad partition of the day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

impl TimeOfDay {
    pub fn from_hour(hour: u32) -> Self {
        match hour % 24 {
            5..=11 => TimeOfDay::Morning,
            12..=17 => TimeOfDay::Afternoon,
            18..=22 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TimeOfDay::Morning => "Buổi sáng",
            TimeOfDay::Afternoon => "Buổi chiều",
            TimeOfDay::Evening => "Buổi tối",
            TimeOfDay::Night => "Đêm muộn",
        }
    }
}

/// Ambient environment noise level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundLevel {
    Quiet,
    Moderate,
    Loud,
}

/// Ambient context captured from environmental sensing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AmbientContext {
    pub sound_level: SoundLevel,
    pub user_active: bool,
    pub is_fullscreen_app: bool,
}

impl Default for AmbientContext {
    fn default() -> Self {
        Self {
            sound_level: SoundLevel::Quiet,
            user_active: true,
            is_fullscreen_app: false,
        }
    }
}

/// Represents the holistic model of the external world surrounding the character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub active_window: Option<WindowContext>,
    pub time_of_day: TimeOfDay,
    pub ambient: AmbientContext,
    pub screen_summary: Option<String>,
    pub last_observation_timestamp: u64,
    pub last_speech_timestamp: u64,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            active_window: None,
            time_of_day: TimeOfDay::Morning,
            ambient: AmbientContext::default(),
            screen_summary: None,
            last_observation_timestamp: 0,
            last_speech_timestamp: 0,
        }
    }
}

impl WorldState {
    /// Update the current active window context.
    pub fn update_window(
        &mut self,
        title: impl Into<String>,
        process_name: impl Into<String>,
        timestamp: u64,
    ) {
        let win = WindowContext::new(title, process_name);
        self.active_window = Some(win);
        self.last_observation_timestamp = timestamp;
    }

    /// Update the latest screen observation summary from Vision perception.
    pub fn update_screen_summary(&mut self, summary: impl Into<String>, timestamp: u64) {
        self.screen_summary = Some(summary.into());
        self.last_observation_timestamp = timestamp;
    }

    /// Record an utterance by the character to reset the autonomous speech cooldown.
    pub fn record_speech(&mut self, timestamp: u64) {
        self.last_speech_timestamp = timestamp;
    }

    /// Check if the user appears deeply immersed or in a high-focus state.
    pub fn is_user_busy(&self) -> bool {
        if let Some(ref win) = self.active_window {
            match win.activity {
                ActivityType::Coding | ActivityType::Gaming => true,
                _ => self.ambient.is_fullscreen_app,
            }
        } else {
            false
        }
    }

    /// Check if current time is late night quiet hours (11 PM - 6 AM).
    pub fn is_quiet_hours(&self) -> bool {
        self.time_of_day == TimeOfDay::Night
    }

    /// Generate a compact descriptive string for prompt injection in `ContextBuilder`.
    pub fn context_description(&self) -> String {
        let mut desc = format!("Thời điểm: {}.", self.time_of_day.description());
        if let Some(ref win) = self.active_window {
            desc.push_str(&format!(
                " Cửa sổ đang mở: '{}' (Tiến trình: {}, Hoạt động: {:?}).",
                win.title, win.process_name, win.activity
            ));
        }
        if let Some(ref s) = self.screen_summary {
            desc.push_str(&format!(" Quan sát màn hình: {}.", s));
        }
        desc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_inference() {
        assert_eq!(
            ActivityType::infer_from("src/main.rs - Visual Studio Code", "Code.exe"),
            ActivityType::Coding
        );
        assert_eq!(
            ActivityType::infer_from("Genshin Impact", "GenshinImpact.exe"),
            ActivityType::Gaming
        );
        assert_eq!(
            ActivityType::infer_from("Aria Lofi Chill - YouTube", "msedge.exe"),
            ActivityType::Media
        );
        assert_eq!(
            ActivityType::infer_from("Wikipedia - Virtual Character", "chrome.exe"),
            ActivityType::Browsing
        );
        assert_eq!(
            ActivityType::infer_from("Báo cáo tháng 9.docx", "WINWORD.EXE"),
            ActivityType::Document
        );
    }

    #[test]
    fn test_time_of_day_partitions() {
        assert_eq!(TimeOfDay::from_hour(7), TimeOfDay::Morning);
        assert_eq!(TimeOfDay::from_hour(14), TimeOfDay::Afternoon);
        assert_eq!(TimeOfDay::from_hour(20), TimeOfDay::Evening);
        assert_eq!(TimeOfDay::from_hour(2), TimeOfDay::Night);
        assert_eq!(TimeOfDay::from_hour(23), TimeOfDay::Night);
    }

    #[test]
    fn test_world_state_context_description_and_busy_logic() {
        let mut world = WorldState::default();
        world.time_of_day = TimeOfDay::Evening;
        world.update_window("crates/vc-runtime/src/lib.rs", "Code.exe", 1000);
        world.update_screen_summary("Người dùng đang biên dịch test suite", 1005);

        assert!(world.is_user_busy());
        assert!(!world.is_quiet_hours());

        let desc = world.context_description();
        assert!(desc.contains("Buổi tối"));
        assert!(desc.contains("Code.exe"));
        assert!(desc.contains("Người dùng đang biên dịch test suite"));

        world.record_speech(1010);
        assert_eq!(world.last_speech_timestamp, 1010);
    }

    #[test]
    fn test_world_state_serialization_roundtrip() {
        let mut world = WorldState::default();
        world.update_window("Discord", "Discord.exe", 500);
        world.ambient.sound_level = SoundLevel::Moderate;

        let json = serde_json::to_string(&world).unwrap();
        let decoded: WorldState = serde_json::from_str(&json).unwrap();

        assert_eq!(world, decoded);
    }
}
