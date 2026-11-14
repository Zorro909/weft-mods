//! Rewriting a shard without its dead records, on a thread of its own: the shard's lock is held
//! only to list where its live records lie and, later, to put the new file in place
//! (`Log::finish_compaction`).
//!
//! The thread reads the listed records through a handle of its own, never the map: records are
//! only ever added after the list's end, so the listed ones don't change meanwhile. It writes
//! them to `<file>.new`. The shard adds the records made meanwhile to that file
//! (`add_changes`) before it takes the old one's place, so a crash at any point leaves one whole
//! file with every change.

use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::ops::Range;
use std::path::Path;
use std::thread::JoinHandle;

use super::Index;
use crate::store::record;

/// Where a live record's key and value lie in the file; the value follows the key.
pub(super) struct Live {
    pub(super) key: Range<usize>,
    pub(super) value: Range<usize>,
}

/// A rewrite under way.
pub(super) struct Compaction {
    thread: JoinHandle<io::Result<Index>>,
    /// Where the records ended when the live ones were listed.
    pub(super) listed_end: usize,
}

/// Writes the records of `bytes`, the shard's file up to where its records end, from
/// `listed_end` on (the changes made since the live records were listed) after them in `new`.
pub(super) fn add_changes(
    index: &mut Index,
    new: &Path,
    bytes: &[u8],
    mut listed_end: usize,
) -> io::Result<()> {
    if listed_end == bytes.len() {
        return Ok(());
    }
    let mut out = BufWriter::new(OpenOptions::new().append(true).open(new)?);
    while let Some(found) = record::read(bytes, listed_end) {
        let value = found.value.map(|value| &bytes[value]);
        write_record(&mut out, index, &bytes[found.key], value)?;
        listed_end += found.len;
    }
    // Not synced: a crash keeps what was written, and power lost before the disk has it
    // leaves a cut-off end, which the next opening repairs.
    out.flush()
}

impl Compaction {
    /// Starts writing the `live` records of `path`, whose records end at `listed_end`, to
    /// `<path>.new` on a thread.
    pub(super) fn start(path: &Path, mut live: Vec<Live>, listed_end: usize) -> Compaction {
        live.sort_unstable_by_key(|record| record.key.start);
        let (path, new) = (path.to_path_buf(), path.with_extension("new"));
        let thread = std::thread::spawn(move || rewrite(&path, &new, &live));
        Compaction { thread, listed_end }
    }

    pub(super) fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }

    /// The rewrite's outcome, once its thread has ended.
    pub(super) fn wait(self) -> io::Result<Index> {
        self.thread
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("the compaction thread panicked")))
    }
}

/// Writes the `live` records of `path`, sorted by place, to `new`, made anew and synced.
fn rewrite(path: &Path, new: &Path, live: &[Live]) -> io::Result<Index> {
    let mut source = BufReader::new(File::open(path)?);
    let mut out = BufWriter::new(File::create(new)?);
    out.write_all(&record::header())?;
    let mut index = Index::new();
    let (mut read_to, mut parts) = (0, Vec::new());
    for record in live {
        source.seek_relative(record.key.start as i64 - read_to as i64)?;
        parts.resize(record.value.end - record.key.start, 0);
        source.read_exact(&mut parts)?;
        read_to = record.value.end;
        let (key, value) = parts.split_at(record.key.len());
        write_record(&mut out, &mut index, key, Some(value))?;
    }
    out.into_inner()
        .map_err(io::IntoInnerError::into_error)?
        .sync_all()?;
    Ok(index)
}

/// Writes the record of `key` and `value` to `out`, the file `index` is of, and indexes it.
fn write_record(
    out: &mut impl Write,
    index: &mut Index,
    key: &[u8],
    value: Option<&[u8]>,
) -> io::Result<()> {
    let mut encoded = vec![0; record::encoded_len(key, value)];
    record::write(&mut encoded, index.end, key, value);
    out.write_all(&encoded)?;
    index.add(key, value);
    Ok(())
}
