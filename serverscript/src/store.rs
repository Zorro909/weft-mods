//! What scripts share: values by key, kept in memory while a world is open.
//!
//! Regions tick in parallel and each may run scripts, so every operation is one step under a
//! short lock, and none replaces what another wrote: a value is added to, appended to, or set
//! where there was none. A script never sees half of another's change.

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::lock;
use crate::value::Value;

/// How many locks the keys are spread over, so regions writing different keys rarely wait.
const SHARDS: usize = 16;

/// The store every script reads and writes.
pub static STORE: Store = Store::new();

type Shard = Mutex<BTreeMap<String, Value>>;

pub struct Store {
    shards: [Shard; SHARDS],
    /// The world session the values belong to (`World::session`).
    session: AtomicU64,
    /// Held while the store is emptied for a new session.
    switching: Mutex<()>,
}

/// Why an operation left the store as it was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoreError {
    /// `add` with something other than two numbers.
    NotANumber,
    /// `append` to a value that is no list.
    NotAList,
}

impl StoreError {
    pub fn message(self) -> &'static str {
        match self {
            StoreError::NotANumber => "store_add needs a number, and a number under the key",
            StoreError::NotAList => "store_append needs a list under the key",
        }
    }
}

impl Store {
    pub const fn new() -> Store {
        Store {
            shards: [const { Mutex::new(BTreeMap::new()) }; SHARDS],
            session: AtomicU64::new(0),
            switching: Mutex::new(()),
        }
    }

    /// Empties the store when `session` is another world than the one it was filled in:
    /// entity ids start over in every world, and scripts keep things by them.
    pub fn enter(&self, session: u64) {
        if self.session.load(Ordering::Acquire) == session {
            return;
        }
        let _switching = lock(&self.switching);
        if self.session.load(Ordering::Acquire) != session {
            self.shards.iter().for_each(|shard| lock(shard).clear());
            self.session.store(session, Ordering::Release);
        }
    }

    fn shard(&self, key: &str) -> &Shard {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        &self.shards[hasher.finish() as usize % SHARDS]
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        lock(self.shard(key)).get(key).cloned()
    }

    /// Adds `amount` to the number under `key`, counting from zero, and gives the new total. Two
    /// whole numbers stay whole and stop at the largest; with a fraction the total is one.
    pub fn add(&self, key: &str, amount: &Value) -> Result<Value, StoreError> {
        let mut shard = lock(self.shard(key));
        let total = match (shard.get(key), amount) {
            (None, Value::Int(_) | Value::Float(_)) => amount.clone(),
            (Some(Value::Int(old)), Value::Int(more)) => Value::Int(old.saturating_add(*more)),
            (Some(Value::Int(old)), Value::Float(more)) => Value::Float(*old as f64 + more),
            (Some(Value::Float(old)), Value::Int(more)) => Value::Float(old + *more as f64),
            (Some(Value::Float(old)), Value::Float(more)) => Value::Float(old + more),
            _ => return Err(StoreError::NotANumber),
        };
        shard.insert(key.to_string(), total.clone());
        Ok(total)
    }

    /// Puts `value` at the end of the list under `key`, starting one, and gives its new length.
    pub fn append(&self, key: &str, value: Value) -> Result<usize, StoreError> {
        let mut shard = lock(self.shard(key));
        let entry = shard
            .entry(key.to_string())
            .or_insert_with(|| Value::List(Vec::new()));
        let Value::List(list) = entry else {
            return Err(StoreError::NotAList);
        };
        list.push(value);
        Ok(list.len())
    }

    /// Sets `key` to `value` if nothing is there; whether it did. Of several asking at once,
    /// exactly one hears yes.
    pub fn set_if_absent(&self, key: &str, value: Value) -> bool {
        let mut shard = lock(self.shard(key));
        if shard.contains_key(key) {
            return false;
        }
        shard.insert(key.to_string(), value);
        true
    }
}

#[cfg(test)]
mod tests {
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
        assert!(store.set_if_absent("name", Value::Text("ada".into())));
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
        assert!(
            store.set_if_absent("winner", Value::Int(1)),
            "nothing was there"
        );
        assert!(
            !store.set_if_absent("winner", Value::Int(2)),
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
                .map(|claimer| {
                    scope.spawn(move || store.set_if_absent("prize", Value::Int(claimer)))
                })
                .collect();
            let won = |claim: thread::ScopedJoinHandle<bool>| claim.join().expect("a claimer ran");
            claims.into_iter().map(won).filter(|won| *won).count()
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
    fn another_world_starts_empty() {
        let store = Store::new();
        store.enter(7);
        store
            .add("coins", &Value::Int(5))
            .expect("a whole number adds");
        store.enter(7);
        assert_eq!(
            store.get("coins"),
            Some(Value::Int(5)),
            "the same world keeps its values"
        );
        store.enter(8);
        assert_eq!(store.get("coins"), None, "a new world has none");
    }

    #[test]
    fn every_error_says_what_it_needs() {
        for error in [StoreError::NotANumber, StoreError::NotAList] {
            assert!(error.message().starts_with("store_"), "it names the call");
        }
    }
}
