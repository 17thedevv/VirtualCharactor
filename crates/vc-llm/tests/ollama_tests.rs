use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use vc_core::CoreError;
use vc_llm::{LlmProvider, LlmRequest, OllamaChatProvider, OllamaConfig};

/// Start a robust mock HTTP server on an ephemeral port.
///
/// Handles reading full HTTP request headers and body according to Content-Length,
/// then sends the specified HTTP response.
fn spawn_mock_ollama_server(
    status_code: u16,
    status_text: &str,
    response_body: &str,
) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind ephemeral port");
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{}", port);

    let (tx, rx) = mpsc::channel();
    let status_text = status_text.to_string();
    let response_body = response_body.to_string();

    thread::spawn(move || {
        // Allow up to 3 connection attempts in case of retry
        for _ in 0..3 {
            if let Ok((stream, _)) = listener.accept() {
                let mut reader = BufReader::new(&stream);
                let mut content_length = 0;

                // 1. Read headers
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0
                        || line == "\r\n"
                        || line == "\n"
                    {
                        break;
                    }
                    let lower = line.to_lowercase();
                    if lower.starts_with("content-length:") {
                        if let Some(val) = line.split(':').nth(1) {
                            content_length = val.trim().parse::<usize>().unwrap_or(0);
                        }
                    }
                }

                // 2. Read body if Content-Length > 0
                let mut body_bytes = vec![0u8; content_length];
                if content_length > 0 {
                    let _ = reader.read_exact(&mut body_bytes);
                }
                let body = String::from_utf8_lossy(&body_bytes).to_string();
                let _ = tx.send(body);

                // 3. Write response
                let mut writer = &stream;
                let response = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status_code,
                    status_text,
                    response_body.len(),
                    response_body
                );
                let _ = writer.write_all(response.as_bytes());
                let _ = writer.flush();

                // Successful interaction
                break;
            }
        }
    });

    // Give OS brief moment to open listening socket
    thread::sleep(Duration::from_millis(20));

    (base_url, rx)
}

#[test]
fn test_ollama_chat_request_mapping_and_successful_response() {
    let mock_resp = r#"{
        "model": "qwen2.5:3b",
        "created_at": "2026-09-18T16:00:00Z",
        "message": {
            "role": "assistant",
            "content": "Chào bạn! Tôi là Aria."
        },
        "done": true,
        "done_reason": "stop",
        "prompt_eval_count": 42,
        "eval_count": 18
    }"#;

    let (base_url, body_rx) = spawn_mock_ollama_server(200, "OK", mock_resp);

    let config = OllamaConfig::new()
        .with_base_url(&base_url)
        .with_model("qwen2.5:3b")
        .with_temperature(0.7)
        .with_max_tokens(512);

    let provider = OllamaChatProvider::new(config);
    assert_eq!(provider.name(), "OllamaChatProvider");
    assert_eq!(provider.model(), "qwen2.5:3b");

    let request = LlmRequest::new("Bạn là ai?")
        .with_system_instruction("Bạn là Aria, một trợ lý AI thông minh.");

    let response = provider
        .generate_text(request)
        .expect("generate_text failed");

    // 1. Verify Response Mapping
    assert_eq!(response.text, "Chào bạn! Tôi là Aria.");
    assert_eq!(response.finish_reason.as_deref(), Some("stop"));

    let usage = response.usage.expect("Usage metadata expected");
    assert_eq!(usage.prompt_tokens, Some(42));
    assert_eq!(usage.completion_tokens, Some(18));
    assert_eq!(usage.total_tokens, Some(60));

    // 2. Verify Request Payload Captured by Server
    let captured_body = body_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("Server did not receive request body");

    let parsed_json: serde_json::Value =
        serde_json::from_str(&captured_body).expect("Invalid JSON payload sent to Ollama");

    assert_eq!(parsed_json["model"], "qwen2.5:3b");
    assert_eq!(parsed_json["stream"], false);

    let messages = parsed_json["messages"]
        .as_array()
        .expect("messages must be an array");
    assert_eq!(messages.len(), 2);

    // System instruction mapping
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(
        messages[0]["content"],
        "Bạn là Aria, một trợ lý AI thông minh."
    );

    // User prompt mapping
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "Bạn là ai?");

    // Options mapping (temperature and num_predict)
    assert_eq!(parsed_json["options"]["temperature"], 0.7);
    assert_eq!(parsed_json["options"]["num_predict"], 512);
}

#[test]
fn test_ollama_chat_request_without_system_instruction() {
    let mock_resp = r#"{
        "model": "qwen2.5:3b",
        "message": {
            "role": "assistant",
            "content": "Phản hồi thông thường"
        },
        "done": true
    }"#;

    let (base_url, body_rx) = spawn_mock_ollama_server(200, "OK", mock_resp);
    let provider = OllamaChatProvider::new(OllamaConfig::new().with_base_url(&base_url));

    let request = LlmRequest::new("Hello");
    let response = provider.generate_text(request).expect("Request failed");

    assert_eq!(response.text, "Phản hồi thông thường");

    let captured_body = body_rx
        .recv_timeout(Duration::from_secs(3))
        .expect("No body received");
    let parsed: serde_json::Value = serde_json::from_str(&captured_body).unwrap();
    let messages = parsed["messages"].as_array().unwrap();

    // Only user message should be sent
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["role"], "user");
    assert_eq!(messages[0]["content"], "Hello");
}

