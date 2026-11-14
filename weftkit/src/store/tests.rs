use std::sync::Arc;

use super::*;
use crate::Pos;
use crate::folder::test_folder;

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
fn changes_are_there_after_opening_again() {
    let folder = test_folder("table-again");
    let long = "a long text ".repeat(10_000);
    let table = names(&folder);
    table.set(&"long".into(), long.clone());
    table.set(&"changed".into(), "old".into());
    table.set(&"changed".into(), "new".into());
    table.set(&"removed".into(), "bye".into());
    table.remove(&"removed".into());
    table.remove(&"never there".into());
    drop(table);
    let (table, damages) = opened(Table::named("names"), &folder);
    assert!(damages.is_empty(), "a closed table is whole");
    let mut entries = Vec::new();
    table.for_each(|key: String, value: String| entries.push((key, value)));
    entries.sort();
    let expected = [
        ("changed".to_string(), "new".to_string()),
        ("long".to_string(), long),
    ];
    assert_eq!(entries, expected, "the newest value of each key, whole");
}

#[test]
fn update_and_get_or_default_start_from_the_default() {
    let folder = test_folder("table-default");
    let (table, _) = opened(Table::<Pos, u32>::named("visits"), &folder);
    let pos = Pos::new(1, -2, 3);
    assert_eq!(table.get_or_default(&pos), 0, "a missing key is 0");
    table.update(&pos, |visits| *visits += 2);
    table.update(&pos, |visits| *visits += 3);
    assert_eq!(table.get_or_default(&pos), 5, "updates add up from 0");
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
fn values_of_another_type_are_refused() {
    let folder = test_folder("table-types");
    names(&folder).set(&"key".into(), "text".into());
    let (table, damages) = opened(Table::<String, u64>::named("names"), &folder);
    assert_eq!(damages[0].refused, 1, "a text is no number");
    assert_eq!(table.get(&"key".into()), None, "so the key is missing");
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
