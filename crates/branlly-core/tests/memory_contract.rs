//! Persistence contract tests for versioned domain snapshots.

use branlly_core::{BranllyConfig, BranllyState, CoreError, MEMORY_SCHEMA_VERSION, MemorySnapshot};

#[test]
fn snapshot_round_trip_preserves_valid_state() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = BranllyState::new(BranllyConfig::default())?;
    state.record_user_message("Bonjour")?;
    state.record_assistant_message("Hmm. Bonjour.")?;

    let json = serde_json::to_string(&MemorySnapshot::current(state.clone()))?;
    let decoded: MemorySnapshot = serde_json::from_str(&json)?;

    assert_eq!(decoded.into_state()?, state);
    Ok(())
}

#[test]
fn future_schema_is_rejected_explicitly() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = MemorySnapshot {
        schema_version: MEMORY_SCHEMA_VERSION + 1,
        state: BranllyState::new(BranllyConfig::default())?,
    };

    assert!(matches!(
        snapshot.into_state(),
        Err(CoreError::UnsupportedSchema { .. })
    ));
    Ok(())
}
