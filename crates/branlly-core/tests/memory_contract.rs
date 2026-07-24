//! Persistence contract tests for versioned domain snapshots.

use branlly_core::{
    BranllyConfig, BranllyState, CoreError, LaunchConfiguration, LaunchItem, LaunchItemKind,
    MEMORY_SCHEMA_VERSION, MemorySnapshot,
};

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
fn v1_snapshot_migrates_with_an_empty_uninitialized_launcher()
-> Result<(), Box<dyn std::error::Error>> {
    let snapshot = MemorySnapshot {
        schema_version: 1,
        state: BranllyState::new(BranllyConfig::default())?,
    };
    let state = snapshot.into_state()?;
    assert!(state.launch_items().is_empty());
    assert!(!state.launcher_initialized());
    Ok(())
}

#[test]
fn removed_defaults_stay_removed_after_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = BranllyState::new(BranllyConfig::default())?;
    state.set_launch_items(Vec::new())?;
    let restored = MemorySnapshot::current(state).into_state()?;
    assert!(restored.launcher_initialized());
    assert!(restored.launch_items().is_empty());
    Ok(())
}

#[test]
fn application_and_future_routine_serialize() -> Result<(), Box<dyn std::error::Error>> {
    let application = LaunchItem {
        id: "app".to_owned(),
        kind: LaunchItemKind::Application,
        name: "App".to_owned(),
        icon: None,
        order: 0,
        platform: None,
        launch: LaunchConfiguration::Application {
            identifier: "app.exe".to_owned(),
            arguments: vec![],
        },
    };
    let routine = LaunchItem {
        id: "routine".to_owned(),
        kind: LaunchItemKind::Routine,
        name: "Routine".to_owned(),
        icon: None,
        order: 1,
        platform: None,
        launch: LaunchConfiguration::Routine {
            routine_id: "future".to_owned(),
        },
    };
    let mut state = BranllyState::new(BranllyConfig::default())?;
    state.set_launch_items(vec![application, routine])?;
    assert_eq!(MemorySnapshot::current(state.clone()).into_state()?, state);
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
