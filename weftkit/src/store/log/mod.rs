//! One shard of a table: a file of records (`record.rs`) mapped into memory, and an index in
//! memory from each key to where its value lies in the file. Keys and values are bytes here;
//! the table encodes them.
//!
//! A change is a record added at the end, so a crash cuts off at most the record being written.
//! Once the records no key needs outweigh the live ones, the live ones are rewritten
//! (`compact.rs`).

mod compact;

use std::collections::HashMap;
use std::fmt;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

use self::compact::{Compaction, Live};
use super::mapped::{FolderLock, MappedFile};
use super::record::{self, HEADER_LEN};

/// Bytes a new file starts with: room for small tables without growing.
const NEW_LEN: usize = 64 * 1024;
/// Dead bytes a file keeps in any case: rewriting a small file to save little isn't worth it.
const DEAD_ALLOWED: usize = 1024 * 1024;

/// A shard's file and the index of its live values.
pub(super) struct Log {
    file: MappedFile,
    index: Index,
    compaction: Option<Compaction>,
}

/// Where each live key's value lies in a file, where its records end, and how many of their
/// bytes no key needs any more.
struct Index {
    keys: HashMap<Box<[u8]>, Range<usize>>,
    end: usize,
    dead: usize,
}

impl Index {
    fn new() -> Index {
        Index {
            keys: HashMap::new(),
            end: HEADER_LEN,
            dead: 0,
        }
    }

    /// Takes in the record of `key` and `value` written at the end; a removal has no value.
    fn add(&mut self, key: &[u8], value: Option<&[u8]>) {
        let record_len = record::encoded_len(key, value);
        self.end += record_len;
        let old = match value {
            Some(value) => self
                .keys
                .insert(key.into(), self.end - value.len()..self.end),
            None => {
                self.dead += record_len;
                self.keys.remove(key)
            }
        };
        self.dead += old.map_or(0, |old| record::KEY_START + key.len() + old.len());
    }

    /// Where the key and value of each live record lie.
    fn live(&self) -> Vec<Live> {
        let live = self.keys.iter().map(|(key, value)| Live {
            key: value.start - key.len()..value.start,
            value: value.clone(),
        });
        live.collect()
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
        let mut index = Index::new();
        let mut refused = 0;
        for found in &scan.records {
            let key = &file.bytes()[found.key.clone()];
            let value = found.value.clone().map(|value| &file.bytes()[value]);
            let kept = value.filter(|value| readable(key, value));
            refused += usize::from(kept.is_none() && value.is_some());
            // After damage the next record lies further on.
            index.end = found.start;
            index.add(key, kept);
        }
        index.end = scan.end.max(HEADER_LEN);
        let mut log = Log {
            file,
            index,
            compaction: None,
        };
        let damaged = scan.damaged > 0 || refused > 0;
        // Backed up before the file is lengthened, so the backup is the file as it was.
        let backup = damaged
            .then(|| back_up(path, log.file.bytes()))
            .transpose()?;
        log.file.grow(NEW_LEN)?;
        let Some(backup) = backup else {
            log.file.bytes_mut()[..HEADER_LEN].copy_from_slice(&record::header());
            return Ok((log, None));
        };
        log.start_compaction(log.index.live());
        log.finish_compaction()?;
        let damage = Damage {
            path: path.to_path_buf(),
            damaged: scan.damaged,
            refused,
            backup,
        };
        Ok((log, Some(damage)))
    }

