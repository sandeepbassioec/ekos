use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub command: Vec<u8>,
}

pub struct RaftLog {
    entries: Vec<LogEntry>,
    commit_index: u64,
    pub last_applied: u64,
}

impl RaftLog {
    pub fn new() -> Self {
        RaftLog {
            entries: Vec::new(),
            commit_index: 0,
            last_applied: 0,
        }
    }

    pub fn append(&mut self, term: u64, command: Vec<u8>) -> u64 {
        let index = self.entries.len() as u64 + 1;
        self.entries.push(LogEntry {
            term,
            index,
            command,
        });
        index
    }

    pub fn get(&self, index: u64) -> Option<&LogEntry> {
        if index == 0 || index > self.entries.len() as u64 {
            return None;
        }
        self.entries.get((index - 1) as usize)
    }

    pub fn last_index(&self) -> u64 {
        self.entries.len() as u64
    }

    pub fn last_term(&self) -> u64 {
        self.entries.last().map(|e| e.term).unwrap_or(0)
    }

    pub fn commit_up_to(&mut self, index: u64) {
        if index <= self.last_index() {
            self.commit_index = index;
        } else {
            self.commit_index = self.last_index();
        }
    }

    pub fn committed_entries(&self) -> &[LogEntry] {
        if self.commit_index == 0 {
            return &[];
        }
        let end = self.commit_index.min(self.entries.len() as u64) as usize;
        &self.entries[..end]
    }

    pub fn truncate_from(&mut self, index: u64) {
        if index == 0 {
            self.entries.clear();
            self.commit_index = 0;
        } else if index <= self.entries.len() as u64 {
            self.entries.truncate((index - 1) as usize);
            if self.commit_index >= index {
                self.commit_index = index.saturating_sub(1);
            }
        }
    }

    pub fn contains(&self, index: u64, term: u64) -> bool {
        match self.get(index) {
            Some(e) => e.term == term,
            None => index == 0, // index=0 is the sentinel "empty" case
        }
    }

    pub fn commit_index(&self) -> u64 {
        self.commit_index
    }
}

impl Default for RaftLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_returns_correct_index() {
        let mut log = RaftLog::new();
        let idx = log.append(1, b"cmd".to_vec());
        assert_eq!(idx, 1);
        let idx2 = log.append(1, b"cmd2".to_vec());
        assert_eq!(idx2, 2);
    }

    #[test]
    fn get_existing() {
        let mut log = RaftLog::new();
        log.append(1, b"hello".to_vec());
        let e = log.get(1).unwrap();
        assert_eq!(e.command, b"hello");
    }

    #[test]
    fn get_missing_returns_none() {
        let log = RaftLog::new();
        assert!(log.get(99).is_none());
    }

    #[test]
    fn last_index_after_3_appends() {
        let mut log = RaftLog::new();
        log.append(1, vec![]);
        log.append(1, vec![]);
        log.append(1, vec![]);
        assert_eq!(log.last_index(), 3);
    }

    #[test]
    fn last_term_correct() {
        let mut log = RaftLog::new();
        log.append(1, vec![]);
        log.append(2, vec![]);
        assert_eq!(log.last_term(), 2);
    }

    #[test]
    fn commit_up_to_advances_commit_index() {
        let mut log = RaftLog::new();
        log.append(1, vec![]);
        log.append(1, vec![]);
        log.commit_up_to(2);
        assert_eq!(log.commit_index(), 2);
    }

    #[test]
    fn committed_entries_returns_correct_slice() {
        let mut log = RaftLog::new();
        log.append(1, b"a".to_vec());
        log.append(1, b"b".to_vec());
        log.append(1, b"c".to_vec());
        log.commit_up_to(2);
        let committed = log.committed_entries();
        assert_eq!(committed.len(), 2);
        assert_eq!(committed[0].command, b"a");
        assert_eq!(committed[1].command, b"b");
    }

    #[test]
    fn truncate_from_removes_entries() {
        let mut log = RaftLog::new();
        log.append(1, b"a".to_vec());
        log.append(1, b"b".to_vec());
        log.append(1, b"c".to_vec());
        log.truncate_from(2);
        assert_eq!(log.last_index(), 1);
        assert!(log.get(2).is_none());
    }

    #[test]
    fn contains_true_for_existing() {
        let mut log = RaftLog::new();
        log.append(3, b"x".to_vec());
        assert!(log.contains(1, 3));
    }

    #[test]
    fn contains_false_for_wrong_term() {
        let mut log = RaftLog::new();
        log.append(3, b"x".to_vec());
        assert!(!log.contains(1, 99));
    }

    #[test]
    fn log_starts_empty() {
        let log = RaftLog::new();
        assert_eq!(log.last_index(), 0);
        assert_eq!(log.last_term(), 0);
        assert!(log.committed_entries().is_empty());
    }

    #[test]
    fn append_100_entries_correct_indices() {
        let mut log = RaftLog::new();
        for i in 1..=100 {
            let idx = log.append(1, vec![i as u8]);
            assert_eq!(idx, i);
        }
        assert_eq!(log.last_index(), 100);
    }

    #[test]
    fn commit_cannot_exceed_last_index() {
        let mut log = RaftLog::new();
        log.append(1, vec![]);
        log.commit_up_to(999);
        assert_eq!(log.commit_index(), 1);
    }
}
