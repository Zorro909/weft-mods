//! The kinds of quarry, one row each: what the frame is built of, how large and how deep it digs,
//! the bits it takes and how much Ore Rubble it brings up. Every kind is the same machine
//! (`Dig`); which one a block entity is comes from its block, never from saved data.
//!
//! A new kind is a row here, its block, its frame block and its rig's textures.

use weft_sdk::{Block, Item, ItemStack, Menu, Sound, Tag};

use super::bit::{Bit, IRON_BIT, STONE_BIT};
use super::depth::Floor;
use super::rig::{GANTRY_RIG, RigLook, TIMBER_RIG};
use super::{EXCAVATOR, Frame, GANTRY_FRAME, LASHED_FRAME, TIMBER_QUARRY};
use crate::lib::BlockState;
use crate::sounds;

/// One kind of quarry.
pub struct Kind {
    pub block: &'static Block,
    /// How long its block takes to break, and the sounds it makes underfoot and breaking.
    pub hardness: f32,
    pub sound_group: &'static str,
    /// The menu its block opens; the screen shows the `bit` and `materials` groups of a kind
    /// that has them.
    pub menu: &'static Menu,
    pub frame: &'static Block,
    /// What each frame block takes from the `materials` group, beside its energy.
    pub frame_cost: &'static [Cost],
    pub standby: Standby,
    pub inside: Inside,
    pub depth: Depth,
    /// The bits it takes, and the most energy each puts into a block a tick.
    pub bits: &'static [Fit],
    /// The energy it puts into a block a tick with no bit, for one that digs without; it then
    /// breaks everything it can reach.
    pub bare: Option<i64>,
    /// Ore Rubble per block of metal ore, and how many times the drops of a gem ore: the
    /// fraction a chance of one more.
    pub rubble_per_ore: f64,
    pub rig: &'static RigLook,
    /// What is heard as it mines a block, and as it puts up a frame block (every fourth).
    pub dig_sound: &'static Sound,
    pub frame_sound: &'static Sound,
}

/// A bit a quarry takes, and the energy it puts into a block a tick with it.
pub struct Fit {
    pub bit: &'static Bit,
    pub draw: i64,
}

/// Some of an item, or of any item with a tag, that a frame block takes.
pub struct Cost {
    pub material: Material,
    pub count: u32,
}