#[test]
fn test_ollama_error_response_in_json() {
    let mock_resp = r#"{
        "error": "model 'qwen2.5:99b' not found, try pulling it first"
    }"#;

    let (base_url, _rx) = spawn_mock_ollama_server(200, "OK", mock_resp);
    let provider = OllamaChatProvider::new(OllamaConfig::new().with_base_url(&base_url));

    let result = provider.generate_text(LlmRequest::new("Hi"));
    assert!(result.is_err());

    match result.unwrap_err() {
        CoreError::ProviderError(msg) => {
            assert!(msg.contains("model 'qwen2.5:99b' not found"));
        }
        other => panic!("Expected ProviderError, got {:?}", other),
    }
}

#[test]
fn test_ollama_http_404_model_not_found() {
    let mock_resp = r#"{"error": "model not found"}"#;
    let (base_url, _rx) = spawn_mock_ollama_server(404, "Not Found", mock_resp);

    let provider = OllamaChatProvider::new(
        OllamaConfig::new()
            .with_base_url(&base_url)
            .with_model("unknown_model"),
    );

    let result = provider.generate_text(LlmRequest::new("Test"));
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found") || err.to_string().contains("404"));
}

#[test]
fn test_ollama_http_400_bad_request() {
    let mock_resp = r#"{"error": "invalid parameter"}"#;
    let (base_url, _rx) = spawn_mock_ollama_server(400, "Bad Request", mock_resp);

    let provider = OllamaChatProvider::new(OllamaConfig::new().with_base_url(&base_url));

    let result = provider.generate_text(LlmRequest::new("Test"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Bad request") || err.to_string().contains("400"));
}

#[test]
fn test_ollama_http_500_internal_server_error() {
    let mock_resp = r#"{"error": "Internal GPU allocation failed"}"#;
    let (base_url, _rx) = spawn_mock_ollama_server(500, "Internal Server Error", mock_resp);

    let provider = OllamaChatProvider::new(
        OllamaConfig::new()
            .with_base_url(&base_url)
            .with_max_retries(0),
    );

    let result = provider.generate_text(LlmRequest::new("Test"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("500") || err.to_string().contains("Internal GPU"));
}

#[test]
fn test_ollama_malformed_json_response() {
    let mock_resp = "THIS IS NOT VALID JSON";
    let (base_url, _rx) = spawn_mock_ollama_server(200, "OK", mock_resp);

    let provider = OllamaChatProvider::new(OllamaConfig::new().with_base_url(&base_url));

    let result = provider.generate_text(LlmRequest::new("Test"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Failed to parse Ollama JSON")
            || err.to_string().contains("parse")
    );
}

#[test]
fn test_ollama_connection_refused() {
    let config = OllamaConfig::new()
        .with_base_url("http://127.0.0.1:1")
        .with_timeout(Duration::from_millis(500))
        .with_max_retries(0);

    let provider = OllamaChatProvider::new(config);
    let result = provider.generate_text(LlmRequest::new("Test"));

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Cannot connect to Ollama")
            || err_str.contains("connection")
            || err_str.contains("network")
    );
}

#[test]
fn test_ollama_client_direct_usage() {
    let (base_url, _rx) = spawn_mock_ollama_server(
        200,
        "OK",
        r#"{"model": "qwen2.5:3b", "message": {"role": "assistant", "content": "Direct client reply"}, "done": true}"#,
    );

    let client = vc_llm::ollama::OllamaClient::new(&base_url, Duration::from_secs(5));
    assert_eq!(client.base_url(), base_url);

    let req = vc_llm::ollama::client::OllamaChatRequest {
        model: "qwen2.5:3b".to_string(),
        messages: vec![vc_llm::ollama::client::OllamaChatMessage::user("Hi")],
        stream: false,
        options: None,
        keep_alive: None,
    };

    let resp = client.send_chat(&req, 0).expect("send_chat failed");
    assert_eq!(resp.message.unwrap().content, "Direct client reply");
}

#[test]
fn test_ollama_streaming_chat_chunks() {
    let ndjson_response = "{\"model\":\"qwen2.5:3b\",\"message\":{\"role\":\"assistant\",\"content\":\"Chào \"},\"done\":false}\n\
{\"model\":\"qwen2.5:3b\",\"message\":{\"role\":\"assistant\",\"content\":\"bạn!\"},\"done\":true,\"done_reason\":\"stop\"}\n";

    let (base_url, _rx) = spawn_mock_ollama_server(200, "OK", ndjson_response);

    let config = OllamaConfig::new()
        .with_base_url(&base_url)
        .with_model("qwen2.5:3b");
    let provider = OllamaChatProvider::new(config);

    let stream = provider
        .stream_text(LlmRequest::new("Hello"))
        .expect("Stream should start successfully");

    let tokens: Vec<String> = stream.map(|r| r.expect("token should be ok")).collect();
    assert_eq!(tokens, vec!["Chào ", "bạn!"]);
    assert_eq!(tokens.concat(), "Chào bạn!");
}
