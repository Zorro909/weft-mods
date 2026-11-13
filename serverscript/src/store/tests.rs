use std::fs;
use std::path::PathBuf;
use std::thread;

use super::*;

#[test]
fn adding_counts_from_zero_and_gives_the_total() {
    let store = Store::new();
    assert_eq!(store.add("coins", &Value::Int(5)), Ok(Value::Int(5)));
    assert_eq!(store.add("coins", &Value::Int(-2)), Ok(Value::Int(3)));
    assert_eq!(store.get("coins"), Some(Value::Int(3)), "the total is kept");
}

#[test]
fn a_fraction_makes_the_total_one() {
    let store = Store::new();
    store
        .add("heat", &Value::Int(1))
        .expect("a whole number adds");
    assert_eq!(store.add("heat", &Value::Float(0.5)), Ok(Value::Float(1.5)));
    assert_eq!(store.add("heat", &Value::Int(1)), Ok(Value::Float(2.5)));
}

#[test]
fn whole_numbers_stop_at_the_largest() {
    let store = Store::new();
    store
        .add("big", &Value::Int(i64::MAX))
        .expect("a whole number adds");
    assert_eq!(store.add("big", &Value::Int(1)), Ok(Value::Int(i64::MAX)));
}

#[test]
fn adding_to_text_changes_nothing() {
    let store = Store::new();
    assert_eq!(
        store.set_if_absent("name", Value::Text("ada".into())),
        Ok(true)
    );
    assert_eq!(
        store.add("name", &Value::Int(1)),
        Err(StoreError::NotANumber)
    );
    assert_eq!(
        store.add("fresh", &Value::Bool(true)),
        Err(StoreError::NotANumber)
    );
    assert_eq!(
        store.get("name"),
        Some(Value::Text("ada".into())),
        "the text stays"
    );
    assert_eq!(store.get("fresh"), None, "a refused amount starts nothing");
}

#[test]
fn appending_starts_a_list_and_gives_its_length() {
    let store = Store::new();
    assert_eq!(store.append("log", Value::Int(1)), Ok(1));
    assert_eq!(store.append("log", Value::Text("two".into())), Ok(2));
    let list = Value::List(vec![Value::Int(1), Value::Text("two".into())]);
    assert_eq!(store.get("log"), Some(list), "in the order appended");
}

#[test]
fn appending_to_a_number_changes_nothing() {
    let store = Store::new();
    store
        .add("count", &Value::Int(1))
        .expect("a whole number adds");
    assert_eq!(
        store.append("count", Value::Int(2)),
        Err(StoreError::NotAList)
    );
    assert_eq!(store.get("count"), Some(Value::Int(1)), "the number stays");
}

#[test]
fn only_the_first_sets_an_absent_key() {
    let store = Store::new();
    assert_eq!(
        store.set_if_absent("winner", Value::Int(1)),
        Ok(true),
        "nothing was there"
    );
    assert_eq!(
        store.set_if_absent("winner", Value::Int(2)),
        Ok(false),
        "something is now"
    );
    assert_eq!(store.get("winner"), Some(Value::Int(1)), "the first stays");
}

#[test]
fn of_many_claiming_at_once_exactly_one_wins() {
    let store = Store::new();
    let store = &store;
    let wins = thread::scope(|scope| {
        let claims: Vec<_> = (0..8)
            .map(|claimer| scope.spawn(move || store.set_if_absent("prize", Value::Int(claimer))))
            .collect();
        let won = |claim: thread::ScopedJoinHandle<Result<bool, StoreError>>| {
            claim.join().expect("a claimer ran")
        };
        claims
            .into_iter()
            .map(won)
            .filter(|won| *won == Ok(true))
            .count()
    });
    assert_eq!(wins, 1, "one claim sets the key");
}

#[test]
fn additions_from_many_threads_all_count() {
    let store = Store::new();
    thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..500 {
                    store
                        .add("hits", &Value::Int(1))
                        .expect("a whole number adds");
                }
            });
        }
    });
    assert_eq!(
        store.get("hits"),
        Some(Value::Int(4000)),
        "no addition is lost"
    );
}

