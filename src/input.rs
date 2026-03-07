use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Press(KeyEvent),
    Tick,
}

pub async fn run_input_loop(tx: mpsc::UnboundedSender<InputEvent>) {
    // crossterm::event::read() blocks, so run in a dedicated thread
    let _ = tokio::task::spawn_blocking(move || loop {
        let ev = if event::poll(std::time::Duration::from_millis(200)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(k)) if k.kind == KeyEventKind::Press => InputEvent::Press(k),
                Ok(_) => InputEvent::Tick,
                Err(_) => break,
            }
        } else {
            InputEvent::Tick
        };

        if tx.send(ev).is_err() {
            break;
        }
    })
    .await;
}
