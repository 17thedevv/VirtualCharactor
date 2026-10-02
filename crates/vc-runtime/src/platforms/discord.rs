use crate::events::{CharacterSpeechOutput, PlatformChatMessage, PlatformEventBus, PlatformType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Discord configuration options for VirtualCharacter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: Option<String>,
    pub bot_user_id: Option<String>,
    pub guild_id: Option<String>,
    pub allowed_channels: Vec<String>,
    pub require_mention: bool,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            bot_token: None,
            bot_user_id: None,
            guild_id: None,
            allowed_channels: Vec::new(),
            require_mention: true,
        }
    }
}

/// Discord user structure representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: Option<String>,
    #[serde(default)]
    pub bot: bool,
}

/// Incoming Discord message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordIncomingMessage {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author: DiscordUser,
    pub content: String,
    #[serde(default)]
    pub mentions: Vec<DiscordUser>,
}

/// Outgoing Discord message payload for Discord REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordOutgoingMessage {
    pub channel_id: String,
    pub content: String,
    pub reply_to_message_id: Option<String>,
}

/// Discord Adapter for VirtualCharacter runtime.
///
/// Under Skill 14 (Relationship) & Skill 25 (Security):
/// - Enforces strict Actor Isolation: Maps Discord user ID -> "discord:<user_id>".
/// - Prevents cross-user memory leakage.
/// - Filters bot loops and unwanted channel traffic.
#[derive(Clone)]
pub struct DiscordAdapter {
    config: DiscordConfig,
    event_bus: PlatformEventBus,
    outbox_history: Arc<Mutex<Vec<DiscordOutgoingMessage>>>,
}

