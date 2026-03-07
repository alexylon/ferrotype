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

const KEYBOARD_ROWS: usize = 5;
const DIM_BORDER: Color = Color::DarkGray;
const ACCENT: Color = Color::Cyan;
const DIM_TEXT: Color = Color::DarkGray;
const CORRECT: Color = Color::Rgb(100, 180, 255);
const INCORRECT: Color = Color::Rgb(255, 170, 60);
const HINT: Color = Color::Rgb(60, 80, 100);

pub struct Regions {
    header: Rect,
    text_area: Rect,
    search_area: Rect,
    keyboard_area: Rect,
}

pub fn compute_regions(area: Rect) -> Regions {
    let [header, body, keyboard_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length((KEYBOARD_ROWS as u16) * 3),
    ])
    .areas(area);

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
    let [kbd_inner] = Layout::horizontal([Constraint::Percentage(90)])
        .flex(Flex::Center)
        .areas(area);

    let row_rects = Layout::new(
        Direction::Vertical,
        vec![Constraint::Ratio(1, KEYBOARD_ROWS as u32); KEYBOARD_ROWS],
    )
    .split(kbd_inner);

    let unit_width = kbd_inner.width / 13;

    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let constraints: Vec<Constraint> = row
                .iter()
                .map(|k| match k.width {
                    KeyWidth::Normal => Constraint::Length(unit_width),
                    KeyWidth::Wide => Constraint::Length(unit_width * 3 / 2),
                    KeyWidth::Spacebar => Constraint::Length(unit_width * 6),
                })
                .collect();
            Layout::new(Direction::Horizontal, constraints)
                .flex(Flex::Center)
                .split(row_rects[i])
        })
        .collect()
}

pub fn draw(
    frame: &mut Frame,
    app: &App,
    regions: &Regions,
    rows: &[Vec<KeyDef>],
    grid_map: &HashMap<KeyCode, GridCoord>,
) {
    let kbd_rects = build_keyboard_rects(regions.keyboard_area, rows);

    let hint_coords: Vec<GridCoord> = app
        .document
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
        .unwrap_or_default();

    draw_header(frame, app, regions.header);
    if app.viewing_history {
        draw_history(frame, app, regions.text_area);
    } else {
        draw_text_panel(frame, app, regions.text_area);
        draw_search_overlay(frame, app, regions.search_area);
    }
    draw_keyboard(frame, rows, &kbd_rects, &hint_coords);
    draw_key_highlight(frame, app, &kbd_rects, grid_map);
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let [left, center, right] = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Min(0),
        Constraint::Percentage(30),
    ])
    .areas(area);

    frame.render_widget(
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(DIM_BORDER)),
        area,
    );

    let elapsed = app.elapsed_secs();
    let mins = (elapsed as u64) / 60;
    let secs = (elapsed as u64) % 60;

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ✓ ", Style::new().fg(CORRECT).bold()),
            Span::styled(
                format!("{}", app.correct_count),
                Style::new().fg(Color::White),
            ),
            Span::styled(format!("  {mins}:{secs:02}"), Style::new().fg(DIM_TEXT)),
        ])),
        left,
    );

    let mut center_spans = vec![Span::styled("ferrotype", Style::new().fg(ACCENT).bold())];
    if let Some(doc) = &app.document {
        let (cur, total) = doc.line_progress();
        center_spans.push(Span::styled(
            format!("  {cur}/{total}"),
            Style::new().fg(DIM_TEXT),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(center_spans)).centered(), center);

    let wpm = app.wpm();
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{wpm:.0} wpm  "),
                Style::new().fg(if wpm > 0.0 { ACCENT } else { DIM_TEXT }),
            ),
            Span::styled(
                format!("{}", app.total_count),
                Style::new().fg(Color::White),
            ),
            Span::styled(" ⌨ ", Style::new().fg(Color::Yellow).bold()),
        ]))
        .right_aligned(),
        right,
    );
}

