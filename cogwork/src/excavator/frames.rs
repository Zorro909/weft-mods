//! The blocks quarries build their frames of. The Steel Excavator's is a cube of riveted steel
//! lattice, built of a steel ingot for every four blocks, and the Gantry's a cube of composite
//! panels on steel, a composite panel for every four: the block that took the ingot or panel is
//! marked `paid` and gives it back when broken, the same on every try, so a machine with a full
//! output can't break it for nothing. The others, the Timber Quarry's lashed logs and the Iron
//! Quarry's bars, are poles joined toward the frame blocks beside them, drawn from a hub-and-arm
//! model (`arms`), and give back what they were built of when broken.
//!
//! `gantry_frame` is the plain lattice the Gantry built with energy alone before composite
//! panels: it stays in worlds that have it, holds up the frames already built and drops nothing.

use std::sync::LazyLock;

use weft_sdk::{Block, Cond, Dir, Pool, Props, Registry, State, World, tool_tier};

use crate::arms::{Arm, HubAndArms};
use crate::lib::{BlockState, Pos, TypedWorld};

pub static GANTRY_FRAME: Block = Block::named("gantry_frame");
pub static LASHED_FRAME: Block = Block::named("lashed_frame");
pub static IRON_FRAME: Block = Block::named("iron_frame");
pub static STEEL_FRAME: Block = Block::named("steel_frame");
pub static COMPOSITE_FRAME: Block = Block::named("composite_frame");

/// The property of a frame block paid for in batches that marks the one that took its batch.
const PAID: &str = "paid";

/// A cube of frame paid for in batches: the block, its texture, and the item its block that took
/// a batch gives back.
struct Batched {
    block: &'static Block,
    gives_back: &'static str,
    hardness: f32,
}

/// The steel lattice and the composite panels on steel.
static BATCHED: [Batched; 2] = [
    Batched {
        block: &STEEL_FRAME,
        gives_back: "steel_ingot",
        hardness: 1.5,
    },
    Batched {
        block: &COMPOSITE_FRAME,
        gives_back: "composite_panel",
        hardness: 2.0,
    },
];

/// A frame block of poles: the block, its model, and the item it drops when broken.
struct Poles {
    block: &'static Block,
    model: &'static LazyLock<HubAndArms>,
    drops: &'static str,
    sound_group: &'static str,
    mined_by: &'static str,
}

/// Log poles lashed with string: the log comes back, the string is lost. Iron bars riveted
/// together: the ingot comes back.
static POLES: [Poles; 2] = [
    Poles {
        block: &LASHED_FRAME,
        model: &crate::arms::LASHED_FRAME,
        drops: "base:oak_log",
        sound_group: "base:block.wood",
        mined_by: "axe",
    },
    Poles {
        block: &IRON_FRAME,
        model: &crate::arms::IRON_FRAME,
        drops: "base:iron_ingot",
        sound_group: "base:block.stone",
        mined_by: "pickaxe",
    },
];

pub fn register(r: &mut Registry) {
    r.block(&GANTRY_FRAME)
        .texture("gantry_frame")
        .cutout()
        .hardness(1.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .drops_nothing()
        .no_item();
    for batched in &BATCHED {
        register_batched(r, batched);
    }
    for poles in &POLES {
        register_poles(r, poles);
    }
}

/// A cube of frame whose block that took a batch gives it back.
fn register_batched(r: &mut Registry, batched: &Batched) {
    r.block(batched.block)
        .prop(PAID, ["false", "true"])
        .texture(batched.block.path())
        .cutout()
        .hardness(batched.hardness)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .no_item();
    r.loot(batched.block).pool(
        Pool::new()
            .when(Cond::harvested())
            .when(paid())
            .item(batched.gives_back),
    );
}

/// Whether the frame block broken is the one that took its batch.
fn paid() -> Cond {
    Cond::state(&format!("{PAID}=true"))
}

fn register_poles(r: &mut Registry, poles: &'static Poles) {
    let mut builder = r.block(poles.block);
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(move |props, state| model(props, state, poles.model))
        .on_neighbor(move |event| refresh(&event.world(), event.pos().into(), poles.block))
        .hardness(1.0)
        .mined_by(poles.mined_by, tool_tier::HAND)
        .sound_group(poles.sound_group)
        .drops(poles.drops, 1)
        .no_item();
}

/// The hub with a pole toward every side it joins, lashed or riveted where poles meet at an
/// angle or end; a pole running straight through shows the plain pole. Its boxes are what it
/// collides with, too.
fn model<'s>(props: &Props, state: State<'s>, model: &HubAndArms) -> State<'s> {
    let joins = |dir: Dir| props.is(dir.name(), "true");
    let arm = |dir: Dir| joins(dir).then_some(Arm::Closed);
    let straight = runs_straight(joins);
    let texture = |name: &str| {
        let name = if straight && name == KNOT { POLE } else { name };
        model.texture(name).to_string()
    };
    let pieces = model.pieces(arm, texture);
    crate::boxes::draw(state.texture(model.texture("particle")), &pieces)
}

