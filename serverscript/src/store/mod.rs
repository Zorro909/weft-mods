//! What scripts keep: values by key, every script's or one player's own, saved in a file
//! beside the scripts.
//!
//! Regions tick in parallel and each may run scripts, so every operation is one step under a
//! short lock. On a shared key none replaces what another wrote: a value is added to, appended
//! to, or set where there was none. A player's keys are set and removed freely: only the region
//! the player is in reaches them.
//!
//! Every change goes to the file as it is made (`journal`), so the store has no size of its own
//! to outgrow and nothing to write when the game stops.

mod journal;
mod pack;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::Mutex;

use weft_sdk::error;

use self::journal::{Change, Record};
use crate::lock;
use crate::value::Value;

/// How many locks the keys are spread over, so regions writing different keys rarely wait.
const SHARDS: usize = 16;

/// The owner of the keys every script shares. A player's are owned by the player's id.
const SHARED: u64 = 0;

/// The store every script reads and writes.
pub static STORE: Store = Store::new();

type Entries = BTreeMap<String, Value>;
type Shard = Mutex<BTreeMap<u64, Entries>>;

pub struct Store {
    shards: [Shard; SHARDS],
    /// Where changes are written; `None` before `open`, and once writing failed.
    file: Mutex<Option<File>>,
}

/// Why an operation left the store as it was.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoreError {
    /// `add` with something other than two numbers.
    NotANumber,
    /// `append` to a value that is no list.
    NotAList,
    /// A key or a value longer than the file can say.
    TooLong,
}

impl StoreError {
    pub fn message(self) -> &'static str {
        match self {
            StoreError::NotANumber => "store_add needs a number, and a number under the key",
            StoreError::NotAList => "store_append needs a list under the key",
            StoreError::TooLong => "a key is at most 255 bytes long",
        }
    }
}

impl Store {
    pub const fn new() -> Store {
        Store {
            shards: [const { Mutex::new(BTreeMap::new()) }; SHARDS],
            file: Mutex::new(None),
        }
    }

    /// Fills the store from the file at `path` and writes every change to it from now on.
    /// Gives what went wrong, for the log. A file that could not be read to its end is kept
    /// beside it as `.bak`, and the store goes on with what was read.
    pub fn open(&self, path: &Path) -> Vec<String> {
        let name = path.display();
        let read = match journal::read(path) {
            Ok(read) => read,
            Err(error) => return vec![format!("{name}: {error}; scripts' data is not saved")],
        };
        let mut problems = Vec::new();
        if !read.whole {
            let copy = path.with_extension("bak");
            let kept = fs::copy(path, &copy).map_or_else(
                |error| format!("no copy could be kept: {error}"),
                |_| format!("a copy is kept as {}", copy.display()),
            );
            problems.push(format!("{name} could not be read to its end; {kept}"));
        }
        read.records
            .into_iter()
            .for_each(|record| self.replay(record));
        match journal::start(path, &self.changes()) {
            Ok(file) => *lock(&self.file) = Some(file),
            Err(error) => problems.push(format!("{name}: {error}; scripts' data is not saved")),
        }
        problems
    }

    /// Does what `record` says, without writing it again.
    fn replay(&self, record: Record) {
        let mut shard = lock(self.shard(record.owner, &record.key));
        let entries = shard.entry(record.owner).or_default();
        match record.change {
            Change::Set(value) => {
                entries.insert(record.key, value);
            }
            Change::Append(item) => {
                let list = entries
                    .entry(record.key)
                    .or_insert_with(|| Value::List(Vec::new()));
                if let Value::List(list) = list {
                    list.push(item);
                }
            }
            Change::Remove => {
                entries.remove(&record.key);
            }
        }
    }

    /// Everything the store holds, as the changes that make it.
    fn changes(&self) -> Vec<u8> {
        let mut changes = Vec::new();
        for shard in &self.shards {
            for (owner, entries) in lock(shard).iter() {
                for (key, value) in entries {
                    changes.extend(journal::set(*owner, key, value).unwrap_or_default());
                }
            }
        }
        changes
    }