pub enum Material {
    Tagged(&'static Tag),
    Item(&'static Item),
}

/// The energy a quarry draws every tick to hold up its frame.
pub enum Standby {
    /// The same for any size: a light frame.
    Flat(i64),
    /// More the longer its sides (`Frame::standby`).
    BySides,
}

/// How large an area a quarry takes, in blocks across the inside.
pub struct Inside {
    /// Its square when no markers or frame say otherwise.
    pub default: i32,
    /// The most across on either axis: markers further apart are taken only that far. None
    /// takes all the world's markers reach.
    pub largest: Option<i32>,
}

/// How deep a quarry digs.
pub enum Depth {
    /// This many layers below its own, or to the world's floor if that comes first.
    Below(i32),
    /// As deep as the world's settings let excavators dig.
    World,
}

/// What a quarry digs with now: the energy it puts into a block a tick, the tier it breaks up
/// to, and its bit, if it has one.
#[derive(Clone, Copy)]
pub struct Drill {
    pub draw: i64,
    pub tier: u32,
    pub bit: Option<&'static Bit>,
}

static LOGS: Tag = Tag::named("logs");
static STRING: Item = Item::named("base:string");
static TIMBER_MENU: Menu = Menu::named("timber_quarry");
static EXCAVATOR_MENU: Menu = Menu::named("excavator");

pub static TIMBER: Kind = Kind {
    block: &TIMBER_QUARRY,
    hardness: 2.0,
    sound_group: "base:block.wood",
    menu: &TIMBER_MENU,
    frame: &LASHED_FRAME,
    frame_cost: &[
        Cost {
            material: Material::Tagged(&LOGS),
            count: 1,
        },
        Cost {
            material: Material::Item(&STRING),
            count: 1,
        },
    ],
    standby: Standby::Flat(4),
    inside: Inside {
        default: 5,
        largest: Some(7),
    },
    depth: Depth::Below(16),
    bits: &[
        Fit {
            bit: &STONE_BIT,
            draw: 12,
        },
        Fit {
            bit: &IRON_BIT,
            draw: 18,
        },
    ],
    bare: None,
    rubble_per_ore: 1.0,
    rig: &TIMBER_RIG,
    dig_sound: &sounds::TIMBER_DIG,
    frame_sound: &sounds::TIMBER_FRAME,
};

/// Today's excavator, the Gantry: it digs as it always has with no bit.
pub static GANTRY: Kind = Kind {
    block: &EXCAVATOR,
    hardness: 4.0,
    sound_group: "base:block.stone",
    menu: &EXCAVATOR_MENU,
    frame: &GANTRY_FRAME,
    frame_cost: &[],
    standby: Standby::BySides,
    inside: Inside {
        default: 9,
        largest: None,
    },
    depth: Depth::World,
    bits: &[],
    bare: Some(120),
    rubble_per_ore: 2.0,
    rig: &GANTRY_RIG,
    dig_sound: &sounds::DRILL,
    frame_sound: &sounds::FRAME_PLACE,
};

pub static KINDS: [&Kind; 2] = [&TIMBER, &GANTRY];

impl Kind {
    /// The kind of quarry `block_state` is, if it is one.
    pub fn of(block_state: BlockState) -> Option<&'static Kind> {
        KINDS.into_iter().find(|kind| block_state.is(kind.block))
    }

    /// Whether `block_state` is a quarry or a frame of any kind, which no quarry mines.
    pub fn is_part(block_state: BlockState) -> bool {
        KINDS
            .iter()
            .any(|kind| block_state.is(kind.block) || block_state.is(kind.frame))
    }

    /// The standby energy a tick to hold up `frame`.
    pub fn standby(&self, frame: &Frame) -> i64 {
        match self.standby {
            Standby::Flat(energy) => energy,
            Standby::BySides => frame.standby(),
        }
    }

    /// The lowest layer it mines under `frame`, by its depth and the `world`'s floor.
    pub fn floor(&self, frame: &Frame, world: Floor) -> Floor {
        match self.depth {
            Depth::Below(layers) => world.raised_to(frame.lo.y - layers),
            Depth::World => world,
        }
    }

    /// What it digs with, with `fitted` in its bit slot; none when it needs a bit it hasn't.
    pub fn drill(&self, fitted: Option<&'static Bit>) -> Option<Drill> {
        let fit = fitted.and_then(|bit| self.fit(bit));
        match (fit, self.bare) {
            (Some(fit), _) => Some(Drill {
                draw: fit.draw,
                tier: fit.bit.tier,
                bit: Some(fit.bit),
            }),
            (None, Some(draw)) => Some(Drill {
                draw,
                tier: u32::MAX,
                bit: None,
            }),
            (None, None) => None,
        }
    }

    /// The most energy it puts into putting up a frame block a tick: its best drill's, whatever
    /// bit is fitted. Clearing the frame's way is mining, at the fitted drill's.
    pub fn build_draw(&self) -> i64 {
        let best_bit = self.bits.iter().map(|fit| fit.draw).max();
        self.bare.max(best_bit).unwrap_or_default()
    }

    /// How it takes `bit`, if it does.
    fn fit(&self, bit: &Bit) -> Option<&Fit> {
        self.bits.iter().find(|fit| std::ptr::eq(fit.bit, bit))
    }

    /// Whether `stack` is something its frame is built of.
    pub fn takes_material(&self, stack: ItemStack) -> bool {
        self.frame_cost
            .iter()
            .any(|cost| cost.material.is(stack.item))
    }

    /// The items its `bit` group lets in, as a slot's `accepts` rule.
    pub fn bit_rule(&self) -> String {
        let names = self.bits.iter().map(|fit| fit.bit.item.name().to_string());
        names.collect::<Vec<_>>().join(" ")
    }

    /// The items its `materials` group lets in, as a slot's `accepts` rule.
    pub fn material_rule(&self) -> String {
        let names = self.frame_cost.iter().map(|cost| cost.material.rule());
        names.collect::<Vec<_>>().join(" ")
    }
}

impl Material {
    /// Whether the item `item` is this material.
    pub fn is(&self, item: u32) -> bool {
        match self {
            Material::Tagged(tag) => tag.on_item(item),
            Material::Item(of) => of.id() == item,
        }
    }

