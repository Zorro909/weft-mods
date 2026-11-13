//! A file mapped into memory, the store's only `unsafe`. What is written into the map is the
//! file's: the kernel writes it out even when the process crashes.
//!
//! Mapping a file is unsafe because another process could change or shorten it under the map.
//! Each `MappedFile` holds its file's exclusive lock, so a second server on the same folder
//! (or a second mod mapping the same file) is refused, and only `grow` changes its length. The
//! lock is advisory on Unix: it keeps out every store, not a program that ignores locks.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use memmap2::MmapMut;

/// A file of this process alone, mapped into memory for reading and writing.
pub(super) struct MappedFile {
    path: PathBuf,
    map: MmapMut,
    file: File,
}

impl MappedFile {
    /// Opens `path`, making it if missing, locked for this process alone and at least `min_len`
    /// bytes long; added bytes are zero.
    pub(super) fn open(path: &Path, min_len: usize) -> io::Result<MappedFile> {
        MappedFile::open_with(path, min_len, true)
    }

    fn open_with(path: &Path, min_len: usize, create: bool) -> io::Result<MappedFile> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .truncate(false)
            .open(path)?;
        lock(&file, path)?;
        #[cfg(unix)]
        still_at(&file, path)?;
        let len = file.metadata()?.len().max(min_len.max(1) as u64);
        file.set_len(len)?;
        let map = map(&file)?;
        Ok(MappedFile {
            path: path.to_path_buf(),
            map,
            file,
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.map
    }

    pub(super) fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.map
    }

    /// Makes the file `len` bytes long, if it is shorter; the bytes added are zero.
    pub(super) fn grow(&mut self, len: usize) -> io::Result<()> {
        if len <= self.map.len() {
            return Ok(());
        }
        self.file.set_len(len as u64)?;
        self.map = map(&self.file)?;
        Ok(())
    }

    /// Puts the finished file `new` in this one's place, mapped. Either file stays locked all the
    /// while; if the process stops on the way, the old file or the new one is whole at this path,
    /// and if the rename fails, this one is left as it was.
    pub(super) fn replace_with(&mut self, new: &Path, min_len: usize) -> io::Result<()> {
        let mut fresh = MappedFile::open_with(new, min_len, false)?;
        std::fs::rename(new, &self.path)?;
        fresh.path = self.path.clone();
        *self = fresh;
        Ok(())
    }
}

/// Takes `file`'s exclusive lock, or says why not with the file's name.
fn lock(file: &File, path: &Path) -> io::Result<()> {
    let Err(error) = file.try_lock() else {
        return Ok(());
    };
    let error = io::Error::from(error);
    let reason = format!(
        "{} is locked, in use by another server or mod? ({error})",
        path.display()
    );
    Err(io::Error::new(error.kind(), reason))
}

/// Refuses `file` if another file took its place at `path` between opening and locking it: a
/// rewrite of another server, which let go of the old file, now nameless, only after the rename.
#[cfg(unix)]
fn still_at(file: &File, path: &Path) -> io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let (opened, current) = (file.metadata()?, std::fs::metadata(path)?);
    if (opened.dev(), opened.ino()) == (current.dev(), current.ino()) {
        return Ok(());
    }
    let reason = format!(
        "{} was replaced while opening, in use by another server or mod?",
        path.display()
    );
    Err(io::Error::new(io::ErrorKind::WouldBlock, reason))
}

