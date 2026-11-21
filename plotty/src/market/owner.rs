//! Whose a stall is: the player who placed its register, kept in the `STALLS` table. Only they
//! set it up and break its register; everyone else who uses any of its blocks gets its shop.

use serde::{Deserialize, Serialize};
use weft_sdk::{Ask, BlockChange, Registry, Verdict, World, actions};
use weftkit::{PlayerKey, Pos, TypedWorld};

use super::blocks::{self, REGISTERS};
use super::pattern;
use super::screens::{SETUP, SHOP};
use super::stall::front_of;
use crate::dims;
use crate::tables::STALLS;

/// Whose a stall is.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct StallEntry {
    pub owner: PlayerKey,
}

pub fn register(r: &mut Registry) {
    r.before_event(&actions::USE_BLOCK, open);
}

/// Records who placed a register.
pub fn placed(world: &World, change: &BlockChange) {
    let Some(owner) = PlayerKey::of(world, change.player) else {
        return;
    };
    let key = (dims::key_of(world), Pos::from(change.pos));
    STALLS.set(&key, StallEntry { owner });
}

pub fn broken(world: &World, change: &BlockChange) {
    STALLS.remove(&(dims::key_of(world), Pos::from(change.pos)));
}

/// Whose the stall whose register is at `pos` is.
pub fn owner_at(world: &World, pos: Pos) -> Option<PlayerKey> {
    owner_in(dims::key_of(world), pos)
}

/// `owner_at` by dimension key.
pub fn owner_in(dim: u64, pos: Pos) -> Option<PlayerKey> {
    STALLS.get(&(dim, pos)).map(|entry| entry.owner)
}

/// Whether `you` own the stall `owner` owns: nobody owns a stall whose owner is unknown.
pub fn is_owner(owner: Option<&PlayerKey>, you: Option<&PlayerKey>) -> bool {
    owner.is_some() && owner == you
}

/// Whether `you` may break a block whose stall register, if it is one, `owner` owns: anyone the
/// rules let, except that a register is its owner's, or a moderator's who switched protection
/// off (`bypass`).
pub fn may_break(owner: Option<&PlayerKey>, you: &PlayerKey, bypass: bool) -> bool {
    bypass || owner.is_none_or(|owner| owner == you)
}

/// Opens the stall a used block belongs to, instead of what using it would do: its owner sets
/// it up, everyone else shops. Asked before the engine opens the register's own menu, which is
/// there only so its slot groups have names.
fn open(ask: &Ask) -> Verdict {
    if blocks::part_of(ask.state()).is_none() {
        return Verdict::Abstain;
    }
    let world = ask.world();
    let Some(register) = register_of(&world, Pos::from(ask.pos())) else {
        return Verdict::Abstain;
    };
    let player = ask.entity();
    let owner = owner_at(&world, register);
    let you = PlayerKey::of(&world, player);
    let menu = if is_owner(owner.as_ref(), you.as_ref()) {
        &SETUP
    } else {
        &SHOP
    };
    world.open_menu(player, menu, world.block_entity(register));
    Verdict::Deny
}

/// The register of the stall holding the block `pos`.
fn register_of(world: &World, pos: Pos) -> Option<Pos> {
    pattern::registers_near(pos).find(|near| {
        let state = world.block_typed(*near);
        REGISTERS.iter().any(|register| state.is(register))
            && pattern::layout(*near, front_of(world, *near))
                .iter()
                .any(|(at, _)| *at == pos)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> PlayerKey {
        PlayerKey::new(text).expect("a test key")
    }

    #[test]
    fn only_the_owner_owns_a_stall() {
        let (ana, bo) = (key("ana"), key("bo"));
        assert!(is_owner(Some(&ana), Some(&ana)), "Ana's own");
        assert!(!is_owner(Some(&ana), Some(&bo)), "not Bo's");
        assert!(!is_owner(None, None), "an unknown owner is nobody");
    }

    #[test]
    fn a_register_is_broken_only_by_its_owner() {
        let (ana, bo) = (key("ana"), key("bo"));
        assert!(may_break(Some(&ana), &ana, false), "the owner");
        assert!(
            !may_break(Some(&ana), &bo, false),
            "a friend trusted on the plot"
        );
        assert!(may_break(Some(&ana), &bo, true), "a moderator with bypass");
        assert!(may_break(None, &bo, false), "no register: the rules decide");
    }
}
