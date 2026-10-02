use crate::events::{CharacterSpeechOutput, PlatformChatMessage, PlatformEventBus, PlatformType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Twitch chat message parser and adapter.
#[derive(Clone)]
pub struct TwitchChatAdapter {
    pub channel: String,
    event_bus: PlatformEventBus,
    outbox_history: Arc<Mutex<Vec<String>>>,
}

impl TwitchChatAdapter {
    pub fn new(channel: impl Into<String>, event_bus: PlatformEventBus) -> Self {
        Self {
            channel: channel.into(),
            event_bus,
            outbox_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Parse an IRC PRIVMSG line from Twitch chat.
    ///
    /// Example format:
    /// `:viewer123!viewer123@viewer123.tmi.twitch.tv PRIVMSG #aria_stream :Aria ơi em đang chơi game gì thế?`
    pub fn parse_irc_line(&self, line: &str, now: u64) -> Option<PlatformChatMessage> {
        let line = line.trim();
        if !line.contains("PRIVMSG") {
            return None;
        }

        // Extract author
        let author = if let Some(stripped) = line.strip_prefix(':') {
            if let Some(idx) = stripped.find('!') {
                &stripped[..idx]
            } else {
                "anonymous"
            }
        } else {
            "anonymous"
        };

        // Extract content after channel
        let parts: Vec<&str> = line.splitn(2, "PRIVMSG").collect();
        if parts.len() < 2 {
            return None;
        }
        let after_privmsg = parts[1].trim();
        let content_idx = after_privmsg.find(':')?;
        let content = after_privmsg[content_idx + 1..].trim();

        if content.is_empty() {
            return None;
        }

        let is_mention = content.to_lowercase().contains("aria")
            || content.contains(&format!("@{}", self.channel));
        let actor_id = format!("twitch:{}", author.to_lowercase());

        let chat_msg = PlatformChatMessage::new(
            PlatformType::Twitch,
            &self.channel,
            actor_id,
            author,
            content,
            now,
        )
        .with_mention(is_mention);

        let _ = self.event_bus.publish_chat(chat_msg.clone());
        Some(chat_msg)
    }

    /// Send reply back to Twitch chat.
    pub fn send_chat_reply(&self, speech: &CharacterSpeechOutput) -> Result<(), String> {
        if speech.platform != PlatformType::Twitch {
            return Err("Not a Twitch speech target".to_string());
        }
        let irc_out = format!("PRIVMSG #{} :{}", self.channel, speech.text);
        if let Ok(mut outbox) = self.outbox_history.lock() {
            outbox.push(irc_out);
        }
        Ok(())
    }

    pub fn get_outbox(&self) -> Vec<String> {
        self.outbox_history
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }
}

/// YouTube Live Chat Adapter.
#[derive(Clone)]
pub struct YouTubeLiveAdapter {
    pub live_chat_id: String,
    event_bus: PlatformEventBus,
    outbox_history: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeSnippet {
    pub author_channel_id: String,
    pub display_name: String,
    pub message_text: String,
    pub is_superchat: bool,
    pub superchat_amount: Option<f64>,
    pub currency: Option<String>,
}

impl YouTubeLiveAdapter {
    pub fn new(live_chat_id: impl Into<String>, event_bus: PlatformEventBus) -> Self {
        Self {
            live_chat_id: live_chat_id.into(),
            event_bus,
            outbox_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Ingest a YouTube Live chat snippet item.
    pub fn process_snippet(&self, snippet: &YouTubeSnippet, now: u64) -> PlatformChatMessage {
        let actor_id = format!("youtube:{}", snippet.author_channel_id);
        let is_mention = snippet.message_text.to_lowercase().contains("aria");

        let mut chat_msg = PlatformChatMessage::new(
            PlatformType::YouTube,
            &self.live_chat_id,
            actor_id,
            &snippet.display_name,
            &snippet.message_text,
            now,
        )
        .with_mention(is_mention);

        if snippet.is_superchat {
            if let Some(amount) = snippet.superchat_amount {
                chat_msg = chat_msg.with_superchat(
                    amount,
                    snippet.currency.clone().unwrap_or_else(|| "USD".into()),
                );
            }
        }

        let _ = self.event_bus.publish_chat(chat_msg.clone());
        chat_msg
    }

    /// Record outgoing chat response.
    pub fn send_chat_reply(&self, speech: &CharacterSpeechOutput) -> Result<(), String> {
        if speech.platform != PlatformType::YouTube {
            return Err("Not a YouTube target".to_string());
        }
        if let Ok(mut outbox) = self.outbox_history.lock() {
            outbox.push(speech.text.clone());
        }
        Ok(())
    }

    pub fn get_outbox(&self) -> Vec<String> {
        self.outbox_history
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_twitch_irc_parsing() {
        let bus = PlatformEventBus::new(16);
        let adapter = TwitchChatAdapter::new("aria_vtuber", bus);

        let line = ":kuro_fan!kuro_fan@kuro_fan.tmi.twitch.tv PRIVMSG #aria_vtuber :Aria ơi giọng em ngọt ngào quá!";
        let parsed = adapter
            .parse_irc_line(line, 1000)
            .expect("Should parse IRC");

        assert_eq!(parsed.platform, PlatformType::Twitch);
        assert_eq!(parsed.author_name, "kuro_fan");
        assert_eq!(parsed.actor_id, "twitch:kuro_fan");
        assert_eq!(parsed.content, "Aria ơi giọng em ngọt ngào quá!");
        assert!(parsed.is_mention);
    }

    #[test]
    fn test_youtube_superchat_ingestion() {
        let bus = PlatformEventBus::new(16);
        let adapter = YouTubeLiveAdapter::new("yt-live-999", bus);

        let snippet = YouTubeSnippet {
            author_channel_id: "UCxyz123".into(),
            display_name: "SuperFan".into(),
            message_text: "Tặng Aria 100k ăn kem nha!".into(),
            is_superchat: true,
            superchat_amount: Some(100_000.0),
            currency: Some("VND".into()),
        };

        let chat = adapter.process_snippet(&snippet, 1005);
        assert_eq!(chat.platform, PlatformType::YouTube);
        assert_eq!(chat.actor_id, "youtube:UCxyz123");
        assert!(chat.is_superchat);
        assert_eq!(chat.donation_amount, Some(100_000.0));
        assert_eq!(chat.currency, Some("VND".into()));
    }
}
