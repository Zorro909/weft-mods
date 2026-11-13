mod file;

use std::thread;

use super::*;

/// A key of a data key that is there once for everyone.
fn shared(key: &str) -> Place<'_> {
    Place {
        owner: SHARED,
        component: "test:shared",
        key,
    }
}

/// A key of a data key on the player with the number `player`.
fn of(player: u64, key: &str) -> Place<'_> {
    Place {
        owner: player,
        component: "test:own",
        key,
    }
}

#[test]
fn adding_counts_from_zero_and_gives_the_total() {
    let store = Store::new();
    assert_eq!(
        store.add(shared("coins"), &Value::Int(5)),
        Ok(Value::Int(5))
    );
    assert_eq!(
        store.add(shared("coins"), &Value::Int(-2)),
        Ok(Value::Int(3))
    );
    assert_eq!(
        store.get(shared("coins")),
        Some(Value::Int(3)),
        "the total is kept"
    );
}

#[test]
fn a_fraction_makes_the_total_one() {
    let store = Store::new();
    store
        .add(shared("heat"), &Value::Int(1))
        .expect("a whole number adds");
    assert_eq!(
        store.add(shared("heat"), &Value::Float(0.5)),
        Ok(Value::Float(1.5))
    );
    assert_eq!(
        store.add(shared("heat"), &Value::Int(1)),
        Ok(Value::Float(2.5))
    );
}

#[test]
fn whole_numbers_stop_at_the_largest() {
    let store = Store::new();
    store
        .add(shared("big"), &Value::Int(i64::MAX))
        .expect("a whole number adds");
    assert_eq!(
        store.add(shared("big"), &Value::Int(1)),
        Ok(Value::Int(i64::MAX))
    );
}

#[test]
fn adding_to_text_changes_nothing() {
    let store = Store::new();
    assert_eq!(
        store.set_if_absent(shared("name"), Value::Text("ada".into())),
        Ok(true)
    );
    assert_eq!(
        store.add(shared("name"), &Value::Int(1)),
        Err(StoreError::NotANumber)
    );
    assert_eq!(
        store.add(shared("fresh"), &Value::Bool(true)),
        Err(StoreError::NotANumber)
    );
    assert_eq!(
        store.get(shared("name")),
        Some(Value::Text("ada".into())),
        "the text stays"
    );
    assert_eq!(
        store.get(shared("fresh")),
        None,
        "a refused amount starts nothing"
    );
}

#[test]
fn appending_starts_a_list_and_gives_its_length() {
    let store = Store::new();
    assert_eq!(store.append(shared("log"), Value::Int(1)), Ok(1));
    assert_eq!(
        store.append(shared("log"), Value::Text("two".into())),
        Ok(2)
    );
    let list = Value::List(vec![Value::Int(1), Value::Text("two".into())]);
    assert_eq!(
        store.get(shared("log")),
        Some(list),
        "in the order appended"
    );
}

#[test]
fn appending_to_a_number_changes_nothing() {
    let store = Store::new();
    store
        .add(shared("count"), &Value::Int(1))
        .expect("a whole number adds");
    assert_eq!(
        store.append(shared("count"), Value::Int(2)),
        Err(StoreError::NotAList)
    );
    assert_eq!(
        store.get(shared("count")),
        Some(Value::Int(1)),
        "the number stays"
    );
}

#[test]
fn only_the_first_sets_an_absent_key() {
    let store = Store::new();
    assert_eq!(
        store.set_if_absent(shared("winner"), Value::Int(1)),
        Ok(true),
        "nothing was there"
    );
    assert_eq!(
        store.set_if_absent(shared("winner"), Value::Int(2)),
        Ok(false),
        "something is now"
    );
    assert_eq!(
        store.get(shared("winner")),
        Some(Value::Int(1)),
        "the first stays"
    );
}

#[test]
fn of_many_claiming_at_once_exactly_one_wins() {
    let store = Store::new();
    let store = &store;
    let wins = thread::scope(|scope| {
        let claims: Vec<_> = (0..8)
            .map(|claimer| {
                scope.spawn(move || store.set_if_absent(shared("prize"), Value::Int(claimer)))
            })
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
                        .add(shared("hits"), &Value::Int(1))
                        .expect("a whole number adds");
                }
            });
        }
    });
    assert_eq!(
        store.get(shared("hits")),
        Some(Value::Int(4000)),
        "no addition is lost"
    );
}

#[test]
fn a_key_longer_than_the_file_can_say_is_refused() {
    let store = Store::new();
    let long = "k".repeat(256);
    assert_eq!(
        store.add(shared(&long), &Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(
        store.append(shared(&long), Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(
        store.set_if_absent(shared(&long), Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(
        store.set(of(7, &long), Value::Int(1)),
        Err(StoreError::TooLong)
    );
    assert_eq!(store.get(shared(&long)), None, "nothing was kept");
}

#[test]
fn a_player_has_keys_of_their_own() {
    let store = Store::new();
    store
        .set(of(7, "camp"), Value::Int(1))
        .expect("a short key is kept");
    store
        .set(of(7, "camp"), Value::Int(2))
        .expect("a short key is kept");
    assert_eq!(
        store.get(of(7, "camp")),
        Some(Value::Int(2)),
        "the last set stays"
    );
    assert_eq!(store.get(of(8, "camp")), None, "another player has none");
    assert_eq!(store.get(shared("camp")), None, "nor does what all share");
    assert!(store.remove(of(7, "camp")), "it was there");
    assert!(!store.remove(of(7, "camp")), "it is gone");
    assert_eq!(store.get(of(7, "camp")), None);
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

#[test]
fn a_data_key_holds_many_keys_and_gives_them_all() {
    let store = Store::new();
    store
        .set(of(7, "x"), Value::Int(1))
        .expect("a short key is kept");
    store
        .set(of(7, "y"), Value::Int(2))
        .expect("a short key is kept");
    store
        .set(of(8, "x"), Value::Int(9))
        .expect("a short key is kept");
    let both = Entries::from([
        ("x".to_string(), Value::Int(1)),
        ("y".to_string(), Value::Int(2)),
    ]);
    assert_eq!(store.all(7, "test:own"), both, "one player's, all of it");
    assert_eq!(store.all(7, "test:other"), Entries::new(), "by its name");
    assert_eq!(
        store.all(SHARED, "test:own"),
        Entries::new(),
        "and its owner"
    );
}

#[test]
fn two_data_keys_keep_the_same_key_apart() {
    let store = Store::new();
    let other = Place {
        component: "other:shared",
        ..shared("count")
    };
    store
        .add(shared("count"), &Value::Int(1))
        .expect("a whole number adds");
    store
        .add(other, &Value::Int(5))
        .expect("a whole number adds");
    assert_eq!(store.get(shared("count")), Some(Value::Int(1)));
    assert_eq!(store.get(other), Some(Value::Int(5)));
}
