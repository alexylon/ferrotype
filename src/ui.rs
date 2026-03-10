use std::collections::HashMap;
use std::rc::Rc;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Frame;

use crate::app::{App, Progress};
use crate::keyboard::*;
use crate::settings::Theme;

const KEYBOARD_ROWS: usize = 5;
const MAX_WIDTH: u16 = 120;

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub dim_border: Color,
    pub accent: Color,
    pub dim_text: Color,
    pub correct: Color,
    pub incorrect: Color,
    pub text: Color,
    pub key_label: Color,
    pub cursor_fg: Color,
    pub cursor_bg: Color,
    pub finger_label: Color,
    pub disabled: Color,
    pub bg: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                dim_border: Color::DarkGray,
                accent: Color::Cyan,
                dim_text: Color::DarkGray,
                correct: Color::Rgb(100, 180, 255),
                incorrect: Color::Rgb(255, 170, 60),
                text: Color::White,
                key_label: Color::Gray,
                cursor_fg: Color::Black,
                cursor_bg: Color::White,
                finger_label: Color::Yellow,
                disabled: Color::Rgb(60, 60, 60),
                bg: Color::Reset,
            },
            Theme::Light => Self {
                dim_border: Color::Rgb(180, 180, 180),
                accent: Color::Rgb(0, 120, 180),
                dim_text: Color::Rgb(140, 140, 140),
                correct: Color::Rgb(30, 120, 200),
                incorrect: Color::Rgb(210, 100, 20),
                text: Color::Black,
                key_label: Color::Rgb(80, 80, 80),
                cursor_fg: Color::White,
                cursor_bg: Color::Rgb(40, 40, 40),
                finger_label: Color::Rgb(180, 130, 0),
                disabled: Color::Rgb(200, 200, 200),
                bg: Color::Rgb(245, 245, 245),
            },
        }
    }
}

pub struct Regions {
    header: Rect,
    text_area: Rect,
    search_area: Rect,
    keyboard_area: Rect,
}

fn clamp_width(area: Rect) -> Rect {
    if area.width <= MAX_WIDTH {
        return area;
    }
    let pad = (area.width - MAX_WIDTH) / 2;
    Rect::new(area.x + pad, area.y, MAX_WIDTH, area.height)
}

pub fn compute_regions(area: Rect, show_keyboard: bool) -> Regions {
    let clamped = clamp_width(area);

    let kbd_height = if show_keyboard {
        (KEYBOARD_ROWS as u16) * 4 + 2 // +2 for keyboard border
    } else {
        0
    };

    let [header, body, keyboard_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(kbd_height),
    ])
    .areas(clamped);

    let [text_area] = Layout::horizontal([Constraint::Percentage(80)])
        .flex(Flex::Center)
        .areas(body);

    let [search_area] = Layout::horizontal([Constraint::Percentage(60)])
        .flex(Flex::Center)
        .areas(body);

    Regions {
        header,
        text_area,
        search_area,
        keyboard_area,
    }
}

fn build_keyboard_rects(area: Rect, rows: &[Vec<KeyDef>]) -> Vec<Rc<[Rect]>> {
    let row_rects = Layout::new(
        Direction::Vertical,
        vec![Constraint::Ratio(1, KEYBOARD_ROWS as u32); KEYBOARD_ROWS],
    )
    .split(area);

    // Largest odd width that fits 13 keys (the widest row) in the area.
    // Odd cell width → odd inner width (cell - 2 borders) → perfect centering.
    let raw = area.width / 13;
    let unit_width = if raw.is_multiple_of(2) {
        raw.saturating_sub(1).max(1)
    } else {
        raw
    };
    let raw_wide = unit_width * 3 / 2;
    let wide_width = if raw_wide % 2 == 0 {
        raw_wide - 1
    } else {
        raw_wide
    };
    let raw_space = unit_width * 6;
    let space_width = if raw_space % 2 == 0 {
        raw_space - 1
    } else {
        raw_space
    };

    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let constraints: Vec<Constraint> = row
                .iter()
                .map(|k| match k.width {
                    KeyWidth::Normal => Constraint::Length(unit_width),
                    KeyWidth::Wide => Constraint::Length(wide_width),
                    KeyWidth::Spacebar => Constraint::Length(space_width),
                })
                .collect();
            Layout::new(Direction::Horizontal, constraints)
                .flex(Flex::Center)
                .split(row_rects[i])
        })
        .collect()
}

