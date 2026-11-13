//! A mod's persistent data: tables of keys and values in files mapped into memory, in
//! `data/<mod>/<table>/` under the game folder (`folder.rs`), one set for the whole server. A change
//! is in the file once the call returns, and the kernel writes it to disk even if the game
//! crashes right after. Values stay in the file; memory holds the keys and where values lie.
//!
//! ```ignore
//! static BALANCES: Table<String, u64> = Table::named("balances");
//! static VISITS: Table<Pos, u32> = Table::write_optimized("visits");
//! BALANCES.update(&name, |coins| *coins += 5);
//! let coins = BALANCES.get_or_default(&name);
//! ```
//!
//! Keys and values are any `serde` types, encoded with `postcard`, which doesn't name fields: a
//! changed type takes a new table name, as a changed `SAVE` component does. So does switching a
//! table to the other lock design, which lays out its files differently.
//!
//! A table opens at its first use, taking its folder's `lock` file. A damaged file is repaired
//! then: an error is logged, the file as it was is kept as a `.bak` beside it, and every record
//! that can still be read stays. A file of a newer format is left alone. A table that can't be
//! opened (another server uses it, no permission, a newer format) panics with the reason, and so
//! does a change that can't be saved (a full disk; on btrfs or ZFS the kernel may stop the
//! server instead). Dead records are rewritten away on a thread of their own.

mod log;
mod mapped;
mod record;
mod shards;

use std::io;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::OnceLock;

use serde::Serialize;
use serde::de::DeserializeOwned;

use self::log::Damage;
use self::shards::{Locking, Shards};

/// A table of the mod that compiles it, from keys `K` to values `V`, read-optimized (`named`) or
/// write-optimized (`write_optimized`).
///
/// A closure handed to a table runs while the table holds a lock: it must not use the table.
pub struct Table<K, V> {
    name: &'static str,
    locking: Locking,
    shards: OnceLock<Shards>,
    types: PhantomData<fn() -> (K, V)>,
}

impl<K: Serialize + DeserializeOwned, V: Serialize + DeserializeOwned> Table<K, V> {
    /// The table `name`, kept in `data/<mod>/<name>/`, behind one lock that many regions can
    /// read through at once.
    pub const fn named(name: &'static str) -> Self {
        Table::with(name, Locking::ReadOptimized)
    }

    /// The table `name`, split into sixteen files with a lock each, for a table that many regions
    /// change at once.
    pub const fn write_optimized(name: &'static str) -> Self {
        Table::with(name, Locking::WriteOptimized)
    }

    const fn with(name: &'static str, locking: Locking) -> Self {
        Table {
            name,
            locking,
            shards: OnceLock::new(),
            types: PhantomData,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let key = self.encode(key);
        self.shards()
            .read(&key, |log| log.get(&key).and_then(decode))
    }

    /// The value of `key`, or `V`'s default where it has none.
    pub fn get_or_default(&self, key: &K) -> V
    where
        V: Default,
    {
        self.get(key).unwrap_or_default()
    }

    pub fn set(&self, key: &K, value: V) {
        let key = self.encode(key);
        let value = self.encode(&value);
        self.saved(self.shards().write(&key, |log| log.set(&key, &value)));
    }

    /// Changes the value of `key`, starting from `V`'s default where it has none, in one step
    /// nobody else's change gets between. `change` must not use the table: it holds the lock.
    pub fn update(&self, key: &K, change: impl FnOnce(&mut V))
    where
        V: Default,
    {
        let key = self.encode(key);
        let saved = self.shards().write(&key, |log| {
            let mut value = log.get(&key).and_then(decode).unwrap_or_default();
            change(&mut value);
            log.set(&key, &self.encode(&value))
        });
        self.saved(saved);
    }

    pub fn remove(&self, key: &K) {
        let key = self.encode(key);
        self.saved(self.shards().write(&key, |log| log.remove(&key)));
    }

    /// Removes every key.
    pub fn clear(&self) {
        self.shards().write_each(|log| self.saved(log.clear()));
    }

    /// `visit` run on each key and value, in no order. `visit` must not use the table, not even to
    /// read: it holds the lock, and a waiting writer would block that read for ever.
    pub fn for_each(&self, mut visit: impl FnMut(K, V)) {
        self.shards().read_each(|log| {
            for (key, value) in log.entries() {
                if let (Some(key), Some(value)) = (decode(key), decode(value)) {
                    visit(key, value);
                }
            }
        });
    }

    fn shards(&self) -> &Shards {
        self.shards.get_or_init(|| {
            let folder = crate::folder::mod_folder("data").join(self.name);
            let (shards, damages) = self.open_in(&folder);
            for damage in damages {
                weft_sdk::error!("table {}: {damage}", self.name);
            }
            shards
        })
    }

    /// The table's files in `folder`, and what was damaged in them.
    fn open_in(&self, folder: &Path) -> (Shards, Vec<Damage>) {
        let readable =
            |key: &[u8], value: &[u8]| decode::<K>(key).is_some() && decode::<V>(value).is_some();
        Shards::open(folder, self.locking, &readable).unwrap_or_else(|error| {
            panic!(
                "could not open table {} in {}: {error}",
                self.name,
                folder.display()
            )
        })
    }

    fn encode<T: Serialize>(&self, value: &T) -> Vec<u8> {
        postcard::to_allocvec(value)
            .unwrap_or_else(|error| panic!("table {} can't encode a value: {error}", self.name))
    }

    fn saved(&self, result: io::Result<()>) {
        if let Err(error) = result {
            panic!("could not save table {}: {error}", self.name);
        }
    }
}

/// `bytes` read as a `T`, all of them.
fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    match postcard::take_from_bytes(bytes) {
        Ok((value, [])) => Some(value),
        _ => None,
    }
}

/// An empty folder of its own under the system's temporary folder.
#[cfg(test)]
pub(crate) fn test_folder(name: &str) -> std::path::PathBuf {
    let folder = std::env::temp_dir().join(format!("weftkit-store-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).expect("a test folder can be made");
    folder
}

#[cfg(test)]
mod tests;
