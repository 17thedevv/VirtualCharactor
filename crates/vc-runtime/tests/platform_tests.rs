use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_llm::mock::MockLlmProvider;
use vc_runtime::events::{
    CharacterSpeechOutput, PlatformChatMessage, PlatformEventBus, PlatformType,
    UnifiedPlatformEvent,
};
use vc_runtime::platforms::discord::{
    DiscordAdapter, DiscordConfig, DiscordIncomingMessage, DiscordUser,
};
use vc_runtime::runtime::RuntimeEngine;

#[tokio::test]
async fn test_unified_event_bus_broadcast() {
    let bus = PlatformEventBus::new(32);
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();

    let chat = PlatformChatMessage::new(
        PlatformType::Discord,
        "chan-1",
        "discord:user-1",
        "Alice",
        "Hello from Discord!",
        1000,
    );

    let sent = bus
        .publish_chat(chat.clone())
        .expect("publish should succeed");
    assert!(sent >= 2, "Both receivers should have received the message");

    let evt1 = rx1.recv().await.expect("rx1 should receive event");
    let evt2 = rx2.recv().await.expect("rx2 should receive event");

    if let UnifiedPlatformEvent::ChatMessage(c1) = evt1 {
        assert_eq!(c1.author_name, "Alice");
        assert_eq!(c1.content, "Hello from Discord!");
    } else {
        panic!("Expected ChatMessage");
    }

    if let UnifiedPlatformEvent::ChatMessage(c2) = evt2 {
        assert_eq!(c2.actor_id, "discord:user-1");
    } else {
        panic!("Expected ChatMessage");
    }
}

#[tokio::test]
async fn test_discord_actor_isolation_and_interaction_e2e() {
    let bus = PlatformEventBus::new(32);
    let config = DiscordConfig {
        bot_token: None, // Headless / test mode
        bot_user_id: Some("bot-aria-99".to_string()),
        guild_id: Some("guild-main".to_string()),
        allowed_channels: vec!["chan-ai-chat".to_string()],
        require_mention: true,
    };
    let adapter = DiscordAdapter::new(config, bus.clone());

    // 1. Receive incoming message from Alice
    let alice_msg = DiscordIncomingMessage {
        id: "msg-101".to_string(),
        channel_id: "chan-ai-chat".to_string(),
        guild_id: Some("guild-main".to_string()),
        author: DiscordUser {
            id: "alice-123".to_string(),
            username: "AliceChan".to_string(),
            discriminator: None,
            bot: false,
        },
        content: "<@bot-aria-99> Aria ơi, chào buổi sáng!".to_string(),
        mentions: vec![DiscordUser {
            id: "bot-aria-99".to_string(),
            username: "Aria".to_string(),
            discriminator: None,
            bot: true,
        }],
    };

    let processed_alice = adapter
        .process_incoming(&alice_msg, 2000)
        .expect("Alice message should be accepted");
    assert_eq!(processed_alice.actor_id, "discord:alice-123");
    assert_eq!(processed_alice.content, "Aria ơi, chào buổi sáng!");

    // 2. Process interaction through RuntimeEngine for Alice
    let char_id = CharacterId::new();
    let personality = Personality::baseline_aria();
    let mut state = CharacterState::default_aria();
    let mut memories = Vec::new();
    let mut rel_alice = Relationship::new_companion(char_id, &processed_alice.actor_id);

    let mock_llm = Arc::new(MockLlmProvider::new(
        "Chào buổi sáng Alice! Hôm nay của bạn thế nào?",
    ));
    let runtime = RuntimeEngine::new(mock_llm);

    let outcome = runtime
        .process_interaction(
            char_id,
            &processed_alice.actor_id,
            &processed_alice.content,
            &personality,
            &mut state,
            &mut rel_alice,
            &mut memories,
            2001,
        )
        .expect("Runtime interaction should succeed");

    assert!(outcome.response_text.contains("Chào buổi sáng"));
    assert_eq!(rel_alice.interaction_count, 1);

    // 3. Send speech response back to Discord channel
    let speech = CharacterSpeechOutput {
        character_id: char_id,
        target_actor_id: processed_alice.actor_id.clone(),
        platform: PlatformType::Discord,
        channel_id: processed_alice.channel_id.clone(),
        text: outcome.response_text.clone(),
        emotion: "Happy".to_string(),
        audio_bytes: None,
        timestamp: 2002,
    };

    let send_res = adapter.send_response(&speech);
    assert!(send_res.is_ok());

    let outbox = adapter.get_outbox_history();
    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].channel_id, "chan-ai-chat");
    assert!(outbox[0].content.contains("Chào buổi sáng"));

    // 4. Verify Actor Isolation: Bob's relationship is completely separate from Alice's
    let bob_actor_id = "discord:bob-456";
    let rel_bob = Relationship::new_companion(char_id, bob_actor_id);
    assert_eq!(rel_bob.interaction_count, 0);
    assert_eq!(rel_alice.interaction_count, 1);
    assert_ne!(rel_bob.target_id, rel_alice.target_id);
}
