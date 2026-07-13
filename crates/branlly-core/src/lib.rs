//! Platform-independent domain model for Branlly.

mod command;
mod config;
mod error;
mod message;
mod state;
mod storage;

pub use command::BranllyCommand;
pub use config::{BranllyConfig, DEFAULT_LOCAL_MODEL, DEFAULT_SYSTEM_PROMPT};
pub use error::CoreError;
pub use message::{Message, Role};
pub use state::{BranllyState, Mood};
pub use storage::{MEMORY_SCHEMA_VERSION, MemorySnapshot, MemoryStore};
