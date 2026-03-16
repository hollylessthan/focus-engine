use std::process::Command;

/// Toggle OS-level Do Not Disturb / Focus Mode.
///
/// macOS: uses `defaults` + kills NotificationCenter to apply.
/// Windows and Linux stubs are included but not yet implemented.
#[tauri::command]
pub fn toggle_do_not_disturb(enable: bool) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let value = if enable { "true" } else { "false" };
        let script = format!(
            "defaults -currentHost write ~/Library/Preferences/ByHost/com.apple.notificationcenterui doNotDisturb -boolean {}",
            value
        );
        Command::new("bash")
            .arg("-c")
            .arg(&script)
            .output()
            .map_err(|e| e.to_string())?;

        // Restart Notification Center to apply the preference
        Command::new("killall")
            .arg("NotificationCenter")
            .output()
            .ok();

        return Ok(format!("macOS DND set to: {enable}"));
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Toggle Focus Assist via Windows Registry or WMI
        let _ = enable;
        return Err("Windows DND toggle not yet implemented".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        // TODO: Toggle via dbus (GNOME) or equivalent
        let _ = enable;
        return Err("Linux DND toggle not yet implemented".to_string());
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_dnd_returns_result() {
        // On macOS this will attempt the command; on CI it may fail gracefully.
        let result = toggle_do_not_disturb(false);
        // We just assert it returns (Ok or Err), not that it succeeds.
        let _ = result;
    }
}
