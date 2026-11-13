//! What scripts keep, saved in a file beside the scripts. As the game keeps components on
//! entities, a script keeps components on holders: a player, or a name of the scripts' own
//! choosing (`Holder`). A component is named `<script>:<name>` (`crate::name`) and holds one
//! value, which a script reads and sets whole.
//!
//! Regions tick in parallel and each may run scripts, so a read and a set are each one step
//! under a short lock. What a name holds is every region's at once: `turn` keeps a script's
//! read, change and write of it together.
//!
//! A set changes memory only. A thread of the store's own writes what changed to the file
//! (`save`, `journal`): at once when nothing was saved in the last second, and a second after
//! the last save otherwise. It writes each changed component as it is then, so a value set
//! every tick costs one write a second, and no tick waits for the disk. A mod is not told when
//! the game stops, so a change made within a second of another may not reach the file.

mod journal;
mod pack;
#[cfg(test)]
mod tests;
pub mod turn;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::thread::{self, Thread};
use std::time::Duration;
use std::{fs, io, mem};

use weft_sdk::error;

use self::journal::Journal;
use crate::lock;
use crate::value::Value;

/// The store every script reads and writes.
pub static STORE: Store = Store::new();

/// How long the saving thread rests after a save: what a change waits for the file at most.
const SAVE_PAUSE: Duration = Duration::from_secs(1);

/// Saves `STORE` from a thread of its own from now on, woken by the first set after a save.
pub fn save_in_background() -> io::Result<()> {
    let saving = || {
        loop {
            thread::park();
            STORE.save();
            thread::sleep(SAVE_PAUSE);
        }
    };
    let saver = thread::Builder::new().name("serverscript-save".into());
    let saver = saver.spawn(saving)?;
    let _ = STORE.saver.set(saver.thread().clone());
    Ok(())
}

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
    /// Where changes are written; `None` before `open`, and once writing failed. Locked before
    /// `kept` by what needs both.
    file: Mutex<Option<Journal>>,
    /// The thread that saves, which a set wakes; none in a store that is saved by hand.
    saver: OnceLock<Thread>,
}

struct Kept {
    held: Held,
    /// The components set since the last save.
    changed: BTreeSet<(Holder, String)>,
}

impl Store {
    pub const fn new() -> Store {
        let kept = Kept {
            held: BTreeMap::new(),
            changed: BTreeSet::new(),
        };
        Store {
            kept: Mutex::new(kept),
            file: Mutex::new(None),
            saver: OnceLock::new(),
        }
    }

    /// Fills the store from the file at `path`, which `save` writes changes to from now on.
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
        let mut file = lock(&self.file);
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
        match Journal::start(path, &kept.all()) {
            Ok(journal) => *file = Some(journal),
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
        let mut kept = lock(&self.kept);
        kept.changed.insert((holder.clone(), component.to_string()));
        kept.put(holder, component, value);
        if let Some(saver) = self.saver.get() {
            saver.unpark();
        }
    }

    /// Writes what changed since the last save to the file: each component as it is now,
    /// however often it was set. When that fails the store goes on in memory, and says so once.
    pub fn save(&self) {
        let mut file = lock(&self.file);
        let Some(journal) = file.as_mut() else {
            return;
        };
        let changes = lock(&self.kept).changes();
        if changes.is_empty() {
            return;
        }
        let mut written = journal.append(&changes);
        if written.is_ok() && journal.outgrown() {
            let all = lock(&self.kept).all();
            written = journal.rewrite(&all);
        }
        if let Err(error) = written {
            error!("scripts' components are no longer saved: {error}");
            *file = None;
        }
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
    fn all(&self) -> Vec<u8> {
        let mut changes = Vec::new();
        for (holder, components) in &self.held {
            for (component, value) in components {
                changes.extend(journal::change(holder, component, Some(value)));
            }
        }
        changes
    }

    /// What was set since this was last asked, as the changes that make it now.
    fn changes(&mut self) -> Vec<u8> {
        let mut changes = Vec::new();
        for (holder, component) in mem::take(&mut self.changed) {
            let components = self.held.get(&holder);
            let value = components.and_then(|components| components.get(&component));
            changes.extend(journal::change(&holder, &component, value));
        }
        changes
    }
}
