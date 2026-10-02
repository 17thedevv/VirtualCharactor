use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::time::Duration;

use crate::provider::LlmError;

/// Internal message representation for the Ollama chat API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OllamaChatMessage {
    pub role: String,
    pub content: String,
}

impl OllamaChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Generation parameter options passed to Ollama.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct OllamaChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
}

/// Request payload for the `/api/chat` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaChatMessage>,
    pub stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaChatOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

/// Response payload returned by the `/api/chat` endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaChatResponse {
    pub model: Option<String>,
    pub created_at: Option<String>,
    pub message: Option<OllamaChatMessage>,
    pub done: Option<bool>,
    pub done_reason: Option<String>,
    pub prompt_eval_count: Option<u32>,
    pub eval_count: Option<u32>,
    pub error: Option<String>,
}

/// Pure HTTP transport client for the Ollama backend service.
///
/// Strictly decoupled from all Character Core domain concepts (Personality, Memory, State).
pub struct OllamaClient {
    base_url: String,
    agent: ureq::Agent,
}

impl OllamaClient {
    /// Create a new Ollama HTTP client.
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Self {
        let mut url = base_url.into();
        if url.ends_with('/') {
            url.pop();
        }

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(timeout)
            .timeout_write(timeout)
            .build();

        Self {
            base_url: url,
            agent,
        }
    }

    /// Return the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check whether the Ollama daemon is reachable and responding.
    pub fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.agent.get(&url).call().is_ok()
    }

    /// Evict a model from VRAM immediately by issuing an empty request with keep_alive: 0.
    /// This is vital for 4GB VRAM hardware to avoid CUDA out-of-memory errors when switching models.
    pub fn unload_model(&self, model: &str) -> Result<(), LlmError> {
        let url = format!("{}/api/chat", self.base_url);
        let payload = serde_json::json!({
            "model": model,
            "messages": [],
            "keep_alive": 0
        });

        match self.agent.post(&url).send_json(payload) {
            Ok(_) => Ok(()),
            Err(e) => Err(LlmError::Other(format!(
                "Failed to unload model '{}': {}",
                model, e
            ))),
        }
    }

    /// Send a chat completion request to `/api/chat` with retry support.
    pub fn send_chat(
        &self,
        request: &OllamaChatRequest,
        max_retries: usize,
    ) -> Result<OllamaChatResponse, LlmError> {
        let url = format!("{}/api/chat", self.base_url);

        let mut attempts = 0;
        loop {
            attempts += 1;
            match self.agent.post(&url).send_json(request) {
                Ok(response) => {
                    let chat_resp: OllamaChatResponse = response.into_json().map_err(|e| {
                        LlmError::Other(format!("Failed to parse Ollama JSON response: {}", e))
                    })?;

                    if let Some(err_msg) = chat_resp.error {
                        return Err(LlmError::Other(format!("Ollama server error: {}", err_msg)));
                    }

                    return Ok(chat_resp);
                }
                Err(ureq::Error::Status(code, resp)) => {
                    let error_text = resp
                        .into_string()
                        .unwrap_or_else(|_| format!("HTTP status {}", code));

                    // Transient 503 error retry with exponential backoff
                    if code == 503 && attempts <= max_retries {
                        let backoff = Duration::from_millis(300 * (1 << (attempts - 1)));
                        std::thread::sleep(backoff);
                        continue;
                    }

                    return Err(match code {
                        404 => LlmError::ModelUnavailable(format!(
                            "Model '{}' not found or Ollama route does not exist. (HTTP 404: {})",
                            request.model, error_text
                        )),
                        400 => LlmError::InvalidRequest(format!(
                            "Bad request to Ollama: {}",
                            error_text
                        )),
                        503 => LlmError::ModelUnavailable(format!(
                            "Ollama service temporarily unavailable: {}",
                            error_text
                        )),
                        _ => LlmError::Other(format!("Ollama HTTP {}: {}", code, error_text)),
                    });
                }
                Err(ureq::Error::Transport(transport_err)) => {
                    let err_str = transport_err.to_string();

                    // Check for connection refused / host unreachable
                    if err_str.contains("connection refused")
                        || err_str.contains("Connection refused")
                        || err_str.contains("No connection could be made")
                        || err_str.contains("failed to connect")
                    {
                        return Err(LlmError::ModelUnavailable(format!(
                            "Cannot connect to Ollama at '{}'. Ensure the Ollama service is running (e.g. 'ollama serve'). Details: {}",
                            self.base_url, err_str
                        )));
                    }

                    // Check for timeout
                    if err_str.contains("timed out") || err_str.contains("Timed out") {
                        return Err(LlmError::Timeout(format!(
                            "Ollama request timed out after connecting to '{}': {}",
                            self.base_url, err_str
                        )));
                    }

                    // Retry generic transport error if retries remain
                    if attempts <= max_retries {
                        let backoff = Duration::from_millis(250 * (1 << (attempts - 1)));
                        std::thread::sleep(backoff);
                        continue;
                    }

                    return Err(LlmError::NetworkError(format!(
                        "Ollama network transport error connecting to '{}': {}",
                        self.base_url, err_str
                    )));
                }
            }
        }
    }

    /// Send a chat completion request to `/api/chat` with stream: true, returning an iterator yielding token deltas.
    pub fn stream_chat(
        &self,
        request: &OllamaChatRequest,
    ) -> Result<OllamaStreamIterator, LlmError> {
        let url = format!("{}/api/chat", self.base_url);
        let mut req_stream = request.clone();
        req_stream.stream = true;

        let response = match self.agent.post(&url).send_json(&req_stream) {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let err_text = resp
                    .into_string()
                    .unwrap_or_else(|_| format!("HTTP status {}", code));
                return Err(match code {
                    404 => LlmError::ModelUnavailable(format!(
                        "Model '{}' not found or Ollama route does not exist. (HTTP 404: {})",
                        request.model, err_text
                    )),
                    400 => LlmError::InvalidRequest(format!("Bad request to Ollama: {}", err_text)),
                    503 => LlmError::ModelUnavailable(format!(
                        "Ollama service temporarily unavailable: {}",
                        err_text
                    )),
                    _ => LlmError::Other(format!("Ollama HTTP {}: {}", code, err_text)),
                });
            }
            Err(ureq::Error::Transport(err)) => {
                let err_str = err.to_string();
                if err_str.contains("connection refused")
                    || err_str.contains("Connection refused")
                    || err_str.contains("No connection could be made")
                    || err_str.contains("failed to connect")
                {
                    return Err(LlmError::ModelUnavailable(format!(
                        "Cannot connect to Ollama at '{}'. Ensure the Ollama service is running. Details: {}",
                        self.base_url, err_str
                    )));
                }
                return Err(LlmError::NetworkError(format!(
                    "Ollama network transport error connecting to '{}': {}",
                    self.base_url, err_str
                )));
            }
        };

        let reader = BufReader::new(response.into_reader());
        Ok(OllamaStreamIterator {
            reader,
            done: false,
        })
    }
}