fn bounding_rect(kbd_rects: &[Rc<[Rect]>]) -> Option<Rect> {
    let mut min_x = u16::MAX;
    let mut min_y = u16::MAX;
    let mut max_x = 0u16;
    let mut max_y = 0u16;
    for row in kbd_rects {
        for cell in row.iter() {
            if cell.width == 0 || cell.height == 0 {
                continue;
            }
            min_x = min_x.min(cell.x);
            min_y = min_y.min(cell.y);
            max_x = max_x.max(cell.x + cell.width);
            max_y = max_y.max(cell.y + cell.height);
        }
    }
    if min_x >= max_x || min_y >= max_y {
        return None;
    }
    Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

pub fn draw(
    frame: &mut Frame,
    app: &App,
    regions: &Regions,
    rows: &[Vec<KeyDef>],
    grid_map: &HashMap<KeyCode, GridCoord>,
) {
    let tc = ThemeColors::from_theme(app.theme);

    let bg_block = Block::default().style(Style::default().bg(tc.bg));
    frame.render_widget(bg_block, frame.area());

    let on_menu = app.document.is_none() && app.error.is_none() && !app.searching;

    let hint_coords: Vec<GridCoord> = if app.show_hints {
        if on_menu {
            // Preview: highlight Enter key so the user sees the effect of display toggles
            grid_map.get(&KeyCode::Enter).copied().into_iter().collect()
        } else {
            app.document
                .as_ref()
                .and_then(|d| d.expected_char())
                .map(|ch| {
                    let mut coords = Vec::new();
                    let key = KeyCode::Char(ch.to_ascii_uppercase());
                    if let Some(&coord) = grid_map.get(&key) {
                        coords.push(coord);
                    }
                    let needs_shift = ch.is_uppercase()
                        || matches!(
                            ch,
                            '!' | '@'
                                | '#'
                                | '$'
                                | '%'
                                | '^'
                                | '&'
                                | '*'
                                | '('
                                | ')'
                                | '_'
                                | '+'
                                | '{'
                                | '}'
                                | '|'
                                | ':'
                                | '"'
                                | '<'
                                | '>'
                                | '?'
                                | '~'
                        );
                    if needs_shift {
                        if let Some(&coord) = grid_map.get(&KeyCode::Modifier(
                            crossterm::event::ModifierKeyCode::LeftShift,
                        )) {
                            coords.push(coord);
                        }
                    }
                    coords
                })
                .unwrap_or_default()
        }
    } else {
        Vec::new()
    };

    let hint_finger = if app.show_fingers {
        hint_coords
            .first()
            .and_then(|&coord| finger_for_coord(coord))
    } else {
        None
    };
    draw_header(frame, app, regions.header, hint_finger, &tc);
    if app.viewing_history {
        draw_history(frame, app, regions.text_area, &tc);
    } else {
        draw_text_panel(frame, app, regions.text_area, &tc);
        draw_search_overlay(frame, app, regions.search_area, &tc);
    }

    if app.show_keyboard {
        let keys_area = Rect::new(
            regions.keyboard_area.x,
            regions.keyboard_area.y + 1,
            regions.keyboard_area.width,
            regions.keyboard_area.height.saturating_sub(2),
        );
        let kbd_rects = build_keyboard_rects(keys_area, rows);
        let highlight_coord: Option<GridCoord> = if app.show_hints {
            app.highlighted_key
                .and_then(|code| grid_map.get(&code))
                .copied()
        } else {
            None
        };
        let highlight_color = if app.last_correct {
            tc.correct
        } else {
            tc.incorrect
        };
        draw_keyboard(
            frame,
            rows,
            &kbd_rects,
            &hint_coords,
            highlight_coord,
            highlight_color,
            app.show_fingers,
            &tc,
        );
    }
}

fn draw_header(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    hint_finger: Option<Finger>,
    tc: &ThemeColors,
) {
    let [left, center, right] = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Min(0),
        Constraint::Percentage(30),
    ])
    .areas(area);

    frame.render_widget(
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(tc.dim_border)),
        area,
    );

    let elapsed = app.elapsed_secs();
    let mins = (elapsed as u64) / 60;
    let secs = (elapsed as u64) % 60;

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {mins}:{secs:02}"), Style::new().fg(tc.dim_text)),
            if app.document.is_some() {
                Span::styled("  Esc", Style::new().fg(tc.accent).bold())
            } else {
                Span::raw("")
            },
            if app.document.is_some() {
                Span::styled(" menu", Style::new().fg(tc.dim_text))
            } else {
                Span::raw("")
            },
        ])),
        left,
    );

    let mut center_spans = vec![Span::styled("clavirio", Style::new().fg(tc.accent).bold())];
    if let Some(doc) = &app.document {
        let (cur, total) = doc.line_progress();
        center_spans.push(Span::styled(
            format!("  {cur}/{total}"),
            Style::new().fg(tc.dim_text),
        ));
    }
    if let Some(finger) = hint_finger {
        let name = match finger {
            Finger::Pinky => "pinky",
            Finger::Ring => "ring",
            Finger::Middle => "middle",
            Finger::Index => "index",
            Finger::Thumb => "thumb",
        };
        center_spans.push(Span::styled(
            format!("  {} ", finger.label()),
            Style::new().fg(tc.finger_label).bold(),
        ));
        center_spans.push(Span::styled(name, Style::new().fg(tc.dim_text)));
    }
    frame.render_widget(Paragraph::new(Line::from(center_spans)).centered(), center);

    let wpm = app.wpm();
    let acc_pct = if app.total_count > 0 {
        app.correct_count as f64 / app.total_count as f64 * 100.0
    } else {
        0.0
    };
    let has_stats = app.total_count > 0;
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{wpm:.0} wpm"),
                Style::new().fg(if wpm > 0.0 { tc.accent } else { tc.dim_text }),
            ),
            Span::styled("  ✓ ", Style::new().fg(tc.correct).bold()),
            Span::styled(
                format!("{}", app.correct_count),
                Style::new().fg(if has_stats { tc.text } else { tc.dim_text }),
            ),
            Span::styled("  ⌨ ", Style::new().fg(tc.finger_label).bold()),
            Span::styled(
                format!("{}", app.total_count),
                Style::new().fg(if has_stats { tc.text } else { tc.dim_text }),
            ),
            Span::styled(
                format!("  {acc_pct:.0}% "),
                Style::new().fg(if has_stats { tc.correct } else { tc.dim_text }),
            ),
        ]))
        .right_aligned(),
        right,
    );
}

