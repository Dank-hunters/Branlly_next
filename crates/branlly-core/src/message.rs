use serde::{Deserialize, Serialize};

/// A role accepted by Ollama's chat protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Instructions defining Branlly's behavior.
    System,
    /// A human message.
    User,
    /// A Branlly response.
    Assistant,
}

/// A normalized conversation entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Author role.
    pub role: Role,
    /// UTF-8 text with leading and trailing whitespace removed.
    pub content: String,
}

impl Message {
    /// Creates a normalized message. Validation is performed by `BranllyState`.
    #[must_use]
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into().trim().to_owned(),
        }
    }
}
