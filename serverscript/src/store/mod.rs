//! What scripts keep, saved in a file beside the scripts. As the game keeps components, a
//! script keeps data keys: each is named `<script>:<name>` (`crate::name`) and holds values by
//! key. A data key is there once for everyone, or once on each player (`Place`).
//!
//! Regions tick in parallel and each may run scripts, so every operation is one step under a
//! short lock. This file offers them all; which of them a script may use where is
//! `crate::bindings::data`'s to say.
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

/// The owner of what is there once for everyone. What is a player's is owned by the
/// player's number (`crate::player`).
pub const SHARED: u64 = 0;

/// The store every script reads and writes.
pub static STORE: Store = Store::new();

/// The values of one data key, by key.
pub type Entries = BTreeMap<String, Value>;
/// Each owner's data keys, by their full names.
type Owners = BTreeMap<u64, BTreeMap<String, Entries>>;
type Shard = Mutex<Owners>;

/// Where a value lies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place<'a> {
    /// `SHARED`, or a player's number.
    pub owner: u64,
    /// The data key's full name, `<script>:<name>`.
    pub component: &'a str,
    pub key: &'a str,
}

impl Place<'_> {
    /// The values of this place's data key, made if there were none.
    fn entries(self, owners: &mut Owners) -> &mut Entries {
        let components = owners.entry(self.owner).or_default();
        components.entry(self.component.to_string()).or_default()
    }
}

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
    /// A data key, a key or a value longer than the file can say.
    TooLong,
}

impl StoreError {
    pub fn message(self) -> &'static str {
        match self {
            StoreError::NotANumber => "data_add needs a number, and a number under the key",
            StoreError::NotAList => "data_append needs a list under the key",
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
        let place = Place {
            owner: record.owner,
            component: &record.component,
            key: &record.key,
        };
        let mut shard = lock(self.shard(place));
        let entries = place.entries(&mut shard);
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
            for (owner, components) in lock(shard).iter() {
                for (component, entries) in components {
                    changes.extend(held(*owner, component, entries));
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

    /// The lock of a place's data key: all its keys are under one, so `all` sees them at one
    /// moment.
    fn shard(&self, place: Place) -> &Shard {
        let mut hasher = DefaultHasher::new();
        (place.owner, place.component).hash(&mut hasher);
        &self.shards[hasher.finish() as usize % SHARDS]
    }

    pub fn get(&self, place: Place) -> Option<Value> {
        let shard = lock(self.shard(place));
        let entries = shard.get(&place.owner)?.get(place.component)?;
        entries.get(place.key).cloned()
    }

    /// Every value of the data key `component` of `owner`, by key.
    pub fn all(&self, owner: u64, component: &str) -> Entries {
        let place = Place {
            owner,
            component,
            key: "",
        };
        let shard = lock(self.shard(place));
        let entries = shard.get(&owner).and_then(|held| held.get(component));
        entries.cloned().unwrap_or_default()
    }

    /// Adds `amount` to the number at `place`, counting from zero, and gives the new total. Two
    /// whole numbers stay whole and stop at the largest; with a fraction the total is one.
    pub fn add(&self, place: Place, amount: &Value) -> Result<Value, StoreError> {
        let mut shard = lock(self.shard(place));
        let entries = place.entries(&mut shard);
        let total = match (entries.get(place.key), amount) {
            (None, Value::Int(_) | Value::Float(_)) => amount.clone(),
            (Some(Value::Int(old)), Value::Int(more)) => Value::Int(old.saturating_add(*more)),
            (Some(Value::Int(old)), Value::Float(more)) => Value::Float(*old as f64 + more),
            (Some(Value::Float(old)), Value::Int(more)) => Value::Float(old + *more as f64),
            (Some(Value::Float(old)), Value::Float(more)) => Value::Float(old + more),
            _ => return Err(StoreError::NotANumber),
        };
        let change = journal::set(place, &total).ok_or(StoreError::TooLong)?;
        entries.insert(place.key.to_string(), total.clone());
        self.write(&change);
        Ok(total)
    }

    /// Puts `value` at the end of the list at `place`, starting one, and gives its new length.
    pub fn append(&self, place: Place, value: Value) -> Result<usize, StoreError> {
        let change = journal::append(place, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(place));
        let entry = place
            .entries(&mut shard)
            .entry(place.key.to_string())
            .or_insert_with(|| Value::List(Vec::new()));
        let Value::List(list) = entry else {
            return Err(StoreError::NotAList);
        };
        list.push(value);
        self.write(&change);
        Ok(list.len())
    }

    /// Sets `place` to `value` if nothing is there; whether it did. Of several asking at once,
    /// exactly one hears yes.
    pub fn set_if_absent(&self, place: Place, value: Value) -> Result<bool, StoreError> {
        let change = journal::set(place, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(place));
        let entries = place.entries(&mut shard);
        if entries.contains_key(place.key) {
            return Ok(false);
        }
        entries.insert(place.key.to_string(), value);
        self.write(&change);
        Ok(true)
    }

    /// Sets `place` to `value`, replacing what was there.
    pub fn set(&self, place: Place, value: Value) -> Result<(), StoreError> {
        let change = journal::set(place, &value).ok_or(StoreError::TooLong)?;
        let mut shard = lock(self.shard(place));
        place
            .entries(&mut shard)
            .insert(place.key.to_string(), value);
        self.write(&change);
        Ok(())
    }

    /// Takes the key at `place` away; whether it was there.
    pub fn remove(&self, place: Place) -> bool {
        let mut shard = lock(self.shard(place));
        let components = shard.get_mut(&place.owner);
        let entries = components.and_then(|held| held.get_mut(place.component));
        let removed = entries.and_then(|entries| entries.remove(place.key));
        if let (Some(_), Some(change)) = (&removed, journal::remove(place)) {
            self.write(&change);
        }
        removed.is_some()
    }
}

/// The changes that set every value of one data key of `owner`.
fn held(owner: u64, component: &str, entries: &Entries) -> Vec<u8> {
    let set = |(key, value): (&String, &Value)| {
        let place = Place {
            owner,
            component,
            key,
        };
        journal::set(place, value).unwrap_or_default()
    };
    entries.iter().flat_map(set).collect()
}
