use anyhow::Result;
use std::process::Command;

// ── Laptop ────────────────────────────────────────────────────────────────────

pub fn get_laptop_brightness() -> Result<u16> {
    let current = String::from_utf8(Command::new("brightnessctl").arg("get").output()?.stdout)?
        .trim()
        .parse::<u32>()?;

    let max = String::from_utf8(Command::new("brightnessctl").arg("max").output()?.stdout)?
        .trim()
        .parse::<u32>()?;

    Ok(((current * 100) / max) as u16)
}

pub fn set_laptop_brightness(value: u16) -> Result<()> {
    Command::new("brightnessctl")
        .args(["set", &format!("{}%", value)])
        .output()?;
    Ok(())
}

// ── DDC ───────────────────────────────────────────────────────────────────────

/// Check if a DDC bus responds (used during backend selection).
pub fn supports_ddc(bus: &str) -> bool {
    Command::new("ddcutil")
        .args(["getvcp", "10", "--bus", bus])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a DDC bus corresponds to a specific monitor.
/// For now this is the same as supports_ddc — a responding bus is a match.
/// In future this could cross-reference EDID data.
pub fn supports_ddc_for_monitor(bus: &str, _monitor_name: &str) -> bool {
    supports_ddc(bus)
}

pub fn set_ddc_brightness(value: u16, bus: &str) -> Result<bool> {
    let output = Command::new("ddcutil")
        .args(["setvcp", "10", &value.to_string(), "--bus", bus])
        .output()?;
    Ok(output.status.success())
}

// ── Gamma / Temperature via hyprsunset ────────────────────────────────────────

/// Set color temperature in Kelvin via hyprsunset.
/// Typical range: 2500K (very warm) to 6500K (daylight).
/// Default/neutral: 6000K.
pub fn set_temperature(kelvin: u32) -> Result<()> {
    // Ensure hyprsunset is running first
    ensure_hyprsunset_running();
    Command::new("hyprctl")
        .args(["hyprsunset", "temperature", &kelvin.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()?;
    Ok(())
}

pub fn set_gamma(percent: u32) -> Result<()> {
    ensure_hyprsunset_running();
    Command::new("hyprctl")
        .args(["hyprsunset", "gamma", &percent.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()?;
    Ok(())
}

pub fn reset_gamma() -> Result<()> {
    ensure_hyprsunset_running();
    Command::new("hyprctl")
        .args(["hyprsunset", "identity"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()?;
    Ok(())
}

pub fn get_temperature() -> Option<u32> {
    let output = Command::new("hyprctl")
        .args(["hyprsunset", "temperature"])
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    // Parse the number out of the output
    stdout
        .split_whitespace()
        .find_map(|w| w.parse::<u32>().ok())
}

fn ensure_hyprsunset_running() {
    let running = Command::new("pgrep")
        .args(["-x", "hyprsunset"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !running {
        let _ = Command::new("uwsm-app")
            .args(["--", "hyprsunset"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
