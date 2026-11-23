//! Golden Woof, the server's money: no item, but a balance per player in the `BALANCES` table,
//! which every region reads and which holds offline players, so a stall can pay its owner while
//! they are away. Every change goes through `transfer` under one lock, so two payments at once
//! never make or lose Woof. Players see their balance in the inventory (`shown.rs`).

mod commands;
mod shown;

use std::sync::Mutex;

use weft_sdk::Registry;
use weftkit::PlayerKey;

use crate::settings;
use crate::tables::BALANCES;

/// The most Woof one player may hold, far above anything a server hands out.
const MAX_BALANCE: u64 = 1_000_000_000_000;

/// Held while a payment reads and writes balances.
static PAYING: Mutex<()> = Mutex::new(());

/// One side of a payment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Account<'a> {
    Player(&'a PlayerKey),
    /// Outside the game: where game masters' Woof comes from and rent goes.
    Outside,
}

/// Why a payment didn't happen.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Refused {
    /// The payer has less than the amount.
    Short,
    /// The payee would hold more than `MAX_BALANCE`.
    TooMuch,
}

impl Refused {
    /// The lang key saying so.
    pub fn key(self) -> &'static str {
        match self {
            Refused::Short => "plotty.woof.short",
            Refused::TooMuch => "plotty.woof.too_much",
        }
    }
}

pub fn register(r: &mut Registry) {
    shown::register(r);
    commands::register(r);
}

/// What `key` holds.
pub fn balance(key: &PlayerKey) -> u64 {
    BALANCES.get(key).unwrap_or(0)
}

/// Gives a player joining for the first time their starting balance.
pub fn start(key: &PlayerKey) {
    let _paying = lock();
    if BALANCES.get(key).is_none() {
        BALANCES.set(key, settings::get().starting_woof);
    }
}

/// Moves `amount` from `from` to `to`, all of it or nothing.
pub fn transfer(from: Account, to: Account, amount: u64) -> Result<(), Refused> {
    if from == to {
        return Ok(());
    }
    let _paying = lock();
    let held = |account: Account| match account {
        Account::Player(key) => Some(balance(key)),
        Account::Outside => None,
    };
    let (from_after, to_after) = moved(held(from), held(to), amount)?;
    if let (Account::Player(key), Some(after)) = (from, from_after) {
        BALANCES.set(key, after);
    }
    if let (Account::Player(key), Some(after)) = (to, to_after) {
        BALANCES.set(key, after);
    }
    Ok(())
}

/// The balances after paying `amount` from `from` to `to`; `None` is outside the game.
fn moved(
    from: Option<u64>,
    to: Option<u64>,
    amount: u64,
) -> Result<(Option<u64>, Option<u64>), Refused> {
    let from = match from {
        Some(held) => Some(held.checked_sub(amount).ok_or(Refused::Short)?),
        None => None,
    };
    let to = match to {
        Some(held) => Some(
            held.checked_add(amount)
                .filter(|after| *after <= MAX_BALANCE)
                .ok_or(Refused::TooMuch)?,
        ),
        None => None,
    };
    Ok((from, to))
}

fn lock() -> std::sync::MutexGuard<'static, ()> {
    PAYING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_payment_moves_the_amount() {
        assert_eq!(
            moved(Some(30), Some(5), 10),
            Ok((Some(20), Some(15))),
            "player to player"
        );
        assert_eq!(
            moved(None, Some(5), 10),
            Ok((None, Some(15))),
            "from outside"
        );
        assert_eq!(
            moved(Some(30), None, 30),
            Ok((Some(0), None)),
            "all of it, out of the game"
        );
    }

    #[test]
    fn nobody_pays_more_than_they_have() {
        assert_eq!(moved(Some(9), Some(0), 10), Err(Refused::Short), "short");
    }

    #[test]
    fn nobody_holds_more_than_the_most() {
        assert_eq!(
            moved(None, Some(MAX_BALANCE), 1),
            Err(Refused::TooMuch),
            "past the most"
        );
        assert_eq!(
            moved(None, Some(u64::MAX), 1),
            Err(Refused::TooMuch),
            "no overflow"
        );
    }
}
