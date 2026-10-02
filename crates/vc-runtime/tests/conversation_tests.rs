use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use vc_llm::provider::{LlmProvider, LlmRequest};
use vc_llm::MockLlmProvider;
use vc_runtime::audio::TextStreamChunker;
use vc_runtime::conversation::{
    ConversationEvent, ConversationState, ConversationStateMachine, TransitionError,
};

#[test]
fn test_conversation_fsm_turn_cycle() {
    let mut fsm = ConversationStateMachine::new();
    assert_eq!(fsm.state(), &ConversationState::Idle);

    // 1. User starts speaking: IDLE -> LISTENING
    let interruption = fsm.on_user_speech_started(1000).expect("listen ok");
    assert!(interruption.is_none());
    assert!(fsm.state().is_listening());

    // 2. Transcript is finalized: LISTENING -> THINKING
    let ev = fsm
        .on_transcript_final(1002, "Hôm nay trời đẹp quá!")
        .expect("think ok");
    assert!(matches!(fsm.state(), ConversationState::Thinking { .. }));
    assert_eq!(
        ev,
        ConversationEvent::ThinkingStarted {
            user_input: "Hôm nay trời đẹp quá!".into(),
            timestamp: 1002,
        }
    );

    // 3. First audio ready: THINKING -> SPEAKING
    let ev_speak = fsm
        .on_first_audio_ready(1003, "utt-100", "Vâng, đúng vậy! Bầu trời rất trong xanh.")
        .expect("speak ok");
    assert!(matches!(fsm.state(), ConversationState::Speaking { .. }));
    assert_eq!(
        ev_speak,
        ConversationEvent::SpeakingStarted {
            utterance_id: "utt-100".into(),
            text: "Vâng, đúng vậy! Bầu trời rất trong xanh.".into(),
            timestamp: 1003,
        }
    );

    // 4. Completed: SPEAKING -> IDLE
    let ev_done = fsm.complete_speech(1005).expect("done ok");
    assert!(fsm.state().is_idle());
    assert_eq!(
        ev_done,
        ConversationEvent::SpeechCompleted {
            utterance_id: "utt-100".into(),
            full_text: "Vâng, đúng vậy! Bầu trời rất trong xanh.".into(),
            timestamp: 1005,
        }
    );
}

#[test]
fn test_state_transitions_all_valid_flows() {
    let mut fsm = ConversationStateMachine::new();

    // 1. IDLE -> LISTENING
    assert!(fsm.state().is_idle());
    let _ = fsm
        .on_user_speech_started(100)
        .expect("idle -> listening ok");
    assert!(fsm.state().is_listening());

    // 2. LISTENING -> THINKING
    let _ = fsm
        .on_transcript_final(102, "Xin chào")
        .expect("listening -> thinking ok");
    assert!(fsm.state().is_thinking());

    // 3. THINKING -> SPEAKING
    let _ = fsm
        .on_first_audio_ready(103, "utt-1", "Chào bạn!")
        .expect("thinking -> speaking ok");
    assert!(fsm.state().is_speaking());

    // 4. SPEAKING -> INTERRUPTED
    let int_res = fsm
        .interrupt(104, Some("Chào".into()))
        .expect("speaking -> interrupted ok");
    assert_eq!(int_res.heard_text, "Chào");
    assert_eq!(int_res.unspoken_text, "bạn!");
    assert!(matches!(fsm.state(), ConversationState::Interrupted { .. }));

    // 5. INTERRUPTED -> THINKING
    let _ = fsm
        .on_transcript_final(105, "Câu hỏi mới")
        .expect("interrupted -> thinking ok");
    assert!(fsm.state().is_thinking());

    // 6. THINKING -> INTERRUPTED (interrupted before audio starts)
    let int_res2 = fsm
        .interrupt(106, None)
        .expect("thinking -> interrupted ok");
    assert_eq!(
        int_res2.preserved_history_text,
        "[Interrupted while thinking]"
    );
    assert!(matches!(fsm.state(), ConversationState::Interrupted { .. }));

    // 7. Reset to IDLE
    fsm.reset(107);
    assert!(fsm.state().is_idle());

    // 8. IDLE -> SPEAKING (proactive utterance)
    let _ = fsm
        .on_first_audio_ready(108, "utt-proactive", "Chào buổi sáng!")
        .expect("idle -> speaking ok");
    assert!(fsm.state().is_speaking());

    // 9. SPEAKING -> IDLE
    let _ = fsm.complete_speech(109).expect("speaking -> idle ok");
    assert!(fsm.state().is_idle());
}

