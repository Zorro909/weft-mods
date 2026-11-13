//! What scripts keep, saved in a file beside the scripts. As the game keeps components on
//! entities, a script keeps components on holders: a player, or a name of the scripts' own
//! choosing (`Holder`). A component is named `<script>:<name>` (`crate::name`) and holds one
//! value, which a script reads and sets whole.
//!
//! Regions tick in parallel and each may run scripts, so a read and a set are each one step
//! under a short lock. What a name holds is every region's at once: `turn` keeps a script's
//! read, change and write of it together.
//!
//! Every change goes to the file as it is made (`journal`), so the store has no size of its own
//! to outgrow and nothing to write when the game stops.

mod journal;
mod pack;
#[cfg(test)]
mod tests;
pub mod turn;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Mutex;

use weft_sdk::error;

use crate::lock;
use crate::value::Value;

/// The store every script reads and writes.
pub static STORE: Store = Store::new();

/// What a component is on.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Holder {
    /// A player, by their number (`crate::player`).
    Player(u64),
    /// A thing of the scripts' own, by the name they gave it.
    Named(String),
}

/// Each holder's components, by their full names.
type Held = BTreeMap<Holder, BTreeMap<String, Value>>;

pub struct Store {
    kept: Mutex<Kept>,
}

struct Kept {
    held: Held,
    /// Where changes are written; `None` before `open`, and once writing failed.
    file: Option<File>,
}

impl Store {
    pub const fn new() -> Store {
        let kept = Kept {
            held: BTreeMap::new(),
            file: None,
        };
        Store {
            kept: Mutex::new(kept),
        }
    }

    /// Fills the store from the file at `path` and writes every change to it from now on.
    /// Gives what went wrong, for the log. A file that could not be read to its end is kept
    /// beside it as `.bak`, and the store goes on with what was read; if no copy can be kept
    /// the file stays as it is and nothing is saved.
    pub fn open(&self, path: &Path) -> Vec<String> {
        let name = path.display();
        let read = match journal::read(path) {
            Ok(read) => read,
            Err(error) => {
                return vec![format!(
                    "{name}: {error}; scripts' components are not saved"
                )];
            }
        };
        let mut kept = lock(&self.kept);
        for record in read.records {
            kept.put(record.holder, &record.component, record.value);
        }
        let mut problems = Vec::new();
        if !read.whole {
            // Writing the file anew drops what could not be read: not without a copy of it.
            match set_aside(path) {
                Ok(copied) => problems.push(copied),
                Err(problem) => return vec![problem],
            }
        }
        match journal::start(path, &kept.changes()) {
            Ok(file) => kept.file = Some(file),
            Err(error) => problems.push(format!(
                "{name}: {error}; scripts' components are not saved"
            )),
        }
        problems
    }

    /// The value of `component` on `holder`.
    pub fn get(&self, holder: &Holder, component: &str) -> Option<Value> {
        let kept = lock(&self.kept);
        kept.held.get(holder)?.get(component).cloned()
    }

    /// Gives `component` on `holder` the `value`, in place of its last; with `None` takes the
    /// component off.
    pub fn set(&self, holder: Holder, component: &str, value: Option<Value>) {
        let change = journal::change(&holder, component, value.as_ref());
        let mut kept = lock(&self.kept);
        kept.put(holder, component, value);
        kept.write(&change);
    }
}

/// Copies the file at `path`, which could not be read to its end, beside it as `.bak`. What to
/// tell the log: `Err` when there is no copy.
fn set_aside(path: &Path) -> Result<String, String> {
    let (name, copy) = (path.display(), path.with_extension("bak"));
    let unread = format!("{name} could not be read to its end");
    match fs::copy(path, &copy) {
        Ok(_) => Ok(format!("{unread}; a copy is kept as {}", copy.display())),
        Err(error) => Err(format!(
            "{unread}, and no copy could be kept: {error}; scripts' components are not saved"
        )),
    }
}

impl Kept {
    fn put(&mut self, holder: Holder, component: &str, value: Option<Value>) {
        let components = self.held.entry(holder).or_default();
        match value {
            Some(value) => components.insert(component.to_string(), value),
            None => components.remove(component),
        };
    }

    /// Everything held, as the changes that make it.
    fn changes(&self) -> Vec<u8> {
        let mut changes = Vec::new();
        for (holder, components) in &self.held {
            for (component, value) in components {
                changes.extend(journal::change(holder, component, Some(value)));
            }
        }
        changes
    }

    /// Puts `change` into the file. When that fails the store goes on in memory, and says so
    /// once.
    fn write(&mut self, change: &[u8]) {
        let Some(file) = self.file.as_mut() else {
            return;
        };
        if let Err(error) = journal::write(file, change) {
            error!("scripts' components are no longer saved: {error}");
            self.file = None;
        }
    }
}