fn draw_text_panel(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    if app.searching {
        return;
    }

    let ideal_height = if app.document.is_none() && app.error.is_none() {
        (crate::lessons::lesson_count() as u16) + 7
    } else {
        7
    };
    let panel_height = ideal_height.min(area.height);
    let inner = if panel_height >= area.height {
        area
    } else {
        let [r] = Layout::vertical([Constraint::Length(panel_height)])
            .flex(Flex::Center)
            .areas(area);
        r
    };

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.dim_border))
        .padding(Padding::symmetric(2, 1));

    if let Some(ref err) = app.error {
        frame.render_widget(
            Paragraph::new(err.as_str())
                .style(Style::new().fg(tc.incorrect))
                .block(block)
                .centered(),
            inner,
        );
        return;
    }

    match &app.document {
        None => {
            let lessons = crate::lessons::lessons_for_layout(app.layout);
            // borders: 2 + padding: 2 + blank: 1 + controls: 1 + settings: 1 = 7
            let chrome = 7_u16;
            let visible_slots = inner.height.saturating_sub(chrome) as usize;

            let scroll =
                if visible_slots >= lessons.len() || app.selected_lesson < visible_slots / 2 {
                    0
                } else if app.selected_lesson + visible_slots / 2 >= lessons.len() {
                    lessons.len().saturating_sub(visible_slots)
                } else {
                    app.selected_lesson.saturating_sub(visible_slots / 2)
                };

            let label_len = |l: &&crate::lessons::Lesson| {
                if l.keys.is_empty() {
                    l.title.len()
                } else {
                    l.title.len() + 2 + l.keys.len() + 1
                }
            };
            let max_label: usize = [
                crate::settings::KeyboardLayout::Qwerty,
                crate::settings::KeyboardLayout::Dvorak,
                crate::settings::KeyboardLayout::Colemak,
            ]
            .iter()
            .flat_map(|lay| crate::lessons::lessons_for_layout(*lay))
            .map(|l| label_len(&l))
            .max()
            .unwrap_or(20);

            let mut lines: Vec<Line> = Vec::new();
            for (i, lesson) in lessons.iter().enumerate().skip(scroll).take(visible_slots) {
                let selected = i == app.selected_lesson;
                let marker = if selected { "▸" } else { " " };
                let title_fg = if selected { tc.text } else { tc.dim_text };
                let marker_fg = if selected { tc.accent } else { tc.dim_text };
                let mut spans = vec![
                    Span::styled(format!(" {marker} "), Style::new().fg(marker_fg).bold()),
                    Span::styled(lesson.title.to_string(), Style::new().fg(title_fg)),
                ];
                if !lesson.keys.is_empty() {
                    spans.push(Span::styled(
                        format!(" ({})", lesson.keys),
                        Style::new().fg(tc.dim_text),
                    ));
                }
                let current_len = if lesson.keys.is_empty() {
                    lesson.title.len()
                } else {
                    lesson.title.len() + 2 + lesson.keys.len() + 1
                };
                if current_len < max_label {
                    spans.push(Span::raw(" ".repeat(max_label - current_len)));
                }
                lines.push(Line::from(spans));
            }
            let on_off = |on: bool| if on { "on" } else { "off" };
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Enter", Style::new().fg(tc.accent)),
                Span::styled(" start  ", Style::new().fg(tc.dim_text)),
                Span::styled("h", Style::new().fg(tc.accent)),
                Span::styled(" history  ", Style::new().fg(tc.dim_text)),
                Span::styled("l", Style::new().fg(tc.accent)),
                Span::styled(format!(" {}  ", app.layout), Style::new().fg(tc.dim_text)),
                Span::styled("^F", Style::new().fg(tc.accent)),
                Span::styled(" file  ", Style::new().fg(tc.dim_text)),
                Span::styled("Esc", Style::new().fg(tc.accent)),
                Span::styled(" quit", Style::new().fg(tc.dim_text)),
            ]));
            let fingers_fg = if app.show_hints {
                tc.dim_text
            } else {
                tc.disabled
            };
            let fingers_key_fg = if app.show_hints {
                tc.accent
            } else {
                tc.disabled
            };
            lines.push(Line::from(vec![
                Span::styled("1", Style::new().fg(fingers_key_fg)),
                Span::styled(
                    format!(" fingers {}  ", on_off(app.show_fingers)),
                    Style::new().fg(fingers_fg),
                ),
                Span::styled("2", Style::new().fg(tc.accent)),
                Span::styled(
                    format!(" hints {}  ", on_off(app.show_hints)),
                    Style::new().fg(tc.dim_text),
                ),
                Span::styled("3", Style::new().fg(tc.accent)),
                Span::styled(
                    format!(" keyboard {}  ", on_off(app.show_keyboard)),
                    Style::new().fg(tc.dim_text),
                ),
                Span::styled("4", Style::new().fg(tc.accent)),
                Span::styled(format!(" {}", app.theme), Style::new().fg(tc.dim_text)),
            ]));
            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
        Some(doc) if doc.progress == Progress::Finished => {
            let pct = if app.total_count > 0 {
                (app.correct_count as f32 / app.total_count as f32) * 100.0
            } else {
                0.0
            };
            let mut lines = vec![Line::from(vec![
                Span::styled("Done! ", Style::new().fg(tc.correct).bold()),
                Span::styled(
                    format!("{:.0} wpm", app.wpm()),
                    Style::new().fg(tc.accent).bold(),
                ),
                Span::styled(format!("  {:.0}% accuracy", pct), Style::new().fg(tc.text)),
                Span::styled(
                    format!("  ({}/{})", app.correct_count, app.total_count),
                    Style::new().fg(tc.dim_text),
                ),
                Span::styled("  r", Style::new().fg(tc.accent).bold()),
                Span::styled(" restart  ", Style::new().fg(tc.dim_text)),
                Span::styled("Ctrl-F", Style::new().fg(tc.accent).bold()),
                Span::styled(" new file", Style::new().fg(tc.dim_text)),
            ])];
            let worst = app.worst_keys(5);
            if !worst.is_empty() {
                let mut spans = vec![Span::styled("Weakest: ", Style::new().fg(tc.dim_text))];
                for (i, (ch, acc)) in worst.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled("  ", Style::new().fg(tc.dim_text)));
                    }
                    let label = if *ch == ' ' {
                        "space".to_string()
                    } else {
                        ch.to_string()
                    };
                    spans.push(Span::styled(label, Style::new().fg(tc.incorrect).bold()));
                    spans.push(Span::styled(
                        format!(" {acc:.0}%"),
                        Style::new().fg(tc.dim_text),
                    ));
                }
                lines.push(Line::from(spans));
            }
            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
        Some(doc) => {
            let pos = doc.cursor_position();
            let byte_pos = doc
                .current_line
                .char_indices()
                .nth(pos)
                .map(|(i, _)| i)
                .unwrap_or(doc.current_line.len());
            let (done, remaining) = doc.current_line.split_at(byte_pos);

            let mut spans = Vec::new();
            if !done.is_empty() {
                spans.push(Span::styled(done, Style::new().fg(tc.correct)));
            }

            let mut chars = remaining.chars();
            if let Some(err_ch) = app.last_error_char {
                if let Some(_expected) = chars.next() {
                    spans.push(Span::styled(
                        err_ch.to_string(),
                        Style::new().fg(tc.cursor_fg).bg(tc.incorrect),
                    ));
                }
                if let Some(cursor_ch) = chars.next() {
                    spans.push(Span::styled(
                        cursor_ch.to_string(),
                        Style::new().fg(tc.cursor_fg).bg(tc.cursor_bg),
                    ));
                }
                let rest: String = chars.collect();
                if !rest.is_empty() {
                    spans.push(Span::styled(rest, Style::new().fg(tc.text)));
                }
            } else if let Some(next_ch) = chars.next() {
                spans.push(Span::styled(
                    next_ch.to_string(),
                    Style::new().fg(tc.cursor_fg).bg(tc.cursor_bg),
                ));
                let rest: String = chars.collect();
                if !rest.is_empty() {
                    spans.push(Span::styled(rest, Style::new().fg(tc.text)));
                }
            }

            let mut lines = vec![Line::from(spans)];
            for upcoming in doc.upcoming_lines(2) {
                lines.push(Line::from(Span::styled(
                    upcoming,
                    Style::new().fg(tc.dim_text),
                )));
            }

            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
    }
}

