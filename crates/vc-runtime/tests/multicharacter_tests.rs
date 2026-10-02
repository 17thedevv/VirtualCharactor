use std::sync::Arc;
use vc_core::character::{Character, CharacterId};
use vc_core::personality::Personality;
use vc_core::state::CharacterState;
use vc_llm::mock::MockLlmProvider;
use vc_runtime::adaptation::{
    FeedbackRating, InteractionFeedback, OfflineAdaptationEngine, PreferenceDataset,
};
use vc_runtime::interaction::InteractionId;
use vc_runtime::multicharacter::{
    CharacterProfile, CharacterToCharacterDialogue, MultiCharacterRegistry,
};
use vc_runtime::runtime::RuntimeEngine;

#[test]
fn test_multicharacter_registry_management_and_isolation() {
    let mut registry = MultiCharacterRegistry::new();

    let aria_char = Character {
        id: CharacterId::new(),
        name: "Aria".to_string(),
    };
    let mut aria_profile = CharacterProfile::new(
        aria_char,
        Personality::baseline_aria(),
        CharacterState::default_aria(),
    );
    aria_profile
        .memories
        .push(vc_core::memory::Memory::new_core(
            "Kỷ niệm ban đầu của Aria",
            100,
        ));

    let hikari_profile = CharacterProfile::preset_hikari();
    let aria_id = aria_profile.character.id;
    let hikari_id = hikari_profile.character.id;

    registry.register(aria_profile);
    registry.register(hikari_profile);

    // Initial active should be Aria
    assert_eq!(registry.active_id(), Some(aria_id));
    assert_eq!(registry.get_active().unwrap().character.name, "Aria");
    assert_eq!(registry.get_active().unwrap().memories.len(), 1);

    // Switch to Hikari
    registry
        .switch_active(hikari_id)
        .expect("Switch to Hikari must succeed");
    assert_eq!(registry.active_id(), Some(hikari_id));
    assert_eq!(registry.get_active().unwrap().character.name, "Hikari");
    assert_eq!(
        registry.get_active().unwrap().memories.len(),
        0,
        "Hikari must not have Aria's memories"
    );

    // Check summaries
    let summaries = registry.list_summaries();
    assert_eq!(summaries.len(), 2);
    let active_summary = summaries.iter().find(|s| s.is_active).unwrap();
    assert_eq!(active_summary.name, "Hikari");
}

#[test]
fn test_character_to_character_autonomous_dialogue() {
    let mut aria = CharacterProfile::new(
        Character {
            id: CharacterId::new(),
            name: "Aria".to_string(),
        },
        Personality::baseline_aria(),
        CharacterState::default_aria(),
    );
    let mut hikari = CharacterProfile::preset_hikari();

    let aria_id = aria.character.id;
    let hikari_id = hikari.character.id;

    let mock_aria = Arc::new(MockLlmProvider::new(
        "Chào em gái Hikari! Hôm nay phòng live của em có vui không?",
    ));
    let mock_hikari = Arc::new(MockLlmProvider::new(
        "Hừ, chị Aria lại trêu em rồi! Nhưng mà hôm nay em gánh team đỉnh lắm á!",
    ));

    let runtime_aria = RuntimeEngine::new(mock_aria);
    let runtime_hikari = RuntimeEngine::new(mock_hikari);

    let orchestrator = CharacterToCharacterDialogue::new(1); // 1 round = Aria -> Hikari
    let history = orchestrator
        .run_dialogue(
            &mut aria,
            &mut hikari,
            "Khởi động buổi live đôi",
            &runtime_aria,
            &runtime_hikari,
            2000,
        )
        .expect("Dialogue execution must succeed");

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].speaker_id, aria_id);
    assert_eq!(history[0].listener_id, hikari_id);
    assert!(history[0].utterance.contains("Chào em gái Hikari"));

    assert_eq!(history[1].speaker_id, hikari_id);
    assert_eq!(history[1].listener_id, aria_id);
    assert!(history[1].utterance.contains("chị Aria"));

    // Verify independent mutual relationships
    let rel_aria_hikari = aria
        .relationships
        .get(&format!("character:{}", hikari_id.0))
        .unwrap();
    let rel_hikari_aria = hikari
        .relationships
        .get(&format!("character:{}", aria_id.0))
        .unwrap();

    assert_eq!(rel_aria_hikari.interaction_count, 1);
    assert_eq!(rel_hikari_aria.interaction_count, 1);
    assert_eq!(
        rel_aria_hikari.target_id,
        format!("character:{}", hikari_id.0)
    );
    assert_eq!(
        rel_hikari_aria.target_id,
        format!("character:{}", aria_id.0)
    );
}

#[test]
fn test_continuous_offline_adaptation_lifecycle() {
    let mut dataset = PreferenceDataset::new();
    let char_id = CharacterId::new();

    // Ingest 4 thumbs up
    for i in 0..4 {
        dataset.add_record(InteractionFeedback::new(
            InteractionId::new(),
            char_id,
            "user-vip",
            format!("Hỏi {}", i),
            format!("Đáp {}", i),
            FeedbackRating::Upvote,
            3000 + i,
        ));
    }

    // Ingest 1 downvote due to wordiness
    dataset.add_record(
        InteractionFeedback::new(
            InteractionId::new(),
            char_id,
            "user-vip",
            "Giải thích ngắn gọn thôi",
            "Bài văn dài 1000 chữ...",
            FeedbackRating::Downvote,
            3010,
        )
        .with_tag("too_long"),
    );

    assert_eq!(dataset.total_count(), 5);
    assert_eq!(dataset.satisfaction_ratio(), 0.8);

    let report = OfflineAdaptationEngine::analyze(&dataset);
    assert_eq!(report.sample_size, 5);
    assert_eq!(report.suggested_verbosity_delta, -0.1);
    assert_eq!(report.suggested_initiative_delta, 0.05);

    // Apply adaptation to personality
    let mut personality = Personality::baseline_aria();
    let initial_playfulness = personality.traits.playfulness.value();
    OfflineAdaptationEngine::apply_to_personality(&mut personality, &report);
    assert!(personality.traits.playfulness.value() > initial_playfulness);

    // Verify dataset export
    let jsonl = dataset.export_jsonl();
    assert_eq!(jsonl.lines().count(), 5);
}
