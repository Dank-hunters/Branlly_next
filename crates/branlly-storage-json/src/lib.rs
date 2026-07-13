//! Local JSON implementation of Branlly's versioned memory port.

use std::{
    fs::{self, File},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use atomic_write_file::AtomicWriteFile;
use branlly_core::{CoreError, MemorySnapshot, MemoryStore};

/// Persists one versioned snapshot at an application-owned path.
#[derive(Debug, Clone)]
pub struct JsonMemoryStore {
    path: PathBuf,
}

impl JsonMemoryStore {
    /// Creates a store without touching the filesystem.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the configured snapshot path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl MemoryStore for JsonMemoryStore {
    fn load(&self) -> Result<Option<MemorySnapshot>, CoreError> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(storage_error("open memory", &error)),
        };
        serde_json::from_reader(BufReader::new(file))
            .map(Some)
            .map_err(|error| CoreError::Storage(format!("decode memory: {error}")))
    }

    fn save(&self, snapshot: &MemorySnapshot) -> Result<(), CoreError> {
        let parent = self.path.parent().ok_or_else(|| {
            CoreError::Storage("memory path must have a parent directory".to_owned())
        })?;
        fs::create_dir_all(parent)
            .map_err(|error| storage_error("create memory directory", &error))?;

        let mut writer = AtomicWriteFile::options()
            .open(&self.path)
            .map_err(|error| storage_error("create temporary memory", &error))?;
        serde_json::to_writer_pretty(&mut writer, snapshot)
            .map_err(|error| CoreError::Storage(format!("encode memory: {error}")))?;
        writer
            .write_all(b"\n")
            .map_err(|error| storage_error("finish memory", &error))?;
        writer
            .commit()
            .map_err(|error| storage_error("commit memory", &error))
    }
}

fn storage_error(context: &str, error: &std::io::Error) -> CoreError {
    CoreError::Storage(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use branlly_core::{BranllyConfig, BranllyState, MemorySnapshot};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn missing_file_is_not_an_error() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let store = JsonMemoryStore::new(directory.path().join("memory.json"));
        assert_eq!(store.load()?, None);
        Ok(())
    }

    #[test]
    fn snapshot_survives_save_and_load() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let store = JsonMemoryStore::new(directory.path().join("nested/memory.json"));
        let mut state = BranllyState::new(BranllyConfig::default())?;
        state.record_user_message("Souviens-toi de ça.")?;
        let snapshot = MemorySnapshot::current(state);

        store.save(&snapshot)?;

        assert_eq!(store.load()?, Some(snapshot));
        Ok(())
    }

    #[test]
    fn malformed_json_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempdir()?;
        let path = directory.path().join("memory.json");
        fs::write(&path, "not-json")?;
        let store = JsonMemoryStore::new(path);
        assert!(matches!(store.load(), Err(CoreError::Storage(_))));
        Ok(())
    }
}
