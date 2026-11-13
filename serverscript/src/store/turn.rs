//! The turn at what names hold. Every region reaches a name's components, so a script that
//! reads one, changes it and writes it back needs the others to wait meanwhile, or two of them
//! counting a visit at the same moment would count one.
//!
//! A script takes the turn when it first reaches a name's component, and has it until it calls
//! into the world or ends. So a thread with the turn only runs the script and the store: it
//! never waits for the game while others wait for it. A player's components need no turn:
//! only the region the player is in reaches them.

use std::cell::RefCell;
use std::sync::{Mutex, MutexGuard};

use crate::lock;

static TURN: Mutex<()> = Mutex::new(());

thread_local! {
    /// The turn, while this thread has it.
    static MINE: RefCell<Option<MutexGuard<'static, ()>>> = const { RefCell::new(None) };
}

/// Waits for the turn, unless this thread has it already.
pub fn take() {
    MINE.with_borrow_mut(|mine| {
        if mine.is_none() {
            *mine = Some(lock(&TURN));
        }
    });
}

/// Gives the turn up, if this thread has it.
pub fn end() {
    MINE.take();
}

/// Gives the turn up when dropped: at the end of a script's call, also one that panicked.
pub struct Ending;

impl Drop for Ending {
    fn drop(&mut self) {
        end();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::store::{Holder, Store};
    use crate::value::Value;

    const THREADS: i64 = 8;
    const ROUNDS: i64 = 500;

    /// Reads the count, adds one and writes it back, as a script does.
    fn count(store: &Store) {
        let server = Holder::Named("server".into());
        let _ending = Ending;
        take();
        let Some(Value::Int(visits)) = store.get(&server, "test:visits") else {
            return store.set(server, "test:visits", Some(Value::Int(1)));
        };
        store.set(server, "test:visits", Some(Value::Int(visits + 1)));
    }

    #[test]
    fn counts_from_many_threads_at_once_all_count() {
        let store = Store::new();
        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| (0..ROUNDS).for_each(|_| count(&store)));
            }
        });
        let server = Holder::Named("server".into());
        let visits = store.get(&server, "test:visits");
        assert_eq!(visits, Some(Value::Int(THREADS * ROUNDS)));
    }

    #[test]
    fn taking_the_turn_twice_does_not_wait_for_itself() {
        let _ending = Ending;
        take();
        take();
    }
}
