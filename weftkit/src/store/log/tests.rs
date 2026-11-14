use super::*;
#[cfg(unix)]
use crate::folder::set_mode;
use crate::folder::test_folder;

fn open(path: &Path) -> (Log, Option<Damage>) {
    open_keeping(path, |_, _| true).expect("the log opens")
}

/// The log at `path`, keeping what passes `readable`, opened under its folder's lock.
fn open_keeping(
    path: &Path,
    readable: impl Fn(&[u8], &[u8]) -> bool,
) -> io::Result<(Log, Option<Damage>)> {
    let lock = FolderLock::take(path.parent().expect("in a folder")).expect("the lock is free");
    Log::open(path, readable, &lock)
}

/// Each key and value of `log` as `key=value`, sorted.
fn contents(log: &Log) -> Vec<String> {
    let text = String::from_utf8_lossy;
    let entries = log
        .entries()
        .map(|(key, value)| format!("{}={}", text(key), text(value)));
    let mut entries: Vec<_> = entries.collect();
    entries.sort();
    entries
}

/// The log at `path` with keys a, b and c, closed again; returns where b's record starts.
fn three_keys(path: &Path) -> usize {
    let (mut log, _) = open(path);
    for key in [b"a", b"b", b"c"] {
        log.set(key, b"value").expect("saved");
    }
    HEADER_LEN + record::encoded_len(b"a", Some(b"value"))
}

/// Waits until the compaction under way has written its file.
#[cfg(unix)]
fn wait_for_compaction(log: &Log) {
    while !log.compaction.as_ref().is_some_and(Compaction::is_finished) {
        std::thread::yield_now();
    }
}

#[test]
fn dead_records_are_rewritten_away() {
    let path = test_folder("log-rewrite").join("table.log");
    let (mut log, _) = open(&path);
    let value_of = |round: u8| vec![round; 64 * 1024];
    for round in 0..40 {
        log.set(b"counter", &value_of(round)).expect("saved");
    }
    log.set(b"other", b"1").expect("saved");
    log.remove(b"other").expect("removed");
    log.finish_compaction()
        .expect("the compaction under way finishes");
    let written = 40 * record::encoded_len(b"counter", Some(&value_of(0)));
    assert!(log.index.end < written, "dead records were rewritten away");
    assert!(
        !path.with_extension("new").exists(),
        "the new file took the old one's place"
    );
    drop(log);
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the rewritten file is whole");
    assert_eq!(
        log.get(b"counter"),
        Some(&value_of(39)[..]),
        "the newest value survived"
    );
}

#[test]
fn a_damaged_file_is_backed_up_and_repaired() {
    let path = test_folder("log-damaged").join("table.log");
    let b = three_keys(&path);
    let mut bytes = std::fs::read(&path).expect("read");
    bytes[b + 14] ^= 0xff;
    std::fs::write(&path, &bytes).expect("written");
    let (log, damage) = open(&path);
    let damage = damage.expect("the damage is found");
    let backup = std::fs::read(&damage.backup).expect("a backup");
    assert_eq!(backup, bytes, "the backup is as it was");
    assert!(
        damage.to_string().contains("table.log.bak"),
        "the message names the backup"
    );
    let b_len = record::encoded_len(b"b", Some(b"value"));
    assert_eq!(damage.damaged, b_len, "the damage is b's record");
    assert_eq!(contents(&log), ["a=value", "c=value"], "the rest stays");
    drop(log);
    assert!(open(&path).1.is_none(), "the repaired file is whole");
}

#[test]
fn the_backup_is_the_file_exactly_as_it_was_under_a_free_name() {
    let path = test_folder("log-backups").join("table.log");
    for older in ["table.log.bak", "table.log.1.bak"] {
        std::fs::write(path.with_file_name(older), b"older").expect("written");
    }
    std::fs::write(&path, b"garbage").expect("written");
    let backup = open(&path).1.expect("garbage is damage").backup;
    assert_eq!(
        backup,
        path.with_file_name("table.log.2.bak"),
        "older backups stay"
    );
    let backup = std::fs::read(backup).expect("a backup");
    assert_eq!(backup, b"garbage", "neither lengthened nor changed");
}

#[test]
fn a_newer_value_the_table_cannot_read_does_not_bring_back_an_older_one() {
    let path = test_folder("log-refused").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"coins", b"old").expect("saved");
    log.set(b"coins", b"NEW").expect("saved");
    drop(log);
    let (log, damage) = open_keeping(&path, |_, value| value != b"NEW").expect("it opens");
    let damage = damage.expect("a refused record is damage");
    let counts = (damage.damaged, damage.refused);
    assert_eq!(counts, (0, 1), "one record refused, no bytes lost");
    assert!(damage.backup.exists(), "the file as it was is kept");
    assert_eq!(
        log.get(b"coins"),
        None,
        "the key is gone, not the old value back"
    );
}

