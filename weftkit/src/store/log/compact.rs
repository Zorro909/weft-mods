//! Rewriting a shard without its dead records, on a thread of its own: the shard's lock is held
//! only to list where its live records lie and, later, to put the new file in place
//! (`Log::finish_compaction`).
//!
//! The thread reads the listed records through a handle of its own, never the map: records are
//! only ever added after the list's end, so the listed ones don't change meanwhile. It writes
//! them one after the other to `<file>.new`. Changes made meanwhile are the records after the
//! list's end: the shard adds them to the new file (`Rewritten::add_changes`) before that takes
//! the old one's place, so a crash at any point leaves one whole file with every change.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use super::{Change, Entry};
use crate::store::record::{self, HEADER_LEN};

/// Where a live record's key and value lie in the file; the value follows the key.
pub(super) struct Live {
    pub(super) key: Range<usize>,
    pub(super) value: Range<usize>,
}

/// A rewrite under way.
pub(super) struct Compaction {
    thread: JoinHandle<io::Result<Rewritten>>,
    /// Where the records ended when the live ones were listed.
    pub(super) listed_end: usize,
    pub(super) new: PathBuf,
}

/// The live records written to the new file: their index there, where they end, and the bytes
/// of records no key needs any more.
pub(super) struct Rewritten {
    pub(super) index: HashMap<Box<[u8]>, Entry>,
    pub(super) end: usize,
    pub(super) dead: usize,
}

impl Rewritten {
    /// Writes `changes`, made since the live records were listed, after them in `new`.
    pub(super) fn add_changes(&mut self, new: &Path, changes: &[Change]) -> io::Result<()> {
        if changes.is_empty() {
            return Ok(());
        }
        let mut out = BufWriter::new(OpenOptions::new().append(true).open(new)?);
        let mut encoded = Vec::new();
        for (key, value) in changes {
            let value = value.as_deref();
            if value.is_none() && !self.index.contains_key(key) {
                continue;
            }
            let record_len = record::encoded_len(key, value);
            encoded.resize(record_len, 0);
            record::write(&mut encoded, self.end, key, value);
            out.write_all(&encoded)?;
            let old = match value {
                Some(value) => {
                    let value = self.end + record_len - value.len()..self.end + record_len;
                    self.index.insert(key.clone(), Entry { record_len, value })
                }
                None => {
                    self.dead += record_len;
                    self.index.remove(key)
                }
            };
            self.dead += old.map_or(0, |old| old.record_len);
            self.end += record_len;
        }
        // Not synced: a crash keeps what was written, and power lost before the disk has it
        // leaves a cut-off end, which the next opening repairs.
        out.flush()
    }
}

impl Compaction {
    /// Starts writing the `live` records of `path`, whose records end at `listed_end`, to
    /// `<path>.new` on a thread.
    pub(super) fn start(path: &Path, mut live: Vec<Live>, listed_end: usize) -> Compaction {
        live.sort_unstable_by_key(|record| record.key.start);
        let new = path.with_extension("new");
        let (path, target) = (path.to_path_buf(), new.clone());
        let thread = std::thread::spawn(move || rewrite(&path, &target, &live));
        Compaction {
            thread,
            listed_end,
            new,
        }
    }

    pub(super) fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }

    /// The rewrite's outcome, once its thread has ended.
    pub(super) fn wait(self) -> io::Result<Rewritten> {
        self.thread
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("the compaction thread panicked")))
    }
}

/// Writes the `live` records of `path`, sorted by place, to `new`, made anew and synced.
fn rewrite(path: &Path, new: &Path, live: &[Live]) -> io::Result<Rewritten> {
    let mut source = BufReader::new(File::open(path)?);
    let mut out = BufWriter::new(File::create(new)?);
    out.write_all(&record::header())?;
    let mut read_to = 0;
    let mut index = HashMap::with_capacity(live.len());
    let mut end = HEADER_LEN;
    let (mut parts, mut encoded) = (Vec::new(), Vec::new());
    for record in live {
        source.seek_relative(record.key.start as i64 - read_to as i64)?;
        parts.resize(record.value.end - record.key.start, 0);
        source.read_exact(&mut parts)?;
        read_to = record.value.end;
        let (key, value) = parts.split_at(record.key.len());
        let record_len = record::encoded_len(key, Some(value));
        encoded.resize(record_len, 0);
        record::write(&mut encoded, end, key, Some(value));
        out.write_all(&encoded)?;
        let value = end + record_len - value.len()..end + record_len;
        index.insert(key.into(), Entry { record_len, value });
        end += record_len;
    }
    out.into_inner()
        .map_err(io::IntoInnerError::into_error)?
        .sync_all()?;
    Ok(Rewritten {
        index,
        end,
        dead: 0,
    })
}
