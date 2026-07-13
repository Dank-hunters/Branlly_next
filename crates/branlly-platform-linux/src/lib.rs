//! Linux platform adapter with explicit X11 and Wayland capability detection.

use async_trait::async_trait;
use branlly_platform::{
    AppLaunchSpec, DeviceInfo, NetworkStatus, Platform, PlatformCapabilities, PlatformError,
    WindowId, WindowInfo,
};

/// Desktop protocol detected from standard environment variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopSession {
    /// X11 exposes the legacy global window operations required by the HUD.
    X11,
    /// Wayland intentionally restricts global window control.
    Wayland,
    /// Headless or unsupported environment.
    Unknown,
}

/// Linux adapter. Service clients will be added behind this stable boundary.
#[derive(Debug, Clone, Copy)]
pub struct LinuxPlatform {
    session: DesktopSession,
}

impl LinuxPlatform {
    /// Detects session protocol from process environment.
    #[must_use]
    pub fn detect() -> Self {
        Self::from_environment(
            std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
            std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
            std::env::var("DISPLAY").ok().as_deref(),
        )
    }

    /// Pure detector used by unit tests and controlled launchers.
    #[must_use]
    pub fn from_environment(
        session_type: Option<&str>,
        wayland_display: Option<&str>,
        x11_display: Option<&str>,
    ) -> Self {
        let normalized = session_type.unwrap_or_default().trim().to_ascii_lowercase();
        let session = if normalized == "wayland" || wayland_display.is_some_and(non_empty) {
            DesktopSession::Wayland
        } else if normalized == "x11" || x11_display.is_some_and(non_empty) {
            DesktopSession::X11
        } else {
            DesktopSession::Unknown
        };
        Self { session }
    }

    /// Returns the detected protocol.
    #[must_use]
    pub const fn session(&self) -> DesktopSession {
        self.session
    }
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

#[async_trait]
impl Platform for LinuxPlatform {
    fn capabilities(&self) -> PlatformCapabilities {
        let x11 = self.session == DesktopSession::X11;
        PlatformCapabilities {
            can_list_windows: x11,
            can_focus_windows: x11,
            can_position_overlay: x11,
            can_follow_pointer: x11,
            can_query_network: true,
            can_query_bluetooth: true,
        }
    }

    async fn list_windows(&self) -> Result<Vec<WindowInfo>, PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux window backend is not implemented yet",
        ))
    }

    async fn focus_window(&self, _id: &WindowId) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux window backend is not implemented yet",
        ))
    }

    async fn close_window(&self, _id: &WindowId) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux window backend is not implemented yet",
        ))
    }

    async fn launch_app(&self, _specification: &AppLaunchSpec) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported(
            "Linux application launcher is not implemented yet",
        ))
    }

    async fn network_status(&self) -> Result<NetworkStatus, PlatformError> {
        Err(PlatformError::Unsupported(
            "NetworkManager integration is not implemented yet",
        ))
    }

    async fn bluetooth_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        Err(PlatformError::Unsupported(
            "BlueZ integration is not implemented yet",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_wins_when_both_display_variables_exist() {
        let platform = LinuxPlatform::from_environment(None, Some("wayland-0"), Some(":0"));
        assert_eq!(platform.session(), DesktopSession::Wayland);
        assert!(!platform.capabilities().can_list_windows);
    }

    #[test]
    fn x11_enables_global_window_capabilities() {
        let platform = LinuxPlatform::from_environment(Some("x11"), None, Some(":0"));
        let capabilities = platform.capabilities();
        assert_eq!(platform.session(), DesktopSession::X11);
        assert!(capabilities.can_list_windows);
        assert!(capabilities.can_focus_windows);
    }

    #[test]
    fn headless_environment_is_safe() {
        let platform = LinuxPlatform::from_environment(None, None, None);
        assert_eq!(platform.session(), DesktopSession::Unknown);
        assert!(!platform.capabilities().can_position_overlay);
    }
}
