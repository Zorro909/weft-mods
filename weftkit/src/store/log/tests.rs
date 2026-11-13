use super::*;
use crate::store::test_folder;

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

fn value(log: &Log, key: &[u8]) -> Option<Vec<u8>> {
    log.get(key).map(<[u8]>::to_vec)
}

/// The log at `path` with keys a, b and c, closed again; returns where b's record starts.
fn three_keys(path: &Path) -> usize {
    let (mut log, _) = open(path);
    for key in [b"a", b"b", b"c"] {
        log.set(key, b"value").expect("saved");
    }
    HEADER_LEN + record::encoded_len(b"a", Some(b"value"))
}

#[test]
fn changes_are_there_after_opening_again() {
    let path = test_folder("log-again").join("table.log");
    let (mut log, damage) = open(&path);
    assert!(damage.is_none(), "a new file is no damage");
    log.set(b"kept", b"1").expect("saved");
    log.set(b"changed", b"old").expect("saved");
    log.set(b"changed", b"new").expect("saved");
    log.set(b"removed", b"1").expect("saved");
    log.remove(b"removed").expect("removed");
    log.remove(b"never there")
        .expect("removing nothing is no fault");
    drop(log);
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "a closed file is whole");
    assert_eq!(value(&log, b"kept"), Some(b"1".to_vec()), "a value stays");
    assert_eq!(
        value(&log, b"changed"),
        Some(b"new".to_vec()),
        "the newest value counts"
    );
    assert_eq!(value(&log, b"removed"), None, "a removal stays");
    assert_eq!(log.entries().count(), 2, "two keys are live");
}

#[test]
fn a_cleared_log_stays_empty() {
    let path = test_folder("log-clear").join("table.log");
    three_keys(&path);
    let (mut log, _) = open(&path);
    log.clear().expect("cleared");
    log.set(b"after", b"1").expect("saved");
    drop(log);
    let (log, _) = open(&path);
    let keys: Vec<_> = log.entries().map(|(key, _)| key.to_vec()).collect();
    assert_eq!(
        keys,
        [b"after".to_vec()],
        "only what came after the clearing"
    );
}

