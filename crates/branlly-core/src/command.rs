use serde::{Deserialize, Serialize};

/// Intents emitted by the domain and handled by platform adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum BranllyCommand {
    /// Launch an application by a stable desktop identifier or executable name.
    LaunchApplication(String),
    /// Display the platform's available windows.
    ListWindows,
    /// Focus a window selected by its opaque platform identifier.
    FocusWindow(String),
    /// Request graceful closure of a window.
    CloseWindow(String),
    /// Open the chat view without invoking the model yet.
    OpenChat,
    /// Open one of the bundled games.
    OpenGame(String),
}
