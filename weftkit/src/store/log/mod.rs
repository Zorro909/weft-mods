//! One shard of a table: a file of records (`record.rs`) mapped into memory, and an index in
//! memory from each key to where its value lies in the file. Keys and values are bytes here;
//! the table encodes them.
//!
//! A change is a record added at the end, so a crash cuts off at most the record being written,
//! which the next opening finds damaged.
//! The records a key no longer needs are dead; once they outweigh the live ones, the live ones
//! are written to a new file that takes the old one's place.

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use super::mapped::MappedFile;
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
}

/// Where a key's newest record lies.
struct Entry {
    record_len: usize,
    value: Range<usize>,
}

/// What opening a damaged file found and kept.
#[derive(Debug)]
pub(super) struct Damage {
    pub(super) path: PathBuf,
    /// Bytes that were no whole record.
    pub(super) damaged: usize,
    /// Whole records whose key or value the table could not read, each taken as a removal of
    /// its key: an older value coming back would be wrong too.
    pub(super) refused: usize,
    /// The file as it was before it was repaired.
    pub(super) backup: PathBuf,
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

impl Log {
    /// Opens the shard at `path`, keeping the records whose key and value pass `readable`. A
    /// damaged file is copied as it was to a `.bak` beside it, then rewritten with what could be
    /// read.
    pub(super) fn open(
        path: &Path,
        readable: impl Fn(&[u8], &[u8]) -> bool,
    ) -> io::Result<(Log, Option<Damage>)> {
        let file = MappedFile::open(path, 1)?;
        // Left by a rewrite cut off before it was finished: the old file is still whole. Only
        // removed once the lock shows no other server is in the middle of one.
        let _ = std::fs::remove_file(path.with_extension("new"));
        let scan = record::scan(file.bytes());
        let mut log = Log {
            file,
            index: HashMap::new(),
            end: scan.end.max(HEADER_LEN),
            dead: 0,
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
        if scan.damaged == 0 && refused == 0 {
            log.file.grow(NEW_LEN)?;
            log.file.bytes_mut()[..HEADER_LEN].copy_from_slice(&record::header());
            return Ok((log, None));
        }
        let backup = back_up(path, log.file.bytes())?;
        log.rewrite()?;
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
        let start = self.append(key, Some(value))?;
        let record_len = record::encoded_len(key, Some(value));
        let value_start = start + record_len - value.len();
        self.index_set(key.into(), record_len, value_start..start + record_len);
        self.rewrite_if_wasteful()
    }

    pub(super) fn remove(&mut self, key: &[u8]) -> io::Result<()> {
        if !self.index.contains_key(key) {
            return Ok(());
        }
        self.append(key, None)?;
        self.index_remove(key, record::encoded_len(key, None));
        self.rewrite_if_wasteful()
    }

    /// Removes every key; if that can't be saved, the log stays as it was.
    pub(super) fn clear(&mut self) -> io::Result<()> {
        let index = std::mem::take(&mut self.index);
        let cleared = self.rewrite();
        if cleared.is_err() {
            self.index = index;
        }
        cleared
    }

    /// Each key and value, in no order.
    pub(super) fn entries(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        let bytes = self.file.bytes();
        self.index
            .iter()
            .map(|(key, entry)| (&key[..], &bytes[entry.value.clone()]))
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
            let message = format!(
                "a record of {} bytes is over 4 GiB",
                key.len().saturating_add(value_len)
            );
            return Err(io::Error::new(io::ErrorKind::InvalidInput, message));
        }
        let start = self.end;
        let end = start + record::encoded_len(key, value);
        let len = self.file.bytes().len();
        if end > len {
            self.file.grow(end.max(len * 2))?;
        }
        record::write(&mut self.file.bytes_mut()[start..end], start, key, value);
        self.end = end;
        Ok(start)
    }

    fn rewrite_if_wasteful(&mut self) -> io::Result<()> {
        let live = self.end - HEADER_LEN - self.dead;
        if self.dead > DEAD_ALLOWED && self.dead > live {
            self.rewrite()?;
        }
        Ok(())
    }

    /// Writes the live records to a new file, which then takes this one's place.
    fn rewrite(&mut self) -> io::Result<()> {
        let new = self.file.path().with_extension("new");
        let mut out = BufWriter::new(std::fs::File::create(&new)?);
        out.write_all(&record::header())?;
        let mut index = HashMap::with_capacity(self.index.len());
        let mut end = HEADER_LEN;
        let mut buffer = Vec::new();
        for (key, entry) in &self.index {
            let value = &self.file.bytes()[entry.value.clone()];
            buffer.resize(entry.record_len, 0);
            record::write(&mut buffer, end, key, Some(value));
            out.write_all(&buffer)?;
            let value_start = end + entry.record_len - value.len();
            let value = value_start..end + entry.record_len;
            index.insert(
                key.clone(),
                Entry {
                    record_len: entry.record_len,
                    value,
                },
            );
            end += entry.record_len;
        }
        out.into_inner()
            .map_err(|error| error.into_error())?
            .sync_all()?;
        self.file.replace_with(&new, end.max(NEW_LEN))?;
        (self.index, self.end, self.dead) = (index, end, 0);
        Ok(())
    }
}

/// Writes `bytes`, the file at `path` as it is, to the first free `<path>.bak`, `<path>.1.bak`,
/// .. and returns where. Not copied from the file itself: on Windows its lock keeps out every
/// other handle.
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

#[cfg(test)]
mod tests;
