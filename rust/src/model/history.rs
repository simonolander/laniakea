use crate::model::border::Border;

pub struct History {
    entries: Vec<HistoryEntry>,
    current_index: usize,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum HistoryEntry {
    ToggleBorder(Border),
}

impl History {
    pub fn new() -> Self {
        History {
            entries: Vec::new(),
            current_index: 0,
        }
    }

    pub fn has_past(&self) -> bool {
        self.current_index > 0
    }

    pub fn has_future(&self) -> bool {
        self.current_index < self.entries.len()
    }

    /// Undo the last action in the history, if any, and return it
    pub fn undo(&mut self) -> Option<&HistoryEntry> {
        if self.has_past() {
            self.current_index -= 1;
            self.entries.get(self.current_index)
        } else {
            None
        }
    }

    /// Redo the last undone action in the history, if any, and return it
    pub fn redo(&mut self) -> Option<&HistoryEntry> {
        if self.has_future() {
            self.current_index += 1;
            self.entries.get(self.current_index - 1)
        } else {
            None
        }
    }

    pub fn push(&mut self, entry: HistoryEntry) {
        if self.has_future() {
            let mut future = self.entries[self.current_index..]
                .iter()
                .cloned()
                .rev()
                .collect();
            self.entries.append(&mut future);
            self.entries.push(entry);
        } else {
            self.entries.push(entry);
        }
        self.current_index = self.entries.len();
        assert!(self.has_past());
        assert!(!self.has_future());
    }
}

#[cfg(test)]
mod tests {
    use crate::model::border::Border;
    use crate::model::history::{History, HistoryEntry};
    use crate::model::position::Position;

    fn some_entry() -> HistoryEntry {
        HistoryEntry::ToggleBorder(Border::new(Position::new(0, 0), Position::new(0, 1)))
    }

    #[test]
    fn should_have_past_after_push() {
        let mut history = History::new();
        history.push(some_entry());
        assert!(history.has_past());
    }

    #[test]
    fn should_have_future_after_undo() {
        let mut history = History::new();
        history.push(some_entry());
        let entry = history.undo();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap(), &some_entry());
        assert!(history.has_future());
    }

    #[test]
    fn should_not_have_future_after_undo_redo() {
        let mut history = History::new();
        let entry = some_entry();
        history.push(entry.clone());
        let undo = history.undo();
        assert_eq!(undo.unwrap(), &entry);
        let redo = history.redo();
        assert_eq!(redo.unwrap(), &entry);
        assert!(!history.has_future());
    }
}
