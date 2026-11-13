use super::*;
use crate::store::test_folder;

/// A fresh folder for `name` and its lock.
fn locked(name: &str) -> (PathBuf, FolderLock) {
    let folder = test_folder(name);
    let lock = FolderLock::take(&folder).expect("the lock is free");
    (folder, lock)
}

#[test]
fn a_folder_is_locked_once() {
    let (folder, _lock) = locked("mapped-lock");
    let error = FolderLock::take(&folder)
        .err()
        .expect("a held lock is refused");
    assert_eq!(
        error.kind(),
        io::ErrorKind::WouldBlock,
        "it says the lock is held"
    );
    assert!(
        error.to_string().contains("lock"),
        "naming the lock file: {error}"
    );
}

#[test]
fn a_lock_is_free_again_once_dropped() {
    let (folder, lock) = locked("mapped-lock-again");
    drop(lock);
    assert!(
        FolderLock::take(&folder).is_ok(),
        "the lock can be taken again"
    );
}

#[test]
fn a_missing_folder_has_no_lock() {
    let folder = test_folder("mapped-no-folder").join("missing");
    assert!(
        FolderLock::take(&folder).is_err(),
        "no folder, no lock file"
    );
}

#[test]
fn a_new_file_is_zeros_of_the_length_asked() {
    let (folder, lock) = locked("mapped-new");
    let path = folder.join("a.log");
    let file = MappedFile::open(&path, 64, &lock).expect("it opens");
    assert_eq!(file.bytes(), [0; 64], "the new file is all zeros");
    assert_eq!(file.path(), path, "it knows its path");
}

#[test]
fn what_is_written_is_there_after_opening_again() {
    let (folder, lock) = locked("mapped-again");
    let path = folder.join("a.log");
    let mut file = MappedFile::open(&path, 16, &lock).expect("it opens");
    file.bytes_mut()[..5].copy_from_slice(b"hello");
    drop(file);
    let file = MappedFile::open(&path, 4, &lock).expect("it opens again");
    assert_eq!(&file.bytes()[..5], b"hello", "the bytes stayed");
    assert_eq!(
        file.bytes().len(),
        16,
        "a smaller length asked doesn't shorten it"
    );
}

#[test]
fn growing_keeps_the_bytes_and_writes_zeros() {
    let (folder, lock) = locked("mapped-grow");
    let mut file = MappedFile::open(&folder.join("a.log"), 4, &lock).expect("it opens");
    file.bytes_mut().copy_from_slice(b"abcd");
    let len = 4 + 2 * ZEROS_LEN + 3;
    file.grow(len).expect("it grows");
    assert_eq!(&file.bytes()[..4], b"abcd", "the old bytes stay");
    assert!(
        file.bytes()[4..].iter().all(|byte| *byte == 0),
        "the new are zero"
    );
    assert_eq!(file.bytes().len(), len, "to the length asked");
    file.grow(2).expect("asking for less is no fault");
    assert_eq!(file.bytes().len(), len, "it never shrinks");
}

/// Unix only: there a file without write permission can't be opened to be changed.
#[cfg(unix)]
#[test]
fn a_file_that_cannot_be_mapped_again_is_read_by_no_one_until_it_is() {
    use std::os::unix::fs::PermissionsExt;
    let (folder, lock) = locked("mapped-unmapped");
    let path = folder.join("a.log");
    let mut file = MappedFile::open(&path, 4, &lock).expect("it opens");
    file.bytes_mut().copy_from_slice(b"abcd");
    let mode = |mode| std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode));
    mode(0o444).expect("made read-only");
    let error = file.grow(8).expect_err("a read-only file doesn't grow");
    assert_eq!(
        error.kind(),
        io::ErrorKind::PermissionDenied,
        "and says why"
    );
    assert_eq!(file.len(), 0, "nor can it be mapped again");
    let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| file.bytes().len()));
    assert!(
        read.is_err(),
        "so reading it panics instead of reading something else"
    );
    let write = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| file.bytes_mut()[0] = 1));
    assert!(write.is_err(), "and so does writing it");
    mode(0o644).expect("made writable");
    file.grow(8).expect("the next growth maps it again");
    assert_eq!(file.bytes(), b"abcd\0\0\0\0", "with what it held");
}

/// Unix only: where a file's blocks are told.
#[cfg(unix)]
#[test]
fn grown_room_is_on_the_disk_not_a_hole() {
    use std::os::unix::fs::MetadataExt;
    let (folder, lock) = locked("mapped-allocated");
    let path = folder.join("a.log");
    let mut file = MappedFile::open(&path, 4, &lock).expect("it opens");
    let len = 4 * ZEROS_LEN;
    file.grow(len).expect("it grows");
    let blocks = std::fs::metadata(&path).expect("found").blocks();
    assert!(
        blocks * 512 >= len as u64,
        "every byte has its block: {blocks} blocks"
    );
}

#[test]
fn a_missing_folder_is_an_error() {
    let (folder, lock) = locked("mapped-nowhere");
    let path = folder.join("missing").join("a.log");
    assert!(
        MappedFile::open(&path, 4, &lock).is_err(),
        "no folder, no file"
    );
}

#[test]
fn a_replaced_file_takes_the_place_of_the_old() {
    let (folder, lock) = locked("mapped-replace");
    let mut file = MappedFile::open(&folder.join("a.log"), 4, &lock).expect("it opens");
    std::fs::write(folder.join("a.new"), b"new").expect("written");
    file.replace_with(&folder.join("a.new"), 4)
        .expect("it is replaced");
    assert_eq!(file.bytes(), b"new\0", "the new bytes, lengthened");
    assert_eq!(file.path(), folder.join("a.log"), "under the old name");
    assert!(!folder.join("a.new").exists(), "the new name is gone");
    assert_eq!(
        std::fs::read(folder.join("a.log")).expect("read"),
        b"new\0",
        "on disk too"
    );
}

#[test]
fn a_missing_new_file_is_not_made() {
    let (folder, lock) = locked("mapped-replace-missing");
    let mut file = MappedFile::open(&folder.join("a.log"), 4, &lock).expect("it opens");
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
    use std::os::unix::fs::PermissionsExt;
    let (folder, lock) = locked("mapped-rename-fails");
    let mut file = MappedFile::open(&folder.join("a.log"), 4, &lock).expect("it opens");
    file.bytes_mut().copy_from_slice(b"old!");
    std::fs::write(folder.join("a.new"), b"new").expect("written");
    let mode = |mode| std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(mode));
    mode(0o555).expect("made read-only");
    let replaced = file.replace_with(&folder.join("a.new"), 4);
    mode(0o755).expect("made writable");
    assert!(replaced.is_err(), "a read-only folder takes no rename");
    assert_eq!(file.bytes(), b"old!", "the old file is mapped again");
    assert_eq!(file.path(), folder.join("a.log"), "under its own name");
}
