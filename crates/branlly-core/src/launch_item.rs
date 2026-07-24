use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A user-configurable entry in the radial launcher.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchItem {
    /// Persistent unique identifier.
    pub id: String,
    /// Entry category.
    pub kind: LaunchItemKind,
    /// User-facing label.
    pub name: String,
    /// Optional system icon reference.
    pub icon: Option<String>,
    /// Stable display position.
    pub order: u32,
    /// Optional platform restriction.
    pub platform: Option<LaunchPlatform>,
    /// Structured launch configuration.
    pub launch: LaunchConfiguration,
}

/// Entry categories supported by the launcher model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchItemKind {
    /// An application executable and its structured arguments.
    Application,
    /// A future routine reference; no routine execution is implemented yet.
    Routine,
}

/// Platform restriction recorded with an item when needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchPlatform {
    /// Linux desktop entry.
    Linux,
    /// Windows Start Menu entry.
    Windows,
}

/// Structured configuration, intentionally never a shell command string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LaunchConfiguration {
    /// Executable identifier and already separated arguments.
    Application {
        /// Executable path or identifier.
        identifier: String,
        /// Arguments passed directly to the executable.
        arguments: Vec<String>,
    },
    /// Future routine identifier.
    Routine {
        /// Stable routine reference.
        routine_id: String,
    },
}

impl LaunchItem {
    /// Validates the persisted launcher item and its matching configuration.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidConfiguration`] for blank identifiers, invalid launch data,
    /// or a mismatch between item category and configuration.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.trim().is_empty() || self.name.trim().is_empty() {
            return Err(CoreError::InvalidConfiguration(
                "launcher item requires id and name".to_owned(),
            ));
        }
        match (&self.kind, &self.launch) {
            (LaunchItemKind::Application, LaunchConfiguration::Application { identifier, .. })
                if !identifier.trim().is_empty() && !identifier.contains('\0') =>
            {
                Ok(())
            }
            (LaunchItemKind::Routine, LaunchConfiguration::Routine { routine_id })
                if !routine_id.trim().is_empty() =>
            {
                Ok(())
            }
            _ => Err(CoreError::InvalidConfiguration(
                "invalid launcher item configuration".to_owned(),
            )),
        }
    }
}
