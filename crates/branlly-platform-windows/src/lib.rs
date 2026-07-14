//! Windows implementation based on structured PowerShell queries and direct process launching.

use async_trait::async_trait;
use branlly_platform::{
    AppLaunchSpec, DeviceInfo, NetworkStatus, Platform, PlatformCapabilities, PlatformError,
    PointerPosition, WindowId, WindowInfo,
};
use serde::Deserialize;
use tokio::process::Command;

/// Windows desktop adapter.
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsPlatform;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ProcessWindow {
    id: String,
    title: String,
    process_id: u32,
    application_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PnpDevice {
    id: String,
    name: String,
    connected: bool,
}

#[async_trait]
impl Platform for WindowsPlatform {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            can_list_windows: true,
            can_focus_windows: true,
            can_position_overlay: true,
            can_follow_pointer: true,
            can_query_network: true,
            can_query_bluetooth: true,
        }
    }

    async fn list_windows(&self) -> Result<Vec<WindowInfo>, PlatformError> {
        let script = r"$items = @(Get-Process | Where-Object {$_.MainWindowHandle -ne 0 -and $_.MainWindowTitle} | ForEach-Object {[pscustomobject]@{Id=('0x{0:X}' -f $_.MainWindowHandle.ToInt64());Title=$_.MainWindowTitle;ProcessId=$_.Id;ApplicationId=$_.ProcessName}}); ConvertTo-Json -Compress -InputObject $items";
        let json = powershell(script, &[]).await?;
        let windows: Vec<ProcessWindow> = serde_json::from_str(json.trim())
            .map_err(|error| PlatformError::Service(format!("invalid window JSON: {error}")))?;
        Ok(windows
            .into_iter()
            .map(|window| WindowInfo {
                id: WindowId(window.id),
                title: window.title,
                application_id: Some(window.application_id),
                process_id: Some(window.process_id),
            })
            .collect())
    }

    async fn focus_window(&self, id: &WindowId) -> Result<(), PlatformError> {
        validate_handle(id)?;
        let script = r"$h=[Convert]::ToInt64($args[0].Substring(2),16); $p=Get-Process | Where-Object {$_.MainWindowHandle.ToInt64() -eq $h} | Select-Object -First 1; if(-not $p){exit 4}; if(-not (New-Object -ComObject WScript.Shell).AppActivate($p.Id)){exit 5}";
        powershell(script, &[&id.0]).await.map(|_| ())
    }

    async fn close_window(&self, id: &WindowId) -> Result<(), PlatformError> {
        validate_handle(id)?;
        let script = r"$h=[Convert]::ToInt64($args[0].Substring(2),16); $p=Get-Process | Where-Object {$_.MainWindowHandle.ToInt64() -eq $h} | Select-Object -First 1; if(-not $p){exit 4}; if(-not $p.CloseMainWindow()){exit 5}";
        powershell(script, &[&id.0]).await.map(|_| ())
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
            .map_err(|error| process_error(identifier, &error))?;
        Ok(())
    }

    async fn network_status(&self) -> Result<NetworkStatus, PlatformError> {
        let script = r"$p=Get-NetConnectionProfile -ErrorAction SilentlyContinue; if(-not $p){'offline'} elseif($p.IPv4Connectivity -contains 'Internet' -or $p.IPv6Connectivity -contains 'Internet'){'online'} else {'local'}";
        Ok(match powershell(script, &[]).await?.trim() {
            "online" => NetworkStatus::Online,
            "local" => NetworkStatus::Local,
            "offline" => NetworkStatus::Offline,
            _ => NetworkStatus::Unknown,
        })
    }

    async fn bluetooth_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        let script = r"$items=@(Get-PnpDevice -Class Bluetooth -ErrorAction SilentlyContinue | Where-Object {$_.FriendlyName} | ForEach-Object {[pscustomobject]@{Id=$_.InstanceId;Name=$_.FriendlyName;Connected=($_.Status -eq 'OK')}}); ConvertTo-Json -Compress -InputObject $items";
        let json = powershell(script, &[]).await?;
        let devices: Vec<PnpDevice> = serde_json::from_str(json.trim())
            .map_err(|error| PlatformError::Service(format!("invalid Bluetooth JSON: {error}")))?;
        Ok(devices
            .into_iter()
            .map(|device| DeviceInfo {
                id: device.id,
                name: device.name,
                connected: device.connected,
                paired: true,
            })
            .collect())
    }

    async fn connected_devices(&self) -> Result<Vec<DeviceInfo>, PlatformError> {
        let script = r"$items=@(Get-PnpDevice -PresentOnly -ErrorAction SilentlyContinue | Where-Object {$_.FriendlyName -and $_.InstanceId -notlike 'ROOT*'} | Select-Object -First 30 | ForEach-Object {[pscustomobject]@{Id=$_.InstanceId;Name=$_.FriendlyName;Connected=$true}}); ConvertTo-Json -Compress -InputObject $items";
        let json = powershell(script, &[]).await?;
        let devices: Vec<PnpDevice> = serde_json::from_str(json.trim())
            .map_err(|error| PlatformError::Service(format!("invalid device JSON: {error}")))?;
        Ok(devices
            .into_iter()
            .map(|device| DeviceInfo {
                id: device.id,
                name: device.name,
                connected: true,
                paired: false,
            })
            .collect())
    }

    async fn pointer_position(&self) -> Result<PointerPosition, PlatformError> {
        let script = r"Add-Type -AssemblyName System.Windows.Forms; $p=[System.Windows.Forms.Cursor]::Position; Write-Output ($p.X.ToString() + ',' + $p.Y.ToString())";
        let output = powershell(script, &[]).await?;
        let (x, y) = output
            .trim()
            .split_once(',')
            .ok_or_else(|| PlatformError::Service("invalid pointer output".to_owned()))?;
        Ok(PointerPosition {
            x: x.parse()
                .map_err(|error| PlatformError::Service(format!("invalid pointer X: {error}")))?,
            y: y.parse()
                .map_err(|error| PlatformError::Service(format!("invalid pointer Y: {error}")))?,
        })
    }
}

fn validate_handle(id: &WindowId) -> Result<(), PlatformError> {
    let value = id.0.strip_prefix("0x").unwrap_or_default();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(PlatformError::InvalidRequest(
            "invalid Windows handle".to_owned(),
        ));
    }
    Ok(())
}

async fn powershell(script: &str, arguments: &[&str]) -> Result<String, PlatformError> {
    let output = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ])
        .args(arguments)
        .output()
        .await
        .map_err(|error| process_error("powershell.exe", &error))?;
    if !output.status.success() {
        return Err(PlatformError::Service(format!(
            "PowerShell exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| PlatformError::Service(format!("invalid PowerShell UTF-8: {error}")))
}

fn process_error(program: &str, error: &std::io::Error) -> PlatformError {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            PlatformError::NotFound(format!("executable not found: {program}"))
        }
        std::io::ErrorKind::PermissionDenied => {
            PlatformError::PermissionDenied(format!("cannot execute {program}"))
        }
        _ => PlatformError::Service(format!("cannot execute {program}: {error}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_opaque_handles_without_command_injection() {
        assert!(validate_handle(&WindowId("0x000F12AB".to_owned())).is_ok());
        assert!(validate_handle(&WindowId("0x12; Stop-Process".to_owned())).is_err());
    }

    #[test]
    fn advertises_only_implemented_windows_features() {
        let capabilities = WindowsPlatform.capabilities();
        assert!(capabilities.can_list_windows);
        assert!(capabilities.can_query_network);
        assert!(capabilities.can_follow_pointer);
    }
}
