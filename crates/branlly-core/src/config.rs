use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Local model used unless the user explicitly chooses another model.
pub const DEFAULT_LOCAL_MODEL: &str = "qwen2.5:3b";

/// Stable identity prompt. UI and platform details must not be inserted here.
pub const DEFAULT_SYSTEM_PROMPT: &str = "Tu es Branlly, un trombone de bureau anthropomorphe, utile, concis et légèrement flemmard. Tu réponds en français sauf demande contraire. Tu n'inventes jamais une action système et tu indiques clairement lorsqu'une capacité est indisponible.";

/// Domain-level configuration shared by every platform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranllyConfig {
    /// Ollama model identifier.
    pub model: String,
    /// Identity and behavioral instructions.
    pub system_prompt: String,
    /// Maximum number of user/assistant messages retained in active context.
    pub history_limit: usize,
    /// Initial energy from 0 to 100.
    pub initial_energy: u8,
}

impl Default for BranllyConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_LOCAL_MODEL.to_owned(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_owned(),
            history_limit: 24,
            initial_energy: 65,
        }
    }
}

impl BranllyConfig {
    /// Validates invariants before a state is created or restored.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidConfiguration`] when a value is blank or out of range.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.model.trim().is_empty() {
            return Err(CoreError::InvalidConfiguration(
                "model must not be empty".to_owned(),
            ));
        }
        if self.system_prompt.trim().is_empty() {
            return Err(CoreError::InvalidConfiguration(
                "system prompt must not be empty".to_owned(),
            ));
        }
        if !(2..=200).contains(&self.history_limit) {
            return Err(CoreError::InvalidConfiguration(
                "history limit must be between 2 and 200".to_owned(),
            ));
        }
        if self.initial_energy > 100 {
            return Err(CoreError::InvalidConfiguration(
                "initial energy must be between 0 and 100".to_owned(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_use_required_model() {
        let config = BranllyConfig::default();
        assert_eq!(config.model, "qwen2.5:3b");
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn rejects_blank_model_and_out_of_range_history() {
        let blank_model = BranllyConfig {
            model: "  ".to_owned(),
            ..BranllyConfig::default()
        };
        assert!(matches!(
            blank_model.validate(),
            Err(CoreError::InvalidConfiguration(_))
        ));

        let invalid_history = BranllyConfig {
            history_limit: 1,
            ..BranllyConfig::default()
        };
        assert!(invalid_history.validate().is_err());
    }
}
