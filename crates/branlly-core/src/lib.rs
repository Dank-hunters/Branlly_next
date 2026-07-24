//! Platform-independent domain model for Branlly.

mod command;
mod config;
mod error;
mod launch_item;
mod message;
mod state;
mod storage;

pub use command::BranllyCommand;
pub use config::{BranllyConfig, DEFAULT_LOCAL_MODEL, DEFAULT_SYSTEM_PROMPT};
pub use error::CoreError;
pub use launch_item::{LaunchConfiguration, LaunchItem, LaunchItemKind, LaunchPlatform};
pub use message::{Message, Role};
pub use state::{BranllyState, Mood};
pub use storage::{MEMORY_SCHEMA_VERSION, MemorySnapshot, MemoryStore};
