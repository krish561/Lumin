use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprMonitor {
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub refresh_rate: f32,
    pub scale: f32,
    pub available_modes: Vec<String>,
    pub disabled: bool,
    // DDC bus — not in hyprctl output, populated separately
    #[serde(skip)]
    pub bus: Option<String>,
}

pub fn get_monitors() -> Result<Vec<HyprMonitor>> {
    let output = Command::new("hyprctl").args(["monitors", "-j"]).output()?;
    let mut monitors: Vec<HyprMonitor> = serde_json::from_slice(&output.stdout)?;

    // Populate DDC bus for each monitor
    for monitor in &mut monitors {
        monitor.bus = find_ddc_bus(&monitor.name);
    }

    Ok(monitors)
}

/// Apply a mode change live via hyprctl keyword.
/// mode_str is e.g. "1920x1080@60.00Hz"
pub fn set_monitor_mode(monitor_name: &str, mode_str: &str, scale: f32) -> Result<()> {
    let keyword = format!("{monitor_name},{mode_str},auto,{scale}");
    Command::new("hyprctl")
        .args(["keyword", "monitor", &keyword])
        .output()?;
    Ok(())
}

/// Try to find the DDC I2C bus for a monitor by name.
/// Scans /sys/class/drm for a matching connector and returns the bus number.
fn find_ddc_bus(monitor_name: &str) -> Option<String> {
    // hyprctl names like "HDMI-A-1" map to drm connectors like "HDMI-A-1"
    // ddcutil uses bus numbers like "1", "2", "3"
    // We probe each bus and check if ddcutil responds
    for bus_num in 0..16 {
        let bus = bus_num.to_string();
        if crate::brightness::supports_ddc_for_monitor(&bus, monitor_name) {
            return Some(bus);
        }
    }
    None
}
