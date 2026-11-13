//! One shard of a table: a file of records (`record.rs`) mapped into memory, and an index in
//! memory from each key to where its value lies in the file. Keys and values are bytes here;
//! the table encodes them.
//!
//! A change is a record added at the end, so a crash cuts off at most the record being written,
//! which the next opening finds damaged. The records a key no longer needs are dead; once they
//! outweigh the live ones, the live ones are written to a new file on a thread of their own
//! (`compact.rs`), which then takes the old one's place.

mod compact;
mod damage;

use std::collections::HashMap;
use std::io;
use std::ops::Range;
use std::path::Path;

use self::compact::{Compaction, Live};
pub(super) use self::damage::Damage;
use super::mapped::{FolderLock, MappedFile};
use super::record::{self, HEADER_LEN};

/// Bytes a new file starts with: room for small tables without growing.
const NEW_LEN: usize = 64 * 1024;
/// Dead bytes a file keeps in any case: rewriting a small file to save little isn't worth it.
const DEAD_ALLOWED: usize = 1024 * 1024;

/// A shard's file and the index of its live values.
pub(super) struct Log {
    file: MappedFile,
    index: HashMap<Box<[u8]>, Entry>,
    /// Where the records end and the next goes.
    end: usize,
    /// Bytes of records no key needs any more.
    dead: usize,
    compaction: Option<Compaction>,
}

/// A key and what was recorded for it: its value, or `None` for a removal.
type Change = (Box<[u8]>, Option<Box<[u8]>>);

/// Where a key's newest record lies.
struct Entry {
    record_len: usize,
    value: Range<usize>,
}

impl Entry {
    /// Where the key and value of a record of `key_len` key bytes lie.
    fn live(&self, key_len: usize) -> Live {
        let key_start = self.value.end - self.record_len + record::KEY_START;
        Live {
            key: key_start..key_start + key_len,
            value: self.value.clone(),
        }
    }
}

impl Log {
    /// Opens the shard at `path` in a folder whose lock is held, keeping the records whose key
    /// and value pass `readable`. A damaged file is copied as it was to a `.bak` beside it, then
    /// rewritten with what could be read. A file of a newer format is refused untouched.
    pub(super) fn open(
        path: &Path,
        readable: impl Fn(&[u8], &[u8]) -> bool,
        held: &FolderLock,
    ) -> io::Result<(Log, Option<Damage>)> {
        let file = MappedFile::open(path, 1, held)?;
        // Left by a rewrite cut off before it was finished: the old file is still whole.
        let _ = std::fs::remove_file(path.with_extension("new"));
        if let Some(format) = record::unknown_format(file.bytes()) {
            let reason = format!(
                "{} has format {format}: a newer version wrote it",
                path.display()
            );
            return Err(io::Error::new(io::ErrorKind::InvalidData, reason));
        }
        let scan = record::scan(file.bytes());
        let mut log = Log {
            file,
            index: HashMap::new(),
            end: scan.end.max(HEADER_LEN),
            dead: 0,
            compaction: None,
        };
        let mut refused = 0;
        for found in &scan.records {
            let key: Box<[u8]> = log.file.bytes()[found.key.clone()].into();
            match found.value.clone() {
                Some(value) if readable(&key, &log.file.bytes()[value.clone()]) => {
                    log.index_set(key, found.len, value)
                }
                Some(_) => {
                    refused += 1;
                    log.index_remove(&key, found.len);
                }
                None => log.index_remove(&key, found.len),
            }
        }
        let damaged = scan.damaged > 0 || refused > 0;
        // Backed up before the file is lengthened, so the backup is the file as it was.
        let backup = damaged
            .then(|| damage::back_up(path, log.file.bytes()))
            .transpose()?;
        log.file.grow(NEW_LEN)?;
        let Some(backup) = backup else {
            log.file.bytes_mut()[..HEADER_LEN].copy_from_slice(&record::header());
            return Ok((log, None));
        };
        log.rewrite_now()?;
        let damage = Damage {
            path: path.to_path_buf(),
            damaged: scan.damaged,
            refused,
            backup,
        };
        Ok((log, Some(damage)))
    }

