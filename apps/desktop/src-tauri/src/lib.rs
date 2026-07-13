//! Tauri composition root. Domain and operating-system adapters meet only here.

use std::{
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use branlly_core::{BranllyConfig, BranllyState, Mood};
use branlly_ollama::OllamaClient;
use branlly_platform::{Platform, PlatformCapabilities};
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

/// Starts the native desktop process.
///
/// # Errors
///
/// Returns a Tauri setup or runtime error instead of panicking.
#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn run() -> tauri::Result<()> {
    let domain = BranllyState::new(BranllyConfig::default()).map_err(|error| {
        tauri::Error::Setup((Box::new(error) as Box<dyn std::error::Error>).into())
    })?;
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
            cancel_chat
        ])
        .run(tauri::generate_context!())
}
