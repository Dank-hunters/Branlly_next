//! Asynchronous Ollama API client with cancellation-safe NDJSON streaming.

use std::{pin::Pin, time::Duration};

use async_stream::try_stream;
use branlly_core::Message;
use futures_util::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use url::Url;

/// Errors returned by configuration, transport, protocol, or cancellation.
#[derive(Debug, Error)]
pub enum OllamaError {
    /// Base endpoint or model configuration is invalid.
    #[error("invalid Ollama configuration: {0}")]
    Configuration(String),
    /// HTTP transport failed.
    #[error("Ollama transport failure: {0}")]
    Transport(#[from] reqwest::Error),
    /// Server returned a non-success status.
    #[error("Ollama returned HTTP {status}: {body}")]
    Http {
        /// HTTP status code.
        status: StatusCode,
        /// Bounded response body or status explanation.
        body: String,
    },
    /// A streaming frame is malformed.
    #[error("invalid Ollama stream frame: {0}")]
    Protocol(String),
    /// Caller cancelled the in-flight request.
    #[error("Ollama request cancelled")]
    Cancelled,
}

/// A model advertised by `/api/tags`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Ollama model name including optional tag.
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [Message],
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatFrame {
    #[serde(default)]
    message: Option<Message>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

/// Heap-safe stream type consumed incrementally by Tauri.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<String, OllamaError>> + Send>>;

/// Reusable Ollama client. Cloning shares reqwest's internal connection pool.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    http: Client,
    base_url: Url,
    model: String,
}

impl OllamaClient {
    /// Creates a client with strict endpoint and model validation.
    ///
    /// # Errors
    ///
    /// Returns [`OllamaError::Configuration`] for an invalid endpoint/model or
    /// [`OllamaError::Transport`] when the HTTP client cannot be constructed.
    pub fn new(
        base_url: &str,
        model: impl Into<String>,
        request_timeout: Duration,
    ) -> Result<Self, OllamaError> {
        let mut base_url =
            Url::parse(base_url).map_err(|error| OllamaError::Configuration(error.to_string()))?;
        if !matches!(base_url.scheme(), "http" | "https") {
            return Err(OllamaError::Configuration(
                "endpoint must use http or https".to_owned(),
            ));
        }
        if base_url.host_str().is_none() {
            return Err(OllamaError::Configuration(
                "endpoint must include a host".to_owned(),
            ));
        }
        if !base_url.path().ends_with('/') {
            let normalized = format!("{}/", base_url.path());
            base_url.set_path(&normalized);
        }
        let model = model.into().trim().to_owned();
        if model.is_empty() {
            return Err(OllamaError::Configuration(
                "model must not be empty".to_owned(),
            ));
        }
        let http = Client::builder().timeout(request_timeout).build()?;
        Ok(Self {
            http,
            base_url,
            model,
        })
    }

    /// Returns the configured model.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Lists models currently installed in Ollama.
    ///
    /// # Errors
    ///
    /// Returns a transport, HTTP status, or JSON protocol error.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, OllamaError> {
        let response = self.http.get(self.endpoint("api/tags")?).send().await?;
        let response = ensure_success(response).await?;
        Ok(response.json::<TagsResponse>().await?.models)
    }

    /// Returns whether the strictly configured model is installed.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::list_models`].
    pub async fn configured_model_is_available(&self) -> Result<bool, OllamaError> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|model| model.name == self.model))
    }

    /// Starts a streamed chat request and yields textual deltas.
    ///
    /// NDJSON records are buffered across arbitrary TCP chunk boundaries. Dropping
    /// the returned stream also drops the response body and cancels network work.
    ///
    /// # Errors
    ///
    /// Returns configuration, transport, HTTP status, cancellation, or stream protocol errors.
    pub async fn chat(
        &self,
        messages: &[Message],
        cancellation: CancellationToken,
    ) -> Result<ChatStream, OllamaError> {
        if messages.is_empty() {
            return Err(OllamaError::Configuration(
                "chat context must not be empty".to_owned(),
            ));
        }
        let request = ChatRequest {
            model: &self.model,
            messages,
            stream: true,
        };
        let response = tokio::select! {
            () = cancellation.cancelled() => return Err(OllamaError::Cancelled),
            response = self.http.post(self.endpoint("api/chat")?).json(&request).send() => response?,
        };
        let response = ensure_success(response).await?;
        let mut bytes = response.bytes_stream();

        let output = try_stream! {
            let mut buffer = Vec::<u8>::with_capacity(4096);
            loop {
                let selected = tokio::select! {
                    () = cancellation.cancelled() => None,
                    next = bytes.next() => Some(next),
                };
                let next = match selected {
                    Some(next) => next,
                    None => Err(OllamaError::Cancelled)?,
                };
                let Some(chunk) = next else { break };
                buffer.extend_from_slice(&chunk?);

                while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                    let line: Vec<u8> = buffer.drain(..=newline).collect();
                    if let Some(delta) = decode_frame(&line)? {
                        yield delta;
                    }
                }
            }
            if buffer.iter().any(|byte| !byte.is_ascii_whitespace()) {
                if let Some(delta) = decode_frame(&buffer)? {
                    yield delta;
                }
            }
        };
        Ok(Box::pin(output))
    }

    fn endpoint(&self, path: &str) -> Result<Url, OllamaError> {
        self.base_url
            .join(path)
            .map_err(|error| OllamaError::Configuration(error.to_string()))
    }
}

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response, OllamaError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let body = response
        .text()
        .await
        .unwrap_or_else(|error| error.to_string());
    let body: String = body.chars().take(1_024).collect();
    Err(OllamaError::Http { status, body })
}

fn decode_frame(line: &[u8]) -> Result<Option<String>, OllamaError> {
    let line = line
        .strip_suffix(b"\n")
        .unwrap_or(line)
        .strip_suffix(b"\r")
        .unwrap_or(line);
    if line.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }
    let frame: ChatFrame =
        serde_json::from_slice(line).map_err(|error| OllamaError::Protocol(error.to_string()))?;
    if let Some(error) = frame.error {
        return Err(OllamaError::Protocol(error));
    }
    if frame.done {
        return Ok(None);
    }
    Ok(frame
        .message
        .and_then(|message| (!message.content.is_empty()).then_some(message.content)))
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn rejects_non_http_endpoint_and_blank_model() {
        assert!(OllamaClient::new("file:///tmp/socket", "qwen", Duration::from_secs(1)).is_err());
        assert!(OllamaClient::new("http://localhost:11434", " ", Duration::from_secs(1)).is_err());
    }

    #[test]
    fn decodes_content_done_and_server_errors() {
        assert_eq!(
            decode_frame(
                b"{\"message\":{\"role\":\"assistant\",\"content\":\"Salut\"},\"done\":false}\n"
            )
            .map_err(|error| error.to_string()),
            Ok(Some("Salut".to_owned()))
        );
        assert_eq!(
            decode_frame(b"{\"done\":true}\n").map_err(|error| error.to_string()),
            Ok(None)
        );
        assert!(decode_frame(b"{\"error\":\"model missing\"}\n").is_err());
    }
}