#[test]
fn the_file_grows_for_values_bigger_than_it() {
    let path = test_folder("log-grow").join("table.log");
    let big = vec![7; 3 * NEW_LEN];
    let (mut log, _) = open(&path);
    log.set(b"big", &big).expect("saved");
    log.set(b"small", b"1").expect("saved");
    drop(log);
    let (log, _) = open(&path);
    assert_eq!(value(&log, b"big"), Some(big), "the big value is whole");
    assert_eq!(
        value(&log, b"small"),
        Some(b"1".to_vec()),
        "and what came after it"
    );
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
    assert!(log.end < written, "dead records were rewritten away");
    assert!(
        !path.with_extension("new").exists(),
        "the new file took the old one's place"
    );
    drop(log);
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the rewritten file is whole");
    assert_eq!(
        value(&log, b"counter"),
        Some(value_of(39)),
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
    assert_eq!(
        std::fs::read(&damage.backup).expect("a backup"),
        bytes,
        "the backup is as it was"
    );
    assert_eq!(
        damage.backup,
        path.with_file_name("table.log.bak"),
        "named .bak"
    );
    assert!(
        damage.to_string().contains("table.log.bak"),
        "the message names the backup"
    );
    assert_eq!(
        value(&log, b"a"),
        Some(b"value".to_vec()),
        "the key before it stays"
    );
    assert_eq!(
        value(&log, b"c"),
        Some(b"value".to_vec()),
        "the key after it stays"
    );
    assert_eq!(value(&log, b"b"), None, "the damaged key is gone");
    drop(log);
    assert!(open(&path).1.is_none(), "the repaired file is whole");
}

#[test]
fn a_second_backup_takes_the_next_name() {
    let path = test_folder("log-backups").join("table.log");
    three_keys(&path);
    std::fs::write(path.with_file_name("table.log.bak"), b"older").expect("written");
    std::fs::write(path.with_file_name("table.log.1.bak"), b"older").expect("written");
    let mut bytes = std::fs::read(&path).expect("read");
    bytes[0] = b'X';
    std::fs::write(&path, &bytes).expect("written");
    let damage = open(&path).1.expect("the damage is found");
    assert_eq!(
        damage.backup,
        path.with_file_name("table.log.2.bak"),
        "older backups stay"
    );
}

#[test]
fn records_the_table_cannot_read_are_refused() {
    let path = test_folder("log-refused").join("table.log");
    three_keys(&path);
    let (log, damage) = open_keeping(&path, |key, _| key != b"b").expect("it opens");
    let damage = damage.expect("a refused record is damage");
    assert_eq!(
        (damage.damaged, damage.refused),
        (0, 1),
        "one record refused, no bytes lost"
    );
    assert_eq!(value(&log, b"b"), None, "the refused key is left out");
    assert!(damage.backup.exists(), "the file as it was is kept");
}

#[test]
fn a_rewrite_cut_off_leaves_the_old_file() {
    let path = test_folder("log-cut-off").join("table.log");
    three_keys(&path);
    std::fs::write(path.with_extension("new"), b"half a rewrite").expect("written");
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the old file is whole");
    assert_eq!(log.entries().count(), 3, "with all its keys");
    assert!(
        !path.with_extension("new").exists(),
        "the unfinished file is gone"
    );
}

#[test]
fn the_backup_is_the_file_exactly_as_it_was() {
    let path = test_folder("log-backup-exact").join("table.log");
    std::fs::write(&path, b"garbage").expect("written");
    let damage = open(&path).1.expect("garbage is damage");
    let backup = std::fs::read(&damage.backup).expect("a backup");
    assert_eq!(backup, b"garbage", "neither lengthened nor changed");
}

#[test]
fn a_newer_value_the_table_cannot_read_does_not_bring_back_an_older_one() {
    let path = test_folder("log-refused-newest").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"coins", b"old").expect("saved");
    log.set(b"coins", b"NEW").expect("saved");
    drop(log);
    let (log, damage) = open_keeping(&path, |_, value| value != b"NEW").expect("it opens");
    assert_eq!(
        damage.expect("the refusal is damage").refused,
        1,
        "one record refused"
    );
    assert_eq!(
        value(&log, b"coins"),
        None,
        "the key is gone, not the old value back"
    );
}

/// Unix only: there a folder without write permission refuses a new file.
#[cfg(unix)]
#[test]
fn a_clearing_that_cannot_be_saved_keeps_the_keys() {
    use std::os::unix::fs::PermissionsExt;
    let folder = test_folder("log-clear-fails");
    let path = folder.join("table.log");
    three_keys(&path);
    let (mut log, _) = open(&path);
    let mode = |mode| std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(mode));
    mode(0o555).expect("made read-only");
    let cleared = log.clear();
    mode(0o755).expect("made writable");
    assert!(cleared.is_err(), "no new file, no clearing");
    assert_eq!(log.entries().count(), 3, "the keys are all still there");
    log.set(b"d", b"value").expect("and the log still works");
}

