//! The store with its file: what is kept is there after a restart, and a file that can't be
//! read is set aside.

use std::fs;
use std::path::PathBuf;

use super::*;

/// A file of this test's own, gone when the test ends.
struct TestFile(PathBuf);

impl TestFile {
    fn named(name: &str) -> TestFile {
        let folder = format!("serverscript-test-{}-{name}", std::process::id());
        TestFile(std::env::temp_dir().join(folder).join("data.bin"))
    }

    fn opened(&self) -> Store {
        let store = Store::new();
        assert_eq!(store.open(&self.0), Vec::<String>::new(), "it opens clean");
        store
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        if let Some(folder) = self.0.parent() {
            let _ = fs::remove_dir_all(folder);
        }
    }
}

fn filled(store: &Store) {
    store.set(server(), "test:visits", Some(Value::Int(2)));
    store.set(server(), "test:visits", Some(Value::Int(5)));
    store.set(Holder::Player(7), "test:camp", Some(camp()));
    store.set(Holder::Player(7), "test:gone", Some(Value::Int(1)));
    store.set(Holder::Player(7), "test:gone", None);
}

fn assert_filled(store: &Store) {
    assert_eq!(store.get(&server(), "test:visits"), Some(Value::Int(5)));
    assert_eq!(store.get(&Holder::Player(7), "test:camp"), Some(camp()));
    let gone = store.get(&Holder::Player(7), "test:gone");
    assert_eq!(gone, None, "what was taken off stays off");
}

#[test]
fn what_was_kept_is_there_after_a_restart() {
    let file = TestFile::named("restart");
    filled(&file.opened());
    assert_filled(&file.opened());
    assert_filled(&file.opened());
}

#[test]
fn a_store_without_a_file_keeps_things_in_memory() {
    let store = Store::new();
    filled(&store);
    assert_filled(&store);
}

#[test]
fn the_file_is_written_anew_at_each_start() {
    let file = TestFile::named("anew");
    let store = file.opened();
    for visits in 1..=100 {
        store.set(server(), "test:visits", Some(Value::Int(visits)));
    }
    let grown = fs::metadata(&file.0).expect("the file is there").len();
    let reopened = file.opened();
    let anew = fs::metadata(&file.0).expect("the file is there").len();
    assert!(anew * 50 < grown, "{anew} bytes hold what {grown} did");
    assert_eq!(
        reopened.get(&server(), "test:visits"),
        Some(Value::Int(100))
    );
}

#[test]
fn a_file_cut_off_keeps_what_came_before_and_a_copy() {
    let file = TestFile::named("cut");
    let store = file.opened();
    store.set(server(), "test:visits", Some(Value::Int(5)));
    store.set(Holder::Player(7), "test:camp", Some(camp()));
    drop(store);
    let bytes = fs::read(&file.0).expect("the file is there");
    fs::write(&file.0, &bytes[..bytes.len() - 1]).expect("the file can be cut");
    let store = Store::new();
    let problems = store.open(&file.0);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("could not be read to its end"));
    let visits = store.get(&server(), "test:visits");
    assert_eq!(visits, Some(Value::Int(5)), "read before it");
    assert_eq!(store.get(&Holder::Player(7), "test:camp"), None, "cut off");
    let copy = fs::read(file.0.with_extension("bak")).expect("a copy is kept");
    assert_eq!(
        copy.len(),
        bytes.len() - 1,
        "the copy is the file as it was"
    );
    let reopened = file.opened();
    assert_eq!(reopened.get(&server(), "test:visits"), Some(Value::Int(5)));
}

#[test]
fn another_file_in_its_place_is_set_aside() {
    let file = TestFile::named("other");
    fs::create_dir_all(file.0.parent().expect("the file is in a folder"))
        .expect("the folder can be made");
    fs::write(&file.0, b"something else").expect("the file can be written");
    let store = Store::new();
    assert_eq!(store.open(&file.0).len(), 1, "it says what it found");
    assert_eq!(
        fs::read(file.0.with_extension("bak")).expect("a copy is kept"),
        b"something else"
    );
    store.set(server(), "test:visits", Some(Value::Int(1)));
    let reopened = file.opened();
    assert_eq!(reopened.get(&server(), "test:visits"), Some(Value::Int(1)));
}

#[test]
fn a_file_that_cannot_be_copied_stays_as_it_is() {
    let file = TestFile::named("uncopied");
    fs::create_dir_all(file.0.with_extension("bak")).expect("a folder takes the copy's place");
    fs::write(&file.0, b"something else").expect("the file can be written");
    let store = Store::new();
    let problems = store.open(&file.0);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("no copy could be kept"));
    store.set(server(), "test:visits", Some(Value::Int(1)));
    let untouched = fs::read(&file.0).expect("the file is there");
    assert_eq!(untouched, b"something else", "nothing was written into it");
    let visits = store.get(&server(), "test:visits");
    assert_eq!(visits, Some(Value::Int(1)), "the store goes on in memory");
}
