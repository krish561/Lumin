use anyhow::Result;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::Command;

pub fn update_software_brightness(monitor_name: &str, percentage: u8) -> std::io::Result<()> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());

    let socket_path =
        PathBuf::from(runtime_dir).join(format!("lumin-overlay-{}.socket", monitor_name));

    let mut stream = UnixStream::connect(socket_path)?;

    stream.write_all(&[percentage])?;

    Ok(())
}
pub fn spawn_overlay(monitor_name: &str, brightness: u8) -> Result<()> {
    let exe = std::env::current_exe()?
        .parent()
        .unwrap()
        .join("lumin-overlay");

    Command::new(exe)
        .arg("--monitor")
        .arg(monitor_name)
        .arg("--brightness")
        .arg(brightness.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null()) // add this
        .stderr(std::process::Stdio::null()) // add this
        .spawn()?;
    Ok(())
}
pub fn kill_overlay(monitor_name: &str) -> std::io::Result<()> {
    let socket_path = get_socket_path(monitor_name);
    // Removing the socket signals the overlay to exit
    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

fn get_socket_path(monitor_name: &str) -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());
    PathBuf::from(runtime_dir).join(format!("lumin-overlay-{}.socket", monitor_name))
}
