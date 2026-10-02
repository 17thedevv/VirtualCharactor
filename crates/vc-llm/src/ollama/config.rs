use std::time::Duration;

/// Default Ollama HTTP server endpoint.
pub const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";

/// Default chat model for local execution on personal hardware (RTX 3050 4GB).
pub const DEFAULT_OLLAMA_CHAT_MODEL: &str = "qwen2.5:3b";

/// Configuration for the Ollama backend service.
///
/// Centralizes all endpoint URLs, model identifiers, timeout parameters, and retry policies.
#[derive(Clone, Debug, PartialEq)]
pub struct OllamaConfig {
    /// Base URL of the Ollama server (e.g. `http://127.0.0.1:11434`).
    pub base_url: String,

    /// Chat model identifier (e.g. `qwen2.5:3b` or `qwen2.5:3b-instruct-q4_K_M`).
    pub model: String,

    /// Optional default sampling temperature (0.0 - 2.0).
    pub temperature: Option<f32>,

    /// Maximum generation tokens / context prediction limit.
    pub max_tokens: Option<u32>,

    /// Network timeout duration for request completion.
    pub timeout: Duration,

    /// Maximum retries on transient network or 503 errors.
    pub max_retries: usize,

    /// Keep-alive duration for loaded models in Ollama memory (e.g., `"5m"`, `"10m"`, `"-1"`).
    pub keep_alive: Option<String>,
}

impl OllamaConfig {
    /// Create a new configuration with default endpoint and model.
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            model: DEFAULT_OLLAMA_CHAT_MODEL.to_string(),
            temperature: None,
            max_tokens: None,
            timeout: Duration::from_secs(60),
            max_retries: 2,
            keep_alive: Some("5m".to_string()),
        }
    }

    /// Set a custom base URL (e.g., for mock tests or remote instances).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }
        self.base_url = url;
        self
    }

    /// Set the target model name.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set maximum tokens limit.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set network timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum retry count.
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set Ollama model keep-alive duration in memory.
    pub fn with_keep_alive(mut self, keep_alive: impl Into<String>) -> Self {
        self.keep_alive = Some(keep_alive.into());
        self
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self::new()
    }
}