fn draw_text_panel(frame: &mut Frame, app: &App, area: Rect) {
    if app.searching {
        return;
    }

    let panel_height = if app.document.is_none() && app.error.is_none() {
        (crate::lessons::LESSONS.len() as u16) + 5
    } else {
        6
    };
    let [inner] = Layout::vertical([Constraint::Length(panel_height)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM_BORDER))
        .padding(Padding::new(2, 2, 1, 0));

    if let Some(ref err) = app.error {
        frame.render_widget(
            Paragraph::new(err.as_str())
                .style(Style::new().fg(INCORRECT))
                .block(block)
                .centered(),
            inner,
        );
        return;
    }

    match &app.document {
        None => {
            let mut lines = vec![Line::from(vec![
                Span::styled("Ctrl-F", Style::new().fg(ACCENT).bold()),
                Span::styled(" open a file  ", Style::new().fg(DIM_TEXT)),
                Span::styled("Esc", Style::new().fg(ACCENT).bold()),
                Span::styled(" quit", Style::new().fg(DIM_TEXT)),
            ])];
            for (i, lesson) in crate::lessons::LESSONS.iter().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}", i + 1), Style::new().fg(ACCENT).bold()),
                    Span::styled(format!("  {}", lesson.label), Style::new().fg(DIM_TEXT)),
                ]));
            }
            lines.push(Line::from(vec![
                Span::styled("  h", Style::new().fg(ACCENT).bold()),
                Span::styled("  History", Style::new().fg(DIM_TEXT)),
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
                Span::styled("Done! ", Style::new().fg(CORRECT).bold()),
                Span::styled(
                    format!("{:.0} wpm", app.wpm()),
                    Style::new().fg(ACCENT).bold(),
                ),
                Span::styled(
                    format!("  {:.0}% accuracy", pct),
                    Style::new().fg(Color::White),
                ),
                Span::styled(
                    format!("  ({}/{})", app.correct_count, app.total_count),
                    Style::new().fg(DIM_TEXT),
                ),
                Span::styled("  r", Style::new().fg(ACCENT).bold()),
                Span::styled(" restart  ", Style::new().fg(DIM_TEXT)),
                Span::styled("Ctrl-F", Style::new().fg(ACCENT).bold()),
                Span::styled(" new file", Style::new().fg(DIM_TEXT)),
            ])];
            let worst = app.worst_keys(5);
            if !worst.is_empty() {
                let mut spans = vec![Span::styled("Weakest: ", Style::new().fg(DIM_TEXT))];
                for (i, (ch, acc)) in worst.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled("  ", Style::new().fg(DIM_TEXT)));
                    }
                    let label = if *ch == ' ' {
                        "space".to_string()
                    } else {
                        ch.to_string()
                    };
                    spans.push(Span::styled(label, Style::new().fg(INCORRECT).bold()));
                    spans.push(Span::styled(
                        format!(" {acc:.0}%"),
                        Style::new().fg(DIM_TEXT),
                    ));
                }
                lines.push(Line::from(spans));
            }
            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
        Some(doc) => {
            let pos = doc.cursor_position();
            let (done, remaining) = doc.current_line.split_at(pos);

            let mut spans = Vec::new();
            if !done.is_empty() {
                spans.push(Span::styled(done, Style::new().fg(CORRECT)));
            }

            let mut chars = remaining.chars();
            if let Some(err_ch) = app.last_error_char {
                if let Some(_expected) = chars.next() {
                    spans.push(Span::styled(
                        err_ch.to_string(),
                        Style::new().fg(Color::Black).bg(INCORRECT),
                    ));
                }
                if let Some(cursor_ch) = chars.next() {
                    spans.push(Span::styled(
                        cursor_ch.to_string(),
                        Style::new().fg(Color::Black).bg(Color::White),
                    ));
                }
                let rest: String = chars.collect();
                if !rest.is_empty() {
                    spans.push(Span::styled(rest, Style::new().fg(Color::White)));
                }
            } else if let Some(next_ch) = chars.next() {
                spans.push(Span::styled(
                    next_ch.to_string(),
                    Style::new().fg(Color::Black).bg(Color::White),
                ));
                let rest: String = chars.collect();
                if !rest.is_empty() {
                    spans.push(Span::styled(rest, Style::new().fg(Color::White)));
                }
            }

            let mut lines = vec![Line::from(spans)];
            for upcoming in doc.upcoming_lines(2) {
                lines.push(Line::from(Span::styled(
                    upcoming,
                    Style::new().fg(DIM_TEXT),
                )));
            }

            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
    }
}