#[test]
fn test_invalid_state_transitions_rejected() {
    let mut fsm = ConversationStateMachine::new();

    // Invalid: IDLE -> complete_speech
    assert!(matches!(
        fsm.complete_speech(100),
        Err(TransitionError::InvalidTransition {
            from: "Idle",
            to: "Idle",
            ..
        })
    ));

    // Invalid: IDLE -> interrupt
    assert!(matches!(
        fsm.interrupt(100, None),
        Err(TransitionError::InvalidTransition {
            from: "Idle",
            to: "Interrupted",
            ..
        })
    ));

    // Move to Listening
    fsm.on_user_speech_started(101).unwrap();

    // Invalid: LISTENING -> complete_speech
    assert!(matches!(
        fsm.complete_speech(102),
        Err(TransitionError::InvalidTransition {
            from: "Listening",
            to: "Idle",
            ..
        })
    ));

    // Invalid: LISTENING -> interrupt
    assert!(matches!(
        fsm.interrupt(102, None),
        Err(TransitionError::InvalidTransition {
            from: "Listening",
            to: "Interrupted",
            ..
        })
    ));

    // Move to Thinking
    fsm.on_transcript_final(103, "Test").unwrap();

    // Invalid: THINKING -> complete_speech
    assert!(matches!(
        fsm.complete_speech(104),
        Err(TransitionError::InvalidTransition {
            from: "Thinking",
            to: "Idle",
            ..
        })
    ));

    // Move to Speaking
    fsm.on_first_audio_ready(105, "utt-test", "Sentence one.")
        .unwrap();

    // Invalid: SPEAKING -> on_first_audio_ready (cannot start speaking while speaking)
    assert!(matches!(
        fsm.on_first_audio_ready(106, "utt-test2", "Sentence two."),
        Err(TransitionError::InvalidTransition {
            from: "Speaking",
            to: "Speaking",
            ..
        })
    ));
}

#[test]
fn test_barge_in_interruption_tracking_and_preservation() {
    let mut fsm = ConversationStateMachine::new();

    fsm.on_user_speech_started(1000).unwrap();
    fsm.on_transcript_final(1001, "Hãy giải thích thuyết lượng tử đi.")
        .unwrap();
    fsm.on_first_audio_ready(
        1002,
        "utt-200",
        "Thuyết lượng tử là lý thuyết vật lý cơ bản miêu tả tự nhiên ở thang vi mô của các nguyên tử.",
    ).unwrap();

    // Partial playback: user heard "Thuyết lượng tử là lý thuyết vật lý"
    fsm.update_heard_progress("Thuyết lượng tử là lý thuyết vật lý");

    // User interrupts by speaking over character: SPEAKING -> INTERRUPTED -> LISTENING
    let interrupt_res = fsm
        .on_user_speech_started(1004)
        .expect("speech triggers interrupt");
    let result = interrupt_res.expect("Must have interruption payload");

    assert_eq!(result.heard_text, "Thuyết lượng tử là lý thuyết vật lý");
    assert!(result.unspoken_text.contains("cơ bản miêu tả"));
    assert_eq!(
        result.preserved_history_text,
        "Thuyết lượng tử là lý thuyết vật lý [Interrupted by user]"
    );
    // State is immediately Listening for the user's interruption turn!
    assert!(fsm.state().is_listening());
}

