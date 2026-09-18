use std::collections::VecDeque;
use std::sync::Mutex;

use crate::provider::{LlmError, LlmProvider, LlmRequest, LlmResponse, LlmUsage};

/// Production-ready Mock LLM Provider for unit and integration testing.
///
/// Under Skill 20 (LLM Provider Engineering):
/// - Allows testing without network or external API keys.
/// - Supports canned responses queue (different responses across turns).
/// - Supports simulated errors (rate limits, timeouts) to test runtime resilience.
/// - Records all incoming requests for test assertions.
pub struct MockLlmProvider {
    pub default_response: String,
    canned_responses: Mutex<VecDeque<String>>,
    simulated_error: Mutex<Option<LlmError>>,
    recorded_requests: Mutex<Vec<LlmRequest>>,
}

impl MockLlmProvider {
    /// Create a new MockLlmProvider with a default response.
    pub fn new(default_response: impl Into<String>) -> Self {
        Self {
            default_response: default_response.into(),
            canned_responses: Mutex::new(VecDeque::new()),
            simulated_error: Mutex::new(None),
            recorded_requests: Mutex::new(Vec::new()),
        }
    }

    /// Create a MockLlmProvider with a predetermined sequence of responses.
    pub fn with_responses(responses: Vec<String>) -> Self {
        let default_response = responses.first().cloned().unwrap_or_else(|| "Default mock response".into());
        let deque = VecDeque::from(responses);
        Self {
            default_response,
            canned_responses: Mutex::new(deque),
            simulated_error: Mutex::new(None),
            recorded_requests: Mutex::new(Vec::new()),
        }
    }

    /// Create a MockLlmProvider that immediately returns an error.
    pub fn failing(error: LlmError) -> Self {
        Self {
            default_response: String::new(),
            canned_responses: Mutex::new(VecDeque::new()),
            simulated_error: Mutex::new(Some(error)),
            recorded_requests: Mutex::new(Vec::new()),
        }
    }

    /// Queue an additional canned response to be returned on subsequent calls.
    pub fn push_canned_response(&self, response: impl Into<String>) {
        let mut canned = self.canned_responses.lock().expect("Lock poisoned");
        canned.push_back(response.into());
    }

    /// Inject or clear a simulated error.
    pub fn set_simulated_error(&self, error: Option<LlmError>) {
        let mut sim_err = self.simulated_error.lock().expect("Lock poisoned");
        *sim_err = error;
    }

    /// Get a snapshot of all recorded requests received by this mock.
    pub fn recorded_requests(&self) -> Vec<LlmRequest> {
        let recorded = self.recorded_requests.lock().expect("Lock poisoned");
        recorded.clone()
    }

    /// Get the most recently recorded request, if any.
    pub fn last_request(&self) -> Option<LlmRequest> {
        let recorded = self.recorded_requests.lock().expect("Lock poisoned");
        recorded.last().cloned()
    }

    /// Get the count of recorded requests.
    pub fn request_count(&self) -> usize {
        let recorded = self.recorded_requests.lock().expect("Lock poisoned");
        recorded.len()
    }

    /// Clear the recorded requests history.
    pub fn clear_recorded(&self) {
        let mut recorded = self.recorded_requests.lock().expect("Lock poisoned");
        recorded.clear();
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new("Hello from MockLlmProvider!")
    }
}

impl LlmProvider for MockLlmProvider {
    fn generate_text(&self, request: LlmRequest) -> vc_core::Result<LlmResponse> {
        // 1. Record incoming request for verification
        {
            let mut recorded = self.recorded_requests.lock().expect("Lock poisoned");
            recorded.push(request.clone());
        }

        // 2. Check for simulated error
        {
            let sim_err = self.simulated_error.lock().expect("Lock poisoned");
            if let Some(ref err) = *sim_err {
                return Err(err.clone().into());
            }
        }

        // 3. Check for canned responses
        let text = {
            let mut canned = self.canned_responses.lock().expect("Lock poisoned");
            canned.pop_front().unwrap_or_else(|| self.default_response.clone())
        };

        // 4. Return mock response with estimated tokens
        let prompt_tokens = (request.prompt.chars().count() / 4).max(1) as u32;
        let completion_tokens = (text.chars().count() / 4).max(1) as u32;

        Ok(LlmResponse {
            text,
            usage: Some(LlmUsage::new(prompt_tokens, completion_tokens)),
            finish_reason: Some("STOP".into()),
        })
    }

    fn name(&self) -> &'static str {
        "MockLlmProvider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_default_response_and_recording() {
        let mock = MockLlmProvider::new("Default reply");
        assert_eq!(mock.request_count(), 0);

        let req = LlmRequest::new("Test prompt").with_system_instruction("System prompt");
        let res = mock.generate_text(req.clone()).expect("Should succeed");

        assert_eq!(res.text, "Default reply");
        assert_eq!(mock.request_count(), 1);
        assert_eq!(mock.last_request(), Some(req));
        assert!(res.usage.is_some());
    }

    #[test]
    fn test_mock_sequence_of_canned_responses() {
        let mock = MockLlmProvider::with_responses(vec![
            "First response".into(),
            "Second response".into(),
        ]);

        let res1 = mock.generate_text(LlmRequest::new("Turn 1")).unwrap();
        assert_eq!(res1.text, "First response");

        let res2 = mock.generate_text(LlmRequest::new("Turn 2")).unwrap();
        assert_eq!(res2.text, "Second response");

        // Third call falls back to default_response
        let res3 = mock.generate_text(LlmRequest::new("Turn 3")).unwrap();
        assert_eq!(res3.text, "First response");
    }

    #[test]
    fn test_mock_simulated_error() {
        let mock = MockLlmProvider::failing(LlmError::Timeout("Connection dropped".into()));
        let res = mock.generate_text(LlmRequest::new("Will fail"));
        assert!(res.is_err());

        // Clear error and verify it recovers
        mock.set_simulated_error(None);
        mock.push_canned_response("Recovered reply");
        let res2 = mock.generate_text(LlmRequest::new("Will succeed")).unwrap();
        assert_eq!(res2.text, "Recovered reply");
    }
}
