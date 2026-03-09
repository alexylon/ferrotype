mod app;
mod history;
mod input;
mod keyboard;
mod lessons;
mod settings;
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
use settings::load_settings;
use ui::{compute_regions, draw};

#[tokio::main(flavor = "current_thread")]
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

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut hup = signal(SignalKind::hangup()).expect("SIGHUP handler");
        tokio::select! {
            _ = term.recv() => {}
            _ = hup.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        // On Windows, just wait forever — graceful shutdown relies on Esc/Ctrl-C
        std::future::pending::<()>().await;
    }
}

async fn run_app() -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut settings = load_settings();
    let mut rows = build_keyboard_rows(settings.keyboard.layout);
    let mut grid_map = build_keycode_grid_map(&rows);
    let mut app = App::new();
    app.layout = settings.keyboard.layout;

    if let Some(path) = std::env::args().nth(1) {
        match app::Document::load(&path) {
            Ok(doc) => {
                app.document = Some(doc);
                app.lesson_id = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path)
                    .to_string();
            }
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
        let event = tokio::select! {
            ev = rx.recv() => ev,
            _ = shutdown_signal() => None,
        };

        let Some(event) = event else {
            app.save_on_exit();
            break;
        };

        if app.handle_event(event) {
            break;
        }

        // Rebuild keyboard if layout changed
        if app.layout != settings.keyboard.layout {
            settings.keyboard.layout = app.layout;
            settings::save_settings(&settings);
            rows = build_keyboard_rows(settings.keyboard.layout);
            grid_map = build_keycode_grid_map(&rows);
        }

        terminal.draw(|frame| {
            let regions = compute_regions(frame.area());
            draw(frame, &app, &regions, &rows, &grid_map);
        })?;
    }

    Ok(())
}
