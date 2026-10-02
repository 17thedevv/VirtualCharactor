use std::sync::Arc;
use vc_core::character::CharacterId;
use vc_core::personality::Personality;
use vc_core::relationship::Relationship;
use vc_core::state::CharacterState;
use vc_llm::MockLlmProvider;
use vc_runtime::audio::{
    AudioInputProvider, EmotionAwareVoiceModulator, InputAudioMode, MockAudioInputProvider,
    MockTtsProvider, TtsProvider, VadStatus,
};
use vc_runtime::runtime::RuntimeEngine;

#[test]
fn test_audio_input_vad_and_push_to_talk() {
    let input_provider = MockAudioInputProvider::default();
    assert_eq!(input_provider.mode(), InputAudioMode::VoiceActivity);

    // Test mode switching to PushToTalk
    input_provider.set_mode(InputAudioMode::PushToTalk).unwrap();
    assert_eq!(input_provider.mode(), InputAudioMode::PushToTalk);

    // Audio capture must be started
    input_provider.start_listening().unwrap();
    assert!(input_provider.is_listening());

    // Stream silence vs speech
    let silence = vec![0.0f32; 1600];
    let speech = vec![0.08f32; 1600];

    assert_eq!(
        input_provider.process_audio_chunk(&silence).unwrap(),
        VadStatus::Silence
    );
    assert_eq!(
        input_provider.process_audio_chunk(&speech).unwrap(),
        VadStatus::SpeechActive
    );

    // End of speech triggered after consecutive silences
    assert_eq!(
        input_provider.process_audio_chunk(&silence).unwrap(),
        VadStatus::SpeechActive
    );
    assert_eq!(
        input_provider.process_audio_chunk(&silence).unwrap(),
        VadStatus::SpeechActive
    );
    let finished = input_provider.process_audio_chunk(&silence).unwrap();

    match finished {
        VadStatus::SpeechFinished(pcm) => {
            assert!(!pcm.is_empty());
        }
        _ => panic!("Expected SpeechFinished"),
    }

    // Queue and poll recognition result
    input_provider.queue_transcription("Aria ơi, bật cho mình một bản nhạc nhé");
    let text = input_provider.poll_transcription();
    assert_eq!(text, Some("Aria ơi, bật cho mình một bản nhạc nhé".into()));
    assert_eq!(input_provider.poll_transcription(), None);

    input_provider.stop_listening().unwrap();
    assert!(!input_provider.is_listening());
}

#[test]
fn test_tts_synthesis_and_lip_sync_visemes() {
    let tts = MockTtsProvider::new(24000);
    let modulator = EmotionAwareVoiceModulator::default();

    let mut state = CharacterState::default_aria();
    // Set joyful emotions
    state.emotion.joy = vc_core::state::emotion::EmotionScore::clamped(0.85);

    let modulation = modulator.modulate(&state.emotion);
    assert!(modulation.pitch_modifier > 1.05);

    let output = tts
        .synthesize("Xin chào các bạn trên livestream", &modulation)
        .unwrap();

    assert_eq!(output.sample_rate, 24000);
    assert!(output.duration_ms > 200);
    assert!(!output.audio_bytes.is_empty());

    // Check viseme lip-sync cues
    assert!(!output.visemes.is_empty());
    for viseme in &output.visemes {
        assert!(viseme.mouth_open >= 0.0 && viseme.mouth_open <= 1.0);
        assert!(viseme.duration_ms > 0);
    }
}

#[test]
fn test_end_to_end_voice_turn_flow() {
    // 1. Setup Audio Input and Output components
    let stt = MockAudioInputProvider::default();
    let tts = MockTtsProvider::default();
    let modulator = EmotionAwareVoiceModulator::default();

    // 2. Setup Core Runtime & domain entities
    let llm_provider = Arc::new(MockLlmProvider::new(
        "Chào bạn! Mình là Aria đây! Rất vui được nói chuyện với bạn!",
    ));
    let runtime = RuntimeEngine::new(llm_provider);

    let char_id = CharacterId::new();
    let personality = Personality::baseline_aria();
    let mut state = CharacterState::default_aria();
    let mut relationship = Relationship::new_companion(char_id, "streamer-user");
    let mut memories = vec![];

    // 3. Simulate speech input from user
    stt.start_listening().unwrap();
    stt.queue_transcription("Chào Aria!");
    let speech_text = stt
        .poll_transcription()
        .expect("Transcribed text should be present");

    // 4. Ingest speech into Runtime interaction lifecycle
    let outcome = runtime
        .process_interaction(
            char_id,
            "streamer-user",
            &speech_text,
            &personality,
            &mut state,
            &mut relationship,
            &mut memories,
            1000,
        )
        .expect("Interaction processing should succeed");

    assert!(!outcome.response_text.is_empty());

    // 5. Modulate voice dynamically from post-turn emotional state
    let modulation = modulator.modulate(&state.emotion);
    let audio_output = tts
        .synthesize(&outcome.response_text, &modulation)
        .expect("TTS synthesis should succeed");

    assert!(!audio_output.audio_bytes.is_empty());
    assert!(!audio_output.visemes.is_empty());
    assert!(audio_output.duration_ms > 0);
    assert_eq!(tts.last_text(), Some(outcome.response_text));
}