impl DiscordAdapter {
    pub fn new(config: DiscordConfig, event_bus: PlatformEventBus) -> Self {
        Self {
            config,
            event_bus,
            outbox_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Parse an incoming Discord message payload and convert to a unified PlatformChatMessage.
    pub fn process_incoming(
        &self,
        msg: &DiscordIncomingMessage,
        now: u64,
    ) -> Option<PlatformChatMessage> {
        // 1. Ignore bot messages to prevent echo loops
        if msg.author.bot {
            return None;
        }

        // 2. Channel filtering if configured
        if !self.config.allowed_channels.is_empty()
            && !self.config.allowed_channels.contains(&msg.channel_id)
        {
            return None;
        }

        // 3. Mention check
        let is_mentioned = if let Some(ref bot_id) = self.config.bot_user_id {
            msg.mentions.iter().any(|m| m.id == *bot_id)
                || msg.content.contains(&format!("<@{}>", bot_id))
                || msg.content.contains(&format!("<@!{}>", bot_id))
        } else {
            !msg.mentions.is_empty()
        };

        if self.config.require_mention && !is_mentioned {
            return None;
        }

        // 4. Sanitize and clean message content
        let mut cleaned_content = msg.content.clone();
        if let Some(ref bot_id) = self.config.bot_user_id {
            cleaned_content = cleaned_content
                .replace(&format!("<@{}>", bot_id), "")
                .replace(&format!("<@!{}>", bot_id), "");
        }
        let cleaned_content = cleaned_content.trim().to_string();

        if cleaned_content.is_empty() {
            return None;
        }

        // 5. Build Actor ID with prefix for namespace isolation
        let actor_id = format!("discord:{}", msg.author.id);

        let mut chat_msg = PlatformChatMessage::new(
            PlatformType::Discord,
            &msg.channel_id,
            actor_id,
            &msg.author.username,
            cleaned_content,
            now,
        )
        .with_mention(is_mentioned);

        if let Some(ref gid) = msg.guild_id {
            chat_msg = chat_msg.with_meta("guild_id", gid);
        }
        chat_msg = chat_msg.with_meta("discord_message_id", &msg.id);

        // 6. Publish to Unified Event Bus
        let _ = self.event_bus.publish_chat(chat_msg.clone());

        Some(chat_msg)
    }

    /// Deliver speech response to Discord.
    pub fn send_response(&self, speech: &CharacterSpeechOutput) -> Result<String, String> {
        if speech.platform != PlatformType::Discord {
            return Err("Target platform is not Discord".to_string());
        }

        let outgoing = DiscordOutgoingMessage {
            channel_id: speech.channel_id.clone(),
            content: speech.text.clone(),
            reply_to_message_id: None,
        };

        // If bot token is provided, attempt HTTP POST via Discord REST API v10
        if let Some(ref token) = self.config.bot_token {
            let url = format!(
                "https://discord.com/api/v10/channels/{}/messages",
                speech.channel_id
            );
            let payload = serde_json::json!({
                "content": speech.text
            });

            match ureq::post(&url)
                .set("Authorization", &format!("Bot {}", token))
                .set("Content-Type", "application/json")
                .send_json(payload)
            {
                Ok(resp) => {
                    if let Ok(body) = resp.into_json::<serde_json::Value>() {
                        let msg_id = body["id"].as_str().unwrap_or("unknown").to_string();
                        self.record_outbox(outgoing);
                        return Ok(msg_id);
                    }
                    self.record_outbox(outgoing);
                    Ok("sent_live".to_string())
                }
                Err(e) => {
                    eprintln!("⚠️ Discord REST API error: {}", e);
                    self.record_outbox(outgoing);
                    Err(format!("Discord API request failed: {}", e))
                }
            }
        } else {
            // Headless / Test mode
            let msg_id = format!("mock-msg-{}", uuid::Uuid::new_v4());
            self.record_outbox(outgoing);
            Ok(msg_id)
        }
    }

    fn record_outbox(&self, msg: DiscordOutgoingMessage) {
        if let Ok(mut history) = self.outbox_history.lock() {
            history.push(msg);
        }
    }

    /// Retrieve sent outbox history (for testing and verification).
    pub fn get_outbox_history(&self) -> Vec<DiscordOutgoingMessage> {
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
    fn test_discord_adapter_mentions_and_actor_isolation() {
        let bus = PlatformEventBus::new(16);
        let config = DiscordConfig {
            bot_token: None,
            bot_user_id: Some("11223344".to_string()),
            guild_id: Some("guild-1".to_string()),
            allowed_channels: vec!["chan-general".to_string()],
            require_mention: true,
        };
        let adapter = DiscordAdapter::new(config, bus);

        // 1. Message from bot should be ignored
        let bot_msg = DiscordIncomingMessage {
            id: "msg-1".into(),
            channel_id: "chan-general".into(),
            guild_id: Some("guild-1".into()),
            author: DiscordUser {
                id: "bot-99".into(),
                username: "MusicBot".into(),
                discriminator: None,
                bot: true,
            },
            content: "Now playing song".into(),
            mentions: vec![],
        };
        assert!(adapter.process_incoming(&bot_msg, 1000).is_none());

        // 2. Message without mention should be ignored when require_mention is true
        let normal_msg = DiscordIncomingMessage {
            id: "msg-2".into(),
            channel_id: "chan-general".into(),
            guild_id: Some("guild-1".into()),
            author: DiscordUser {
                id: "user-1".into(),
                username: "Tuan".into(),
                discriminator: None,
                bot: false,
            },
            content: "Hey everyone!".into(),
            mentions: vec![],
        };
        assert!(adapter.process_incoming(&normal_msg, 1001).is_none());

        // 3. Message with mention in allowed channel should be ingested with actor isolation
        let mention_msg = DiscordIncomingMessage {
            id: "msg-3".into(),
            channel_id: "chan-general".into(),
            guild_id: Some("guild-1".into()),
            author: DiscordUser {
                id: "user-12345".into(),
                username: "TuanDev".into(),
                discriminator: None,
                bot: false,
            },
            content: "<@11223344> chào em, Aria!".into(),
            mentions: vec![DiscordUser {
                id: "11223344".into(),
                username: "AriaBot".into(),
                discriminator: None,
                bot: true,
            }],
        };
        let processed = adapter.process_incoming(&mention_msg, 1002);
        assert!(processed.is_some());
        let chat = processed.unwrap();
        assert_eq!(chat.actor_id, "discord:user-12345");
        assert_eq!(chat.content, "chào em, Aria!");
        assert!(chat.is_mention);
        assert_eq!(chat.platform, PlatformType::Discord);
    }
}
