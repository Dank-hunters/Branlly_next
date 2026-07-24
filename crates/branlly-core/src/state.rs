use serde::{Deserialize, Serialize};

use crate::{BranllyConfig, CoreError, LaunchItem, Message, Role};

/// Current emotional presentation. It has no platform dependency.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    /// Low energy and minimal animation.
    Sleepy,
    /// Default presentation.
    #[default]
    Neutral,
    /// User interaction is in progress.
    Curious,
    /// A useful exchange completed successfully.
    Happy,
    /// A recoverable error occurred.
    Irritated,
}

/// Complete serializable domain state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranllyState {
    config: BranllyConfig,
    mood: Mood,
    energy: u8,
    conversation: Vec<Message>,
    #[serde(default)]
    launch_items: Vec<LaunchItem>,
    #[serde(default)]
    launcher_initialized: bool,
}

impl BranllyState {
    /// Builds a fresh state after checking every configuration invariant.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidConfiguration`] when `config` is invalid.
    pub fn new(config: BranllyConfig) -> Result<Self, CoreError> {
        config.validate()?;
        let energy = config.initial_energy;
        Ok(Self {
            config,
            mood: Mood::Neutral,
            energy,
            conversation: Vec::new(),
            launch_items: Vec::new(),
            launcher_initialized: false,
        })
    }

    /// Revalidates data loaded from an untrusted persistence source.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidConfiguration`] when persisted invariants are violated.
    pub fn validate(&self) -> Result<(), CoreError> {
        self.config.validate()?;
        if self.energy > 100 {
            return Err(CoreError::InvalidConfiguration(
                "persisted energy must be between 0 and 100".to_owned(),
            ));
        }
        if self.conversation.len() > self.config.history_limit {
            return Err(CoreError::InvalidConfiguration(
                "persisted conversation exceeds history limit".to_owned(),
            ));
        }
        if self
            .launch_items
            .iter()
            .any(|item| item.validate().is_err())
            || self
                .launch_items
                .windows(2)
                .any(|pair| pair[0].order >= pair[1].order)
            || self.launch_items.iter().enumerate().any(|(index, item)| {
                self.launch_items[..index]
                    .iter()
                    .any(|other| other.id == item.id)
            })
        {
            return Err(CoreError::InvalidConfiguration(
                "persisted launcher items are invalid".to_owned(),
            ));
        }
        if self
            .conversation
            .iter()
            .any(|message| message.content.trim().is_empty() || message.role == Role::System)
        {
            return Err(CoreError::InvalidConfiguration(
                "persisted conversation contains an invalid message".to_owned(),
            ));
        }
        Ok(())
    }

    /// Returns immutable shared configuration.
    #[must_use]
    pub const fn config(&self) -> &BranllyConfig {
        &self.config
    }

    /// Returns the current mood.
    #[must_use]
    pub const fn mood(&self) -> Mood {
        self.mood
    }

    /// Returns energy in the inclusive range 0..=100.
    #[must_use]
    pub const fn energy(&self) -> u8 {
        self.energy
    }

    /// Returns active user/assistant context in chronological order.
    #[must_use]
    pub fn conversation(&self) -> &[Message] {
        &self.conversation
    }

    /// Returns launcher entries in stable display order.
    #[must_use]
    pub fn launch_items(&self) -> &[LaunchItem] {
        &self.launch_items
    }

    /// Replaces launcher entries after validation and stable ordering.
    /// Whether the default launcher migration was completed.
    #[must_use]
    pub const fn launcher_initialized(&self) -> bool {
        self.launcher_initialized
    }

    /// Replaces launcher entries after validation and stable ordering.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidConfiguration`] for duplicate or invalid entries, or when the
    /// item count cannot be represented by the persisted ordering type.
    pub fn set_launch_items(&mut self, mut items: Vec<LaunchItem>) -> Result<(), CoreError> {
        items.sort_by_key(|item| item.order);
        for (index, item) in items.iter_mut().enumerate() {
            item.order = u32::try_from(index).map_err(|_| {
                CoreError::InvalidConfiguration("too many launcher items".to_owned())
            })?;
            item.validate()?;
        }
        if items
            .iter()
            .enumerate()
            .any(|(index, item)| items[..index].iter().any(|other| other.id == item.id))
        {
            return Err(CoreError::InvalidConfiguration(
                "duplicate launcher item".to_owned(),
            ));
        }
        self.launch_items = items;
        self.launcher_initialized = true;
        Ok(())
    }

