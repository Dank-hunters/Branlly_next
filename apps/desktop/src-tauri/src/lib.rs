//! Tauri composition root. Domain and operating-system adapters meet only here.

use std::sync::Mutex;

use branlly_core::{BranllyConfig, BranllyState, Mood};
use branlly_platform::{Platform, PlatformCapabilities};
use serde::Serialize;
use tauri::State;

#[cfg(target_os = "linux")]
use branlly_platform_linux::LinuxPlatform as NativePlatform;

#[cfg(target_os = "windows")]
use branlly_platform_windows::WindowsPlatform as NativePlatform;

struct RuntimeState {
    domain: Mutex<BranllyState>,
    platform: NativePlatform,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapStatus {
    model: String,
    mood: Mood,
    energy: u8,
    capabilities: PlatformCapabilities,
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri command arguments are extractor values.
fn bootstrap_status(state: State<'_, RuntimeState>) -> Result<BootstrapStatus, String> {
    let domain = state
        .domain
        .lock()
        .map_err(|_| "Branlly state lock is poisoned".to_owned())?;
    Ok(BootstrapStatus {
        model: domain.config().model.clone(),
        mood: domain.mood(),
        energy: domain.energy(),
        capabilities: state.platform.capabilities(),
    })
}

/// Starts the native desktop process.
///
/// # Errors
///
/// Returns a Tauri setup or runtime error instead of panicking.
#[cfg(target_os = "linux")]
pub fn run() -> tauri::Result<()> {
    let domain = BranllyState::new(BranllyConfig::default()).map_err(|error| {
        tauri::Error::Setup((Box::new(error) as Box<dyn std::error::Error>).into())
    })?;
    tauri::Builder::default()
        .manage(RuntimeState {
            domain: Mutex::new(domain),
            platform: NativePlatform::detect(),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![bootstrap_status])
        .run(tauri::generate_context!())
}

/// Windows bootstrap will be activated after its platform adapter implements the shared port.
///
/// # Errors
///
/// Returns a setup error while the Windows adapter remains intentionally incomplete.
#[cfg(target_os = "windows")]
pub fn run() -> tauri::Result<()> {
    let error = std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Windows adapter is not implemented yet",
    );
    Err(tauri::Error::Setup(
        (Box::new(error) as Box<dyn std::error::Error>).into(),
    ))
}