#[test]
fn test_continuation_scope_and_timeout() {
    let mut fsm = ConversationStateMachine::new();

    // User says "Aria ơi..." -> Prompt ack "Dạ?"
    fsm.await_continuation(2000, "Dạ?", 4)
        .expect("continuation started");
    assert!(matches!(
        fsm.state(),
        ConversationState::AwaitingContinuation { .. }
    ));

    // User continues speaking at 2002s (within 4s)
    let interrupt = fsm.on_user_speech_started(2002).expect("user continued");
    assert!(interrupt.is_none());
    assert!(fsm.state().is_listening());

    // In a new scenario, let's test continuation timeout
    let mut fsm2 = ConversationStateMachine::new();
    fsm2.await_continuation(3000, "Dạ?", 4).unwrap();
    assert!(fsm2.check_timeouts(3002).is_none());
    let timeout_ev = fsm2.check_timeouts(3005).expect("must expire at 3005");
    assert_eq!(
        timeout_ev,
        ConversationEvent::ContinuationExpired { timestamp: 3005 }
    );
    assert!(fsm2.state().is_idle());
}

#[test]
fn test_cancellation_mid_stream() {
    let cancellation_flag = Arc::new(AtomicBool::new(false));
    let mut fsm = ConversationStateMachine::new();

    // Start turn
    fsm.on_user_speech_started(100).unwrap();
    fsm.on_transcript_final(101, "Kể chuyện dài đi.").unwrap();

    let full_text = "Ngày xửa ngày xưa, ở một vương quốc xa xôi có một vị vua anh minh.";
    fsm.on_first_audio_ready(102, "utt-cancel", full_text)
        .unwrap();

    // Playback first sentence
    fsm.update_heard_progress("Ngày xửa ngày xưa, ở một vương quốc xa xôi");

    // Cancellation triggered (e.g., Barge-In signal received)
    cancellation_flag.store(true, Ordering::SeqCst);

    if cancellation_flag.load(Ordering::SeqCst) {
        let res = fsm.interrupt(103, None).expect("interrupt ok");
        assert_eq!(res.heard_text, "Ngày xửa ngày xưa, ở một vương quốc xa xôi");
        assert!(res.unspoken_text.contains("có một vị vua anh minh"));
    }

    assert!(matches!(fsm.state(), ConversationState::Interrupted { .. }));
}

#[test]
fn test_streaming_event_propagation_pipeline() {
    let mock = MockLlmProvider::new("Chào bạn nhé! Rất vui được gặp lại bạn.");
    let request = LlmRequest::new("Chào!");
    let stream = mock.stream_text(request).expect("start stream");

    let mut chunker = TextStreamChunker::default_tts();
    let mut fsm = ConversationStateMachine::new();
    fsm.on_user_speech_started(100).unwrap();
    fsm.on_transcript_final(101, "Chào!").unwrap();

    let mut is_first_sentence = true;
    let mut generated_sentences = Vec::new();

    for token_res in stream {
        let token = token_res.expect("token ok");
        let sentences = chunker.feed(&token);
        for s in sentences {
            if is_first_sentence {
                is_first_sentence = false;
                fsm.on_first_audio_ready(102, "utt-stream", &s)
                    .expect("audio ready");
            }
            fsm.update_heard_progress(&s);
            generated_sentences.push(s);
        }
    }

    if let Some(trailing) = chunker.flush() {
        if is_first_sentence {
            fsm.on_first_audio_ready(103, "utt-stream", &trailing)
                .expect("audio ready");
        }
        fsm.update_heard_progress(&trailing);
        generated_sentences.push(trailing);
    }

    // Assert that sentences were chunked properly
    assert!(!generated_sentences.is_empty());
    assert!(fsm.state().is_speaking());

    // Speech completed
    let done_ev = fsm.complete_speech(105).expect("speech completed");
    assert_eq!(
        done_ev,
        ConversationEvent::SpeechCompleted {
            utterance_id: "utt-stream".into(),
            full_text: generated_sentences[0].clone(),
            timestamp: 105,
        }
    );
    assert!(fsm.state().is_idle());
}
