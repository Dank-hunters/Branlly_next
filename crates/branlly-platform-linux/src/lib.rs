//! Linux platform adapter with explicit X11 and Wayland capability detection.

use std::path::Path;

use async_trait::async_trait;
use branlly_platform::{
    AppLaunchSpec, DeviceInfo, NetworkStatus, Platform, PlatformCapabilities, PlatformError,
    PointerPosition, WindowId, WindowInfo,
};
use tokio::process::Command;

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
        let has_wmctrl = x11 && command_exists("wmctrl");
        PlatformCapabilities {
            can_list_windows: has_wmctrl,
            can_focus_windows: has_wmctrl,
            can_position_overlay: x11,
            can_follow_pointer: x11 && command_exists("xdotool"),
            can_query_network: command_exists("nmcli"),
            can_query_bluetooth: command_exists("bluetoothctl"),
        }
    }

    async fn list_windows(&self) -> Result<Vec<WindowInfo>, PlatformError> {
        let output = run_command("wmctrl", &["-l", "-p"]).await?;
        Ok(output.lines().filter_map(parse_wmctrl_window).collect())
    }

    async fn focus_window(&self, id: &WindowId) -> Result<(), PlatformError> {
        validate_window_id(id)?;
        run_command("wmctrl", &["-ia", &id.0]).await.map(|_| ())
    }

    async fn close_window(&self, id: &WindowId) -> Result<(), PlatformError> {
        validate_window_id(id)?;
        run_command("wmctrl", &["-ic", &id.0]).await.map(|_| ())
    }

    async fn launch_app(&self, specification: &AppLaunchSpec) -> Result<(), PlatformError> {
        let identifier = specification.identifier.trim();
        if identifier.is_empty() || identifier.contains('\0') {
            return Err(PlatformError::InvalidRequest(
                "invalid executable identifier".to_owned(),
            ));
        }
        Command::new(identifier)
            .args(&specification.arguments)
            .spawn()
            .map_err(|error| map_process_error(identifier, &error))?;
        Ok(())
    }

    async fn network_status(&self) -> Result<NetworkStatus, PlatformError> {
        let status = run_command("nmcli", &["-t", "-f", "CONNECTIVITY", "general"]).await?;
        Ok(match status.trim() {
            "full" => NetworkStatus::Online,
            "limited" | "portal" => NetworkStatus::Local,
            "none" => NetworkStatus::Offline,
            _ => NetworkStatus::Unknown,
        })
    }

    async fn bluetooth_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        let paired = run_command("bluetoothctl", &["devices", "Paired"]).await?;
        let connected = run_command("bluetoothctl", &["devices", "Connected"])
            .await
            .unwrap_or_default();
        let connected_ids: std::collections::HashSet<_> = connected
            .lines()
            .filter_map(parse_bluetooth_line)
            .map(|device| device.id)
            .collect();
        Ok(paired
            .lines()
            .filter_map(parse_bluetooth_line)
            .map(|mut device| {
                device.connected = connected_ids.contains(&device.id);
                device
            })
            .collect())
    }

    async fn connected_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        let output = run_command("lsusb", &[]).await?;
        Ok(output.lines().filter_map(parse_usb_line).collect())
    }

    async fn pointer_position(&self) -> Result<PointerPosition, PlatformError> {
        let output = run_command("xdotool", &["getmouselocation", "--shell"]).await?;
        let mut x = None;
        let mut y = None;
        for line in output.lines() {
            if let Some(value) = line.strip_prefix("X=") {
                x = value.parse().ok();
            }
            if let Some(value) = line.strip_prefix("Y=") {
                y = value.parse().ok();
            }
        }
        x.zip(y)
            .map(|(x, y)| PointerPosition { x, y })
            .ok_or_else(|| PlatformError::Service("invalid xdotool pointer output".to_owned()))
    }
}

fn command_exists(command: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|directory| Path::new(&directory).join(command).is_file())
    })
}

