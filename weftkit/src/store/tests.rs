use std::sync::Arc;

use serde::Deserialize;

use super::*;
use crate::Pos;

/// `table` opened in `folder`, with what it found damaged there.
fn opened<K, V>(table: Table<K, V>, folder: &Path) -> (Table<K, V>, Vec<Damage>)
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    let (shards, damages) = table.open_in(folder);
    assert!(table.shards.set(shards).is_ok(), "a table opens once");
    (table, damages)
}

fn names(folder: &Path) -> Table<String, String> {
    opened(Table::named("names"), folder).0
}

#[test]
fn values_of_any_length_are_there_after_opening_again() {
    let folder = test_folder("table-again");
    let long = "a long text ".repeat(10_000);
    let table = names(&folder);
    table.set(&"long".into(), long.clone());
    table.set(&"short".into(), "hi".into());
    table.set(&"removed".into(), "bye".into());
    table.remove(&"removed".into());
    drop(table);
    let table = names(&folder);
    assert_eq!(
        table.get(&"long".into()),
        Some(long),
        "a long text stays whole"
    );
    assert_eq!(
        table.get(&"short".into()).as_deref(),
        Some("hi"),
        "a short one too"
    );
    assert_eq!(
        table.get(&"removed".into()),
        None,
        "a removed key stays removed"
    );
}

#[test]
fn update_and_get_or_default_start_from_the_default() {
    let folder = test_folder("table-default");
    let (table, _) = opened(Table::<Pos, u32>::named("visits"), &folder);
    let pos = Pos::new(1, -2, 3);
    assert_eq!(
        table.get_or_default(&pos),
        0,
        "a missing key reads as the default"
    );
    table.update(&pos, |visits| *visits += 2);
    table.update(&pos, |visits| *visits += 3);
    assert_eq!(
        table.get_or_default(&pos),
        5,
        "updates add up from the default"
    );
    assert_eq!(
        table.get(&Pos::new(0, 0, 0)),
        None,
        "other keys stay missing"
    );
}

#[test]
fn clear_and_for_each_reach_every_shard() {
    let folder = test_folder("table-clear");
    let (table, _) = opened(Table::<u32, u32>::write_optimized("squares"), &folder);
    for number in 0..100 {
        table.set(&number, number * number);
    }
    let mut sum = 0;
    table.for_each(|number, square| {
        assert_eq!(square, number * number, "each key with its value");
        sum += number;
    });
    assert_eq!(sum, (0..100).sum(), "every key is visited once");
    table.clear();
    let mut left = 0;
    table.for_each(|_, _| left += 1);
    assert_eq!(left, 0, "clearing removes every key");
}

#[test]
fn updates_from_many_threads_lose_nothing() {
    let folder = test_folder("table-threads");
    for table in [Table::named("read"), Table::write_optimized("write")] {
        let table_folder = folder.join(table.name);
        let (table, _) = opened::<u32, u64>(table, &table_folder);
        let table = Arc::new(table);
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let table = Arc::clone(&table);
                std::thread::spawn(move || {
                    for step in 0..500 {
                        table.update(&(step % 4), |count| *count += 1);
                    }
                })
            })
            .collect();
        threads
            .into_iter()
            .for_each(|thread| thread.join().expect("the thread ends"));
        let total: u64 = (0..4).map(|key| table.get_or_default(&key)).sum();
        assert_eq!(total, 8 * 500, "every update counted");
    }
}

#[test]
fn a_damaged_table_keeps_what_it_can_and_a_backup() {
    let folder = test_folder("table-damaged");
    let table = names(&folder);
    table.set(&"first".into(), "1".into());
    table.set(&"second".into(), "2".into());
    drop(table);
    let file = folder.join("table.log");
    let mut bytes = std::fs::read(&file).expect("read");
    bytes[record::HEADER_LEN + 14] ^= 0xff;
    std::fs::write(&file, bytes).expect("written");
    let (table, damages) = opened(Table::<String, String>::named("names"), &folder);
    assert_eq!(damages.len(), 1, "the damage is reported");
    assert!(
        folder.join("table.log.bak").exists(),
        "the file as it was is kept"
    );
    assert_eq!(
        table.get(&"first".into()),
        None,
        "the damaged record is gone"
    );
    assert_eq!(
        table.get(&"second".into()).as_deref(),
        Some("2"),
        "the rest is kept"
    );
}

#[test]
fn values_of_another_type_are_refused() {
    let folder = test_folder("table-types");
    names(&folder).set(&"key".into(), "text".into());
    let (table, damages) = opened(Table::<String, u64>::named("names"), &folder);
    assert_eq!(damages[0].refused, 1, "a text is no number");
    assert_eq!(
        table.get(&"key".into()),
        None,
        "so the key reads as missing"
    );
}

#[test]
#[should_panic(expected = "could not open table names")]
fn a_table_that_cannot_open_panics() {
    let folder = test_folder("table-blocked");
    std::fs::write(folder.join("file"), b"").expect("written");
    names(&folder.join("file"));
}

#[test]
#[should_panic(expected = "could not save table names")]
fn a_change_that_cannot_be_saved_panics() {
    let table = Table::<String, String>::named("names");
    table.saved(Err(io::Error::other("the disk is full")));
}

/// A value whose encoding always fails.
#[derive(Deserialize)]
struct Unsaveable;

impl Serialize for Unsaveable {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("never"))
    }
}

#[test]
#[should_panic(expected = "table odd can't encode a value")]
fn a_value_that_cannot_be_encoded_panics() {
    let table = Table::<u32, Unsaveable>::named("odd");
    table.encode(&Unsaveable);
}