    /// Puts `change` into the file. When that fails the store goes on in memory, and says so
    /// once.
    fn write(&self, change: &[u8]) {
        let mut file = lock(&self.file);
        let Some(open) = file.as_mut() else {
            return;
        };
        if let Err(error) = journal::write(open, change) {
            error!("scripts' data is no longer saved: {error}");
            *file = None;
        }
    }

    fn shard(&self, owner: u64, key: &str) -> &Shard {
        let mut hasher = DefaultHasher::new();
        (owner, key).hash(&mut hasher);
        &self.shards[hasher.finish() as usize % SHARDS]
    }

    fn value(&self, owner: u64, key: &str) -> Option<Value> {
        lock(self.shard(owner, key)).get(&owner)?.get(key).cloned()
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.value(SHARED, key)
    }

    /// Adds `amount` to the number under `key`, counting from zero, and gives the new total. Two
    /// whole numbers stay whole and stop at the largest; with a fraction the total is one.
    pub fn add(&self, key: &str, amount: &Value) -> Result<Value, StoreError> {
        let mut shard = lock(self.shard(SHARED, key));
        let entries = shard.entry(SHARED).or_default();
        let total = match (entries.get(key), amount) {
            (None, Value::Int(_) | Value::Float(_)) => amount.clone(),
            (Some(Value::Int(old)), Value::Int(more)) => Value::Int(old.saturating_add(*more)),
            (Some(Value::Int(old)), Value::Float(more)) => Value::Float(*old as f64 + more),
            (Some(Value::Float(old)), Value::Int(more)) => Value::Float(old + *more as f64),
            (Some(Value::Float(old)), Value::Float(more)) => Value::Float(old + more),
            _ => return Err(StoreError::NotANumber),
        };
        let change = journal::set(SHARED, key, &total).ok_or(StoreError::TooLong)?;
        entries.insert(key.to_string(), total.clone());
        self.write(&change);
        Ok(total)
    }

    /// Puts `value` at the end of the list under `key`, starting one, and gives its new length.
    pub fn append(&self, key: &str, value: Value) -> Result<usize, StoreError> {
        let change = journal::append(SHARED, key, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(SHARED, key));
        let entries = shard.entry(SHARED).or_default();
        let entry = entries
            .entry(key.to_string())
            .or_insert_with(|| Value::List(Vec::new()));
        let Value::List(list) = entry else {
            return Err(StoreError::NotAList);
        };
        list.push(value);
        self.write(&change);
        Ok(list.len())
    }

    /// Sets `key` to `value` if nothing is there; whether it did. Of several asking at once,
    /// exactly one hears yes.
    pub fn set_if_absent(&self, key: &str, value: Value) -> Result<bool, StoreError> {
        let change = journal::set(SHARED, key, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(SHARED, key));
        let entries = shard.entry(SHARED).or_default();
        if entries.contains_key(key) {
            return Ok(false);
        }
        entries.insert(key.to_string(), value);
        self.write(&change);
        Ok(true)
    }

    /// What the player with the id `player` has under `key`.
    pub fn player_get(&self, player: u64, key: &str) -> Option<Value> {
        self.value(player, key)
    }

    /// Sets `key` of the player with the id `player`, replacing what was there.
    pub fn player_set(&self, player: u64, key: &str, value: Value) -> Result<(), StoreError> {
        let change = journal::set(player, key, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(player, key));
        let entries = shard.entry(player).or_default();
        entries.insert(key.to_string(), value);
        self.write(&change);
        Ok(())
    }

    /// Takes `key` off the player with the id `player`; whether it was there.
    pub fn player_remove(&self, player: u64, key: &str) -> bool {
        let mut shard = lock(self.shard(player, key));
        let removed = shard
            .get_mut(&player)
            .and_then(|entries| entries.remove(key));
        if let (Some(_), Some(change)) = (&removed, journal::remove(player, key)) {
            self.write(&change);
        }
        removed.is_some()
    }
}