fn draw_search_overlay(frame: &mut Frame, app: &App, area: Rect) {
    if !app.searching {
        return;
    }

    let [inner] = Layout::vertical([Constraint::Length(3)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(ACCENT))
        .title(Span::styled(" File path ", Style::new().fg(ACCENT).bold()))
        .padding(Padding::horizontal(1));

    let cursor_line = Line::from(vec![
        Span::raw(&app.file_path_buf),
        Span::styled("▌", Style::new().fg(ACCENT)),
    ]);

    frame.render_widget(Paragraph::new(cursor_line).block(block), inner);
}

fn draw_history(frame: &mut Frame, app: &App, area: Rect) {
    let records = &app.history;
    let show_count = 10;
    let recent: Vec<_> = records.iter().rev().take(show_count).collect();
    let panel_h = (recent.len() as u16 + 5).min(area.height);

    let [inner] = Layout::vertical([Constraint::Length(panel_h)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(DIM_BORDER))
        .title(Span::styled(" History ", Style::new().fg(ACCENT).bold()))
        .padding(Padding::new(2, 2, 1, 0));

    let mut lines = Vec::new();

    if recent.is_empty() {
        lines.push(Line::from(Span::styled(
            "No sessions yet",
            Style::new().fg(DIM_TEXT),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("{:<18} {:>5}  {:>5}  {:>6}", "date", "wpm", "acc", "time"),
            Style::new().fg(DIM_TEXT),
        )));

        for r in &recent {
            let display_ts = if r.timestamp.len() >= 16 {
                &r.timestamp[..16]
            } else {
                &r.timestamp
            };
            let mins = (r.duration_secs as u64) / 60;
            let secs = (r.duration_secs as u64) % 60;
            let status = if r.completed { "" } else { "*" };
            lines.push(Line::from(Span::styled(
                format!(
                    "{:<18} {:>5.0}  {:>4.0}%  {:>2}:{:02}{}",
                    display_ts, r.wpm, r.accuracy, mins, secs, status
                ),
                Style::new().fg(Color::White),
            )));
        }

        // Averages from completed sessions
        let completed: Vec<_> = records.iter().filter(|r| r.completed).collect();
        if !completed.is_empty() {
            let avg_wpm: f64 =
                completed.iter().map(|r| r.wpm).sum::<f64>() / completed.len() as f64;
            let avg_acc: f64 =
                completed.iter().map(|r| r.accuracy).sum::<f64>() / completed.len() as f64;
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Avg: ", Style::new().fg(DIM_TEXT)),
                Span::styled(format!("{avg_wpm:.0} wpm"), Style::new().fg(ACCENT).bold()),
                Span::styled("  ", Style::new().fg(DIM_TEXT)),
                Span::styled(
                    format!("{avg_acc:.0}% acc"),
                    Style::new().fg(CORRECT).bold(),
                ),
                Span::styled(
                    format!("  ({} sessions)", completed.len()),
                    Style::new().fg(DIM_TEXT),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Esc to go back  (* = incomplete)",
        Style::new().fg(DIM_TEXT),
    )));

    frame.render_widget(Paragraph::new(lines).block(block), inner);
}

fn draw_keyboard(
    frame: &mut Frame,
    rows: &[Vec<KeyDef>],
    kbd_rects: &[Rc<[Rect]>],
    hint_coords: &[GridCoord],
) {
    for (row_idx, row) in rows.iter().enumerate() {
        let Some(row_rects) = kbd_rects.get(row_idx) else {
            continue;
        };

        for (col_idx, key_def) in row.iter().enumerate() {
            let Some(&cell) = row_rects.get(col_idx) else {
                continue;
            };

            let is_hint = hint_coords.contains(&(row_idx, col_idx));

            let border_color = if is_hint { ACCENT } else { DIM_BORDER };
            let block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(border_color))
                .style(if is_hint {
                    Style::new().bg(HINT)
                } else {
                    Style::new()
                });

            frame.render_widget(block, cell);

            let label = key_def.label;
            frame.render_widget(
                Paragraph::new(Span::styled(label, Style::new().fg(Color::Gray)))
                    .centered()
                    .block(Block::new().padding(Padding::top(cell.height.saturating_sub(2) / 2))),
                cell,
            );

            if let Some(sec) = key_def.secondary {
                let sec_label = match sec {
                    KeyCode::Char(c) => c.to_string(),
                    _ => String::new(),
                };
                if !sec_label.is_empty() {
                    frame.render_widget(
                        Paragraph::new(Span::styled(sec_label, Style::new().fg(DIM_TEXT)))
                            .block(Block::new().padding(Padding::new(2, 0, 0, 0))),
                        cell,
                    );
                }
            }
        }
    }
}

fn draw_key_highlight(
    frame: &mut Frame,
    app: &App,
    kbd_rects: &[Rc<[Rect]>],
    grid_map: &HashMap<KeyCode, GridCoord>,
) {
    if app.highlighted_key.is_none() {
        return;
    }

    let code = match app.highlighted_key {
        Some(c) => c,
        None => return,
    };

    let &(row, col) = match grid_map.get(&code) {
        Some(coord) => coord,
        None => return,
    };

    let rect = kbd_rects.get(row).and_then(|r| r.get(col));
    let Some(&cell) = rect else { return };

    let color = if app.last_correct { CORRECT } else { INCORRECT };

    frame.render_widget(
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(color))
            .style(Style::new().bg(color).fg(Color::Black)),
        cell,
    );
}
