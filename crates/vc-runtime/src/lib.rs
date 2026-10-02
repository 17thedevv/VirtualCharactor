pub mod adaptation;
pub mod attention;
pub mod audio;
pub mod autonomous;
pub mod character;
pub mod chat;
pub mod computer;
pub mod consolidation;
pub mod conversation;
pub mod error;
pub mod events;
pub mod in_memory_store;
pub mod interaction;
pub mod mock_decision;
pub mod multicharacter;
pub mod platforms;
pub mod rag;
pub mod resource_manager;
pub mod rule_decision_engine;
pub mod rule_emotion_engine;
pub mod runtime;
pub mod session;
pub mod stream;
pub mod tools;
pub mod vision;

pub use adaptation::{
    AdaptationReport, FeedbackRating, InteractionFeedback, OfflineAdaptationEngine,
    PreferenceDataset,
};
pub use attention::{
    AttentionConfig, AttentionEngine, AttentionEvaluation, AttentionEvent, AttentionScore,
    TemporalAttentionGate, TemporalGateConfig, TemporalGateDecision, VisualTemporalEvent,
};
pub use audio::{
    calculate_rms_volume_slices, clean_narrative_text, AudioInputProvider, AudioOutput,
    CpalAudioInputProvider, EmotionAwareVoiceModulator, InputAudioMode, MockAudioInputProvider,
    MockTtsProvider, OrderedTtsQueue, SynthesizedSentenceAudio, TextStreamChunker, TtsProvider,
    VadConfig, VadStatus, VisemeCue, VoiceModulation, WhisperCpuTranscriber, WhisperResult,
    WindowsSapiTtsProvider,
};
pub use autonomous::{
    AutonomousActionType, AutonomousLifeConfig, AutonomousLifeEngine, AutonomousProposal,
    SubconsciousReflexAction, SubconsciousReflexOutput,
};
pub use computer::{
    ActionResult, ComputerAction, ComputerExecutor, MockComputerExecutor, MouseButton,
    ScreenVerificationLoop, ToolPermissionPolicy, ToolRiskLevel, VerificationResult,
    WindowsComputerExecutor,
};
pub use consolidation::{ConsolidationReport, MemoryConsolidator};
pub use conversation::{
    ConversationEvent, ConversationState, ConversationStateMachine, InterruptionResult,
    TransitionError,
};
pub use error::{Result, RuntimeError};
pub use events::{
    CharacterSpeechOutput, PlatformChatMessage, PlatformEventBus, PlatformType,
    UnifiedPlatformEvent,
};
pub use multicharacter::{
    CharacterProfile, CharacterSummary, CharacterToCharacterDialogue, DialogueTurn,
    MultiCharacterRegistry,
};
pub use platforms::{
    DiscordAdapter, DiscordConfig, DiscordIncomingMessage, DiscordOutgoingMessage, DiscordUser,
    TwitchChatAdapter, YouTubeLiveAdapter, YouTubeSnippet,
};
pub use rag::{ConversationArchiver, IndexingReport, LoreIndexer, SemanticTextChunker};
pub use resource_manager::{ExecutionMode, ResourceError, ResourceManager, ResourceSnapshot};
pub use stream::{
    ChatPriorityConfig, ChatPriorityEngine, ChatPriorityItem, DirectorDecision, StreamDirector,
    StreamDirectorConfig,
};
pub use tools::{KnowledgeLearner, TeachingFact, WebSearchResult, WebSearchTool};
pub use vision::{
    CaptureFrame, MockScreenCaptureProvider, MockVisionProvider, OllamaVisionProvider,
    ScreenCaptureProvider, VisionObservation, VisionProvider, VisionRouter, VisionRouterConfig,
    VisualDiffDetector, WindowsCaptureConfig, WindowsScreenCaptureProvider,
};