/// An iterator reading newline-delimited JSON chunks from an Ollama `/api/chat` stream.
pub struct OllamaStreamIterator {
    reader: BufReader<Box<dyn std::io::Read + Send + Sync>>,
    done: bool,
}

impl Iterator for OllamaStreamIterator {
    type Item = Result<String, LlmError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut line = String::new();
        loop {
            line.clear();
            match self.reader.read_line(&mut line) {
                Ok(0) => {
                    self.done = true;
                    return None;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<OllamaChatResponse>(trimmed) {
                        Ok(chunk) => {
                            if let Some(err) = chunk.error {
                                self.done = true;
                                return Some(Err(LlmError::Other(format!(
                                    "Ollama stream error: {}",
                                    err
                                ))));
                            }
                            if chunk.done == Some(true) {
                                self.done = true;
                            }
                            if let Some(msg) = chunk.message {
                                if !msg.content.is_empty() {
                                    return Some(Ok(msg.content));
                                }
                            }
                            if self.done {
                                return None;
                            }
                        }
                        Err(e) => {
                            self.done = true;
                            return Some(Err(LlmError::Other(format!(
                                "Failed to parse Ollama JSON chunk: {}. Line: '{}'",
                                e, trimmed
                            ))));
                        }
                    }
                }
                Err(e) => {
                    self.done = true;
                    return Some(Err(LlmError::NetworkError(format!(
                        "Error reading from Ollama stream: {}",
                        e
                    ))));
                }
            }
        }
    }
}
