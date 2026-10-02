use crate::ollama::client::{
    OllamaChatMessage, OllamaChatOptions, OllamaChatRequest, OllamaClient,
};
use crate::ollama::config::OllamaConfig;
use crate::provider::{LlmProvider, LlmRequest, LlmResponse, LlmUsage};

/// Production implementation of `LlmProvider` backed by an Ollama server.
///
/// Adapts generic `LlmRequest` instances into Ollama `/api/chat` calls,
/// decoupling the character runtime and domain core from Ollama-specific REST APIs.
pub struct OllamaChatProvider {
    config: OllamaConfig,
    client: OllamaClient,
}

impl OllamaChatProvider {
    /// Create an `OllamaChatProvider` from the given configuration.
    pub fn new(config: OllamaConfig) -> Self {
        let client = OllamaClient::new(&config.base_url, config.timeout);
        Self { config, client }
    }

    /// Create an `OllamaChatProvider` with a specific model and default settings.
    pub fn with_model(model: impl Into<String>) -> Self {
        let config = OllamaConfig::new().with_model(model);
        Self::new(config)
    }

    /// Access the underlying configuration.
    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }

    /// Access the target model name.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Access the base URL.
    pub fn base_url(&self) -> &str {
        self.client.base_url()
    }

    /// Check if the Ollama service is reachable.
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }

    /// Evict this chat model from VRAM immediately using keep_alive: 0.
    pub fn unload(&self) -> Result<(), crate::provider::LlmError> {
        self.client.unload_model(&self.config.model)
    }
}

impl Default for OllamaChatProvider {
    fn default() -> Self {
        Self::new(OllamaConfig::default())
    }
}

impl std::fmt::Debug for OllamaChatProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OllamaChatProvider")
            .field("config", &self.config)
            .finish()
    }
}

impl LlmProvider for OllamaChatProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse> {
        let mut messages = Vec::new();

        // 1. Map system instruction if provided
        if let Some(system_content) = request.system_instruction {
            if !system_content.trim().is_empty() {
                messages.push(OllamaChatMessage::system(system_content));
            }
        }

        // 2. Map user prompt
        messages.push(OllamaChatMessage::user(request.prompt));

        // 3. Assemble options (temperature and max_tokens)
        let temperature = request.temperature.or(self.config.temperature);
        let num_predict = request.max_tokens.or(self.config.max_tokens);

        let options = if temperature.is_some() || num_predict.is_some() {
            Some(OllamaChatOptions {
                temperature,
                num_predict,
            })
        } else {
            None
        };

        // 4. Build Ollama Chat request DTO
        let chat_request = OllamaChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: false,
            options,
            keep_alive: self.config.keep_alive.clone(),
        };

        // 5. Send chat request through HTTP client
        let response = self
            .client
            .send_chat(&chat_request, self.config.max_retries)
            .map_err(|err| -> vc_core::CoreError { err.into() })?;

        // 6. Extract generated message content
        let text = response.message.map(|m| m.content).unwrap_or_default();

        // 7. Extract token usage metadata
        let usage = if response.prompt_eval_count.is_some() || response.eval_count.is_some() {
            let prompt_tokens = response.prompt_eval_count;
            let completion_tokens = response.eval_count;
            let total_tokens = match (prompt_tokens, completion_tokens) {
                (Some(p), Some(c)) => Some(p + c),
                (Some(p), None) => Some(p),
                (None, Some(c)) => Some(c),
                (None, None) => None,
            };

            Some(LlmUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            })
        } else {
            None
        };

        // 8. Extract finish reason if provided
        let finish_reason = response.done_reason;

        Ok(LlmResponse {
            text,
            usage,
            finish_reason,
        })
    }

    fn stream_text(
        &self,
        request: LlmRequest,
    ) -> Result<crate::provider::LlmTokenStream, crate::provider::LlmError> {
        let mut messages = Vec::new();

        if let Some(system_content) = request.system_instruction {
            if !system_content.trim().is_empty() {
                messages.push(OllamaChatMessage::system(system_content));
            }
        }

        messages.push(OllamaChatMessage::user(request.prompt));

        let temperature = request.temperature.or(self.config.temperature);
        let num_predict = request.max_tokens.or(self.config.max_tokens);

        let options = if temperature.is_some() || num_predict.is_some() {
            Some(OllamaChatOptions {
                temperature,
                num_predict,
            })
        } else {
            None
        };

        let chat_request = OllamaChatRequest {
            model: self.config.model.clone(),
            messages,
            stream: true,
            options,
            keep_alive: self.config.keep_alive.clone(),
        };

        let stream_iter = self.client.stream_chat(&chat_request)?;
        Ok(Box::new(stream_iter))
    }

    fn name(&self) -> &'static str {
        "OllamaChatProvider"
    }
}