#[test]
fn a_rewrite_left_unfinished_leaves_the_old_file() {
    let path = test_folder("log-unfinished").join("table.log");
    three_keys(&path);
    let (mut log, _) = open(&path);
    log.start_compaction(log.index.live());
    drop(log);
    let new = path.with_extension("new");
    assert!(!new.exists(), "a compaction under way goes with its log");
    std::fs::write(&new, b"half a rewrite").expect("written");
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the old file is whole");
    assert_eq!(log.entries().count(), 3, "with all its keys");
    assert!(!new.exists(), "a rewrite cut off is gone on opening");
}

#[test]
fn changes_made_while_compacting_are_kept() {
    let path = test_folder("log-compact-meanwhile").join("table.log");
    let (mut log, _) = open(&path);
    let first: [(&[u8], &[u8]); 3] = [(b"kept", b"1"), (b"chan", b"old"), (b"gone", b"1")];
    for (key, value) in first {
        log.set(key, value).expect("saved");
    }
    log.start_compaction(log.index.live());
    log.append(b"chan", Some(b"new")).expect("saved");
    log.append(b"gone", None).expect("removed");
    log.append(b"made", Some(b"1")).expect("saved");
    log.finish_compaction().expect("finished");
    let expected = ["chan=new", "kept=1", "made=1"];
    assert_eq!(contents(&log), expected, "changes made meanwhile count");
    drop(log);
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the compacted file is whole");
    assert_eq!(contents(&log), expected, "in the file too");
}

#[test]
fn a_file_of_a_newer_format_is_refused_untouched() {
    let path = test_folder("log-newer").join("table.log");
    let mut bytes = record::header().to_vec();
    bytes[8] = 9;
    bytes.extend(b"what a newer version keeps");
    std::fs::write(&path, &bytes).expect("written");
    let error = open_keeping(&path, |_, _| true)
        .err()
        .expect("a newer format is refused");
    let kind = error.kind();
    assert_eq!(kind, io::ErrorKind::InvalidData, "as data this can't read");
    let file = std::fs::read(&path).expect("read");
    assert_eq!(file, bytes, "the file is untouched");
}

#[test]
fn a_cleared_log_stays_empty() {
    let path = test_folder("log-clear").join("table.log");
    three_keys(&path);
    let (mut log, _) = open(&path);
    log.start_compaction(log.index.live());
    log.clear().expect("cleared");
    log.set(b"after", b"1").expect("saved");
    drop(log);
    let (log, _) = open(&path);
    let after = "only what came after, nothing a compaction under way kept";
    assert_eq!(contents(&log), ["after=1"], "{after}");
}

/// Unix only: there a folder without write permission refuses a new file.
#[cfg(unix)]
#[test]
fn a_clearing_that_cannot_be_saved_keeps_the_keys() {
    let folder = test_folder("log-clear-fails");
    let path = folder.join("table.log");
    three_keys(&path);
    let (mut log, _) = open(&path);
    set_mode(&folder, 0o555);
    let cleared = log.clear();
    set_mode(&folder, 0o755);
    assert!(cleared.is_err(), "no new file, no clearing");
    assert_eq!(log.entries().count(), 3, "the keys are all still there");
    log.set(b"d", b"value").expect("and the log still works");
}

/// Unix only: there a folder without write permission refuses a rename.
#[cfg(unix)]
#[test]
fn a_compaction_that_cannot_take_the_files_place_leaves_the_log_as_it_was() {
    let folder = test_folder("log-compact-no-rename");
    let path = folder.join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"1").expect("saved");
    log.start_compaction(log.index.live());
    wait_for_compaction(&log);
    log.append(b"b", Some(b"2")).expect("saved meanwhile");
    set_mode(&folder, 0o555);
    let finished = log.finish_compaction();
    set_mode(&folder, 0o755);
    assert!(finished.is_err(), "the rename was refused");
    let expected = ["a=1", "b=2"];
    assert_eq!(
        contents(&log),
        expected,
        "the old file, with what came meanwhile"
    );
    log.set(b"c", b"3").expect("and the log still works");
}

/// Unix only: there a file without write permission can be renamed but not opened to be mapped.
#[cfg(unix)]
#[test]
fn a_clearing_whose_file_cannot_be_mapped_stays_cleared() {
    let path = test_folder("log-clear-unmapped").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"old").expect("saved");
    log.start_compaction(Vec::new());
    wait_for_compaction(&log);
    set_mode(&path.with_extension("new"), 0o444);
    let cleared = log.finish_compaction();
    assert!(
        cleared.is_err(),
        "the cleared file took the old one's place but isn't mapped"
    );
    set_mode(&path, 0o644);
    log.set(b"b", b"new").expect("the next write maps it again");
    assert_eq!(contents(&log), ["b=new"], "the cleared key stays gone");
}
