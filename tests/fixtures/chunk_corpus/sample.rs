//! A small synthetic Rust fixture for the chunker corpus, not lifted from
//! any real source file. ASCII-only by construction, so char offsets and
//! byte offsets coincide (see build_chunk_fixtures.py's own note).

use std::collections::HashMap;

/// One entry the ledger tracks.
pub struct Entry {
    pub key: String,
    pub value: i64,
    pub tag: Option<String>,
}

impl Entry {
    pub fn new(key: &str, value: i64) -> Self {
        Entry {
            key: key.to_string(),
            value,
            tag: None,
        }
    }

    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tag = Some(tag.to_string());
        self
    }
}

/// A tiny in-memory ledger used only to give the chunker something with
/// real shape: several functions, a struct, an enum, and a small amount of
/// control flow, spread across enough bytes to force at least one merge
/// boundary at the desired chunk length.
pub struct Ledger {
    entries: HashMap<String, Entry>,
    total: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    NotFound(String),
    DuplicateKey(String),
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            entries: HashMap::new(),
            total: 0,
        }
    }

    pub fn insert(&mut self, entry: Entry) -> Result<(), LedgerError> {
        if self.entries.contains_key(&entry.key) {
            return Err(LedgerError::DuplicateKey(entry.key.clone()));
        }
        self.total += entry.value;
        self.entries.insert(entry.key.clone(), entry);
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Result<Entry, LedgerError> {
        match self.entries.remove(key) {
            Some(entry) => {
                self.total -= entry.value;
                Ok(entry)
            }
            None => Err(LedgerError::NotFound(key.to_string())),
        }
    }

    pub fn total(&self) -> i64 {
        self.total
    }

    pub fn tagged(&self, tag: &str) -> Vec<&Entry> {
        let mut out = Vec::new();
        for entry in self.entries.values() {
            if entry.tag.as_deref() == Some(tag) {
                out.push(entry);
            }
        }
        out
    }
}

fn describe(error: &LedgerError) -> String {
    match error {
        LedgerError::NotFound(key) => format!("no such entry: {key}"),
        LedgerError::DuplicateKey(key) => format!("duplicate key: {key}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_then_remove_round_trips() {
        let mut ledger = Ledger::new();
        ledger.insert(Entry::new("a", 10)).unwrap();
        assert_eq!(ledger.total(), 10);
        let removed = ledger.remove("a").unwrap();
        assert_eq!(removed.value, 10);
        assert_eq!(ledger.total(), 0);
    }

    #[test]
    fn duplicate_insert_is_an_error() {
        let mut ledger = Ledger::new();
        ledger.insert(Entry::new("a", 1)).unwrap();
        let err = ledger.insert(Entry::new("a", 2)).unwrap_err();
        assert_eq!(err, LedgerError::DuplicateKey("a".to_string()));
    }
}