    pub(super) fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let value = self.index.keys.get(key)?;
        Some(&self.file.bytes()[value.clone()])
    }

    pub(super) fn set(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        self.append(key, Some(value))?;
        self.compact_when_due()
    }

    pub(super) fn remove(&mut self, key: &[u8]) -> io::Result<()> {
        if self.index.keys.contains_key(key) {
            self.append(key, None)?;
        }
        self.compact_when_due()
    }

    /// Removes every key: a compaction keeping none. If that fails before its file took this
    /// one's place, the log stays as it was.
    pub(super) fn clear(&mut self) -> io::Result<()> {
        self.abandon_compaction();
        self.start_compaction(Vec::new());
        self.finish_compaction()
    }

    /// Each key and value, in no order.
    pub(super) fn entries(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        let bytes = self.file.bytes();
        self.index
            .keys
            .iter()
            .map(|(key, value)| (&key[..], &bytes[value.clone()]))
    }

    /// Writes the record of `key` and `value` at the end, growing the file if needed, and
    /// indexes it.
    fn append(&mut self, key: &[u8], value: Option<&[u8]>) -> io::Result<()> {
        let value_len = value.map_or(0, <[u8]>::len);
        if !record::fits(key.len(), value_len) {
            let size = key.len().saturating_add(value_len);
            let reason = format!("a record of {size} bytes is over 4 GiB");
            return Err(io::Error::new(io::ErrorKind::InvalidInput, reason));
        }
        let start = self.index.end;
        let end = start + record::encoded_len(key, value);
        let len = self.file.len();
        if end > len {
            self.file.grow(end.max(len * 2))?;
        }
        record::write(&mut self.file.bytes_mut()[start..end], start, key, value);
        self.index.add(key, value);
        Ok(())
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
        let dead = self.index.dead;
        let live = self.index.end - HEADER_LEN - dead;
        if self.compaction.is_none() && dead > DEAD_ALLOWED && dead > live {
            self.start_compaction(self.index.live());
        }
        Ok(())
    }

    /// Starts a compaction keeping the `live` records.
    fn start_compaction(&mut self, live: Vec<Live>) {
        self.compaction = Some(Compaction::start(self.file.path(), live, self.index.end));
    }

    /// Waits for the compaction, writes the changes made since it started into its file, and
    /// puts that in this one's place. If any of it fails before the rename, its file is removed
    /// and this one stays.
    fn finish_compaction(&mut self) -> io::Result<()> {
        let Some(compaction) = self.compaction.take() else {
            return Ok(());
        };
        let new = self.file.path().with_extension("new");
        let listed_end = compaction.listed_end;
        let finished = compaction.wait().and_then(|mut index| {
            let changes = &self.file.bytes()[..self.index.end];
            compact::add_changes(&mut index, &new, changes, listed_end)?;
            let replaced = self.file.replace_with(&new, index.end.max(NEW_LEN));
            // Renamed, even if mapping it failed: the file is the new one, and its index with it.
            if !new.exists() {
                self.index = index;
            }
            replaced
        });
        if finished.is_err() {
            let _ = std::fs::remove_file(&new);
        }
        finished
    }

    /// Waits for a compaction under way and throws its file away.
    fn abandon_compaction(&mut self) {
        if let Some(compaction) = self.compaction.take() {
            let _ = compaction.wait();
            let _ = std::fs::remove_file(self.file.path().with_extension("new"));
        }
    }
}

/// What opening a damaged file found and kept.
#[derive(Debug)]
pub(in crate::store) struct Damage {
    pub(in crate::store) path: PathBuf,
    /// Bytes that were no whole record.
    pub(in crate::store) damaged: usize,
    /// Whole records whose key or value the table could not read, each taken as a removal of
    /// its key: an older value coming back would be wrong too.
    pub(in crate::store) refused: usize,
    /// The file as it was before it was repaired.
    pub(in crate::store) backup: PathBuf,
}

impl fmt::Display for Damage {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(
            out,
            "{} was damaged ({} bytes unreadable, {} records of other types); kept what could be \
             read, and the file as it was is {}",
            self.path.display(),
            self.damaged,
            self.refused,
            self.backup.display()
        )
    }
}

/// Writes `bytes`, the file at `path` as it is, to the first free `<path>.bak`, `<path>.1.bak`,
/// .., and gives that path.
fn back_up(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let mut backup = path.with_file_name(format!("{name}.bak"));
    let mut number = 1;
    while backup.exists() {
        backup = path.with_file_name(format!("{name}.{number}.bak"));
        number += 1;
    }
    std::fs::write(&backup, bytes)?;
    Ok(backup)
}

impl Drop for Log {
    /// A compaction still running would write its file after a new opening of the shard.
    fn drop(&mut self) {
        self.abandon_compaction();
    }
}

#[cfg(test)]
mod tests;