fn draw_search_overlay(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    if !app.searching {
        return;
    }

    let [inner] = Layout::vertical([Constraint::Length(3)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.accent))
        .title(Span::styled(
            " File path ",
            Style::new().fg(tc.accent).bold(),
        ))
        .padding(Padding::horizontal(1));

    let cursor_line = Line::from(vec![
        Span::raw(&app.file_path_buf),
        Span::styled("▌", Style::new().fg(tc.accent)),
    ]);

    frame.render_widget(Paragraph::new(cursor_line).block(block), inner);
}

/// Turn "2026-03-06T22:01:05" into "Mar 06  22:01"
pub(crate) fn friendly_timestamp(ts: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    // Expected format: YYYY-MM-DDThh:mm:ss
    if ts.len() >= 16 {
        let mo: usize = ts[5..7].parse().unwrap_or(1);
        let day = &ts[8..10];
        let time = &ts[11..16]; // hh:mm
        let month = MONTHS.get(mo.wrapping_sub(1)).unwrap_or(&"???");
        format!("{month} {day}  {time}")
    } else {
        ts.to_string()
    }
}

fn draw_history(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    let records = &app.history;

    let panel_h = area.height;
    let [inner] = Layout::vertical([Constraint::Length(panel_h)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.dim_border))
        .title(Span::styled(" History ", Style::new().fg(tc.accent).bold()))
        .padding(Padding::symmetric(2, 1));

    let mut lines = Vec::new();

    if records.is_empty() {
        lines.push(Line::from(Span::styled(
            "No sessions yet",
            Style::new().fg(tc.dim_text),
        )));
    } else {
        // chrome: borders 2 + padding 2 + header 1 + avg ~3 + footer 2 = 10
        let chrome = 10_u16;
        let visible_slots = inner.height.saturating_sub(chrome) as usize;

        let scroll_pos = app.history_scroll;
        let scroll = if visible_slots >= records.len() || scroll_pos < visible_slots / 2 {
            0
        } else if scroll_pos + visible_slots / 2 >= records.len() {
            records.len().saturating_sub(visible_slots)
        } else {
            scroll_pos.saturating_sub(visible_slots / 2)
        };

        lines.push(Line::from(Span::styled(
            format!(
                "{:<18} {:>5}  {:>5}  {:>6}  {}",
                "date", "wpm", "acc", "time", "lesson"
            ),
            Style::new().fg(tc.dim_text),
        )));

        for (i, r) in records.iter().enumerate().skip(scroll).take(visible_slots) {
            let display_ts = friendly_timestamp(&r.timestamp);
            let mins = (r.duration_secs as u64) / 60;
            let secs = (r.duration_secs as u64) % 60;
            let status = if r.completed { "" } else { "*" };
            let lessons = crate::lessons::lessons_for_layout(app.layout);
            let lesson_label = lessons
                .iter()
                .find(|l| l.id == r.id)
                .map(|l| l.title)
                .unwrap_or(&r.id);
            let lesson_display = if r.id.is_empty() { "—" } else { lesson_label };
            let selected = i == scroll_pos;
            let fg = if selected { tc.text } else { tc.dim_text };
            let marker = if selected { "▸ " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!(
                    "{}{:<18} {:>5.0}  {:>4.0}%  {:>2}:{:02}{}  {}",
                    marker, display_ts, r.wpm, r.accuracy, mins, secs, status, lesson_display
                ),
                Style::new().fg(fg),
            )));
        }

        let completed: Vec<_> = records.iter().filter(|r| r.completed).collect();
        if !completed.is_empty() {
            let avg_wpm: f64 =
                completed.iter().map(|r| r.wpm).sum::<f64>() / completed.len() as f64;
            let avg_acc: f64 =
                completed.iter().map(|r| r.accuracy).sum::<f64>() / completed.len() as f64;
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Avg: ", Style::new().fg(tc.dim_text)),
                Span::styled(
                    format!("{avg_wpm:.0} wpm"),
                    Style::new().fg(tc.accent).bold(),
                ),
                Span::styled("  ", Style::new().fg(tc.dim_text)),
                Span::styled(
                    format!("{avg_acc:.0}% acc"),
                    Style::new().fg(tc.correct).bold(),
                ),
                Span::styled(
                    format!("  ({} sessions)", completed.len()),
                    Style::new().fg(tc.dim_text),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::new().fg(tc.accent)),
        Span::styled(" scroll  ", Style::new().fg(tc.dim_text)),
        Span::styled("Esc", Style::new().fg(tc.accent)),
        Span::styled(" back  ", Style::new().fg(tc.dim_text)),
        Span::styled("*", Style::new().fg(tc.accent)),
        Span::styled(" = incomplete", Style::new().fg(tc.dim_text)),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_timestamp_full_iso() {
        assert_eq!(friendly_timestamp("2026-03-06T22:01:05"), "Mar 06  22:01");
    }

    #[test]
    fn friendly_timestamp_january() {
        assert_eq!(friendly_timestamp("2025-01-15T09:30:00"), "Jan 15  09:30");
    }

    #[test]
    fn friendly_timestamp_december() {
        assert_eq!(friendly_timestamp("2025-12-31T23:59:59"), "Dec 31  23:59");
    }

    #[test]
    fn friendly_timestamp_short_string_passthrough() {
        assert_eq!(friendly_timestamp("short"), "short");
    }

    #[test]
    fn friendly_timestamp_invalid_month_shows_fallback() {
        // month "00" → wrapping_sub(1) overflows → fallback "???"
        assert_eq!(friendly_timestamp("2025-00-01T12:00:00"), "??? 01  12:00");
    }
}

