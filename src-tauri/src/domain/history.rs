//! Bounded canonical per-area history. Deltas use byte offsets on UTF-8 boundaries.
use crate::{
    app::document::{DocumentHistory, HistoryAvailability},
    domain::DictationTarget,
};

const ENTRIES: usize = 256;
const BYTES: usize = 16 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    Manual,
    Formatting,
    Dictation,
    Reformat,
}
#[derive(Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManualOrigin {
    #[default]
    Manual,
    Formatting,
}
impl From<ManualOrigin> for Origin {
    fn from(origin: ManualOrigin) -> Self {
        match origin {
            ManualOrigin::Manual => Self::Manual,
            ManualOrigin::Formatting => Self::Formatting,
        }
    }
}
struct Entry {
    area: DictationTarget,
    _origin: Origin,
    at: usize,
    before: String,
    after: String,
    undone: bool,
}
#[derive(Default)]
pub struct History {
    entries: Vec<Entry>,
    bytes: usize,
    evicted: bool,
}
impl History {
    pub fn clear(&mut self, area: Option<DictationTarget>) {
        if area.is_none() {
            self.evicted = false;
        }
        self.entries
            .retain(|entry| area.is_some_and(|area| entry.area != area));
        self.recount();
    }
    fn recount(&mut self) {
        self.bytes = self
            .entries
            .iter()
            .map(|e| e.before.len() + e.after.len())
            .sum();
    }
    /// Returns whether capacity eviction occurred, never truncates text.
    pub fn record(
        &mut self,
        area: DictationTarget,
        origin: Origin,
        before: &str,
        after: &str,
    ) -> bool {
        if before == after {
            return false;
        }
        self.entries
            .retain(|entry| entry.area != area || !entry.undone);
        let mut start = before
            .bytes()
            .zip(after.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while !before.is_char_boundary(start) || !after.is_char_boundary(start) {
            start -= 1;
        }
        let mut suffix = before[start..]
            .bytes()
            .rev()
            .zip(after[start..].bytes().rev())
            .take_while(|(a, b)| a == b)
            .count();
        while !before.is_char_boundary(before.len() - suffix)
            || !after.is_char_boundary(after.len() - suffix)
        {
            suffix -= 1;
        }
        self.entries.push(Entry {
            area,
            _origin: origin,
            at: start,
            before: before[start..before.len() - suffix].into(),
            after: after[start..after.len() - suffix].into(),
            undone: false,
        });
        self.recount();
        let mut evicted = false;
        while self.entries.len() > ENTRIES || self.bytes > BYTES {
            let entry = self.entries.remove(0);
            if entry.undone {
                self.entries
                    .retain(|other| other.area != entry.area || !other.undone);
            }
            self.recount();
            evicted = true;
        }
        self.evicted |= evicted;
        evicted
    }
    pub fn capacity_reached(&self) -> bool {
        self.evicted
    }
    pub fn apply(&mut self, area: DictationTarget, redo: bool, current: &str) -> Option<String> {
        let index = if redo {
            self.entries.iter().position(|e| e.area == area && e.undone)
        } else {
            self.entries
                .iter()
                .rposition(|e| e.area == area && !e.undone)
        }?;
        let entry = &mut self.entries[index];
        let (expected, replacement) = if redo {
            (&entry.before, &entry.after)
        } else {
            (&entry.after, &entry.before)
        };
        let end = entry.at.checked_add(expected.len())?;
        if current.get(entry.at..end)? != expected {
            return None;
        }
        let mut text = current.to_owned();
        text.replace_range(entry.at..end, replacement);
        entry.undone = !redo;
        Some(text)
    }
    pub fn availability(&self) -> DocumentHistory {
        let area = |area| HistoryAvailability {
            can_undo: self.entries.iter().any(|e| e.area == area && !e.undone),
            can_redo: self.entries.iter().any(|e| e.area == area && e.undone),
        };
        DocumentHistory {
            content: area(DictationTarget::Content),
            context: area(DictationTarget::Context),
            instructions: area(DictationTarget::Instructions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn byte_budget_is_global_and_never_truncates_a_large_delta() {
        let mut history = History::default();
        let mut text = String::new();
        for index in 0..20 {
            let next = if index % 2 == 0 {
                "é".repeat(524_288)
            } else {
                "😀".repeat(262_144)
            };
            history.record(DictationTarget::Content, Origin::Reformat, &text, &next);
            text = next;
            assert!(history.bytes <= BYTES);
            assert!(history.entries.len() <= ENTRIES);
        }
        let exact = text.clone();
        text = history
            .apply(DictationTarget::Content, false, &text)
            .unwrap();
        assert_eq!(
            history
                .apply(DictationTarget::Content, true, &text)
                .unwrap(),
            exact
        );
    }
    #[test]
    fn utf8_deltas_independent_redo_and_global_eviction() {
        let mut history = History::default();
        for i in 0..300 {
            history.record(
                DictationTarget::Content,
                Origin::Manual,
                &format!("é{i}👋"),
                &format!("é{}👋", i + 1),
            );
        }
        let mut text = "é300👋".to_string();
        for _ in 0..256 {
            text = history
                .apply(DictationTarget::Content, false, &text)
                .unwrap();
        }
        assert_eq!(text, "é44👋");
        assert!(
            history
                .apply(DictationTarget::Content, false, &text)
                .is_none()
        );
        for _ in 0..256 {
            text = history
                .apply(DictationTarget::Content, true, &text)
                .unwrap();
        }
        assert_eq!(text, "é300👋");
        history.record(DictationTarget::Context, Origin::Dictation, "", "context");
        assert!(history.availability().context.can_undo);
        assert!(
            history
                .apply(DictationTarget::Context, false, "wrong")
                .is_none()
        );
    }
}
