//! Operating-system ports used by Branlly's application layer.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Opaque identifier owned by a platform adapter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub String);

/// A top-level application window visible to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowInfo {
    /// Adapter-owned identifier.
    pub id: WindowId,
    /// Human-readable title.
    pub title: String,
    /// Desktop application identifier when available.
    pub application_id: Option<String>,
    /// Owning process identifier when available.
    pub process_id: Option<u32>,
}

/// Stable application launch request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredApplication {
    /// Adapter-owned stable source identifier.
    pub id: String,
    /// User-facing display name.
    pub name: String,
    /// Optional icon reference supplied by the system.
    pub icon: Option<String>,
    /// Structured executable and arguments.
    pub launch: AppLaunchSpec,
}

/// Stable application launch request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppLaunchSpec {
    /// Freedesktop application id, executable, or Windows application id.
    pub identifier: String,
    /// Explicit arguments; never interpolated through a shell.
    pub arguments: Vec<String>,
}

/// Network connectivity summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkStatus {
    /// No active network is known.
    Offline,
    /// A network is connected without confirmed internet access.
    Local,
    /// Internet connectivity is available.
    Online,
    /// The adapter cannot determine status.
    Unknown,
}

/// Discoverable or paired Bluetooth device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Adapter-owned stable id, such as a `BlueZ` object path.
    pub id: String,
    /// User-facing device name.
    pub name: String,
    /// Whether the device is currently connected.
    pub connected: bool,
    /// Whether the device is paired.
    pub paired: bool,
}

/// Features available in the current desktop session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // Independent OS capabilities are intentionally orthogonal.
pub struct PlatformCapabilities {
    /// Global enumeration of top-level windows.
    pub can_list_windows: bool,
    /// Programmatic focus of another application's window.
    pub can_focus_windows: bool,
    /// Reliable absolute positioning of transparent overlays.
    pub can_position_overlay: bool,
    /// Global pointer tracking outside Branlly's own surface.
    pub can_follow_pointer: bool,
    /// Network status integration.
    pub can_query_network: bool,
    /// Bluetooth integration.
    pub can_query_bluetooth: bool,
    /// Installed application discovery is available.
    pub can_discover_applications: bool,
}

/// Absolute pointer position in physical desktop pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PointerPosition {
    /// Horizontal desktop coordinate.
    pub x: i32,
    /// Vertical desktop coordinate.
    pub y: i32,
}

/// Typed adapter failures suitable for UI presentation and telemetry.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PlatformError {
    /// The current compositor or OS does not expose the requested feature.
    #[error("capability unavailable: {0}")]
    Unsupported(&'static str),
    /// An opaque identifier no longer refers to a live object.
    #[error("resource not found: {0}")]
    NotFound(String),
    /// The OS denied access.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// An external system service failed.
    #[error("platform service failure: {0}")]
    Service(String),
    /// User input failed adapter validation.
    #[error("invalid platform request: {0}")]
    InvalidRequest(String),
}

/// OS abstraction consumed by Tauri commands and application services.
#[async_trait]
pub trait Platform: Send + Sync {
    /// Reports features before the UI presents an action.
    fn capabilities(&self) -> PlatformCapabilities;

    /// Lists top-level user windows when supported.
    ///
    /// # Errors
    ///
    /// Returns a typed OS capability, permission, or service error.
    async fn list_windows(&self) -> Result<Vec<WindowInfo>, PlatformError>;
    /// Focuses one previously listed window.
    ///
    /// # Errors
    ///
    /// Returns a typed OS capability, permission, or stale-resource error.
    async fn focus_window(&self, id: &WindowId) -> Result<(), PlatformError>;
    /// Requests graceful closure, escalating only with explicit user consent.
    ///
    /// # Errors
    ///
    /// Returns a typed OS capability, permission, or stale-resource error.
    async fn close_window(&self, id: &WindowId) -> Result<(), PlatformError>;
    /// Starts an application without passing through a command shell.
    ///
    /// # Errors
    ///
    /// Returns a typed validation, permission, or service error.
    async fn launch_app(&self, specification: &AppLaunchSpec) -> Result<(), PlatformError>;
    /// Lists launchable desktop applications without scanning arbitrary disks.
    async fn discover_applications(&self) -> Result<Vec<DiscoveredApplication>, PlatformError>;
    /// Queries network connectivity.
    ///
    /// # Errors
    ///
    /// Returns a typed capability, permission, or network service error.
    async fn network_status(&self) -> Result<NetworkStatus, PlatformError>;
    /// Lists known Bluetooth devices.
    ///
    /// # Errors
    ///
    /// Returns a typed capability, permission, or Bluetooth service error.
    async fn bluetooth_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError>;
    /// Lists connected physical peripherals exposed by the operating system.
    ///
    /// # Errors
    ///
    /// Returns a typed capability or platform service error.
    async fn connected_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError>;
    /// Returns the global pointer position when the desktop protocol permits it.
    ///
    /// # Errors
    ///
    /// Returns a typed capability or platform service error.
    async fn pointer_position(&self) -> Result<PointerPosition, PlatformError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_default_to_safe_disabled_values() {
        assert_eq!(
            PlatformCapabilities::default(),
            PlatformCapabilities {
                can_list_windows: false,
                can_focus_windows: false,
                can_position_overlay: false,
                can_follow_pointer: false,
                can_query_network: false,
                can_query_bluetooth: false,
                can_discover_applications: false,
            }
        );
    }

    #[test]
    fn launch_arguments_remain_structured_after_serialization() -> Result<(), serde_json::Error> {
        let specification = AppLaunchSpec {
            identifier: "org.mozilla.firefox".to_owned(),
            arguments: vec![
                "https://example.test/a b".to_owned(),
                "; rm -rf /".to_owned(),
            ],
        };
        let json = serde_json::to_string(&specification)?;
        let restored: AppLaunchSpec = serde_json::from_str(&json)?;
        assert_eq!(restored, specification);
        Ok(())
    }
}
