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

#[cfg(target_os = "macos")]
const MODIFIER_ROW: [KeyDef; 7] = [
    key!(KeyCode::Modifier(ModifierKeyCode::LeftControl), "⌃"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftSuper), "⌘"),
    KeyDef {
        primary: KeyCode::Char(' '),
        secondary: None,
        width: KeyWidth::Spacebar,
        label: "",
    },
    key!(KeyCode::Modifier(ModifierKeyCode::RightSuper), "⌘"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightControl), "⌃"),
];

#[cfg(not(target_os = "macos"))]
const MODIFIER_ROW: [KeyDef; 7] = [
    key!(KeyCode::Modifier(ModifierKeyCode::LeftControl), "⌃"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftSuper), "Win"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftAlt), "Alt"),
    KeyDef {
        primary: KeyCode::Char(' '),
        secondary: None,
        width: KeyWidth::Spacebar,
        label: "",
    },
    key!(KeyCode::Modifier(ModifierKeyCode::RightAlt), "Alt"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightSuper), "Win"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightControl), "⌃"),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finger {
    Pinky,
    Ring,
    Middle,
    Index,
    Thumb,
}

impl Finger {
    pub fn label(self) -> &'static str {
        match self {
            Finger::Pinky => "P",
            Finger::Ring => "R",
            Finger::Middle => "M",
            Finger::Index => "I",
            Finger::Thumb => "T",
        }
    }
}

/// Returns the finger for a key at the given grid coordinate.
pub fn finger_for_coord(coord: GridCoord) -> Option<Finger> {
    let (row, col) = coord;
    match row {
        // Number row: 1  2  3  4  5  6  7  8  9  0  -  =  backslash
        0 => match col {
            0 => Some(Finger::Pinky),
            1 => Some(Finger::Ring),
            2 => Some(Finger::Middle),
            3 | 4 => Some(Finger::Index),
            5 | 6 => Some(Finger::Index),
            7 => Some(Finger::Middle),
            8 => Some(Finger::Ring),
            _ => Some(Finger::Pinky),
        },
        // Top row: Q  W  E  R  T  Y  U  I  O  P  [  ]
        1 => match col {
            0 => Some(Finger::Pinky),
            1 => Some(Finger::Ring),
            2 => Some(Finger::Middle),
            3 | 4 => Some(Finger::Index),
            5 | 6 => Some(Finger::Index),
            7 => Some(Finger::Middle),
            8 => Some(Finger::Ring),
            _ => Some(Finger::Pinky),
        },
        // Home row: A  S  D  F  G  H  J  K  L  ;  '  Enter
        2 => match col {
            0 => Some(Finger::Pinky),
            1 => Some(Finger::Ring),
            2 => Some(Finger::Middle),
            3 | 4 => Some(Finger::Index),
            5 | 6 => Some(Finger::Index),
            7 => Some(Finger::Middle),
            8 => Some(Finger::Ring),
            _ => Some(Finger::Pinky),
        },
        // Bottom row: LShift  Z  X  C  V  B  N  M  ,  .  /  RShift
        3 => match col {
            0 | 1 => Some(Finger::Pinky),
            2 => Some(Finger::Ring),
            3 => Some(Finger::Middle),
            4 | 5 => Some(Finger::Index),
            6 | 7 => Some(Finger::Index),
            8 => Some(Finger::Middle),
            9 => Some(Finger::Ring),
            _ => Some(Finger::Pinky),
        },
        // Modifier row: space = thumb
        4 => Some(Finger::Thumb),
        _ => None,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_map_contains_primary_keys() {
        let rows = build_keyboard_rows();
        let map = build_keycode_grid_map(&rows);
        // 'A' is in home row (row 2), first key (col 0)
        assert_eq!(map.get(&KeyCode::Char('A')), Some(&(2, 0)));
        // 'Q' is in top row (row 1), first key (col 0)
        assert_eq!(map.get(&KeyCode::Char('Q')), Some(&(1, 0)));
    }

    #[test]
    fn grid_map_contains_secondary_keys() {
        let rows = build_keyboard_rows();
        let map = build_keycode_grid_map(&rows);
        // '!' is secondary of '1', which is number row (row 0), col 0
        assert_eq!(map.get(&KeyCode::Char('!')), Some(&(0, 0)));
        // '@' is secondary of '2', row 0, col 1
        assert_eq!(map.get(&KeyCode::Char('@')), Some(&(0, 1)));
    }

    #[test]
    fn grid_map_secondary_shares_coord_with_primary() {
        let rows = build_keyboard_rows();
        let map = build_keycode_grid_map(&rows);
        // ';' and ':' should map to the same grid position
        let semi = map.get(&KeyCode::Char(';'));
        let colon = map.get(&KeyCode::Char(':'));
        assert!(semi.is_some());
        assert_eq!(semi, colon);
    }

    #[test]
    fn grid_map_spacebar_mapped() {
        let rows = build_keyboard_rows();
        let map = build_keycode_grid_map(&rows);
        assert!(map.contains_key(&KeyCode::Char(' ')));
    }
}
