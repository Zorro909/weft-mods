use super::*;
use crate::store::test_folder;

fn open(path: &Path) -> (Log, Option<Damage>) {
    Log::open(path, |_, _| true).expect("the log opens")
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
    assert!(
        log.dead <= DEAD_ALLOWED,
        "the dead records were rewritten away"
    );
    assert!(
        log.file.bytes().len() < 4 * 1024 * 1024,
        "so the file stays small"
    );
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
    let (log, damage) = Log::open(&path, |key, _| key != b"b").expect("it opens");
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
fn a_file_in_use_is_an_error() {
    let path = test_folder("log-busy").join("table.log");
    let _first = open(&path);
    assert!(
        Log::open(&path, |_, _| true).is_err(),
        "a second server is refused"
    );
}

#[test]
fn a_second_opening_leaves_the_first_ones_rewrite_alone() {
    let path = test_folder("log-rewrite-alone").join("table.log");
    let _first = open(&path);
    std::fs::write(path.with_extension("new"), b"a rewrite in progress").expect("written");
    assert!(
        Log::open(&path, |_, _| true).is_err(),
        "the second server is refused"
    );
    assert!(
        path.with_extension("new").exists(),
        "before it touched the rewrite"
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
    let (log, damage) = Log::open(&path, |_, value| value != b"NEW").expect("it opens");
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
