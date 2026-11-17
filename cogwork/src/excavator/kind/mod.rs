//! The kinds of quarry, one row each: what the frame is built of, how large and how deep it digs,
//! the bits it takes and how much Ore Rubble it brings up. Every kind is the same machine
//! (`Dig`); which one a block entity is comes from its block, never from saved data.
//!
//! A new kind is a row of `rows`, its block, its frame block and its rig's textures.

pub mod rows;

pub use rows::KINDS;
use weft_sdk::{Block, Item, ItemStack, Menu, Sound, Tag};

use super::Frame;
use super::bit::Bit;
use super::depth::Floor;
use super::rig::RigLook;
use crate::lib::BlockState;

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
    /// What each batch of frame blocks takes from the `materials` group, beside its energy.
    pub frame_cost: &'static [Cost],
    /// The frame blocks one `frame_cost` pays for. It is taken as the first of them goes up,
    /// and that block gives it back when broken (`frames`).
    pub frame_batch: u32,
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

/// Some of an item, or of any item with a tag, that a batch of frame blocks takes.
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
    /// Down to the world's lowest layer for excavators (`excavator_lowest_y`), even where its
    /// settings let excavators dig on with none.
    Lowest,
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

impl Kind {
    /// The kind of quarry `block_state` is, if it is one.
    pub fn of(block_state: BlockState) -> Option<&'static Kind> {
        KINDS.into_iter().find(|kind| block_state.is(kind.block))
    }

    /// Whether `block_state` is a quarry or a frame of any kind, which no quarry mines: the
    /// Gantry's old lattice too, which holds up frames built before composite panels.
    pub fn is_part(block_state: BlockState) -> bool {
        block_state.is(&super::GANTRY_FRAME)
            || KINDS
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

    /// The lowest layer it mines under `frame`, by its depth, the `world`'s floor and the
    /// world's `lowest_y` for excavators.
    pub fn floor(&self, frame: &Frame, world: Floor, lowest_y: i32) -> Floor {
        match self.depth {
            Depth::Below(layers) => world.raised_to(frame.lo.y - layers),
            Depth::Lowest => world.raised_to(lowest_y),
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

    /// Whether `bit` goes into its bit slot.
    pub fn takes_bit(&self, bit: &Bit) -> bool {
        self.fit(bit).is_some()
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
