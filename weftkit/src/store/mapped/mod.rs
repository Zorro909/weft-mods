//! A file mapped into memory, the store's only `unsafe`, and the lock that makes mapping it
//! sound. What is written into the map is the file's: the kernel writes it out even when the
//! process crashes.
//!
//! Mapping a file is unsafe because another process could change or shorten it under the map.
//! A table's files are only mapped while its folder's lock file is held (`FolderLock`), which
//! keeps out every other store, in this server or another. The lock file never moves, so the
//! files under it can be replaced. It is advisory on Unix: it keeps out every store, not a
//! program that ignores locks. Windows changes no file that is mapped, so a file's map is let go
//! before the file is lengthened or replaced.

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use memmap2::MmapMut;

/// Zeros written at a time when a file is lengthened.
const ZEROS_LEN: usize = 64 * 1024;

/// The lock of a table's folder, held as long as any of its files is mapped.
pub(super) struct FolderLock {
    _file: File,
}

impl FolderLock {
    /// Takes the lock of `folder`, which exists, or says why not.
    pub(super) fn take(folder: &Path) -> io::Result<FolderLock> {
        let path = folder.join("lock");
        let file = open_file(&path, true)?;
        let Err(error) = file.try_lock() else {
            return Ok(FolderLock { _file: file });
        };
        let error = io::Error::from(error);
        let reason = format!(
            "{} is held: is another server or mod using this table? ({error})",
            path.display()
        );
        Err(io::Error::new(error.kind(), reason))
    }
}

/// A file mapped into memory for reading and writing.
pub(super) struct MappedFile {
    path: PathBuf,
    /// `None` while the file is changed, and after mapping it again failed: then reading it
    /// panics and the next `grow` tries again.
    map: Option<MmapMut>,
}

impl MappedFile {
    /// Opens `path` in a folder whose lock is held, making it if missing, at least `min_len`
    /// bytes long; added bytes are zero.
    pub(super) fn open(path: &Path, min_len: usize, _held: &FolderLock) -> io::Result<MappedFile> {
        let file = open_file(path, true)?;
        lengthen(&file, min_len.max(1))?;
        let map = Some(map(&file)?);
        Ok(MappedFile {
            path: path.to_path_buf(),
            map,
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn bytes(&self) -> &[u8] {
        let path = &self.path;
        let map = self.map.as_deref();
        map.unwrap_or_else(|| panic!("{} could not be mapped again", path.display()))
    }

    pub(super) fn bytes_mut(&mut self) -> &mut [u8] {
        let path = &self.path;
        let map = self.map.as_deref_mut();
        map.unwrap_or_else(|| panic!("{} could not be mapped again", path.display()))
    }

    /// The bytes mapped: none while the file isn't.
    pub(super) fn len(&self) -> usize {
        self.map.as_ref().map_or(0, |map| map.len())
    }

    /// Makes the file `len` bytes long, if it is shorter; the bytes added are zero. If that
    /// fails, the file is mapped again as far as it got.
    pub(super) fn grow(&mut self, len: usize) -> io::Result<()> {
        if len <= self.len() {
            return Ok(());
        }
        self.unmap();
        let lengthened = open_file(&self.path, false).and_then(|file| lengthen(&file, len));
        self.remap()?;
        lengthened
    }

    /// Puts the finished file `new` in this one's place, maps it and makes it at least
    /// `min_len` bytes long. If the process stops on the way, the old file or the new one is
    /// whole at this path; if the rename fails, the old one is mapped again. A missing `new` is
    /// an error, never an empty file.
    pub(super) fn replace_with(&mut self, new: &Path, min_len: usize) -> io::Result<()> {
        self.unmap();
        let renamed = std::fs::rename(new, &self.path);
        self.remap()?;
        renamed?;
        self.grow(min_len)
    }

    /// Lets go of the file's map, so the file can be changed.
    fn unmap(&mut self) {
        self.map = None;
    }

    /// Maps the file at this path, whole.
    fn remap(&mut self) -> io::Result<()> {
        self.map = Some(map(&open_file(&self.path, false)?)?);
        Ok(())
    }
}

/// `path` opened for reading and writing, made if missing and `create` says so.
fn open_file(path: &Path, create: bool) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .truncate(false)
        .open(path)
}

/// Makes `file` `len` bytes long, if it is shorter, by writing zeros at its end: a hole would
/// only be given room when a page of the map is written, and on a full disk the kernel then
/// stops the process instead of returning an error here. Written, not set: a file that fails
/// halfway is only as long as the zeros that made it. Copy-on-write file systems (btrfs, ZFS)
/// may still stop the process.
fn lengthen(file: &File, len: usize) -> io::Result<()> {
    let mut writer = file;
    let mut at = writer.seek(SeekFrom::End(0))?;
    let len = len as u64;
    let zeros = [0; ZEROS_LEN];
    while at < len {
        let chunk = (len - at).min(ZEROS_LEN as u64) as usize;
        writer.write_all(&zeros[..chunk])?;
        at += chunk as u64;
    }
    Ok(())
}

/// `file` mapped into memory, whole.
fn map(file: &File) -> io::Result<MmapMut> {
    // SAFETY: a file is only mapped in a folder whose lock is held (`MappedFile::open` takes
    // it, and a table drops its maps before its lock), which keeps every other store out. This
    // process lengthens or replaces a file only after `unmap`, under `&mut self`, so no slice of
    // the old map is borrowed then and no map is left, and nothing changes the file under one.
    unsafe { MmapMut::map_mut(file) }
}

#[cfg(test)]
mod tests;