/// The model's names for the plain pole and for the joint.
const POLE: &str = "pole";
const KNOT: &str = "knot";

/// Whether a frame block joining toward `joins` is a pole running straight through it: two
/// arms, opposite each other.
fn runs_straight(joins: impl Fn(Dir) -> bool) -> bool {
    let arms: Vec<Dir> = Dir::ALL.into_iter().filter(|&dir| joins(dir)).collect();
    matches!(arms[..], [a, b] if a.opposite() == b)
}

/// Puts a frame block of `frame` at `cell`, joined to the frame blocks beside it, and marked
/// `paid` if it took its batch and its block is paid for in batches.
pub fn put(world: &World, cell: Pos, frame: &Block, paid: bool) {
    let block_state = BlockState::of(frame).with_flag(PAID, paid);
    world.set_block_typed(cell, block_state);
    refresh(world, cell, frame);
}

/// Joins the `frame` block at `pos`, if it is one with poles, toward the frame blocks beside it.
fn refresh(world: &World, pos: Pos, frame: &Block) {
    let block_state = world.block_typed(pos);
    if !block_state.is(frame) || block_state.prop(Dir::Up.name()).is_none() {
        return;
    }
    let mut next = block_state;
    for dir in Dir::ALL {
        let joins = world.block_typed(pos.toward(dir)).is(frame);
        next = next.with_flag(dir.name(), joins);
    }
    if next != block_state {
        world.set_block_typed(pos, next);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_joints_are_lashed() {
        let along = |dirs: &'static [Dir]| move |dir| dirs.contains(&dir);
        assert!(runs_straight(along(&[Dir::East, Dir::West])), "a beam");
        assert!(runs_straight(along(&[Dir::Up, Dir::Down])), "a post");
        assert!(!runs_straight(along(&[Dir::East, Dir::Up])), "a corner");
        assert!(
            !runs_straight(along(&[Dir::East, Dir::West, Dir::Up])),
            "a post's foot"
        );
        assert!(!runs_straight(along(&[Dir::East])), "an end");
    }

    #[test]
    fn a_batched_frame_block_gives_the_same_back_on_every_try() {
        assert_eq!(
            paid().text(),
            "state(paid=true)",
            "the block that took the batch, never a roll a full output could wait out"
        );
        for batched in &BATCHED {
            let name = batched.block.path();
            let texture = format!("textures/block/{name}.png");
            assert!(crate::has_asset(&texture), "{name}: its look");
            let key = format!("block.cogwork.{name}");
            assert!(crate::translated(&key), "{name}: its name");
        }
    }

    #[test]
    fn every_frame_block_has_its_name_and_textures() {
        for poles in &POLES {
            let name = poles.block.path();
            assert!(
                crate::translated(&format!("block.cogwork.{name}")),
                "{name}"
            );
            for texture in ["particle", POLE, KNOT] {
                let file = format!("textures/block/{}.png", poles.model.texture(texture));
                assert!(crate::has_asset(&file), "{name}: no {file}");
            }
        }
    }
}
