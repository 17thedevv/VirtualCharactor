use vc_llm::ollama::{OllamaChatProvider, OllamaConfig};
use vc_llm::provider::{LlmProvider, LlmRequest};

/// REAL OLLAMA TEST
///
/// Connects to the local Ollama daemon at http://127.0.0.1:11434 with model `qwen2.5:3b`.
/// Streams real token chunks over HTTP and asserts token arrival without fake delays.
#[test]
fn test_real_local_ollama_streaming_e2e() {
    let config = OllamaConfig::new()
        .with_base_url("http://127.0.0.1:11434")
        .with_model("qwen2.5:3b");
    let provider = OllamaChatProvider::new(config);

    if !provider.is_available() {
        eprintln!("⚠️ REAL OLLAMA TEST = ENVIRONMENT-BLOCKED (daemon not reachable)");
        return;
    }

    let request = LlmRequest::new("Reply with the single word: 'Antigravity'.")
        .with_temperature(0.1)
        .with_max_tokens(20);

    let stream = match provider.stream_text(request) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to open real Ollama stream: {}", e);
        }
    };

    let mut token_count = 0;
    let mut full_output = String::new();

    for token_res in stream {
        let token = token_res.expect("Real Ollama stream yielded an error chunk");
        token_count += 1;
        full_output.push_str(&token);
    }

    println!(
        "✅ [REAL OLLAMA TEST] Successfully received {} tokens from local Ollama: '{}'",
        token_count,
        full_output.trim()
    );

    assert!(token_count > 0, "Real Ollama stream must produce tokens");
    assert!(
        full_output.to_lowercase().contains("antigravity"),
        "Real Ollama response must contain expected word, got: '{}'",
        full_output
    );
}
