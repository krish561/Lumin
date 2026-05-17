mod app;
mod brightness;
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

    enable_raw_mode()?;

    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(current_window_mode());

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        app.dismiss_expired_notifications();

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => {
                    app.cleanup();
                    break;
                }
                KeyCode::Esc => app.close_section(),
                KeyCode::Char('1') => app.toggle_section(ActiveSection::Displays),
                KeyCode::Char('2') => app.toggle_section(ActiveSection::Profiles),
                KeyCode::Char('3') => app.toggle_section(ActiveSection::Gamma),
                KeyCode::Char('4') => app.toggle_section(ActiveSection::Night),
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                KeyCode::Right => app.increase(),
                KeyCode::Left => app.decrease(),
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

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;

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