/// `file` mapped into memory, whole.
fn map(file: &File) -> io::Result<MmapMut> {
    // SAFETY: the file is locked for this process alone (`lock`), which every store respects,
    // and its length only changes in `grow`, which maps it again at once while no slice of the
    // old map is borrowed (`&mut self`), so nothing changes it under the map.
    unsafe { MmapMut::map_mut(file) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::test_folder;

    #[test]
    fn a_new_file_is_zeros_of_the_length_asked() {
        let path = test_folder("mapped-new").join("a.log");
        let file = MappedFile::open(&path, 64).expect("it opens");
        assert_eq!(file.bytes(), [0; 64], "the new file is all zeros");
        assert_eq!(file.path(), path, "it knows its path");
    }

    #[test]
    fn what_is_written_is_there_after_opening_again() {
        let path = test_folder("mapped-again").join("a.log");
        let mut file = MappedFile::open(&path, 16).expect("it opens");
        file.bytes_mut()[..5].copy_from_slice(b"hello");
        drop(file);
        let file = MappedFile::open(&path, 4).expect("it opens again");
        assert_eq!(&file.bytes()[..5], b"hello", "the bytes stayed");
        assert_eq!(
            file.bytes().len(),
            16,
            "a smaller length asked doesn't shorten it"
        );
    }

    #[test]
    fn growing_keeps_the_bytes_and_adds_zeros() {
        let path = test_folder("mapped-grow").join("a.log");
        let mut file = MappedFile::open(&path, 4).expect("it opens");
        file.bytes_mut().copy_from_slice(b"abcd");
        file.grow(8).expect("it grows");
        assert_eq!(
            file.bytes(),
            b"abcd\0\0\0\0",
            "the old bytes stay, the new are zero"
        );
        file.grow(2).expect("asking for less is no fault");
        assert_eq!(file.bytes().len(), 8, "it never shrinks");
    }

    #[test]
    fn a_second_opening_is_refused() {
        let path = test_folder("mapped-twice").join("a.log");
        let _first = MappedFile::open(&path, 4).expect("it opens");
        let error = MappedFile::open(&path, 4)
            .err()
            .expect("a file in use opens only once");
        assert_eq!(
            error.kind(),
            io::ErrorKind::WouldBlock,
            "it says the file is locked"
        );
        assert!(
            error.to_string().contains("a.log"),
            "it names the file: {error}"
        );
    }

    #[test]
    fn a_missing_folder_is_an_error() {
        let path = test_folder("mapped-nowhere").join("missing").join("a.log");
        assert!(MappedFile::open(&path, 4).is_err(), "no folder, no file");
    }

    #[test]
    fn a_replaced_file_takes_the_place_and_lock_of_the_old() {
        let folder = test_folder("mapped-replace");
        let mut file = MappedFile::open(&folder.join("a.log"), 4).expect("it opens");
        std::fs::write(folder.join("a.new"), b"new").expect("written");
        file.replace_with(&folder.join("a.new"), 4)
            .expect("it is replaced");
        assert_eq!(file.bytes(), b"new\0", "the new bytes, lengthened");
        assert_eq!(file.path(), folder.join("a.log"), "under the old name");
        assert!(!folder.join("a.new").exists(), "the new name is gone");
        assert!(
            MappedFile::open(&folder.join("a.log"), 4).is_err(),
            "and it is locked"
        );
    }

    #[test]
    fn a_missing_new_file_is_not_made() {
        let folder = test_folder("mapped-replace-missing");
        let mut file = MappedFile::open(&folder.join("a.log"), 4).expect("it opens");
        let missing = folder.join("a.new");
        assert!(
            file.replace_with(&missing, 4).is_err(),
            "no new file, no replacement"
        );
        assert!(!missing.exists(), "an empty one is not made in its place");
        file.bytes_mut()[0] = 7;
        assert_eq!(file.bytes()[0], 7, "the old file is still mapped");
    }

    /// Unix only: there a folder without write permission refuses a rename.
    #[cfg(unix)]
    #[test]
    fn a_failed_rename_keeps_the_old_file() {
        let folder = test_folder("mapped-rename-fails");
        let mut file = MappedFile::open(&folder.join("a.log"), 4).expect("it opens");
        file.bytes_mut().copy_from_slice(b"old!");
        std::fs::write(folder.join("a.new"), b"new").expect("written");
        set_mode(&folder, 0o555);
        let replaced = file.replace_with(&folder.join("a.new"), 4);
        set_mode(&folder, 0o755);
        assert!(replaced.is_err(), "a read-only folder takes no rename");
        assert_eq!(file.bytes(), b"old!", "the old file is still mapped");
        assert_eq!(file.path(), folder.join("a.log"), "under its own name");
    }

    #[cfg(unix)]
    fn set_mode(folder: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(folder, permissions).expect("permissions set");
    }

    /// Unix only: where another server's rewrite can replace the file between opening and locking.
    #[cfg(unix)]
    #[test]
    fn a_file_replaced_before_it_was_locked_is_refused() {
        let folder = test_folder("mapped-replaced-early");
        let path = folder.join("a.log");
        std::fs::write(&path, b"old").expect("written");
        let opened = File::open(&path).expect("opened");
        assert!(still_at(&opened, &path).is_ok(), "the file is still there");
        std::fs::write(folder.join("a.new"), b"new").expect("written");
        std::fs::rename(folder.join("a.new"), &path).expect("replaced");
        let error = still_at(&opened, &path).expect_err("the old file has no name any more");
        assert_eq!(
            error.kind(),
            io::ErrorKind::WouldBlock,
            "refused as a file in use"
        );
        assert!(error.to_string().contains("a.log"), "naming it: {error}");
    }
}