    /// Produces the complete request context, including the system prompt.
    #[must_use]
    pub fn chat_context(&self) -> Vec<Message> {
        let mut messages = Vec::with_capacity(self.conversation.len() + 1);
        messages.push(Message::new(Role::System, &self.config.system_prompt));
        messages.extend(self.conversation.iter().cloned());
        messages
    }

    /// Records normalized user input and transitions to a listening state.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyMessage`] when input only contains whitespace.
    pub fn record_user_message(&mut self, content: impl Into<String>) -> Result<(), CoreError> {
        self.push_message(Message::new(Role::User, content))?;
        self.mood = Mood::Curious;
        self.energy = self.energy.saturating_sub(1);
        Ok(())
    }

    /// Records a model response and transitions to a positive state.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyMessage`] when output only contains whitespace.
    pub fn record_assistant_message(
        &mut self,
        content: impl Into<String>,
    ) -> Result<(), CoreError> {
        self.push_message(Message::new(Role::Assistant, content))?;
        self.mood = Mood::Happy;
        self.energy = self.energy.saturating_add(1).min(100);
        Ok(())
    }

    /// Records a recoverable failure without corrupting conversation history.
    pub fn mark_recoverable_error(&mut self) {
        self.mood = Mood::Irritated;
        self.energy = self.energy.saturating_sub(2);
    }

    /// Applies passive energy decay and derives the sleepy state at zero.
    pub fn idle_tick(&mut self) {
        self.energy = self.energy.saturating_sub(1);
        self.mood = if self.energy == 0 {
            Mood::Sleepy
        } else {
            Mood::Neutral
        };
    }

    fn push_message(&mut self, message: Message) -> Result<(), CoreError> {
        if message.content.is_empty() {
            return Err(CoreError::EmptyMessage);
        }
        self.conversation.push(message);
        let overflow = self
            .conversation
            .len()
            .saturating_sub(self.config.history_limit);
        if overflow > 0 {
            self.conversation.drain(..overflow);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_limit(history_limit: usize) -> BranllyState {
        let config = BranllyConfig {
            history_limit,
            ..BranllyConfig::default()
        };
        match BranllyState::new(config) {
            Ok(state) => state,
            Err(error) => unreachable!("test configuration must be valid: {error}"),
        }
    }

    #[test]
    fn launcher_items_are_stable_and_reject_duplicates() {
        let mut state = state_with_limit(4);
        let item = LaunchItem {
            id: "firefox".to_owned(),
            kind: crate::LaunchItemKind::Application,
            name: "Firefox".to_owned(),
            icon: None,
            order: 9,
            platform: None,
            launch: crate::LaunchConfiguration::Application {
                identifier: "firefox".to_owned(),
                arguments: vec![],
            },
        };
        assert!(state.set_launch_items(vec![item.clone()]).is_ok());
        assert_eq!(state.launch_items()[0].order, 0);
        assert!(state.set_launch_items(vec![item.clone(), item]).is_err());
    }

    #[test]
    fn chat_context_always_starts_with_system_prompt() {
        let state = state_with_limit(4);
        let context = state.chat_context();
        assert_eq!(context.len(), 1);
        assert_eq!(context[0].role, Role::System);
        assert!(!context[0].content.is_empty());
    }

    #[test]
    fn messages_are_trimmed_and_history_is_bounded() {
        let mut state = state_with_limit(2);
        assert_eq!(state.record_user_message("  un  "), Ok(()));
        assert_eq!(state.record_assistant_message("deux"), Ok(()));
        assert_eq!(state.record_user_message("trois"), Ok(()));

        assert_eq!(state.conversation().len(), 2);
        assert_eq!(state.conversation()[0].content, "deux");
        assert_eq!(state.conversation()[1].content, "trois");
    }

    #[test]
    fn empty_messages_do_not_mutate_state() {
        let mut state = state_with_limit(4);
        let before = state.clone();
        assert_eq!(
            state.record_user_message(" \n "),
            Err(CoreError::EmptyMessage)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn energy_is_saturating_and_mood_transitions_are_deterministic() {
        let config = BranllyConfig {
            initial_energy: 1,
            ..BranllyConfig::default()
        };
        let mut state = match BranllyState::new(config) {
            Ok(state) => state,
            Err(error) => unreachable!("test configuration must be valid: {error}"),
        };

        state.idle_tick();
        state.idle_tick();
        assert_eq!(state.energy(), 0);
        assert_eq!(state.mood(), Mood::Sleepy);

        assert_eq!(state.record_assistant_message("Réveillé."), Ok(()));
        assert_eq!(state.energy(), 1);
        assert_eq!(state.mood(), Mood::Happy);
    }
}