#[test]
fn changes_made_while_compacting_are_kept() {
    let path = test_folder("log-compact-meanwhile").join("table.log");
    let (mut log, _) = open(&path);
    let first: [(&[u8], &[u8]); 3] = [(b"kept", b"1"), (b"chan", b"old"), (b"gone", b"1")];
    for (key, value) in first {
        log.set(key, value).expect("saved");
    }
    log.start_compaction();
    log.put(b"chan", b"new").expect("saved");
    log.take_out(b"gone").expect("removed");
    log.put(b"made", b"1").expect("saved");
    log.finish_compaction().expect("finished");
    let check = |log: &Log| {
        assert_eq!(
            value(log, b"kept"),
            Some(b"1".to_vec()),
            "a listed key stays"
        );
        assert_eq!(
            value(log, b"chan"),
            Some(b"new".to_vec()),
            "a change meanwhile counts"
        );
        assert_eq!(value(log, b"gone"), None, "so does a removal");
        assert_eq!(
            value(log, b"made"),
            Some(b"1".to_vec()),
            "and a key made meanwhile"
        );
    };
    check(&log);
    drop(log);
    let (log, damage) = open(&path);
    assert!(damage.is_none(), "the compacted file is whole");
    check(&log);
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
    assert_eq!(
        error.kind(),
        io::ErrorKind::InvalidData,
        "as data this can't read"
    );
    assert_eq!(
        std::fs::read(&path).expect("read"),
        bytes,
        "the file is untouched"
    );
}

#[test]
fn a_compaction_under_way_is_dropped_with_its_log() {
    let path = test_folder("log-compact-dropped").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"1").expect("saved");
    log.start_compaction();
    drop(log);
    assert!(
        !path.with_extension("new").exists(),
        "its file goes with it"
    );
    let (log, _) = open(&path);
    assert_eq!(
        value(&log, b"a"),
        Some(b"1".to_vec()),
        "the log is as it was"
    );
}

#[test]
fn clearing_throws_away_a_compaction_under_way() {
    let path = test_folder("log-compact-cleared").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"1").expect("saved");
    log.start_compaction();
    log.clear().expect("cleared");
    assert_eq!(
        log.entries().count(),
        0,
        "the compaction brings nothing back"
    );
}

/// Unix only: there a folder without write permission refuses a rename.
#[cfg(unix)]
#[test]
fn a_compaction_that_cannot_take_the_files_place_leaves_the_log_as_it_was() {
    use std::os::unix::fs::PermissionsExt;
    let folder = test_folder("log-compact-no-rename");
    let path = folder.join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"1").expect("saved");
    log.start_compaction();
    while !log.compaction.as_ref().is_some_and(Compaction::is_finished) {
        std::thread::yield_now();
    }
    log.put(b"b", b"2").expect("saved meanwhile");
    let mode = |mode| std::fs::set_permissions(&folder, std::fs::Permissions::from_mode(mode));
    mode(0o555).expect("made read-only");
    let finished = log.finish_compaction();
    mode(0o755).expect("made writable");
    assert!(finished.is_err(), "the rename was refused");
    assert_eq!(
        value(&log, b"a"),
        Some(b"1".to_vec()),
        "the old file is read again"
    );
    assert_eq!(
        value(&log, b"b"),
        Some(b"2".to_vec()),
        "with what was saved meanwhile"
    );
    log.set(b"c", b"3").expect("and the log still works");
}

/// Unix only: there a file without write permission can be renamed but not opened to be mapped.
#[cfg(unix)]
#[test]
fn a_clearing_whose_file_cannot_be_mapped_stays_cleared() {
    use std::os::unix::fs::PermissionsExt;
    let path = test_folder("log-clear-unmapped").join("table.log");
    let (mut log, _) = open(&path);
    log.set(b"a", b"old").expect("saved");
    log.start_compaction_of(Vec::new());
    while !log.compaction.as_ref().is_some_and(Compaction::is_finished) {
        std::thread::yield_now();
    }
    let new = path.with_extension("new");
    let mode =
        |path: &Path, mode| std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
    mode(&new, 0o444).expect("made read-only");
    let cleared = log.finish_compaction();
    assert!(
        cleared.is_err(),
        "the cleared file took the old one's place but isn't mapped"
    );
    mode(&path, 0o644).expect("made writable");
    log.set(b"b", b"new").expect("the next write maps it again");
    assert_eq!(value(&log, b"a"), None, "the cleared key stays gone");
    assert_eq!(
        value(&log, b"b"),
        Some(b"new".to_vec()),
        "beside what came after"
    );
}