    /// It in a slot's `accepts` rule.
    fn rule(&self) -> String {
        match self {
            Material::Tagged(tag) => format!("#{}", tag.name()),
            Material::Item(item) => item.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gantry_digs_as_ever_with_no_bit() {
        let drill = GANTRY.drill(None).expect("it digs bare");
        assert_eq!(drill.draw, 120, "120 a tick, as today");
        assert_eq!(drill.tier, u32::MAX, "it breaks all it reaches");
        assert!(drill.bit.is_none(), "and wears nothing");
    }

    #[test]
    fn the_timber_quarry_needs_a_bit_and_digs_harder_with_iron() {
        assert!(TIMBER.drill(None).is_none(), "no bit, no digging");
        let stone = TIMBER.drill(Some(&STONE_BIT)).expect("a stone bit fits");
        let iron = TIMBER.drill(Some(&IRON_BIT)).expect("an iron bit fits");
        assert_eq!((stone.draw, iron.draw), (12, 18), "work draw by the bit");
        assert!(iron.tier > stone.tier, "iron breaks more");
    }

    #[test]
    fn a_bit_that_does_not_fit_is_no_bit() {
        assert!(
            !GANTRY
                .bits
                .iter()
                .any(|fit| std::ptr::eq(fit.bit, &STONE_BIT))
        );
        let drill = GANTRY.drill(Some(&STONE_BIT)).expect("bare");
        assert!(drill.bit.is_none(), "the gantry ignores a stone bit");
    }

    #[test]
    fn frames_go_up_at_the_best_drill_s_pace() {
        assert_eq!(GANTRY.build_draw(), 120, "as today");
        assert_eq!(TIMBER.build_draw(), 18, "an iron bit's, bit or none");
    }

    #[test]
    fn a_timber_quarry_stops_sixteen_below() {
        let frame = Frame::from(([0, 10, 0], [6, 14, 6]));
        let floor = TIMBER.floor(&frame, Floor::at(-64));
        assert!(!floor.is_above(-6), "16 below its layer of 10");
        assert!(floor.is_above(-7), "and no further");
        let shallow = TIMBER.floor(&frame, Floor::at(0));
        assert!(shallow.is_above(-1), "the world's floor comes first");
        assert_eq!(
            GANTRY.floor(&frame, Floor::at(-64)),
            Floor::at(-64),
            "the world's"
        );
    }

    #[test]
    fn standby_is_flat_for_the_timber_quarry() {
        let frame = Frame::from(([0, 0, 0], [8, 4, 8]));
        assert_eq!(TIMBER.standby(&frame), 4, "a light frame");
        assert_eq!(GANTRY.standby(&frame), 12 + 7 + 7, "by its sides");
    }

    #[test]
    fn the_slot_rules_name_what_goes_in() {
        assert_eq!(TIMBER.material_rule(), "#logs base:string");
        // Outside the game an item's mod is not known: only the names after it.
        let rule = TIMBER.bit_rule();
        let bits: Vec<&str> = rule.split(' ').collect();
        let named = |name| bits.iter().any(|bit| bit.ends_with(name));
        assert!(named(":stone_bit") && named(":iron_bit"), "{bits:?}");
    }
}
