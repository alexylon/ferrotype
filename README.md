# clavirio

![](https://github.com/alexylon/clavirio/actions/workflows/rust.yml/badge.svg) &nbsp; [![Crates.io](https://img.shields.io/crates/v/clavirio.svg?color=blue)](https://crates.io/crates/clavirio)

[www.clavir.io](https://www.clavir.io)

*From Latin **clavis** (key) — a terminal typing tutor.*

<div align="center"><img src="/assets/images/screenshot.png" style="width: 800px;" alt="clavirio"></div>

A terminal typing tutor built with Rust and [ratatui](https://ratatui.rs). Practice with built-in lessons or any text file while a virtual keyboard tracks your keystrokes in real time.

## Features

- **15 built-in lessons** — progressive drills from home row basics to full paragraphs and code, ordered by difficulty
- **Live stats** — WPM, accuracy %, elapsed time, keystrokes, and line progress update as you type
- **Virtual keyboard** — highlights the expected next key (including Shift) with a finger hint (**P**inky, **R**ing, **M**iddle, **I**ndex, **T**humb) on the top border; adapts to macOS and PC layouts
- **Error feedback** — wrong keystrokes are shown inline and block progress until corrected with Backspace
- **Completion summary** — final WPM, accuracy percentage, and your weakest keys
- **Session history** — results saved to `~/.clavirio/history.json` with per-lesson tracking, scrollable history view, and averages across completed sessions
- **Progress tracking** — the menu cursor remembers your last built-in lesson: points to it if unfinished, advances to the next if completed
- **Custom text** — load any text file via `Ctrl-F` or as a CLI argument
- **Graceful shutdown** — in-progress sessions are saved on SIGTERM/SIGHUP

## Lessons

| # | Lesson | Focus |
|---|--------|-------|
| 1 | f j d k | Index fingers |
| 2 | d k (+ f j) | Index + middle fingers |
| 3 | s l ; (+ f j d k) | Ring + pinky fingers |
| 4 | a s d f j k l ; | Full home row |
| 5 | g h (home row) | Home row complete |
| 6 | e i r u | Top row reach |
| 7 | q w e r t y u i o p | Full top row |
| 8 | z x c v b n m , . | Bottom row |
| 9 | All Letters | Pangrams |
| 10 | Capitals & Shift | Mixed case |
| 11 | 0-9 Numbers | Numbers in context |
| 12 | Punctuation & Symbols | Special characters |
| 13 | Common Words | High-frequency words |
| 14 | Full Paragraphs | Real-world text |
| 15 | Code (Rust) | Programming syntax |

## Build & Run

```
cargo build --release
cargo run
```

Optionally pass a file directly:

```
cargo run -- sample.txt
```

## Terminal Size

clavirio is a terminal UI application — your terminal window should be large enough to display all elements (text panel, keyboard, stats).
On a laptop screen this usually means a maximized terminal window.

## Controls

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate lesson menu / scroll history |
| `Enter` | Start selected lesson |
| `h` | View session history (main menu) |
| `Ctrl-F` | Open file path input |
| `Ctrl-R` | Restart current text |
| `Ctrl-C` | Save and quit immediately |
| `Esc` | Save and back to menu / quit |
| `Backspace` | Correct a mistake |
| `r` | Restart (completion screen) |
