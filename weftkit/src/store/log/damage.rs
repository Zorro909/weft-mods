//! What opening a damaged shard reports, and the backup of the file as it was.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

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
/// .. and returns where.
pub(super) fn back_up(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
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
