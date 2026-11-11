//! Liquids as cogwork's own blocks hold them: liquid tubes, tanks and the pump. Each keeps one
//! liquid and how many millibuckets of it, and every tick liquid moves between holders that
//! touch, from the fuller to the emptier by how full each is (`share`).
//!
//! A liquid is kept by the key of its name, as ghosts keep items: a liquid's item is named like
//! it, so `ghost::item_of` finds that item again.

mod bucket;
mod share;
mod vessel;

pub use bucket::use_bucket;
pub use share::{Level, share};
pub use vessel::{Holders, Vessel, is_vessel, reaches};
use weft_sdk::{Component, Registry, World, component_flags, item_name, liquid_of, value};

use crate::ghost::{self, ItemKey, NO_ITEM};
use crate::lib::{BlockState, Pos, TypedWorld};

/// The key of a liquid: of its name, as `ghost` keys items.
pub type LiquidKey = ItemKey;
pub const NO_LIQUID: LiquidKey = NO_ITEM;

/// Millibuckets in a bucket, and in a source block of a liquid.
pub const BUCKET: u32 = 1000;
/// A holder with less than this left takes another liquid, the rest of the old one lost: so a
/// little left behind never blocks a line. A big holder may keep more (`dribble`).
pub const DRIBBLE: u32 = 10;

value! {
    /// What one holder keeps: `liquid` (`NO_LIQUID` for none), `amount` millibuckets of it, and
    /// `shown`, what its fill block shows now, so the world changes only when that does.
    #[derive(Default)]
    pub struct Held {
        pub liquid: u64,
        pub amount: u32,
        pub shown: u32,
    }
}
/// The liquid a block of this mod holds: a liquid tube's, a tank's share, a pump's.
pub static HELD: Component<Held> = Component::named("cogwork:held_liquid");

impl Held {
    /// Whether it holds nothing worth keeping.
    pub fn is_empty(&self) -> bool {
        self.liquid == NO_LIQUID || self.amount < DRIBBLE
    }

    /// Whether it is as good as empty in a holder of `capacity`: less is left than sharing can
    /// hand on from there (`dribble`).
    pub fn is_empty_in(&self, capacity: u32) -> bool {
        self.is_empty() || self.amount < dribble(capacity)
    }

    /// Whether `liquid` may go into a holder of `capacity` holding this: the same liquid, or
    /// it is as good as empty there.
    pub fn takes(&self, liquid: LiquidKey, capacity: u32) -> bool {
        self.liquid == liquid || self.is_empty_in(capacity)
    }

    /// Puts `amount` of `liquid` in, which `takes` allowed.
    pub fn add(&mut self, liquid: LiquidKey, amount: u32) {
        if self.liquid != liquid {
            // What its fill shows is of the old liquid: shown anew.
            *self = Held {
                liquid,
                ..Held::default()
            };
        }
        self.amount += amount;
    }

    /// Takes `amount` out, forgetting the liquid once none is left.
    pub fn remove(&mut self, amount: u32) {
        self.amount -= amount.min(self.amount);
        if self.amount == 0 {
            self.liquid = NO_LIQUID;
        }
    }
}

/// What a holder of `capacity` may be left with that sharing never hands on, and one more: beside
/// an empty liquid tube, the smallest holder, sharing rounds less than this down to nothing.
fn dribble(capacity: u32) -> u32 {
    let tube = crate::liquid_tube::CAPACITY;
    DRIBBLE.max(capacity.div_ceil(tube) + 1)
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &HELD,
        Held::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&HELD, vessel::tick);
}

/// The key of the liquid named `name` (`cogwork:oil`).
pub fn key_named(name: &str) -> LiquidKey {
    ghost::key_of_name(name)
}

/// The liquid the block `block_state` is, and whether it is a source of it.
pub fn in_world(block_state: BlockState) -> Option<(LiquidKey, bool)> {
    liquid_of(block_state.id())?;
    let source = block_state.prop("level") == Some("0");
    Some((key_named(block_state.block_name()), source))
}

/// The name of the liquid `liquid`, if a loaded mod has it.
pub fn name(liquid: LiquidKey) -> Option<&'static str> {
    match ghost::item_of(liquid) {
        0 => None,
        item => Some(item_name(item)),
    }
}

/// The liquid at `pos`, if it is one.
pub fn at(world: &World, pos: Pos) -> Option<(LiquidKey, bool)> {
    in_world(world.block_typed(pos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tank::CELL_CAPACITY;

    const WATER: LiquidKey = 7;
    const OIL: LiquidKey = 8;

    #[test]
    fn what_sharing_cannot_hand_on_lets_another_liquid_in() {
        let tube = crate::liquid_tube::CAPACITY;
        let biggest_tank = CELL_CAPACITY * crate::tank::merge::MOST as u32;
        for capacity in [tube, crate::pump::CAPACITY, CELL_CAPACITY, biggest_tank] {
            // Beside an empty liquid tube, the smallest holder, sharing hands on nothing of this.
            let stays = |amount| {
                let from = Level { amount, capacity };
                share(
                    from,
                    Level {
                        amount: 0,
                        capacity: tube,
                    },
                    u32::MAX,
                ) == 0
            };
            let stuck = (0..).take_while(|&amount| stays(amount)).last().unwrap();
            let left = Held {
                liquid: WATER,
                amount: stuck,
                shown: 0,
            };
            assert!(left.takes(OIL, capacity), "{stuck} mB stuck in {capacity}");
        }
    }

    #[test]
    fn what_a_holder_keeps_is_saved_as_laid_out() {
        assert_eq!(std::mem::offset_of!(Held, liquid), 0, "its liquid");
        assert_eq!(std::mem::offset_of!(Held, amount), 8, "how much");
        assert_eq!(std::mem::offset_of!(Held, shown), 12, "what its fill shows");
    }
}
