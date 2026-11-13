//! A table's two lock designs. Read-optimized: one file behind a `RwLock`, so any number of
//! regions read at once and a write waits for them. Write-optimized: sixteen files, each behind
//! a `Mutex` and chosen by the key's hash, so regions changing different keys seldom wait for
//! each other; a read locks its shard as a write does.
//!
//! The table's folder holds a lock file besides, taken before any file is opened and let go
//! after all are closed, so no other server or mod uses the table meanwhile.

use std::io;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock};

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

/// A table's open files, behind their locks, and its folder's lock.
pub(super) struct Shards {
    /// Before the lock, so the files are closed before it is let go.
    files: Files,
    _lock: FolderLock,
}

enum Files {
    One(RwLock<Log>),
    Many(Box<[Mutex<Log>]>),
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
        let mut damages = Vec::new();
        let mut open = |file: String| -> io::Result<Log> {
            let (log, damage) = Log::open(&folder.join(file), readable, &lock)?;
            damages.extend(damage);
            Ok(log)
        };
        let files = match locking {
            Locking::ReadOptimized => Files::One(RwLock::new(open("table.log".into())?)),
            Locking::WriteOptimized => Files::Many(
                (0..SHARDS)
                    .map(|shard| open(format!("shard-{shard:02}.log")).map(Mutex::new))
                    .collect::<io::Result<_>>()?,
            ),
        };
        let shards = Shards { files, _lock: lock };
        Ok((shards, damages))
    }

    /// `look` run on the file of `key`, which nobody changes meanwhile.
    pub(super) fn read<R>(&self, key: &[u8], look: impl FnOnce(&Log) -> R) -> R {
        match &self.files {
            Files::One(lock) => look(&lock.read().unwrap_or_else(PoisonError::into_inner)),
            Files::Many(shards) => look(&locked(&shards[shard_of(key)])),
        }
    }

    /// `change` run on the file of `key`, which nobody else uses meanwhile.
    pub(super) fn write<R>(&self, key: &[u8], change: impl FnOnce(&mut Log) -> R) -> R {
        match &self.files {
            Files::One(lock) => change(&mut lock.write().unwrap_or_else(PoisonError::into_inner)),
            Files::Many(shards) => change(&mut locked(&shards[shard_of(key)])),
        }
    }

    /// `look` run on each file in turn.
    pub(super) fn read_each(&self, mut look: impl FnMut(&Log)) {
        match &self.files {
            Files::One(lock) => look(&lock.read().unwrap_or_else(PoisonError::into_inner)),
            Files::Many(shards) => shards.iter().for_each(|shard| look(&locked(shard))),
        }
    }

    /// `change` run on each file in turn.
    pub(super) fn write_each(&self, mut change: impl FnMut(&mut Log)) {
        match &self.files {
            Files::One(lock) => change(&mut lock.write().unwrap_or_else(PoisonError::into_inner)),
            Files::Many(shards) => shards.iter().for_each(|shard| change(&mut locked(shard))),
        }
    }
}

/// The shard a key's records go to: a hash that stays the same on every machine and release.
fn shard_of(key: &[u8]) -> usize {
    crc32fast::hash(key) as usize % SHARDS
}

/// `mutex` locked, even if a panic left it poisoned: a log is whole between calls.
fn locked(mutex: &Mutex<Log>) -> MutexGuard<'_, Log> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::test_folder;

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
        let key = 7u32.to_le_bytes();
        assert_eq!(
            shards.read(&key, |log| log.get(&key).map(<[u8]>::to_vec)),
            Some(b"v".to_vec())
        );
        assert_eq!(
            std::fs::read_dir(&folder).expect("listed").count(),
            SHARDS + 1,
            "one file each, and the lock"
        );
    }

    #[test]
    fn a_read_optimized_table_is_one_file() {
        let folder = test_folder("shards-one");
        let (shards, _) =
            Shards::open(&folder, Locking::ReadOptimized, &|_, _| true).expect("it opens");
        shards
            .write(b"k", |log| log.set(b"k", b"v"))
            .expect("saved");
        shards.write_each(|log| log.clear().expect("cleared"));
        let mut count = 0;
        shards.read_each(|log| count += log.entries().count());
        assert_eq!(count, 0, "clearing each file empties the table");
        assert!(folder.join("table.log").exists(), "the file is table.log");
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
    fn a_folder_that_cannot_be_made_is_an_error() {
        let folder = test_folder("shards-blocked");
        std::fs::write(folder.join("file"), b"").expect("written");
        let opened = Shards::open(
            &folder.join("file").join("table"),
            Locking::ReadOptimized,
            &|_, _| true,
        );
        assert!(opened.is_err(), "a file is in the way");
    }

    #[test]
    fn a_table_in_use_is_refused_without_touching_its_files() {
        let folder = test_folder("shards-in-use");
        let _first = Shards::open(&folder, Locking::ReadOptimized, &|_, _| true).expect("it opens");
        std::fs::write(folder.join("table.new"), b"a rewrite in progress").expect("written");
        let second = Shards::open(&folder, Locking::ReadOptimized, &|_, _| true);
        assert!(second.is_err(), "a second server is refused");
        assert!(
            folder.join("table.new").exists(),
            "before it touched the rewrite"
        );
    }
}