    pub(super) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let entry = self.index.get(key)?;
        Some(&self.file.bytes()[entry.value.clone()])
    }

    pub(super) fn set(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        self.put(key, value)?;
        self.compact_when_due()
    }

    pub(super) fn remove(&mut self, key: &[u8]) -> io::Result<()> {
        self.take_out(key)?;
        self.compact_when_due()
    }

    /// Removes every key: a compaction keeping none. If that fails before its file took this
    /// one's place, the log stays as it was.
    pub(super) fn clear(&mut self) -> io::Result<()> {
        self.abandon_compaction();
        self.start_compaction_of(Vec::new());
        self.finish_compaction()
    }

    /// Each key and value, in no order.
    pub(super) fn entries(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        let bytes = self.file.bytes();
        self.index
            .iter()
            .map(|(key, entry)| (&key[..], &bytes[entry.value.clone()]))
    }

    /// Writes the record setting `key` to `value` and indexes it.
    fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        let start = self.append(key, Some(value))?;
        let record_len = record::encoded_len(key, Some(value));
        let value_start = start + record_len - value.len();
        self.index_set(key.into(), record_len, value_start..start + record_len);
        Ok(())
    }

    /// Writes the record removing `key`, if it has a value, and takes it out of the index.
    fn take_out(&mut self, key: &[u8]) -> io::Result<()> {
        if self.index.contains_key(key) {
            self.append(key, None)?;
            self.index_remove(key, record::encoded_len(key, None));
        }
        Ok(())
    }

    fn index_set(&mut self, key: Box<[u8]>, record_len: usize, value: Range<usize>) {
        let entry = Entry { record_len, value };
        if let Some(old) = self.index.insert(key, entry) {
            self.dead += old.record_len;
        }
    }

    /// Takes `key` out of the index; its removal record of `record_len` bytes is dead too.
    fn index_remove(&mut self, key: &[u8], record_len: usize) {
        let old = self.index.remove(key).map_or(0, |old| old.record_len);
        self.dead += old + record_len;
    }

    /// Writes the record of `key` and `value` at the end, growing the file if needed; returns
    /// where it starts.
    fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> io::Result<usize> {
        let value_len = value.map_or(0, <[u8]>::len);
        if !record::fits(key.len(), value_len) {
            let size = key.len().saturating_add(value_len);
            let reason = format!("a record of {size} bytes is over 4 GiB");
            return Err(io::Error::new(io::ErrorKind::InvalidInput, reason));
        }
        let start = self.end;
        let end = start + record::encoded_len(key, value);
        let len = self.file.len();
        if end > len {
            self.file.grow(end.max(len * 2))?;
        }
        record::write(&mut self.file.bytes_mut()[start..end], start, key, value);
        self.end = end;
        Ok(start)
    }

    /// Puts a finished compaction in place, or starts one once dead records outweigh the live.
    fn compact_when_due(&mut self) -> io::Result<()> {
        if self
            .compaction
            .as_ref()
            .is_some_and(Compaction::is_finished)
        {
            return self.finish_compaction();
        }
        let live = self.end - HEADER_LEN - self.dead;
        if self.compaction.is_none() && self.dead > DEAD_ALLOWED && self.dead > live {
            self.start_compaction();
        }
        Ok(())
    }

    fn start_compaction(&mut self) {
        let live = self.index.iter();
        let live = live.map(|(key, entry)| entry.live(key.len())).collect();
        self.start_compaction_of(live);
    }

    /// Starts a compaction keeping the `live` records.
    fn start_compaction_of(&mut self, live: Vec<Live>) {
        self.compaction = Some(Compaction::start(self.file.path(), live, self.end));
    }

    /// Waits for the compaction, writes the changes made since it started into its file, and
    /// puts that in this one's place. If any of it fails before the rename, its file is removed
    /// and this one stays.
    fn finish_compaction(&mut self) -> io::Result<()> {
        let Some(compaction) = self.compaction.take() else {
            return Ok(());
        };
        let (listed_end, new) = (compaction.listed_end, compaction.new.clone());
        let since = self.changes_since(listed_end);
        let rewritten = compaction.wait().and_then(|mut rewritten| {
            rewritten.add_changes(&new, &since)?;
            Ok(rewritten)
        });
        let rewritten = rewritten.inspect_err(|_| {
            let _ = std::fs::remove_file(&new);
        })?;
        let replaced = self.file.replace_with(&new, rewritten.end.max(NEW_LEN));
        if replaced.is_err() && new.exists() {
            let _ = std::fs::remove_file(&new);
            return replaced;
        }
        // Renamed, even if mapping it failed: the file is the new one, and its index with it.
        (self.index, self.end, self.dead) = (rewritten.index, rewritten.end, rewritten.dead);
        replaced
    }

    /// The changes recorded from `start` to the end, in order.
    fn changes_since(&self, mut start: usize) -> Vec<Change> {
        let bytes = self.file.bytes();
        let mut changes = Vec::new();
        while let Some(found) = record::read(&bytes[..self.end], start) {
            let value = found.value.map(|value| bytes[value].into());
            changes.push((bytes[found.key].into(), value));
            start += found.len;
        }
        changes
    }

    /// Rewrites the file without its dead records before going on.
    fn rewrite_now(&mut self) -> io::Result<()> {
        self.abandon_compaction();
        self.start_compaction();
        self.finish_compaction()
    }

    /// Waits for a compaction under way and throws its file away.
    fn abandon_compaction(&mut self) {
        if let Some(compaction) = self.compaction.take() {
            let new = compaction.new.clone();
            let _ = compaction.wait();
            let _ = std::fs::remove_file(new);
        }
    }
}

impl Drop for Log {
    /// A compaction still running would write its file after a new opening of the shard.
    fn drop(&mut self) {
        self.abandon_compaction();
    }
}

#[cfg(test)]
mod tests;