fn draw_keyboard(
    frame: &mut Frame,
    rows: &[Vec<KeyDef>],
    kbd_rects: &[Rc<[Rect]>],
    hint_coords: &[GridCoord],
    highlight_coord: Option<GridCoord>,
    highlight_color: Color,
    show_fingers: bool,
    tc: &ThemeColors,
) {
    if let Some(outer) = bounding_rect(kbd_rects) {
        let pad_x: u16 = 2;
        let pad_y: u16 = 1;
        let padded = Rect::new(
            outer.x.saturating_sub(pad_x),
            outer.y.saturating_sub(pad_y),
            (outer.width + pad_x * 2).min(
                frame
                    .area()
                    .width
                    .saturating_sub(outer.x.saturating_sub(pad_x)),
            ),
            (outer.height + pad_y * 2).min(
                frame
                    .area()
                    .height
                    .saturating_sub(outer.y.saturating_sub(pad_y)),
            ),
        );
        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(tc.dim_border)),
            padded,
        );
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let Some(row_rects) = kbd_rects.get(row_idx) else {
            continue;
        };

        for (col_idx, key_def) in row.iter().enumerate() {
            let Some(&cell) = row_rects.get(col_idx) else {
                continue;
            };

            let is_hint = hint_coords.contains(&(row_idx, col_idx));
            let is_highlight = highlight_coord == Some((row_idx, col_idx));

            let border_color = if is_highlight {
                highlight_color
            } else if is_hint {
                tc.correct
            } else {
                tc.dim_border
            };
            let block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(border_color));

            let inner = block.inner(cell);
            frame.render_widget(block, cell);

            if is_hint && show_fingers {
                if let Some(finger) = finger_for_coord((row_idx, col_idx)) {
                    frame.render_widget(
                        Paragraph::new(Span::styled(
                            finger.label(),
                            Style::new().fg(tc.finger_label),
                        ))
                        .centered(),
                        cell,
                    );
                }
            }

            let label = key_def.label;
            let has_secondary = key_def.secondary.and_then(|s| match s {
                KeyCode::Char(c) => Some(c),
                _ => None,
            });

            let buf = frame.buffer_mut();
            let label_w = label.chars().count() as u16;
            let cx = inner.x + inner.width.saturating_sub(label_w) / 2;

            let label_fg = if is_highlight {
                tc.cursor_fg
            } else if is_hint {
                tc.correct
            } else {
                tc.key_label
            };
            let sec_fg = if is_highlight {
                tc.cursor_fg
            } else {
                tc.dim_text
            };

            if let Some(sec_char) = has_secondary {
                // Two-label key: secondary at top, primary at bottom half
                let cy = inner.y + inner.height.saturating_sub(1);
                if cx < inner.x + inner.width && cy < inner.y + inner.height {
                    buf.set_string(cx, cy, label, Style::new().fg(label_fg));
                }
                let s = sec_char.to_string();
                let sw = s.chars().count() as u16;
                let sx = inner.x + inner.width.saturating_sub(sw) / 2;
                if sx < inner.x + inner.width && inner.y < cy {
                    buf.set_string(sx, inner.y, &s, Style::new().fg(sec_fg));
                }
            } else {
                let cy = inner.y + inner.height / 2;
                if cx < inner.x + inner.width && cy < inner.y + inner.height {
                    buf.set_string(cx, cy, label, Style::new().fg(label_fg));
                }
            }
        }
    }
}
