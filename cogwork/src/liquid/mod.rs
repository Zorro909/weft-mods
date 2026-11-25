//! Liquids as cogwork's own blocks hold them: liquid tubes, tanks and the pump. Each keeps one
//! liquid and how many millibuckets of it, and liquid moves between holders that touch, from
//! where it stands or pushes higher to where it stands lower (`flow`). A full holder passes on
//! the push of what fills it, so liquid stands in a tube before it rises, and a pump pushes it
//! up. Steam rises instead: it spreads to where holders are emptier, whatever their height.
//! Holders that have settled sleep until a neighbour changes (`turn`). A machine may also hand
//! liquid it keeps apart straight into the holder beside it, or draw it out of one (`pipe`).
//!
//! A liquid is kept by the key of its name, as ghosts keep items: a liquid's item is named like
//! it, so `ghost::item_of` finds that item again.

mod bucket;
mod flow;
mod pipe;
mod turn;
mod vessel;

pub use bucket::{hand_over, use_bucket};
pub use flow::{Level, flow};
pub use pipe::{draw, pour};
pub use vessel::{Holders, Vessel, is_vessel, reaches};
use weft_sdk::{Component, Registry, World, component_flags, item_name, liquid_of, value};

use crate::ghost::{self, ItemKey, NO_ITEM};
use crate::lib::{BlockState, Pos, TypedWorld};

/// The key of a liquid: of its name, as `ghost` keys items.
pub type LiquidKey = ItemKey;
pub const NO_LIQUID: LiquidKey = NO_ITEM;

/// Millibuckets in a bucket, and in a source block of a liquid.
pub const BUCKET: u32 = 1000;
/// The most millibuckets one holder hands one neighbour a turn.
pub const MOST_A_TURN: u32 = 100;

/// Whether a holder of `capacity` with `amount` in is as good as full: no more than a turn short.
/// A full line of tubes hands a turn on and gets one back, so it stays that close; it is full,
/// and passes on the push of what fills it.
pub fn is_full(amount: u32, capacity: u32) -> bool {
    amount + MOST_A_TURN >= capacity
}

/// A holder with less than this left takes another liquid, the rest of the old one lost: so a
/// little left behind never blocks a line. A big holder may keep more (`dribble`). Steam and
/// sulfuric acid are the exception (`Held::takes`).
pub const DRIBBLE: u32 = 10;

/// The head of a holder that has told none for what it holds now: no push to anyone.
pub const NO_HEAD: i64 = i64::MIN;

value! {
    /// What one holder keeps: `liquid` (`NO_LIQUID` for none), `amount` millibuckets of it,
    /// `shown`, what its fill block shows now, so the world changes only when that does; `head`,
    /// the head it last told its neighbours (`flow`), and `asleep`, 1 while it has settled.
    #[derive(Default)]
    pub struct Held {
        pub liquid: u64,
        pub amount: u32,
        pub shown: u32,
        pub head: i64,
        pub asleep: u32,
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
    /// it is as good as empty there. A liquid `kept_apart` never takes the place of another, nor
    /// another liquid its place, while a millibucket of the one is left: it would be lost unseen.
    pub fn takes(&self, liquid: LiquidKey, capacity: u32) -> bool {
        let apart = self.amount > 0 && (kept_apart(self.liquid) || kept_apart(liquid));
        self.liquid == liquid || self.is_empty_in(capacity) && !apart
    }

    /// Puts `amount` of `liquid` in, which `takes` allowed, and wakes it.
    pub fn add(&mut self, liquid: LiquidKey, amount: u32) {
        if self.liquid != liquid {
            // What its fill shows and the head it told are of the old liquid.
            *self = Held {
                liquid,
                head: NO_HEAD,
                ..Held::default()
            };
        }
        self.amount += amount;
        self.asleep = 0;
    }

    /// Takes `amount` out, forgetting the liquid once none is left, and wakes it.
    pub fn remove(&mut self, amount: u32) {
        self.amount -= amount.min(self.amount);
        if self.amount == 0 {
            self.liquid = NO_LIQUID;
        }
        self.asleep = 0;
    }
}

/// What a holder of `capacity` may be left with that never flows on, and one more: beside an
/// empty liquid tube on the same floor, the smallest holder, `flow` rounds less than this down to
/// nothing. Its head is rounded too, which may keep one more.
fn dribble(capacity: u32) -> u32 {
    let tube = crate::liquid_tube::CAPACITY;
    DRIBBLE.max(capacity.div_ceil(tube) + 2)
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &HELD,
        Held::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&HELD, turn::tick);
}

