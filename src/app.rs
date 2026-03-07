use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::input::InputEvent;

fn chrono_now() -> String {
    let now = time::OffsetDateTime::now_utc()
        .to_offset(time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC));
    let format = time::format_description::well_known::Iso8601::DEFAULT;
    now.format(&format)
        .map(|s| s[..19].to_string())
        .unwrap_or_else(|_| "1970-01-01T00:00:00".into())
}

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
    current_chars: Vec<char>,
    pub progress: Progress,
}

impl Document {
    pub fn from_text(text: &str) -> Result<Self, String> {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        if lines.is_empty() {
            return Err("Text is empty".into());
        }
        let first_line = lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();
        let current_chars = first_line.chars().collect();
        Ok(Self {
            lines,
            line_idx: 0,
            char_idx: 0,
            current_line: first_line,
            current_chars,
            progress: Progress::Active,
        })
    }

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

        Self::from_text(&content)
    }

    pub fn reset(&mut self) {
        self.line_idx = 0;
        self.char_idx = 0;
        self.current_line = self
            .lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();
        self.current_chars = self.current_line.chars().collect();
        self.progress = Progress::Active;
    }

    pub fn cursor_position(&self) -> usize {
        self.char_idx
    }

    pub fn expected_char(&self) -> Option<char> {
        self.current_chars.get(self.char_idx).copied()
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

    pub fn line_progress(&self) -> (usize, usize) {
        let total = self.lines.iter().filter(|l| !l.is_empty()).count();
        let current = self.lines[..self.line_idx]
            .iter()
            .filter(|l| !l.is_empty())
            .count()
            + 1;
        (current, total)
    }

    pub fn advance(&mut self) {
        self.char_idx += 1;

        if self.char_idx >= self.current_chars.len() {
            self.line_idx += 1;
            loop {
                match self.lines.get(self.line_idx) {
                    Some(line) if !line.is_empty() => {
                        self.char_idx = 0;
                        self.current_line = line.clone();
                        self.current_chars = self.current_line.chars().collect();
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
    pub last_error_char: Option<char>,
    pub highlighted_key: Option<KeyCode>,
    pub highlight_until: Option<Instant>,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub key_stats: HashMap<char, (u32, u32)>,
    pub viewing_history: bool,
    pub history: Vec<crate::history::SessionRecord>,
    pub selected_lesson: usize,
    pub lesson_name: String,
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
            last_error_char: None,
            highlighted_key: None,
            highlight_until: None,
            start_time: None,
            end_time: None,
            key_stats: HashMap::new(),
            viewing_history: false,
            history: Vec::new(),
            selected_lesson: 0,
            lesson_name: String::new(),
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

    fn save_history(&self, completed: bool) {
        if self.total_count == 0 {
            return;
        }
        let elapsed = self.elapsed_secs();
        let accuracy = if self.total_count > 0 {
            self.correct_count as f64 / self.total_count as f64 * 100.0
        } else {
            0.0
        };
        crate::history::save_session(crate::history::SessionRecord {
            timestamp: chrono_now(),
            wpm: self.wpm(),
            accuracy,
            correct: self.correct_count,
            total: self.total_count,
            duration_secs: elapsed,
            completed,
            lesson: self.lesson_name.clone(),
        });
    }

    pub fn worst_keys(&self, count: usize) -> Vec<(char, f32)> {
        let mut keys: Vec<(char, f32)> = self
            .key_stats
            .iter()
            .filter(|(_, (hits, misses))| *misses > 0 && (*hits + *misses) >= 2)
            .map(|(&ch, (hits, misses))| {
                let accuracy = *hits as f32 / (*hits + *misses) as f32 * 100.0;
                (ch, accuracy)
            })
            .collect();
        keys.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        keys.truncate(count);
        keys
    }

    pub fn is_finished(&self) -> bool {
        self.document
            .as_ref()
            .is_some_and(|d| d.progress == Progress::Finished)
    }

    pub fn save_on_exit(&self) {
        if self.document.is_some() && !self.is_finished() {
            self.save_history(false);
        }
    }

    fn restart(&mut self) {
        if let Some(doc) = self.document.as_mut() {
            doc.reset();
        }
        self.correct_count = 0;
        self.total_count = 0;
        self.start_time = None;
        self.end_time = None;
        self.key_stats.clear();
        self.last_error_char = None;
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
            InputEvent::Press(key) => self.handle_key(key),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.searching {
            match key.code {
                KeyCode::Enter => {
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
                            self.key_stats.clear();
                            self.lesson_name = path.rsplit('/').next().unwrap_or(&path).to_string();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                KeyCode::Esc => {
                    self.searching = false;
                    self.file_path_buf.clear();
                }
                _ if key.modifiers == KeyModifiers::CONTROL => {}
                KeyCode::Char(c) => self.file_path_buf.push(c),
                KeyCode::Backspace => {
                    self.file_path_buf.pop();
                }
                _ => {}
            }
            return false;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.searching = true;
                self.file_path_buf.clear();
                self.error = None;
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                if self.document.is_some() {
                    self.restart();
                }
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.viewing_history {
                    self.viewing_history = false;
                    return false;
                }
                if self.document.is_none() && self.error.is_none() {
                    return true;
                }
                if !self.is_finished() {
                    self.save_history(false);
                }
                self.document = None;
                self.error = None;
                self.correct_count = 0;
                self.total_count = 0;
                self.start_time = None;
                self.end_time = None;
                self.key_stats.clear();
                self.last_error_char = None;
            }
            _ if self.viewing_history => {}
            _ if self.document.is_none() && self.error.is_none() => {
                self.handle_menu_key(key.code);
            }
            (KeyCode::Char('r'), _) if self.is_finished() => {
                self.restart();
            }
            (KeyCode::Char(typed), _) if self.last_error_char.is_none() => {
                self.handle_typed_char(typed);
            }
            (KeyCode::Backspace, _) => {
                if self.last_error_char.is_some() {
                    self.last_error_char = None;
                }
            }
            _ => {}
        }
        false
    }

    fn handle_menu_key(&mut self, code: KeyCode) {
        let lesson_count = crate::lessons::LESSONS.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_lesson = self.selected_lesson.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_lesson + 1 < lesson_count {
                    self.selected_lesson += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(lesson) = crate::lessons::LESSONS.get(self.selected_lesson) {
                    match Document::from_text(lesson.text) {
                        Ok(doc) => {
                            self.document = Some(doc);
                            self.error = None;
                            self.correct_count = 0;
                            self.total_count = 0;
                            self.start_time = None;
                            self.end_time = None;
                            self.key_stats.clear();
                            self.lesson_name = lesson.label.to_string();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
            }
            KeyCode::Char('h') => {
                self.history = crate::history::load_history();
                self.viewing_history = true;
            }
            _ => {}
        }
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
        let entry = self.key_stats.entry(expected).or_insert((0, 0));

        if typed == expected {
            entry.0 += 1;
            self.correct_count += 1;
            self.last_correct = true;
            self.last_error_char = None;
            if let Some(doc) = self.document.as_mut() {
                doc.advance();
                if doc.progress == Progress::Finished {
                    self.end_time = Some(Instant::now());
                    self.save_history(true);
                }
            }
        } else {
            entry.1 += 1;
            self.last_correct = false;
            self.last_error_char = Some(typed);
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
