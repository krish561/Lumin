mod app;
mod monitor;
mod ui;

use std::io;

use anyhow::Result;

use app::App;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    let monitors = monitor::get_monitors()?;

    println!("{:#?}", monitors);

    return Ok(());

    enable_raw_mode()?;

    execute!(io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());

    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,

                KeyCode::Up => {
                    app.previous();
                }

                KeyCode::Down => {
                    app.next();
                }

                KeyCode::Right => {
                    app.increase();
                }

                KeyCode::Left => {
                    app.decrease();
                }

                _ => {}
            }
        }
    }

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

