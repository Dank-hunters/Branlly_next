use serde::{Deserialize, Serialize};

use crate::{BranllyState, CoreError};

/// Current on-disk format version.
pub const MEMORY_SCHEMA_VERSION: u32 = 2;

/// Versioned persistence envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Enables explicit future migrations.
    pub schema_version: u32,
    /// Persisted domain state.
    pub state: BranllyState,
}

impl MemorySnapshot {
    /// Wraps a validated state in the current schema.
    #[must_use]
    pub const fn current(state: BranllyState) -> Self {
        Self {
            schema_version: MEMORY_SCHEMA_VERSION,
            state,
        }
    }

    /// Accepts the previous schema, whose absent launcher fields deserialize to defaults, and rejects unknown formats.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::UnsupportedSchema`] or a state validation error.
    pub fn into_state(self) -> Result<BranllyState, CoreError> {
        if !(1..=MEMORY_SCHEMA_VERSION).contains(&self.schema_version) {
            return Err(CoreError::UnsupportedSchema {
                found: self.schema_version,
                expected: MEMORY_SCHEMA_VERSION,
            });
        }
        self.state.validate()?;
        Ok(self.state)
    }
}

/// Persistence port implemented by JSON/XDG and `AppData` adapters.
pub trait MemoryStore: Send + Sync {
    /// Loads the memory, returning `None` when it does not exist yet.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] when the adapter cannot load valid data.
    fn load(&self) -> Result<Option<MemorySnapshot>, CoreError>;
    /// Atomically persists a snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] when atomic persistence fails.
    fn save(&self, snapshot: &MemorySnapshot) -> Result<(), CoreError>;
}
