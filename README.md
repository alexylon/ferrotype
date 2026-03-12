<div align="center"><img src="/assets/images/icon_gunmetal_small.svg" style="width: 80px;" alt="clavirio"></div>

<h1 align="center"><code>clavirio</code></h1>

![](https://github.com/alexylon/clavirio/actions/workflows/rust.yml/badge.svg) &nbsp; [![Crates.io](https://img.shields.io/crates/v/clavirio.svg?color=blue)](https://crates.io/crates/clavirio)

[www.clavir.io](https://www.clavir.io)

### Learn touch typing without leaving the terminal.

Progressive lessons for QWERTY, Dvorak & Colemak. 
Real-time stats, a virtual keyboard with finger hints, and session history — in a fast, 
lightweight binary built with Rust and [ratatui](https://ratatui.rs).

*clavirio* — from Latin *clavis* (key).

<div align="center"><img src="/assets/images/screenshot_dark.png" style="width: 800px;" alt="clavirio"></div>

<div align="center"><img src="/assets/images/screenshot_light.png" style="width: 800px;" alt="clavirio"></div>

## Features

- **15 built-in lessons** — progressive drills from home row basics to full paragraphs and code; lessons 1–9 are tailored to each keyboard layout, lessons 10–15 are shared
- **Live stats** — WPM, accuracy %, elapsed time, keystrokes, and line progress update as you type
- **Keyboard layouts** — supports QWERTY, Dvorak, and Colemak; switch with `l` on the main menu
- **Virtual keyboard** — highlights the expected next key (including Shift) with a finger hint (**P**inky, **R**ing, **M**iddle, **I**ndex, **T**humb) on the top border; adapts to macOS and PC layouts
- **Error feedback** — wrong keystrokes are shown inline and block progress until corrected with Backspace
- **Completion summary** — final WPM, accuracy percentage, and your weakest keys
- **Session history** — results saved to `~/.clavirio/history.json` with per-lesson tracking, scrollable history view, and averages across completed sessions
- **Display settings** — toggle fingers, hints, keyboard, and dark/light theme from the main menu (`1`–`4`); all preferences saved to `~/.clavirio/settings.toml`
- **Progress tracking** — the menu cursor remembers your last built-in lesson: points to it if unfinished, advances to the next if completed
- **Custom text** — load any text file via `Ctrl-F` or as a CLI argument
- **Graceful shutdown** — in-progress sessions are saved on SIGTERM/SIGHUP

## Lessons

Lessons 1–9 are **layout-specific** (each layout has its own drills matched to that layout's finger positions). Lessons 10–15 are **shared** across all layouts.

| # | Lesson | QWERTY | Dvorak | Colemak |
|---|--------|--------|--------|---------|
| 1 | Index Keys | f j | u h | t n |
| 2 | Middle Keys | d k (+ f j) | e t (+ u h) | s e (+ t n) |
| 3 | Ring & Pinky | s l ; (+ f j d k) | o n s (+ a) | r i o (+ a) |
| 4 | Home Row | a s d f j k l ; | a o e u i d h t n s | a r s t d h n e i o |
| 5 | Home Reach | g h | i d | d h |
| 6 | Top Intro | e i r u | p c r l | f p l u |
| 7 | Top Row | q w e r t y u i o p | ' , . p y f g c r l | q w f p g j l u y ; |
| 8 | Bottom Row | z x c v b n m , . | ; q j k x b m w v z | z x c v b k m , . |
| 9 | All Letters | Pangrams | Pangrams | Pangrams |

| # | Lesson | Focus |
|---|--------|-------|
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
| `l` | Cycle keyboard layout: QWERTY → Dvorak → Colemak |
| `1` | Toggle finger hints (requires hints on) |
| `2` | Toggle key hints |
| `3` | Toggle virtual keyboard |
| `4` | Toggle dark/light theme |
| `h` | View session history |
| `Ctrl-F` | Open file path input |
| `Ctrl-R` | Restart current text |
| `Ctrl-C` | Save and quit immediately |
| `Esc` | Save and back to menu / quit |
| `Backspace` | Correct a mistake |
| `r` | Restart (completion screen) |

## Settings

Preferences are stored in `~/.clavirio/settings.toml` and saved automatically.

```toml
[keyboard]
layout = "qwerty"      # qwerty, dvorak, colemak

[display]
show_keyboard = true
show_hints = true
show_fingers = true
theme = "dark"          # dark, light
```
