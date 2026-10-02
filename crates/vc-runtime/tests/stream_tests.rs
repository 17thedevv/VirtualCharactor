use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_llm::mock::MockLlmProvider;
use vc_runtime::events::{CharacterSpeechOutput, PlatformEventBus, PlatformType};
use vc_runtime::platforms::stream::{TwitchChatAdapter, YouTubeLiveAdapter, YouTubeSnippet};
use vc_runtime::runtime::RuntimeEngine;
use vc_runtime::stream::{
    ChatPriorityConfig, ChatPriorityEngine, DirectorDecision, StreamDirector, StreamDirectorConfig,
};

#[test]
fn test_youtube_and_twitch_chat_ingestion() {
    let bus = PlatformEventBus::new(32);
    let yt_adapter = YouTubeLiveAdapter::new("yt-stream-001", bus.clone());
    let twitch_adapter = TwitchChatAdapter::new("aria_live", bus.clone());

    // 1. YouTube SuperChat
    let yt_msg = yt_adapter.process_snippet(
        &YouTubeSnippet {
            author_channel_id: "chan-vip".into(),
            display_name: "VIP_Supporter".into(),
            message_text: "Aria hát bài cưng thích đi!".into(),
            is_superchat: true,
            superchat_amount: Some(50.0),
            currency: Some("USD".into()),
        },
        1000,
    );
    assert_eq!(yt_msg.platform, PlatformType::YouTube);
    assert!(yt_msg.is_superchat);
    assert_eq!(yt_msg.actor_id, "youtube:chan-vip");

    // 2. Twitch message
    let twitch_line = ":pixel_gamer!pixel_gamer@pixel_gamer.tmi.twitch.tv PRIVMSG #aria_live :Game này khó quá Aria ơi";
    let twitch_msg = twitch_adapter
        .parse_irc_line(twitch_line, 1001)
        .expect("should parse twitch line");
    assert_eq!(twitch_msg.platform, PlatformType::Twitch);
    assert_eq!(twitch_msg.author_name, "pixel_gamer");
    assert_eq!(twitch_msg.actor_id, "twitch:pixel_gamer");
}

#[test]
fn test_chat_priority_and_topic_aggregation() {
    let mut engine = ChatPriorityEngine::new(ChatPriorityConfig::default());

    // Ingest various messages
    let spam1 = vc_runtime::events::PlatformChatMessage::new(
        PlatformType::YouTube,
        "stream",
        "spammer",
        "Spammer",
        "Spam emote",
        100,
    );
    let spam2 = vc_runtime::events::PlatformChatMessage::new(
        PlatformType::YouTube,
        "stream",
        "spammer",
        "Spammer",
        "Spam emote",
        101,
    );
    let question = vc_runtime::events::PlatformChatMessage::new(
        PlatformType::Twitch,
        "stream",
        "user-q",
        "CuriousGamer",
        "Aria ơi em đang chơi game gì vậy?",
        102,
    );
    let superchat = vc_runtime::events::PlatformChatMessage::new(
        PlatformType::YouTube,
        "stream",
        "donor",
        "Generous",
        "Tặng Aria cốc trà sữa nè!",
        103,
    )
    .with_superchat(10.0, "USD");

    engine.evaluate_and_ingest(spam1, 100);
    let eval_spam2 = engine.evaluate_and_ingest(spam2, 101);
    assert!(
        eval_spam2.is_duplicate,
        "Second identical message must be flagged as duplicate"
    );

    engine.evaluate_and_ingest(question, 102);
    engine.evaluate_and_ingest(superchat, 103);

    // Topics should identify gaming
    let topics = engine.aggregate_topics();
    assert_eq!(*topics.get("gaming").unwrap_or(&0), 1);

    // Top message must be the superchat
    let top = engine
        .pop_highest_priority()
        .expect("should have top message");
    assert!(top.message.is_superchat);
    assert_eq!(top.message.author_name, "Generous");
}