#[test]
fn a_key_longer_than_the_file_can_say_is_refused() {
    let store = Store::new();
    let long = "k".repeat(256);
    assert_eq!(store.add(&long, &Value::Int(1)), Err(StoreError::TooLong));
    assert_eq!(store.append(&long, Value::Int(1)), Err(StoreError::TooLong));
    assert_eq!(
        store.set_if_absent(&long, Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(
        store.player_set(7, &long, Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(store.get(&long), None, "nothing was kept");
}

#[test]
fn a_player_has_keys_of_their_own() {
    let store = Store::new();
    store
        .player_set(7, "camp", Value::Int(1))
        .expect("a short key is kept");
    store
        .player_set(7, "camp", Value::Int(2))
        .expect("a short key is kept");
    assert_eq!(
        store.player_get(7, "camp"),
        Some(Value::Int(2)),
        "the last set stays"
    );
    assert_eq!(store.player_get(8, "camp"), None, "another player has none");
    assert_eq!(store.get("camp"), None, "nor do the shared keys");
    assert!(store.player_remove(7, "camp"), "it was there");
    assert!(!store.player_remove(7, "camp"), "it is gone");
    assert_eq!(store.player_get(7, "camp"), None);
}

#[test]
fn every_error_has_a_message() {
    let errors = [
        StoreError::NotANumber,
        StoreError::NotAList,
        StoreError::TooLong,
    ];
    for error in errors {
        assert!(!error.message().is_empty());
    }
}

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
    store
        .add("visits", &Value::Int(2))
        .expect("a whole number adds");
    store
        .add("visits", &Value::Int(3))
        .expect("a whole number adds");
    store
        .append("log", Value::Text("one".into()))
        .expect("a list starts");
    store
        .append("log", Value::Text("two".into()))
        .expect("the list grows");
    store
        .set_if_absent("owner", Value::Bool(true))
        .expect("a short key is kept");
    store
        .player_set(7, "camp", Value::List(vec![Value::Float(0.5)]))
        .expect("a short key is kept");
    store
        .player_set(7, "gone", Value::Int(1))
        .expect("a short key is kept");
    store.player_remove(7, "gone");
}

fn assert_filled(store: &Store) {
    assert_eq!(store.get("visits"), Some(Value::Int(5)));
    let log = vec![Value::Text("one".into()), Value::Text("two".into())];
    assert_eq!(store.get("log"), Some(Value::List(log)));
    assert_eq!(store.get("owner"), Some(Value::Bool(true)));
    let camp = Value::List(vec![Value::Float(0.5)]);
    assert_eq!(store.player_get(7, "camp"), Some(camp));
    assert_eq!(
        store.player_get(7, "gone"),
        None,
        "a removed key stays gone"
    );
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
    for _ in 0..100 {
        store
            .add("visits", &Value::Int(1))
            .expect("a whole number adds");
    }
    let grown = fs::metadata(&file.0).expect("the file is there").len();
    let reopened = file.opened();
    let anew = fs::metadata(&file.0).expect("the file is there").len();
    assert!(anew * 50 < grown, "{anew} bytes hold what {grown} did");
    assert_eq!(reopened.get("visits"), Some(Value::Int(100)));
}

#[test]
fn a_file_cut_off_keeps_what_came_before_and_a_copy() {
    let file = TestFile::named("cut");
    filled(&file.opened());
    let bytes = fs::read(&file.0).expect("the file is there");
    fs::write(&file.0, &bytes[..bytes.len() - 1]).expect("the file can be cut");
    let store = Store::new();
    let problems = store.open(&file.0);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("could not be read to its end"));
    assert_eq!(store.get("visits"), Some(Value::Int(5)), "read before it");
    let copy = fs::read(file.0.with_extension("bak")).expect("a copy is kept");
    assert_eq!(
        copy.len(),
        bytes.len() - 1,
        "the copy is the file as it was"
    );
    assert_eq!(file.opened().get("visits"), Some(Value::Int(5)));
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
    store
        .add("visits", &Value::Int(1))
        .expect("a whole number adds");
    assert_eq!(file.opened().get("visits"), Some(Value::Int(1)));
}
