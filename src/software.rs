use anyhow::Result;

use std::process::Command;

pub fn set_software_brightness(value: u16) -> Result<()> {
    let opacity = 1.0 - (value as f32 / 100.0);

    Command::new("hyprctl")
        .args(["keyword", "decoration:dim_strength", &opacity.to_string()])
        .output()?;

    Ok(())
}
