mod app;
mod brightness;
mod config;
mod monitor;
mod software;
mod ui;

use std::env;
use std::io;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};

use app::{ActiveSection, App, WindowMode};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    if should_spawn_floating_window() {
        spawn_floating_window()?;
        return Ok(());
    }

    // Cleanup stale overlays before starting
    cleanup_stale_overlays()?;

    enable_raw_mode()?;

    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(current_window_mode());

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;

    result
}

fn run_app(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        app.dismiss_expired_notifications();
        app.check_mode_revert();
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    app.cleanup();
                    break;
                }
                KeyCode::Enter => {
                    if app.ui.active_section == Some(ActiveSection::Displays) {
                        app.confirm_mode();
                    } else if app.ui.active_section == Some(ActiveSection::Profiles) {
                        app.apply_profile(app.profiles_cursor);
                    }
                }
                // Gamma section controls
                KeyCode::Char('g') => {
                    if app.ui.active_section == Some(ActiveSection::Gamma) {
                        app.decrease_gamma();
                    }
                }
                KeyCode::Char('G') => {
                    if app.ui.active_section == Some(ActiveSection::Gamma) {
                        app.increase_gamma();
                    }
                }
                KeyCode::Char('r') => {
                    if app.ui.active_section == Some(ActiveSection::Displays) {
                        app.retry_ddc();
                    } else if app.ui.active_section == Some(ActiveSection::Gamma) {
                        app.reset_gamma();
                    }
                }
                // Night section toggle
                KeyCode::Char('n') => {
                    if app.ui.active_section == Some(ActiveSection::Night) {
                        app.toggle_night_light();
                    }
                }
                // Temperature: reuse Left/Right when Night or Gamma is open
                // (already handled by increase/decrease but needs section guard)
                KeyCode::Char('s') => {
                    if app.ui.active_section == Some(ActiveSection::Displays) {
                        app.force_software();
                    }
                }
                KeyCode::Char('w') => {
                    if app.ui.active_section == Some(ActiveSection::Profiles) {
                        let name = if app.config.profiles.is_empty() {
                            "custom".to_string()
                        } else {
                            let presets = ["indoor", "outdoor", "gaming", "night", "custom"];
                            let existing: Vec<&str> = app
                                .config
                                .profiles
                                .iter()
                                .map(|p| p.name.as_str())
                                .collect();
                            presets
                                .iter()
                                .find(|&&n| !existing.contains(&n))
                                .map(|&n| n.to_string())
                                .unwrap_or_else(|| format!("custom-{}", app.config.profiles.len()))
                        };
                        app.save_profile(name);
                    }
                }
                KeyCode::Char('d') => {
                    if app.ui.active_section == Some(ActiveSection::Profiles) {
                        app.delete_profile(app.profiles_cursor);
                    }
                }
                KeyCode::Esc => app.close_section(),
                KeyCode::Char('1') => app.toggle_section(ActiveSection::Displays),
                KeyCode::Char('2') => app.toggle_section(ActiveSection::Profiles),
                KeyCode::Char('3') => app.toggle_section(ActiveSection::Gamma),
                KeyCode::Char('4') => app.toggle_section(ActiveSection::Night),
                KeyCode::Up => {
                    if app.ui.active_section == Some(ActiveSection::Profiles) {
                        app.profiles_previous();
                    } else {
                        app.previous();
                    }
                }
                KeyCode::Down => {
                    if app.ui.active_section == Some(ActiveSection::Profiles) {
                        app.profiles_next();
                    } else {
                        app.next();
                    }
                }
                KeyCode::Right => {
                    if app.ui.active_section == Some(ActiveSection::Night)
                        || app.ui.active_section == Some(ActiveSection::Gamma)
                    {
                        app.increase_temperature();
                    } else {
                        app.increase();
                    }
                }
                KeyCode::Left => {
                    if app.ui.active_section == Some(ActiveSection::Night)
                        || app.ui.active_section == Some(ActiveSection::Gamma)
                    {
                        app.decrease_temperature();
                    } else {
                        app.decrease();
                    }
                }
                KeyCode::Tab => {
                    if app.ui.active_section == Some(ActiveSection::Displays) {
                        app.cycle_refresh_rate(false);
                    }
                }
                KeyCode::BackTab => {
                    if app.ui.active_section == Some(ActiveSection::Displays) {
                        app.cycle_refresh_rate(true);
                    }
                }
                _ => {}
            },
            Event::Mouse(mouse) => {
                let area = terminal.size()?.into();

                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        if let Some(section) = ui::section_at(area, mouse.column, mouse.row) {
                            app.toggle_section(section);
                        } else if let Some(brightness) =
                            ui::brightness_at(area, &app, mouse.column, mouse.row)
                        {
                            app.set_brightness(brightness);
                        } else if let Some(index) =
                            ui::device_at(area, &app, mouse.column, mouse.row)
                        {
                            app.select(index);
                            app.close_section();
                        } else {
                            app.close_section();
                        }
                    }
                    MouseEventKind::Drag(MouseButton::Left) => {
                        if let Some(brightness) =
                            ui::brightness_at(area, &app, mouse.column, mouse.row)
                        {
                            app.set_brightness(brightness);
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        app.previous();
                    }
                    MouseEventKind::ScrollDown => {
                        app.next();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

fn should_spawn_floating_window() -> bool {
    env::var_os("LUMIN_FLOATING_TUI").is_none()
        && env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some()
        && Path::new("/usr/bin/hyprctl").exists()
        && terminal_exists()
}

fn current_window_mode() -> WindowMode {
    if env::var_os("LUMIN_FLOATING_TUI").is_some() {
        WindowMode::Floating
    } else {
        WindowMode::Inline
    }
}

fn spawn_floating_window() -> Result<()> {
    let exe = env::current_exe().context("resolve lumin executable")?;
    let Some(terminal_command) = terminal_command(&exe) else {
        return Ok(());
    };

    let command = format!("[float; pin; center; size 900 620] {terminal_command}");
    Command::new("hyprctl")
        .args(["dispatch", "exec", &command])
        .spawn()
        .context("spawn floating lumin window through hyprctl")?;

    Ok(())
}

fn terminal_exists() -> bool {
    Path::new("/usr/bin/ghostty").exists()
        || Path::new("/usr/local/bin/ghostty").exists()
        || Path::new("/usr/bin/alacritty").exists()
        || Path::new("/usr/local/bin/alacritty").exists()
}

fn terminal_command(exe: &Path) -> Option<String> {
    let exe = shell_quote(&exe.to_string_lossy());

    if Path::new("/usr/bin/ghostty").exists() || Path::new("/usr/local/bin/ghostty").exists() {
        return Some(format!(
            "ghostty --title=lumin -e env LUMIN_FLOATING_TUI=1 {exe}"
        ));
    }

    if Path::new("/usr/bin/alacritty").exists() || Path::new("/usr/local/bin/alacritty").exists() {
        return Some(format!(
            "alacritty --title lumin -e env LUMIN_FLOATING_TUI=1 {exe}"
        ));
    }

    None
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Clean up stale overlay sockets and processes at startup
/// This handles cases where overlays didn't exit cleanly
fn cleanup_stale_overlays() -> Result<()> {
    let runtime_dir = env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| env::temp_dir().to_string_lossy().into_owned());

    let runtime_path = Path::new(&runtime_dir);

    // Find all lumin-overlay-*.socket files and remove them
    if let Ok(entries) = std::fs::read_dir(runtime_path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("lumin-overlay-") && name.ends_with(".socket") {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}
