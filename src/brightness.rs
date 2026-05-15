use anyhow::Result;

use std::process::Command;

pub fn get_laptop_brightness() -> Result<u16> {
    let output = Command::new("brightnessctl").arg("info").output()?;

    let stdout = String::from_utf8(output.stdout)?;

    let percentage = stdout
        .split('(')
        .nth(1)
        .and_then(|part| part.split('%').next())
        .unwrap_or("0")
        .parse::<u16>()?;

    Ok(percentage)
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
pub fn set_ddc_brightness(value: u16) -> Result<bool> {
    let output = Command::new("ddcutil")
        .args(["setvcp", "10", &value.to_string(), "--bus", "2"])
        .output()?;

    Ok(output.status.success())
}