/// The key of the liquid named `name` (`cogwork:oil`).
pub fn key_named(name: &str) -> LiquidKey {
    ghost::key_of_name(name)
}

/// Whether `liquid` is water, all the grinding mill and the washer take.
pub fn is_water(liquid: LiquidKey) -> bool {
    liquid == key_named("base:water")
}

/// Whether `liquid` rises: steam, which stands in a holder by how full it is, not how high
/// (`flow::pressure`).
pub fn rises(liquid: LiquidKey) -> bool {
    liquid == crate::steam::key()
}

/// Whether `liquid` is kept apart from every other liquid in a holder (`Held::takes`): steam and
/// sulfuric acid. Water, oil and the rest give way to each other as before.
fn kept_apart(liquid: LiquidKey) -> bool {
    rises(liquid) || liquid == crate::acid::key()
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
            // Beside an empty liquid tube on its floor, the smallest holder, none of this flows.
            let stays = |amount| {
                let from = Level {
                    head: flow::surface(0, amount, capacity),
                    per_block: capacity,
                };
                let empty_tube = Level {
                    head: 0,
                    per_block: tube,
                };
                flow(from, empty_tube, u32::MAX) == 0
            };
            let stuck = (0..).take_while(|&amount| stays(amount)).last().unwrap();
            let left = Held {
                liquid: WATER,
                amount: stuck,
                ..Held::default()
            };
            assert!(left.takes(OIL, capacity), "{stuck} mB stuck in {capacity}");
        }
    }

    fn holding(liquid: LiquidKey, amount: u32) -> Held {
        Held {
            liquid,
            amount,
            ..Held::default()
        }
    }

    #[test]
    fn steam_never_takes_the_place_of_another_liquid_nor_it_of_steam() {
        let tube = crate::liquid_tube::CAPACITY;
        let steam = crate::steam::key();
        assert!(!holding(WATER, 9).takes(steam, tube), "9 mB of water stays");
        assert!(!holding(steam, 1).takes(WATER, tube), "1 mB of steam stays");
        assert!(holding(WATER, 0).takes(steam, tube), "none left");
        assert!(holding(NO_LIQUID, 0).takes(WATER, tube), "an empty tube");
        assert!(
            holding(OIL, 9).takes(WATER, tube),
            "oil gives way as before"
        );
    }

    #[test]
    fn acid_never_takes_the_place_of_another_liquid_nor_it_of_acid() {
        let tube = crate::liquid_tube::CAPACITY;
        let (acid, steam) = (crate::acid::key(), crate::steam::key());
        assert!(!holding(acid, 5).takes(WATER, tube), "5 mB of acid stays");
        assert!(!holding(acid, 1).takes(OIL, tube), "1 mB of acid stays");
        assert!(!holding(WATER, 1).takes(acid, tube), "1 mB of water stays");
        assert!(
            !holding(acid, 1).takes(steam, tube),
            "nor steam in its place"
        );
        assert!(holding(acid, 0).takes(WATER, tube), "none left");
        assert!(holding(acid, 400).takes(acid, tube), "more acid");
        assert!(holding(NO_LIQUID, 0).takes(acid, tube), "an empty tube");
    }

    #[test]
    fn a_new_liquid_forgets_the_head_told_of_the_old() {
        let mut tube = Held {
            head: 40 * flow::UNIT,
            ..Held::default()
        };
        tube.add(crate::steam::key(), 1_000);
        assert_eq!(tube.head, NO_HEAD, "steam has no height");
        tube.head = flow::UNIT;
        tube.add(crate::steam::key(), 1);
        assert_eq!(tube.head, flow::UNIT, "more of the same keeps its head");
    }

    #[test]
    fn what_a_holder_keeps_is_saved_as_laid_out() {
        assert_eq!(std::mem::offset_of!(Held, liquid), 0, "its liquid");
        assert_eq!(std::mem::offset_of!(Held, amount), 8, "how much");
        assert_eq!(std::mem::offset_of!(Held, shown), 12, "what its fill shows");
        assert_eq!(std::mem::offset_of!(Held, head), 16, "the head it told");
        assert_eq!(std::mem::offset_of!(Held, asleep), 24, "whether it sleeps");
    }
}
