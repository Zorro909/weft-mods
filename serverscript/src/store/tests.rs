mod file;

use super::*;

fn server() -> Holder {
    Holder::Named("server".into())
}

fn camp() -> Value {
    let entries = [
        ("at".to_string(), Value::List(vec![Value::Float(0.5)])),
        ("returns".to_string(), Value::Int(2)),
    ];
    Value::Map(entries.into())
}

#[test]
fn a_component_holds_the_value_last_set() {
    let store = Store::new();
    assert_eq!(store.get(&server(), "test:stats"), None, "nothing yet");
    store.set(server(), "test:stats", Some(Value::Int(1)));
    store.set(server(), "test:stats", Some(camp()));
    assert_eq!(store.get(&server(), "test:stats"), Some(camp()));
}

#[test]
fn setting_nothing_takes_the_component_off() {
    let store = Store::new();
    store.set(server(), "test:stats", Some(Value::Int(1)));
    store.set(server(), "test:stats", None);
    assert_eq!(store.get(&server(), "test:stats"), None);
    store.set(server(), "test:never", None);
    assert_eq!(
        store.get(&server(), "test:never"),
        None,
        "nor one never set"
    );
}

#[test]
fn holders_keep_the_same_component_apart() {
    let store = Store::new();
    store.set(Holder::Player(7), "test:camp", Some(Value::Int(7)));
    store.set(Holder::Player(8), "test:camp", Some(Value::Int(8)));
    store.set(Holder::Named("7".into()), "test:camp", Some(Value::Int(0)));
    assert_eq!(
        store.get(&Holder::Player(7), "test:camp"),
        Some(Value::Int(7))
    );
    assert_eq!(
        store.get(&Holder::Player(8), "test:camp"),
        Some(Value::Int(8))
    );
    let named = store.get(&Holder::Named("7".into()), "test:camp");
    assert_eq!(named, Some(Value::Int(0)), "a name is no player's number");
}

#[test]
fn a_holder_has_many_components() {
    let store = Store::new();
    store.set(server(), "welcome:stats", Some(Value::Int(1)));
    store.set(server(), "camps:stats", Some(Value::Int(2)));
    assert_eq!(store.get(&server(), "welcome:stats"), Some(Value::Int(1)));
    assert_eq!(store.get(&server(), "camps:stats"), Some(Value::Int(2)));
}
