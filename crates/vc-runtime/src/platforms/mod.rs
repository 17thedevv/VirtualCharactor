pub mod discord;
pub mod stream;

pub use discord::{
    DiscordAdapter, DiscordConfig, DiscordIncomingMessage, DiscordOutgoingMessage, DiscordUser,
};
pub use stream::{TwitchChatAdapter, YouTubeLiveAdapter, YouTubeSnippet};
