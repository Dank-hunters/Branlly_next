//! Tauri composition root. Domain and operating-system adapters meet only here.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use branlly_core::{BranllyConfig, BranllyState, MemorySnapshot, MemoryStore, Mood};
use branlly_ollama::OllamaClient;
use branlly_platform::{
    AppLaunchSpec, DeviceInfo, NetworkStatus, Platform, PlatformCapabilities, PointerPosition,
    WindowId, WindowInfo,
};
use branlly_storage_json::JsonMemoryStore;
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{State, ipc::Channel};
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "linux")]
use branlly_platform_linux::LinuxPlatform as NativePlatform;

#[cfg(target_os = "windows")]
use branlly_platform_windows::WindowsPlatform as NativePlatform;

#[cfg(target_os = "linux")]
fn native_platform() -> NativePlatform {
    NativePlatform::detect()
}

#[cfg(target_os = "windows")]
fn native_platform() -> NativePlatform {
    NativePlatform
}

struct RuntimeState {
    domain: Mutex<BranllyState>,
    platform: NativePlatform,
    ollama: OllamaClient,
    active_chat: Mutex<Option<ActiveChat>>,
    next_chat_id: AtomicU64,
    memory: JsonMemoryStore,
}

struct ActiveChat {
    id: u64,
    cancellation: CancellationToken,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapStatus {
    model: String,
    mood: Mood,
    energy: u8,
    capabilities: PlatformCapabilities,
    ollama_available: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
enum ChatEvent {
    Delta(String),
    Complete,
    Error(String),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemSnapshot {
    network: NetworkStatus,
    bluetooth_devices: Vec<DeviceInfo>,
    connected_devices: Vec<DeviceInfo>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WikiResult {
    title: String,
    description: String,
    url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CleanupReport {
    removed_entries: u64,
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri command arguments are extractor values.
async fn bootstrap_status(state: State<'_, RuntimeState>) -> Result<BootstrapStatus, String> {
    let (model, mood, energy) = {
        let domain = state
            .domain
            .lock()
            .map_err(|_| "Branlly state lock is poisoned".to_owned())?;
        (
            domain.config().model.clone(),
            domain.mood(),
            domain.energy(),
        )
    };
    let ollama_available = state
        .ollama
        .configured_model_is_available()
        .await
        .unwrap_or(false);
    Ok(BootstrapStatus {
        model,
        mood,
        energy,
        capabilities: state.platform.capabilities(),
        ollama_available,
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri extracts state and IPC channels by value.
async fn chat(
    message: String,
    on_event: Channel<ChatEvent>,
    state: State<'_, RuntimeState>,
) -> Result<(), String> {
    let message = message.trim();
    if message.is_empty() || message.chars().count() > 4_000 {
        return Err("Le message doit contenir entre 1 et 4000 caractères.".to_owned());
    }

    let context = {
        let mut domain = lock_domain(&state)?;
        domain
            .record_user_message(message)
            .map_err(|error| error.to_string())?;
        state
            .memory
            .save(&MemorySnapshot::current(domain.clone()))
            .map_err(|error| error.to_string())?;
        domain.chat_context()
    };

    let cancellation = CancellationToken::new();
    let chat_id = state.next_chat_id.fetch_add(1, Ordering::Relaxed);
    {
        let mut active = state
            .active_chat
            .lock()
            .map_err(|_| "Chat cancellation lock is poisoned".to_owned())?;
        if let Some(previous) = active.replace(ActiveChat {
            id: chat_id,
            cancellation: cancellation.clone(),
        }) {
            previous.cancellation.cancel();
        }
    }

    let mut stream = match state.ollama.chat(&context, cancellation.clone()).await {
        Ok(stream) => stream,
        Err(error) => {
            return finish_chat_error(&state, &on_event, chat_id, error.to_string());
        }
    };
    let mut response = String::new();
    while let Some(delta) = stream.next().await {
        match delta {
            Ok(delta) => {
                response.push_str(&delta);
                if let Err(error) = on_event.send(ChatEvent::Delta(delta)) {
                    cancellation.cancel();
                    clear_active_chat(&state, chat_id);
                    return Err(error.to_string());
                }
            }
            Err(error) => {
                return finish_chat_error(&state, &on_event, chat_id, error.to_string());
            }
        }
    }

    {
        let mut domain = lock_domain(&state)?;
        domain
            .record_assistant_message(response)
            .map_err(|error| error.to_string())?;
        state
            .memory
            .save(&MemorySnapshot::current(domain.clone()))
            .map_err(|error| error.to_string())?;
    }
    clear_active_chat(&state, chat_id);
    on_event
        .send(ChatEvent::Complete)
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri extracts managed state by value.
fn cancel_chat(state: State<'_, RuntimeState>) -> Result<(), String> {
    let active = state
        .active_chat
        .lock()
        .map_err(|_| "Chat cancellation lock is poisoned".to_owned())?;
    if let Some(active_chat) = active.as_ref() {
        active_chat.cancellation.cancel();
    }
    Ok(())
}

fn lock_domain(state: &RuntimeState) -> Result<std::sync::MutexGuard<'_, BranllyState>, String> {
    state
        .domain
        .lock()
        .map_err(|_| "Branlly state lock is poisoned".to_owned())
}

fn finish_chat_error(
    state: &RuntimeState,
    on_event: &Channel<ChatEvent>,
    chat_id: u64,
    detail: String,
) -> Result<(), String> {
    if let Ok(mut domain) = state.domain.lock() {
        domain.mark_recoverable_error();
    }
    let _ = on_event.send(ChatEvent::Error(detail.clone()));
    clear_active_chat(state, chat_id);
    Err(detail)
}

fn clear_active_chat(state: &RuntimeState, chat_id: u64) {
    if let Ok(mut active) = state.active_chat.lock() {
        let is_current = active.as_ref().is_some_and(|current| current.id == chat_id);
        if is_current {
            *active = None;
        }
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn list_windows(state: State<'_, RuntimeState>) -> Result<Vec<WindowInfo>, String> {
    state
        .platform
        .list_windows()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn focus_window(id: String, state: State<'_, RuntimeState>) -> Result<(), String> {
    state
        .platform
        .focus_window(&WindowId(id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn close_window(id: String, state: State<'_, RuntimeState>) -> Result<(), String> {
    state
        .platform
        .close_window(&WindowId(id))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn pointer_position(state: State<'_, RuntimeState>) -> Result<PointerPosition, String> {
    state
        .platform
        .pointer_position()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn system_snapshot(state: State<'_, RuntimeState>) -> Result<SystemSnapshot, String> {
    let network = state
        .platform
        .network_status()
        .await
        .unwrap_or(NetworkStatus::Unknown);
    let bluetooth_devices = state.platform.bluetooth_devices().await.unwrap_or_default();
    let connected_devices = state.platform.connected_devices().await.unwrap_or_default();
    Ok(SystemSnapshot {
        network,
        bluetooth_devices,
        connected_devices,
    })
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
async fn launch_shortcut(id: String, state: State<'_, RuntimeState>) -> Result<(), String> {
    let specification =
        shortcut_specification(&id).ok_or_else(|| "Raccourci inconnu ou interdit.".to_owned())?;
    state
        .platform
        .launch_app(&specification)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn cleanup_temp() -> Result<CleanupReport, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let cutoff = std::time::SystemTime::now()
            .checked_sub(Duration::from_secs(24 * 60 * 60))
            .ok_or_else(|| "Horloge système invalide.".to_owned())?;
        let mut removed_entries = 0;
        let entries = std::fs::read_dir(std::env::temp_dir()).map_err(|error| error.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.modified().is_ok_and(|modified| modified < cutoff) {
                let result = if metadata.is_dir() {
                    std::fs::remove_dir_all(&path)
                } else {
                    std::fs::remove_file(&path)
                };
                if result.is_ok() {
                    removed_entries += 1;
                }
            }
        }
        Ok(CleanupReport { removed_entries })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn wiki_search(query: String) -> Result<Vec<WikiResult>, String> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > 120 {
        return Err("La recherche doit contenir entre 1 et 120 caractères.".to_owned());
    }
    let response: serde_json::Value = reqwest::Client::new()
        .get("https://fr.wikipedia.org/w/api.php")
        .query(&[
            ("action", "opensearch"),
            ("search", query),
            ("limit", "5"),
            ("namespace", "0"),
            ("format", "json"),
        ])
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    let titles = response.get(1).and_then(serde_json::Value::as_array);
    let descriptions = response.get(2).and_then(serde_json::Value::as_array);
    let urls = response.get(3).and_then(serde_json::Value::as_array);
    let Some((titles, descriptions, urls)) = titles
        .zip(descriptions)
        .zip(urls)
        .map(|((a, b), c)| (a, b, c))
    else {
        return Err("Réponse Wikipédia invalide.".to_owned());
    };
    Ok(titles
        .iter()
        .zip(descriptions)
        .zip(urls)
        .filter_map(|((title, description), url)| {
            Some(WikiResult {
                title: title.as_str()?.to_owned(),
                description: description.as_str()?.to_owned(),
                url: url.as_str()?.to_owned(),
            })
        })
        .collect())
}

#[cfg(target_os = "linux")]
fn shortcut_specification(id: &str) -> Option<AppLaunchSpec> {
    let target = shortcut_target(id)?;
    Some(AppLaunchSpec {
        identifier: "xdg-open".to_owned(),
        arguments: vec![target.to_owned()],
    })
}

#[cfg(target_os = "windows")]
fn shortcut_specification(id: &str) -> Option<AppLaunchSpec> {
    let target = shortcut_target(id)?;
    Some(AppLaunchSpec {
        identifier: "explorer.exe".to_owned(),
        arguments: vec![target.to_owned()],
    })
}

fn shortcut_target(id: &str) -> Option<&'static str> {
    match id {
        "discord" => Some("https://discord.com/app"),
        "steam" => Some("steam://open/main"),
        "twitch" => Some("https://www.twitch.tv"),
        "youtube-music" => Some("https://music.youtube.com"),
        "stremio" => Some("https://web.stremio.com"),
        "disney" => Some("https://www.disneyplus.com"),
        _ => None,
    }
}

fn setup_error(error: impl std::error::Error + 'static) -> tauri::Error {
    tauri::Error::Setup((Box::new(error) as Box<dyn std::error::Error>).into())
}

/// Starts the native desktop process.
///
/// # Errors
///
/// Returns a Tauri setup or runtime error instead of panicking.
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn run() -> tauri::Result<()> {
    let data_directory = dirs::data_local_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("branlly-next");
    let memory = JsonMemoryStore::new(data_directory.join("memory.json"));
    let domain = match memory.load().map_err(setup_error)? {
        Some(snapshot) => snapshot.into_state().map_err(setup_error)?,
        None => BranllyState::new(BranllyConfig::default()).map_err(setup_error)?,
    };
    let ollama = OllamaClient::new(
        "http://127.0.0.1:11434",
        domain.config().model.clone(),
        Duration::from_secs(90),
    )
    .map_err(|error| tauri::Error::Setup((Box::new(error) as Box<dyn std::error::Error>).into()))?;

    tauri::Builder::default()
        .manage(RuntimeState {
            domain: Mutex::new(domain),
            platform: native_platform(),
            ollama,
            active_chat: Mutex::new(None),
            next_chat_id: AtomicU64::new(1),
            memory,
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
        .invoke_handler(tauri::generate_handler![
            bootstrap_status,
            chat,
            cancel_chat,
            list_windows,
            focus_window,
            close_window,
            system_snapshot,
            launch_shortcut,
            wiki_search,
            cleanup_temp,
            pointer_position
        ])
        .run(tauri::generate_context!())
}
