use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardLayout {
    #[default]
    Qwerty,
    Dvorak,
    Colemak,
}

impl KeyboardLayout {
    pub fn cycle(self) -> Self {
        match self {
            Self::Qwerty => Self::Dvorak,
            Self::Dvorak => Self::Colemak,
            Self::Colemak => Self::Qwerty,
        }
    }
}

impl fmt::Display for KeyboardLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Qwerty => write!(f, "QWERTY"),
            Self::Dvorak => write!(f, "Dvorak"),
            Self::Colemak => write!(f, "Colemak"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyboardSettings {
    #[serde(default)]
    pub layout: KeyboardLayout,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    #[serde(default = "default_true")]
    pub show_keyboard: bool,
    #[serde(default = "default_true")]
    pub show_hints: bool,
    #[serde(default = "default_true")]
    pub show_fingers: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            show_keyboard: true,
            show_hints: true,
            show_fingers: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub keyboard: KeyboardSettings,
    #[serde(default)]
    pub display: DisplaySettings,
}

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".clavirio").join("settings.toml")
}

pub fn load_settings() -> Settings {
    let path = settings_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(settings) {
        let tmp = path.with_extension("toml.tmp");
        if fs::write(&tmp, &content).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_is_qwerty() {
        let s = Settings::default();
        assert_eq!(s.keyboard.layout, KeyboardLayout::Qwerty);
    }

    #[test]
    fn cycle_layout() {
        assert_eq!(KeyboardLayout::Qwerty.cycle(), KeyboardLayout::Dvorak);
        assert_eq!(KeyboardLayout::Dvorak.cycle(), KeyboardLayout::Colemak);
        assert_eq!(KeyboardLayout::Colemak.cycle(), KeyboardLayout::Qwerty);
    }

    #[test]
    fn roundtrip_toml() {
        let s = Settings {
            keyboard: KeyboardSettings {
                layout: KeyboardLayout::Dvorak,
            },
            ..Settings::default()
        };
        let content = toml::to_string(&s).unwrap();
        let parsed: Settings = toml::from_str(&content).unwrap();
        assert_eq!(parsed.keyboard.layout, KeyboardLayout::Dvorak);
    }

    #[test]
    fn deserialize_empty_toml() {
        let parsed: Settings = toml::from_str("").unwrap();
        assert_eq!(parsed.keyboard.layout, KeyboardLayout::Qwerty);
    }

    #[test]
    fn display_names() {
        assert_eq!(KeyboardLayout::Qwerty.to_string(), "QWERTY");
        assert_eq!(KeyboardLayout::Dvorak.to_string(), "Dvorak");
        assert_eq!(KeyboardLayout::Colemak.to_string(), "Colemak");
    }
}
