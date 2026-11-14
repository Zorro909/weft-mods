//! A table's two lock designs: its files, each behind a `RwLock`, so any number of regions read
//! one at once and a write waits for them. Read-optimized: one file. Write-optimized: sixteen,
//! chosen by the key's hash, so regions changing different keys seldom wait for each other.

use std::io;
use std::path::Path;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::log::{Damage, Log};
use super::mapped::FolderLock;

/// The files of a write-optimized table. Part of the file layout: a key's file follows from it.
const SHARDS: usize = 16;

/// Which lock design a table has.
#[derive(Clone, Copy, Debug)]
pub(super) enum Locking {
    ReadOptimized,
    WriteOptimized,
}

/// A table's open files, each behind its lock, and its folder's lock.
pub(super) struct Shards {
    /// Before the lock, so the files are closed before it is let go.
    files: Box<[RwLock<Log>]>,
    _lock: FolderLock,
}

impl Shards {
    /// Opens the files of a table in `folder`, making it if missing; see `Log::open` for
    /// `readable` and the damage found.
    pub(super) fn open(
        folder: &Path,
        locking: Locking,
        readable: &dyn Fn(&[u8], &[u8]) -> bool,
    ) -> io::Result<(Shards, Vec<Damage>)> {
        std::fs::create_dir_all(folder)?;
        let lock = FolderLock::take(folder)?;
        let names = match locking {
            Locking::ReadOptimized => vec!["table.log".to_string()],
            Locking::WriteOptimized => (0..SHARDS)
                .map(|shard| format!("shard-{shard:02}.log"))
                .collect(),
        };
        let (mut files, mut damages) = (Vec::new(), Vec::new());
        for name in names {
            let (log, damage) = Log::open(&folder.join(name), readable, &lock)?;
            files.push(RwLock::new(log));
            damages.extend(damage);
        }
        let shards = Shards {
            files: files.into(),
            _lock: lock,
        };
        Ok((shards, damages))
    }

    /// `look` run on the file of `key`, which nobody changes meanwhile.
    pub(super) fn read<R>(&self, key: &[u8], look: impl FnOnce(&Log) -> R) -> R {
        look(&read(self.file_of(key)))
    }

    /// `change` run on the file of `key`, which nobody else uses meanwhile.
    pub(super) fn write<R>(&self, key: &[u8], change: impl FnOnce(&mut Log) -> R) -> R {
        change(&mut write(self.file_of(key)))
    }

    /// `look` run on each file in turn.
    pub(super) fn read_each(&self, mut look: impl FnMut(&Log)) {
        self.files.iter().for_each(|file| look(&read(file)));
    }

    /// `change` run on each file in turn.
    pub(super) fn write_each(&self, mut change: impl FnMut(&mut Log)) {
        self.files.iter().for_each(|file| change(&mut write(file)));
    }

    /// The file a key's records go to: a hash that stays the same on every machine and release.
    fn file_of(&self, key: &[u8]) -> &RwLock<Log> {
        &self.files[crc32fast::hash(key) as usize % self.files.len()]
    }
}

/// `file` locked for reading, even if a panic left it poisoned: a log is whole between calls.
fn read(file: &RwLock<Log>) -> RwLockReadGuard<'_, Log> {
    file.read().unwrap_or_else(PoisonError::into_inner)
}

/// `file` locked for changing, even if a panic left it poisoned.
fn write(file: &RwLock<Log>) -> RwLockWriteGuard<'_, Log> {
    file.write().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::folder::test_folder;

    #[test]
    fn a_write_optimized_table_spreads_its_keys_over_sixteen_files() {
        let folder = test_folder("shards-spread");
        let (shards, _) =
            Shards::open(&folder, Locking::WriteOptimized, &|_, _| true).expect("the shards open");
        for number in 0..64u32 {
            let key = number.to_le_bytes();
            shards
                .write(&key, |log| log.set(&key, b"v"))
                .expect("saved");
        }
        let mut used = 0;
        shards.read_each(|log| used += usize::from(log.entries().next().is_some()));
        assert_eq!(used, SHARDS, "every shard holds some keys");
        let files = std::fs::read_dir(&folder).expect("listed").count();
        assert_eq!(files, SHARDS + 1, "one file each, and the lock");
    }

    #[test]
    fn a_poisoned_lock_still_opens() {
        let folder = test_folder("shards-poison");
        for locking in [Locking::ReadOptimized, Locking::WriteOptimized] {
            let (shards, _) =
                Shards::open(&folder.join(format!("{locking:?}")), locking, &|_, _| true)
                    .expect("it opens");
            let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                shards.write(b"k", |_| panic!("a mod's closure fails"))
            }));
            assert!(poisoned.is_err(), "the panic came through");
            shards
                .write(b"k", |log| log.set(b"k", b"v"))
                .expect("saved after the panic");
            shards.write_each(|log| log.clear().expect("cleared after the panic"));
        }
    }

    #[test]
    fn a_table_in_use_is_refused_without_touching_its_files() {
        let folder = test_folder("shards-in-use");
        let _first = Shards::open(&folder, Locking::ReadOptimized, &|_, _| true).expect("it opens");
        std::fs::write(folder.join("table.new"), b"a rewrite in progress").expect("written");
        let second = Shards::open(&folder, Locking::ReadOptimized, &|_, _| true);
        let error = second.err().expect("a second server is refused");
        assert!(
            error.to_string().contains("lock"),
            "naming the lock: {error}"
        );
        assert!(
            folder.join("table.new").exists(),
            "before it touched the rewrite"
        );
    }
}
