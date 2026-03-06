use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Quit,
    OpenSearch,
    Search,
    SubmitSearch,
    CancelSearch,
    Typing,
}

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub key: KeyEvent,
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Press(Action),
    Tick,
}

pub async fn run_input_loop(tx: mpsc::UnboundedSender<InputEvent>) {
    let mut stream = EventStream::new();
    let mut in_search = false;

    loop {
        let event = tokio::select! {
            maybe = stream.next() => maybe,
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {
                let _ = tx.send(InputEvent::Tick);
                continue;
            }
        };

        let key = match event {
            Some(Ok(Event::Key(k))) if k.kind == KeyEventKind::Press => k,
            Some(Ok(_)) => {
                let _ = tx.send(InputEvent::Tick);
                continue;
            }
            Some(Err(_)) | None => break,
        };

        let mode = classify_key(&key, &mut in_search);

        if tx.send(InputEvent::Press(Action { key, mode })).is_err() {
            break;
        }
    }
}

fn classify_key(key: &KeyEvent, in_search: &mut bool) -> Mode {
    if *in_search {
        match key.code {
            KeyCode::Enter => {
                *in_search = false;
                Mode::SubmitSearch
            }
            KeyCode::Esc => {
                *in_search = false;
                Mode::CancelSearch
            }
            _ => Mode::Search,
        }
    } else {
        match (key.code, key.modifiers) {
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                *in_search = true;
                Mode::OpenSearch
            }
            (KeyCode::Esc, KeyModifiers::NONE) => Mode::Quit,
            _ => Mode::Typing,
        }
    }
}