async fn run_command(program: &str, arguments: &[&str]) -> Result<String, PlatformError> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .await
        .map_err(|error| map_process_error(program, &error))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(PlatformError::Service(format!(
            "{program} exited with {}: {detail}",
            output.status
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| PlatformError::Service(format!("invalid UTF-8 from {program}: {error}")))
}

fn map_process_error(program: &str, error: &std::io::Error) -> PlatformError {
    if error.kind() == std::io::ErrorKind::NotFound {
        PlatformError::Unsupported("required Linux command is not installed")
    } else if error.kind() == std::io::ErrorKind::PermissionDenied {
        PlatformError::PermissionDenied(format!("cannot execute {program}"))
    } else {
        PlatformError::Service(format!("cannot execute {program}: {error}"))
    }
}

fn parse_wmctrl_window(line: &str) -> Option<WindowInfo> {
    let mut fields = line.split_whitespace();
    let id = fields.next()?;
    let _desktop = fields.next()?;
    let process_id = fields.next()?.parse().ok();
    let application_id = fields.next().map(str::to_owned);
    let title = fields.collect::<Vec<_>>().join(" ");
    if title.is_empty() {
        return None;
    }
    Some(WindowInfo {
        id: WindowId(id.to_owned()),
        title,
        application_id,
        process_id,
    })
}

fn validate_window_id(id: &WindowId) -> Result<(), PlatformError> {
    let value = id.0.strip_prefix("0x").unwrap_or_default();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(PlatformError::InvalidRequest(
            "invalid X11 window id".to_owned(),
        ));
    }
    Ok(())
}

fn parse_usb_line(line: &str) -> Option<DeviceInfo> {
    let (_, device) = line.split_once(" ID ")?;
    let (id, name) = device.split_once(' ')?;
    Some(DeviceInfo {
        id: id.to_owned(),
        name: name.trim().to_owned(),
        connected: true,
        paired: false,
    })
}

fn parse_bluetooth_line(line: &str) -> Option<DeviceInfo> {
    let mut fields = line.splitn(3, ' ');
    if fields.next()? != "Device" {
        return None;
    }
    let id = fields.next()?.trim().to_owned();
    let name = fields.next()?.trim().to_owned();
    if id.is_empty() || name.is_empty() {
        return None;
    }
    Some(DeviceInfo {
        id,
        name,
        connected: false,
        paired: true,
    })
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
    fn x11_capabilities_follow_installed_backends() {
        let platform = LinuxPlatform::from_environment(Some("x11"), None, Some(":0"));
        let capabilities = platform.capabilities();
        assert_eq!(platform.session(), DesktopSession::X11);
        assert!(capabilities.can_position_overlay);
        assert_eq!(capabilities.can_list_windows, command_exists("wmctrl"));
        assert_eq!(capabilities.can_focus_windows, command_exists("wmctrl"));
        assert_eq!(capabilities.can_follow_pointer, command_exists("xdotool"));
    }

    #[test]
    fn headless_environment_is_safe() {
        let platform = LinuxPlatform::from_environment(None, None, None);
        assert_eq!(platform.session(), DesktopSession::Unknown);
        assert!(!platform.capabilities().can_position_overlay);
    }

    #[test]
    fn service_capabilities_follow_installed_tools() {
        let capabilities =
            LinuxPlatform::from_environment(Some("x11"), None, Some(":0")).capabilities();
        assert_eq!(capabilities.can_query_network, command_exists("nmcli"));
        assert_eq!(
            capabilities.can_query_bluetooth,
            command_exists("bluetoothctl")
        );
    }

    #[test]
    fn parses_windows_without_trusting_titles_as_identifiers() {
        let window = parse_wmctrl_window("0x03a00007  0  4312 host Firefox - Documentation");
        assert_eq!(
            window.as_ref().map(|item| item.id.0.as_str()),
            Some("0x03a00007")
        );
        assert_eq!(
            window.as_ref().map(|item| item.process_id),
            Some(Some(4312))
        );
        assert_eq!(
            window.map(|item| item.title),
            Some("Firefox - Documentation".to_owned())
        );
        assert!(validate_window_id(&WindowId("; rm -rf /".to_owned())).is_err());
    }

    #[test]
    fn parses_usb_output() {
        let device = parse_usb_line("Bus 001 Device 002: ID 1234:5678 Clavier USB");
        assert_eq!(
            device.as_ref().map(|item| item.name.as_str()),
            Some("Clavier USB")
        );
        assert_eq!(device.map(|item| item.connected), Some(true));
    }

    #[test]
    fn parses_bluetooth_output() {
        let device = parse_bluetooth_line("Device AA:BB:CC:DD:EE:FF Casque Audio");
        assert_eq!(
            device.as_ref().map(|item| item.name.as_str()),
            Some("Casque Audio")
        );
        assert_eq!(device.map(|item| item.paired), Some(true));
    }
}
