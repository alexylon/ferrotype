use std::fs;
use std::path::Path;
use std::time::Instant;

use crossterm::event::KeyCode;

use crate::input::{Action, InputEvent, Mode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Active,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Document {
    lines: Vec<String>,
    line_idx: usize,
    char_idx: usize,
    pub current_line: String,
    pub progress: Progress,
}

impl Document {
    pub fn load(raw_path: &str) -> Result<Self, String> {
        let trimmed = raw_path.trim();
        let resolved = if Path::new(trimmed).is_absolute() {
            Path::new(trimmed).to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Cannot resolve working directory: {e}"))?
                .join(trimmed)
        };

        if !resolved.exists() {
            return Err(format!("File not found: {}", resolved.display()));
        }

        let content =
            fs::read_to_string(&resolved).map_err(|e| format!("Cannot read file: {e}"))?;

        let lines: Vec<String> = content.lines().map(String::from).collect();

        if lines.is_empty() {
            return Err("File is empty".into());
        }

        let first_line = lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();

        Ok(Self {
            lines,
            line_idx: 0,
            char_idx: 0,
            current_line: first_line,
            progress: Progress::Active,
        })
    }

    pub fn cursor_position(&self) -> usize {
        self.char_idx
    }

    pub fn expected_char(&self) -> Option<char> {
        self.current_line.chars().nth(self.char_idx)
    }

    pub fn upcoming_lines(&self, count: usize) -> Vec<&str> {
        let mut result = Vec::new();
        let mut idx = self.line_idx + 1;
        while result.len() < count {
            match self.lines.get(idx) {
                Some(line) if !line.is_empty() => result.push(line.as_str()),
                Some(_) => {}
                None => break,
            }
            idx += 1;
        }
        result
    }

    pub fn advance(&mut self) {
        self.char_idx += 1;

        if self.char_idx >= self.current_line.len() {
            self.line_idx += 1;
            loop {
                match self.lines.get(self.line_idx) {
                    Some(line) if !line.is_empty() => {
                        self.char_idx = 0;
                        self.current_line = line.clone();
                        self.progress = Progress::Active;
                        return;
                    }
                    Some(_) => self.line_idx += 1,
                    None => {
                        self.progress = Progress::Finished;
                        return;
                    }
                }
            }
        }
    }
}

pub struct App {
    pub document: Option<Document>,
    pub file_path_buf: String,
    pub searching: bool,
    pub error: Option<String>,
    pub correct_count: u32,
    pub total_count: u32,
    pub last_correct: bool,
    pub highlighted_key: Option<KeyCode>,
    pub highlight_until: Option<Instant>,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
}

impl App {
    pub fn new() -> Self {
        Self {
            document: None,
            file_path_buf: String::new(),
            searching: false,
            error: None,
            correct_count: 0,
            total_count: 0,
            last_correct: false,
            highlighted_key: None,
            highlight_until: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn wpm(&self) -> f64 {
        let start = match self.start_time {
            Some(t) => t,
            None => return 0.0,
        };
        let end = self.end_time.unwrap_or_else(Instant::now);
        let secs = end.duration_since(start).as_secs_f64();
        if secs < 1.0 {
            return 0.0;
        }
        (self.correct_count as f64 / 5.0) / (secs / 60.0)
    }

    pub fn elapsed_secs(&self) -> f64 {
        match self.start_time {
            Some(t) => {
                let end = self.end_time.unwrap_or_else(Instant::now);
                end.duration_since(t).as_secs_f64()
            }
            None => 0.0,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::Tick => {
                if let Some(until) = self.highlight_until {
                    if Instant::now() >= until {
                        self.highlighted_key = None;
                        self.highlight_until = None;
                    }
                }
                false
            }
            InputEvent::Press(action) => self.handle_action(action),
        }
    }

    fn handle_action(&mut self, action: Action) -> bool {
        match action.mode {
            Mode::Quit => return true,

            Mode::OpenSearch => {
                self.searching = true;
                self.file_path_buf.clear();
                self.error = None;
            }

            Mode::Search => {
                if action.key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                    return false;
                }
                match action.key.code {
                    KeyCode::Char(c) => self.file_path_buf.push(c),
                    KeyCode::Backspace => {
                        self.file_path_buf.pop();
                    }
                    _ => {}
                }
            }

            Mode::SubmitSearch => {
                self.searching = false;
                let path = self.file_path_buf.clone();
                self.file_path_buf.clear();
                match Document::load(&path) {
                    Ok(doc) => {
                        self.document = Some(doc);
                        self.error = None;
                        self.correct_count = 0;
                        self.total_count = 0;
                        self.start_time = None;
                        self.end_time = None;
                    }
                    Err(e) => self.error = Some(e),
                }
            }

            Mode::CancelSearch => {
                self.searching = false;
                self.file_path_buf.clear();
            }

            Mode::Typing => {
                if let KeyCode::Char(typed) = action.key.code {
                    self.handle_typed_char(typed);
                }
            }
        }
        false
    }

    fn handle_typed_char(&mut self, typed: char) {
        let expected = match self.document.as_ref().and_then(|d| d.expected_char()) {
            Some(c) => c,
            None => return,
        };

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        self.total_count += 1;

        if typed == expected {
            self.correct_count += 1;
            self.last_correct = true;
            if let Some(doc) = self.document.as_mut() {
                doc.advance();
                if doc.progress == Progress::Finished {
                    self.end_time = Some(Instant::now());
                }
            }
        } else {
            self.last_correct = false;
        }

        self.highlight_until = Some(Instant::now() + std::time::Duration::from_millis(400));
        let display_char = if typed.is_whitespace() {
            ' '
        } else {
            typed.to_ascii_uppercase()
        };
        self.highlighted_key = Some(KeyCode::Char(display_char));
    }
}
