use crossterm::event::{KeyCode, ModifierKeyCode};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyWidth {
    Normal,
    Wide,
    Spacebar,
}

pub type GridCoord = (usize, usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct KeyDef {
    pub primary: KeyCode,
    pub secondary: Option<KeyCode>,
    pub width: KeyWidth,
    pub label: &'static str,
}

macro_rules! key {
    ($code:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: None,
            width: KeyWidth::Normal,
            label: $label,
        }
    };
    ($code:expr, $shift:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: Some($shift),
            width: KeyWidth::Normal,
            label: $label,
        }
    };
}

macro_rules! wide {
    ($code:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: None,
            width: KeyWidth::Wide,
            label: $label,
        }
    };
}

const NUMBER_ROW: [KeyDef; 13] = [
    key!(KeyCode::Char('1'), KeyCode::Char('!'), "1"),
    key!(KeyCode::Char('2'), KeyCode::Char('@'), "2"),
    key!(KeyCode::Char('3'), KeyCode::Char('#'), "3"),
    key!(KeyCode::Char('4'), KeyCode::Char('$'), "4"),
    key!(KeyCode::Char('5'), KeyCode::Char('%'), "5"),
    key!(KeyCode::Char('6'), KeyCode::Char('^'), "6"),
    key!(KeyCode::Char('7'), KeyCode::Char('&'), "7"),
    key!(KeyCode::Char('8'), KeyCode::Char('*'), "8"),
    key!(KeyCode::Char('9'), KeyCode::Char('('), "9"),
    key!(KeyCode::Char('0'), KeyCode::Char(')'), "0"),
    key!(KeyCode::Char('-'), KeyCode::Char('_'), "-"),
    key!(KeyCode::Char('='), KeyCode::Char('+'), "="),
    key!(KeyCode::Char('\\'), KeyCode::Char('|'), "\\"),
];

const TOP_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('Q'), "Q"),
    key!(KeyCode::Char('W'), "W"),
    key!(KeyCode::Char('E'), "E"),
    key!(KeyCode::Char('R'), "R"),
    key!(KeyCode::Char('T'), "T"),
    key!(KeyCode::Char('Y'), "Y"),
    key!(KeyCode::Char('U'), "U"),
    key!(KeyCode::Char('I'), "I"),
    key!(KeyCode::Char('O'), "O"),
    key!(KeyCode::Char('P'), "P"),
    key!(KeyCode::Char('['), KeyCode::Char('{'), "["),
    key!(KeyCode::Char(']'), KeyCode::Char('}'), "]"),
];

const HOME_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('A'), "A"),
    key!(KeyCode::Char('S'), "S"),
    key!(KeyCode::Char('D'), "D"),
    key!(KeyCode::Char('F'), "F"),
    key!(KeyCode::Char('G'), "G"),
    key!(KeyCode::Char('H'), "H"),
    key!(KeyCode::Char('J'), "J"),
    key!(KeyCode::Char('K'), "K"),
    key!(KeyCode::Char('L'), "L"),
    key!(KeyCode::Char(';'), KeyCode::Char(':'), ";"),
    key!(KeyCode::Char('\''), KeyCode::Char('"'), "'"),
    wide!(KeyCode::Enter, "⏎"),
];

const BOTTOM_ROW: [KeyDef; 12] = [
    wide!(KeyCode::Modifier(ModifierKeyCode::LeftShift), "⇧"),
    key!(KeyCode::Char('Z'), "Z"),
    key!(KeyCode::Char('X'), "X"),
    key!(KeyCode::Char('C'), "C"),
    key!(KeyCode::Char('V'), "V"),
    key!(KeyCode::Char('B'), "B"),
    key!(KeyCode::Char('N'), "N"),
    key!(KeyCode::Char('M'), "M"),
    key!(KeyCode::Char(','), KeyCode::Char('<'), ","),
    key!(KeyCode::Char('.'), KeyCode::Char('>'), "."),
    key!(KeyCode::Char('/'), KeyCode::Char('?'), "/"),
    wide!(KeyCode::Modifier(ModifierKeyCode::RightShift), "⇧"),
];

const MODIFIER_ROW: [KeyDef; 7] = [
    key!(KeyCode::Modifier(ModifierKeyCode::LeftControl), "Ctrl"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftSuper), "⌘"),
    KeyDef {
        primary: KeyCode::Char(' '),
        secondary: None,
        width: KeyWidth::Spacebar,
        label: "␣",
    },
    key!(KeyCode::Modifier(ModifierKeyCode::RightSuper), "⌘"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightControl), "Ctrl"),
];

pub fn build_keyboard_rows() -> Vec<Vec<KeyDef>> {
    vec![
        NUMBER_ROW.to_vec(),
        TOP_ROW.to_vec(),
        HOME_ROW.to_vec(),
        BOTTOM_ROW.to_vec(),
        MODIFIER_ROW.to_vec(),
    ]
}

pub fn build_keycode_grid_map(rows: &[Vec<KeyDef>]) -> HashMap<KeyCode, GridCoord> {
    let mut map = HashMap::new();

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, key_def) in row.iter().enumerate() {
            map.insert(key_def.primary, (row_idx, col_idx));
            if let Some(secondary) = key_def.secondary {
                map.insert(secondary, (row_idx, col_idx));
            }
        }
    }

    map
}
