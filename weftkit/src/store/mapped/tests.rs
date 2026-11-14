use super::*;
const ZEROS_LEN: usize = 64 * 1024;
use crate::folder::test_folder;

/// A fresh folder for `name` and its lock.
fn locked(name: &str) -> (PathBuf, FolderLock) {
    let folder = test_folder(name);
    let lock = FolderLock::take(&folder).expect("the lock is free");
    (folder, lock)
}

#[test]
fn growing_keeps_the_bytes_and_writes_zeros_on_the_disk() {
    let (folder, lock) = locked("mapped-grow");
    let path = folder.join("a.log");
    let mut file = MappedFile::open(&path, 4, &lock).expect("it opens");
    file.bytes_mut().copy_from_slice(b"abcd");
    let len = 4 + 2 * ZEROS_LEN + 3;
    file.grow(len).expect("it grows");
    // Unix only: where a file's blocks are told. Counted before the bytes are read, which would
    // fill holes.
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let blocks = std::fs::metadata(&path).expect("found").blocks();
        assert!(blocks * 512 >= len as u64, "no hole: {blocks} blocks");
    }
    assert_eq!(&file.bytes()[..4], b"abcd", "the old bytes stay");
    let zeros = file.bytes()[4..].iter().all(|byte| *byte == 0);
    assert!(zeros, "the new are zero");
    assert_eq!(file.bytes().len(), len, "to the length asked");
    file.grow(2).expect("asking for less is no fault");
    assert_eq!(file.bytes().len(), len, "it never shrinks");
}

/// Unix only: there a file without write permission can't be opened to be changed.
#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_mapped_again_is_read_by_no_one_until_it_is() {
    use crate::folder::set_mode;
    let (folder, lock) = locked("mapped-unmapped");
    let path = folder.join("a.log");
    let mut file = MappedFile::open(&path, 4, &lock).expect("it opens");
    file.bytes_mut().copy_from_slice(b"abcd");
    set_mode(&path, 0o444);
    let grown = file.grow(8);
    set_mode(&path, 0o644);
    assert!(grown.is_err(), "a read-only file doesn't grow");
    let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| file.bytes().len()));
    assert!(read.is_err(), "nor is it mapped, so reading it panics");
    let write = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| file.bytes_mut()[0] = 1));
    assert!(write.is_err(), "and so does writing it");
    file.grow(8).expect("the next growth maps it again");
    assert_eq!(file.bytes(), b"abcd\0\0\0\0", "with what it held");
}

#[test]
fn a_missing_new_file_is_not_made() {
    let (folder, lock) = locked("mapped-replace-missing");
    let mut file = MappedFile::open(&folder.join("a.log"), 4, &lock).expect("it opens");
    let missing = folder.join("a.new");
    let replaced = file.replace_with(&missing, 4);
    assert!(replaced.is_err(), "no new file, no replacement");
    assert!(!missing.exists(), "an empty one is not made in its place");
    file.bytes_mut()[0] = 7;
    assert_eq!(file.bytes()[0], 7, "the old file is still mapped");
}
