//! HTTP contract tests using an isolated mock Ollama server.

use std::time::Duration;

use branlly_core::{Message, Role};
use branlly_ollama::OllamaClient;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

#[tokio::test]
async fn lists_models_and_requires_exact_configured_tag() -> Result<(), Box<dyn std::error::Error>>
{
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [{"name": "qwen2.5:3b"}, {"name": "llama3.2:1b"}]
        })))
        .mount(&server)
        .await;

    let client = OllamaClient::new(&server.uri(), "qwen2.5:3b", Duration::from_secs(2))?;
    assert!(client.configured_model_is_available().await?);
    assert_eq!(client.list_models().await?.len(), 2);
    Ok(())
}

#[tokio::test]
async fn streams_ndjson_content_in_order() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    let context = vec![Message::new(Role::System, "Tu es Branlly")];
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_json(serde_json::json!({
            "model": "qwen2.5:3b",
            "messages": context,
            "stream": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            concat!(
                "{\"message\":{\"role\":\"assistant\",\"content\":\"Hmm. \"},\"done\":false}\n",
                "{\"message\":{\"role\":\"assistant\",\"content\":\"Bonjour.\"},\"done\":false}\n",
                "{\"done\":true}\n"
            ),
            "application/x-ndjson",
        ))
        .mount(&server)
        .await;

    let client = OllamaClient::new(&server.uri(), "qwen2.5:3b", Duration::from_secs(2))?;
    let mut stream = client.chat(&context, CancellationToken::new()).await?;
    let mut output = String::new();
    while let Some(delta) = stream.next().await {
        output.push_str(&delta?);
    }
    assert_eq!(output, "Hmm. Bonjour.");
    Ok(())
}

#[tokio::test]
async fn cancellation_before_send_is_reported() -> Result<(), Box<dyn std::error::Error>> {
    let client = OllamaClient::new(
        "http://127.0.0.1:11434",
        "qwen2.5:3b",
        Duration::from_secs(2),
    )?;
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let error = client
        .chat(&[Message::new(Role::System, "test")], cancellation)
        .await;
    assert!(matches!(error, Err(branlly_ollama::OllamaError::Cancelled)));
    Ok(())
}
