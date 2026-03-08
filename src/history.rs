use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub timestamp: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub correct: u32,
    pub total: u32,
    pub duration_secs: f64,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub lesson: String,
}

fn history_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".ferrotype").join("history.json")
}

pub fn save_session(record: SessionRecord) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut records: Vec<SessionRecord> = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    records.push(record);

    if let Ok(json) = serde_json::to_string_pretty(&records) {
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, &json).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}

pub fn load_history() -> Vec<SessionRecord> {
    let path = history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_record_roundtrip() {
        let record = SessionRecord {
            timestamp: "2026-03-08T12:00:00".into(),
            wpm: 45.0,
            accuracy: 97.5,
            correct: 195,
            total: 200,
            duration_secs: 120.0,
            completed: true,
            lesson: "home row".into(),
        };
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: SessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.timestamp, "2026-03-08T12:00:00");
        assert_eq!(deserialized.wpm, 45.0);
        assert_eq!(deserialized.accuracy, 97.5);
        assert_eq!(deserialized.correct, 195);
        assert_eq!(deserialized.total, 200);
        assert!(deserialized.completed);
        assert_eq!(deserialized.lesson, "home row");
    }

    #[test]
    fn deserialize_without_optional_fields() {
        // Simulates loading history from before lesson/completed fields existed
        let json = r#"{
            "timestamp": "2026-01-01T00:00:00",
            "wpm": 30.0,
            "accuracy": 90.0,
            "correct": 100,
            "total": 111,
            "duration_secs": 60.0
        }"#;
        let record: SessionRecord = serde_json::from_str(json).unwrap();
        assert!(!record.completed); // default
        assert!(record.lesson.is_empty()); // default
    }

    #[test]
    fn deserialize_array_of_records() {
        let json = r#"[
            {"timestamp":"t1","wpm":40.0,"accuracy":95.0,"correct":50,"total":53,"duration_secs":30.0},
            {"timestamp":"t2","wpm":50.0,"accuracy":98.0,"correct":100,"total":102,"duration_secs":60.0,"completed":true,"lesson":"f j d k"}
        ]"#;
        let records: Vec<SessionRecord> = serde_json::from_str(json).unwrap();
        assert_eq!(records.len(), 2);
        assert!(!records[0].completed);
        assert!(records[0].lesson.is_empty());
        assert!(records[1].completed);
        assert_eq!(records[1].lesson, "f j d k");
    }
}
