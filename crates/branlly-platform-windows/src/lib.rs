//! Windows adapter boundary.
//!
//! Behavior remains explicitly unsupported until the Linux-first contracts stabilize.

use async_trait::async_trait;
use branlly_platform::{
    AppLaunchSpec, DeviceInfo, NetworkStatus, Platform, PlatformCapabilities, PlatformError,
    WindowId, WindowInfo,
};

/// Placeholder used by the Win32 composition root.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsPlatform;

#[async_trait]
impl Platform for WindowsPlatform {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::default()
    }

    async fn list_windows(&self) -> Result<Vec<WindowInfo>, PlatformError> {
        Err(not_implemented())
    }

    async fn focus_window(&self, _id: &WindowId) -> Result<(), PlatformError> {
        Err(not_implemented())
    }

    async fn close_window(&self, _id: &WindowId) -> Result<(), PlatformError> {
        Err(not_implemented())
    }

    async fn launch_app(&self, _specification: &AppLaunchSpec) -> Result<(), PlatformError> {
        Err(not_implemented())
    }

    async fn network_status(&self) -> Result<NetworkStatus, PlatformError> {
        Err(not_implemented())
    }

    async fn bluetooth_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        Err(not_implemented())
    }
}

const fn not_implemented() -> PlatformError {
    PlatformError::Unsupported("Windows adapter is not implemented yet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_never_advertises_unimplemented_capabilities() {
        assert_eq!(
            WindowsPlatform.capabilities(),
            PlatformCapabilities::default()
        );
    }
}
