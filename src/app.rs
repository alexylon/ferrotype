use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::KeyCode;

use crate::input::{Action, InputEvent, Mode};

fn chrono_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let local_secs = utc_to_local(secs as i64) as u64;
    let days = local_secs / 86400;
    let time = local_secs % 86400;
    let h = time / 3600;
    let m = (time % 3600) / 60;
    let s = time % 60;
    // days since 1970-01-01
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}")
}

#[cfg(unix)]
fn utc_to_local(epoch: i64) -> i64 {
    use std::mem::MaybeUninit;
    extern "C" {
        fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    }
    #[repr(C)]
    struct Tm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        _tm_zone: *const i8,
    }
    let mut tm = MaybeUninit::<Tm>::uninit();
    unsafe {
        let ptr = localtime_r(&epoch, tm.as_mut_ptr());
        if ptr.is_null() {
            return epoch;
        }
        epoch + (*ptr).tm_gmtoff
    }
}

#[cfg(windows)]
fn utc_to_local(epoch: i64) -> i64 {
    // Windows: use _localtime64_s to get local broken-down time,
    // then compute offset by diffing against UTC components.
    // Fallback: just return UTC.
    extern "C" {
        fn _localtime64_s(result: *mut CTm, timep: *const i64) -> i32;
        fn _gmtime64_s(result: *mut CTm, timep: *const i64) -> i32;
    }
    #[repr(C)]
    #[derive(Default)]
    struct CTm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
    }
    let mut local = CTm::default();
    let mut utc = CTm::default();
    unsafe {
        if _localtime64_s(&mut local, &epoch) != 0 || _gmtime64_s(&mut utc, &epoch) != 0 {
            return epoch;
        }
    }
    let local_mins = (local.tm_yday * 24 + local.tm_hour) * 60 + local.tm_min;
    let utc_mins = (utc.tm_yday * 24 + utc.tm_hour) * 60 + utc.tm_min;
    let offset_secs = (local_mins - utc_mins) as i64 * 60;
    epoch + offset_secs
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut y = 1970;
    loop {
        let year_days = if is_leap(y) { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        y += 1;
    }
    let leap = is_leap(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            mo = i as u64 + 1;
            break;
        }
        days -= md;
    }
    (y, mo, days + 1)
}

fn is_leap(y: u64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
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
        Ok(Self {
            lines,
            line_idx: 0,
            char_idx: 0,
            current_line: first_line,
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
        self.progress = Progress::Active;
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

    pub fn retreat(&mut self) -> bool {
        if self.char_idx > 0 {
            self.char_idx -= 1;
            return true;
        }
        false
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
            InputEvent::Press(action) => self.handle_action(action),
        }
    }

    fn handle_action(&mut self, action: Action) -> bool {
        match action.mode {
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
                        self.key_stats.clear();
                    }
                    Err(e) => self.error = Some(e),
                }
            }

            Mode::CancelSearch => {
                self.searching = false;
                self.file_path_buf.clear();
            }

            Mode::Restart => {
                if self.document.is_some() {
                    self.restart();
                }
            }

            Mode::MainMenu => {
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

            Mode::Typing => match action.key.code {
                _ if self.viewing_history => {}
                KeyCode::Char('h') if self.document.is_none() => {
                    self.history = crate::history::load_history();
                    self.viewing_history = true;
                }
                KeyCode::Char(typed) if self.document.is_none() => {
                    self.try_select_lesson(typed);
                }
                KeyCode::Char('r') if self.is_finished() => {
                    self.restart();
                }
                KeyCode::Char(typed) if self.last_error_char.is_none() => {
                    self.handle_typed_char(typed);
                }
                KeyCode::Backspace => {
                    if self.last_error_char.is_some() {
                        self.last_error_char = None;
                    } else if let Some(doc) = self.document.as_mut() {
                        if doc.retreat() {
                            self.correct_count = self.correct_count.saturating_sub(1);
                        }
                    }
                }
                _ => {}
            },
        }
        false
    }

    fn try_select_lesson(&mut self, ch: char) {
        let idx = match ch.to_digit(10) {
            Some(d) if d >= 1 => (d - 1) as usize,
            _ => return,
        };
        if let Some(lesson) = crate::lessons::LESSONS.get(idx) {
            match Document::from_text(lesson.text) {
                Ok(doc) => {
                    self.document = Some(doc);
                    self.error = None;
                    self.correct_count = 0;
                    self.total_count = 0;
                    self.start_time = None;
                    self.end_time = None;
                    self.key_stats.clear();
                }
                Err(e) => self.error = Some(e),
            }
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
