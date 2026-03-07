mod app;
mod history;
mod input;
mod keyboard;
mod lessons;
mod ui;

use std::io::{stdout, Result};

use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;

use app::App;
use input::run_input_loop;
use keyboard::{build_keyboard_rows, build_keycode_grid_map};
use ui::{compute_regions, draw};

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(std::io::stderr(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    let result = run_app().await;

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    result
}

async fn run_app() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let rows = build_keyboard_rows();
    let grid_map = build_keycode_grid_map(&rows);
    let mut app = App::new();

    if let Some(path) = std::env::args().nth(1) {
        match app::Document::load(&path) {
            Ok(doc) => app.document = Some(doc),
            Err(e) => app.error = Some(e),
        }
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(run_input_loop(tx));

    terminal.draw(|frame| {
        let regions = compute_regions(frame.area());
        draw(frame, &app, &regions, &rows, &grid_map);
    })?;

    loop {
        let Some(event) = rx.recv().await else { break };

        if app.handle_event(event) {
            break;
        }

        terminal.draw(|frame| {
            let regions = compute_regions(frame.area());
            draw(frame, &app, &regions, &rows, &grid_map);
        })?;
    }

    Ok(())
}
