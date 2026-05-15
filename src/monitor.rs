use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HyprMonitor {
    pub name: String,
    pub description: String,
    pub bus: Option<String>,
}

use anyhow::Result;

use std::process::Command;

pub fn get_monitors() -> Result<Vec<HyprMonitor>> {
    let output = Command::new("hyprctl").args(["monitors", "-j"]).output()?;

    let monitors: Vec<HyprMonitor> = serde_json::from_slice(&output.stdout)?;

    Ok(monitors)
}
