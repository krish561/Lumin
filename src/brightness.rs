use anyhow::Result;

use std::process::Command;

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

pub fn supports_ddc(bus: &str) -> bool {
    Command::new("ddcutil")
        .args(["getvcp", "10", "--bus", bus])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn set_ddc_brightness(value: u16, bus: &str) -> Result<bool> {
    let output = Command::new("ddcutil")
        .args(["setvcp", "10", &value.to_string(), "--bus", bus])
        .output()?;

    Ok(output.status.success())
}
