use anyhow::{bail, Result};
use std::process::{Command, Stdio};

pub fn set_hyprland_wallpaper(path: &str) -> Result<()> {
    // Try swww first (most common Hyprland wallpaper daemon)
    if is_available("swww") {
        let out = Command::new("swww")
            .args(["img", path])
            .stderr(Stdio::piped())
            .output()?;
        if out.status.success() {
            return Ok(());
        }
        // Daemon not running — start it and retry once
        let daemon_started = Command::new("swww-daemon")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if daemon_started {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let status = Command::new("swww").args(["img", path]).status()?;
            if status.success() {
                return Ok(());
            }
        }
    }

    // Fall back to hyprpaper via hyprctl
    if is_available("hyprctl") {
        let preload = Command::new("hyprctl")
            .args(["hyprpaper", "preload", path])
            .stdout(Stdio::null())
            .status()?;
        if preload.success() {
            let wallpaper_arg = format!(",{}", path);
            let set = Command::new("hyprctl")
                .args(["hyprpaper", "wallpaper", &wallpaper_arg])
                .stdout(Stdio::null())
                .status()?;
            if set.success() {
                return Ok(());
            }
        }
    }

    bail!("No supported Hyprland wallpaper tool found (tried: swww, hyprpaper via hyprctl)")
}

fn is_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
