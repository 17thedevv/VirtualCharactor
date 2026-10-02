use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use uuid::Uuid;
use vc_core::character::CharacterId;

/// Supported platforms in the VirtualCharacter multi-platform ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlatformType {
    Web,
    Discord,
    YouTube,
    Twitch,
    Cli,
    System,
}

impl std::fmt::Display for PlatformType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Web => write!(f, "Web"),
            Self::Discord => write!(f, "Discord"),
            Self::YouTube => write!(f, "YouTube"),
            Self::Twitch => write!(f, "Twitch"),
            Self::Cli => write!(f, "Cli"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Unified, normalized representation of an incoming chat message across all platforms.
///
/// Ensures cross-platform consistency while preserving platform-specific provenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformChatMessage {
    pub message_id: String,
    pub platform: PlatformType,
    pub channel_id: String,
    pub actor_id: String,
    pub author_name: String,
    pub content: String,
    pub is_mention: bool,
    pub is_direct_message: bool,
    pub is_superchat: bool,
    pub donation_amount: Option<f64>,
    pub currency: Option<String>,
    pub metadata: HashMap<String, String>,
    pub timestamp: u64,
}

impl PlatformChatMessage {
    pub fn new(
        platform: PlatformType,
        channel_id: impl Into<String>,
        actor_id: impl Into<String>,
        author_name: impl Into<String>,
        content: impl Into<String>,
        now: u64,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4().to_string(),
            platform,
            channel_id: channel_id.into(),
            actor_id: actor_id.into(),
            author_name: author_name.into(),
            content: content.into(),
            is_mention: false,
            is_direct_message: false,
            is_superchat: false,
            donation_amount: None,
            currency: None,
            metadata: HashMap::new(),
            timestamp: now,
        }
    }

    pub fn with_mention(mut self, is_mention: bool) -> Self {
        self.is_mention = is_mention;
        self
    }

    pub fn with_dm(mut self, is_dm: bool) -> Self {
        self.is_direct_message = is_dm;
        self
    }

    pub fn with_superchat(mut self, amount: f64, currency: impl Into<String>) -> Self {
        self.is_superchat = true;
        self.donation_amount = Some(amount);
        self.currency = Some(currency.into());
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), val.into());
        self
    }
}

/// Character speech or reaction output targeted to a platform channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSpeechOutput {
    pub character_id: CharacterId,
    pub target_actor_id: String,
    pub platform: PlatformType,
    pub channel_id: String,
    pub text: String,
    pub emotion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_bytes: Option<Vec<u8>>,
    pub timestamp: u64,
}

/// Universal event envelope passing through the VirtualCharacter runtime bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedPlatformEvent {
    ChatMessage(PlatformChatMessage),
    CharacterSpeech(CharacterSpeechOutput),
    SystemNotification {
        title: String,
        message: String,
        level: String,
        timestamp: u64,
    },
    AttentionTrigger {
        reason: String,
        score: f32,
        timestamp: u64,
    },
}

impl UnifiedPlatformEvent {
    pub fn platform(&self) -> PlatformType {
        match self {
            Self::ChatMessage(msg) => msg.platform,
            Self::CharacterSpeech(spk) => spk.platform,
            Self::SystemNotification { .. } => PlatformType::System,
            Self::AttentionTrigger { .. } => PlatformType::System,
        }
    }
}

/// Asynchronous, high-throughput event bus supporting publish-subscribe architecture.
///
/// Designed under Skill 23 (Event Engineering) to decouple platform I/O from core intelligence loops.
#[derive(Clone)]
pub struct PlatformEventBus {
    sender: broadcast::Sender<UnifiedPlatformEvent>,
}

impl PlatformEventBus {
    /// Initialize a new bus with the given channel capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Default capacity of 512 events.
    pub fn default_bus() -> Self {
        Self::new(512)
    }

    /// Publish an event to all active subscribers.
    pub fn publish(
        &self,
        event: UnifiedPlatformEvent,
    ) -> Result<usize, broadcast::error::SendError<UnifiedPlatformEvent>> {
        self.sender.send(event)
    }

    /// Publish a chat message directly.
    pub fn publish_chat(
        &self,
        msg: PlatformChatMessage,
    ) -> Result<usize, broadcast::error::SendError<UnifiedPlatformEvent>> {
        self.publish(UnifiedPlatformEvent::ChatMessage(msg))
    }

    /// Publish character speech directly.
    pub fn publish_speech(
        &self,
        speech: CharacterSpeechOutput,
    ) -> Result<usize, broadcast::error::SendError<UnifiedPlatformEvent>> {
        self.publish(UnifiedPlatformEvent::CharacterSpeech(speech))
    }

    /// Create a new receiver to listen to all events on the bus.
    pub fn subscribe(&self) -> broadcast::Receiver<UnifiedPlatformEvent> {
        self.sender.subscribe()
    }

    /// Number of active subscriber receivers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for PlatformEventBus {
    fn default() -> Self {
        Self::default_bus()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_pub_sub() {
        let bus = PlatformEventBus::new(32);
        let mut rx = bus.subscribe();

        let msg = PlatformChatMessage::new(
            PlatformType::Discord,
            "chan-123",
            "discord:user-456",
            "Alice",
            "Hello Aria!",
            1000,
        )
        .with_mention(true);

        bus.publish_chat(msg.clone()).expect("send should succeed");

        let received = rx.recv().await.expect("receive should succeed");
        match received {
            UnifiedPlatformEvent::ChatMessage(recv_msg) => {
                assert_eq!(recv_msg.platform, PlatformType::Discord);
                assert_eq!(recv_msg.author_name, "Alice");
                assert_eq!(recv_msg.content, "Hello Aria!");
                assert!(recv_msg.is_mention);
            }
            _ => panic!("Expected ChatMessage"),
        }
    }
}