#[test]
fn test_stream_director_e2e_pacing_and_silence_handling() {
    let mut director = StreamDirector::new(StreamDirectorConfig {
        silence_threshold_secs: 10,
        min_speech_interval_secs: 3,
        max_chat_per_minute: 10,
    });
    let mut priority_engine = ChatPriorityEngine::default_engine();

    // Turn 1: SuperChat arrives at t=100
    let sc = vc_runtime::events::PlatformChatMessage::new(
        PlatformType::YouTube,
        "live-1",
        "user-sc",
        "FanA",
        "Chào Aria tỷ tỷ!",
        100,
    )
    .with_superchat(25.0, "USD");
    priority_engine.evaluate_and_ingest(sc, 100);

    let dec1 = director.evaluate_turn(&mut priority_engine, None, 100);
    match dec1 {
        DirectorDecision::RespondToChat {
            message,
            priority_score,
        } => {
            assert_eq!(message.author_name, "FanA");
            assert!(priority_score >= 100.0);
        }
        _ => panic!("Expected RespondToChat"),
    }

    // Turn 2: Immediate follow up at t=101 -> within 3s cooldown -> WaitIdle
    let dec2 = director.evaluate_turn(&mut priority_engine, None, 101);
    assert_eq!(dec2, DirectorDecision::WaitIdle);

    // Turn 3: Urgent screen event at t=104 (e.g. Boss defeated) -> ReactToScreen
    let screen_event = Some(("Boss defeated in Elden Ring!".to_string(), 0.85));
    let dec3 = director.evaluate_turn(&mut priority_engine, screen_event, 104);
    match dec3 {
        DirectorDecision::ReactToScreen {
            description,
            urgency,
        } => {
            assert!(description.contains("Boss defeated"));
            assert_eq!(urgency, 0.85);
        }
        _ => panic!("Expected ReactToScreen"),
    }

    // Turn 4: Silence for > 10 seconds at t=116 -> triggers SelfInitiatedBanter
    let dec4 = director.evaluate_turn(&mut priority_engine, None, 116);
    assert!(matches!(dec4, DirectorDecision::SelfInitiatedBanter { .. }));
}

#[tokio::test]
async fn test_stream_director_runtime_orchestration_turn() {
    let bus = PlatformEventBus::new(16);
    let yt_adapter = YouTubeLiveAdapter::new("live-room", bus.clone());
    let mut priority_engine = ChatPriorityEngine::default_engine();
    let mut director = StreamDirector::default_director();

    // Ingest a high-value donation question
    let yt_msg = yt_adapter.process_snippet(
        &YouTubeSnippet {
            author_channel_id: "donor-99".into(),
            display_name: "SuperFan99".into(),
            message_text: "Aria stream vui quá! Hôm nay em có mệt không?".into(),
            is_superchat: true,
            superchat_amount: Some(30.0),
            currency: Some("USD".into()),
        },
        500,
    );
    priority_engine.evaluate_and_ingest(yt_msg, 500);

    // Director evaluates next action
    let decision = director.evaluate_turn(&mut priority_engine, None, 500);

    if let DirectorDecision::RespondToChat { message, .. } = decision {
        // Run full cognitive pipeline with RuntimeEngine
        let char_id = CharacterId::new();
        let personality = Personality::baseline_aria();
        let mut state = CharacterState::default_aria();
        let mut rel = Relationship::new_companion(char_id, &message.actor_id);
        let mut memories = Vec::new();

        let mock_llm = Arc::new(MockLlmProvider::new(
            "Cảm ơn SuperFan99 nhé! Nhìn thấy mọi người vui là em hết mệt ngay!",
        ));
        let runtime = RuntimeEngine::new(mock_llm);

        let outcome = runtime
            .process_interaction(
                char_id,
                &message.actor_id,
                &message.content,
                &personality,
                &mut state,
                &mut rel,
                &mut memories,
                501,
            )
            .expect("Runtime interaction must succeed");

        assert!(outcome.response_text.contains("hết mệt ngay"));

        // Dispatch reply back to YouTube chat
        let speech = CharacterSpeechOutput {
            character_id: char_id,
            target_actor_id: message.actor_id,
            platform: PlatformType::YouTube,
            channel_id: "live-room".to_string(),
            text: outcome.response_text,
            emotion: "Happy".to_string(),
            audio_bytes: None,
            timestamp: 502,
        };
        let res = yt_adapter.send_chat_reply(&speech);
        assert!(res.is_ok());

        let outbox = yt_adapter.get_outbox();
        assert_eq!(outbox.len(), 1);
        assert!(outbox[0].contains("SuperFan99"));
    } else {
        panic!("Director should have selected RespondToChat");
    }
}
