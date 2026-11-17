//! The blocks quarries build their frames of. The Gantry's is a plain lattice cube that drops
//! nothing. The others are poles joined toward the frame blocks beside them, drawn from a
//! hub-and-arm model (`arms`), and give back what they were built of when broken.

use std::sync::LazyLock;

use weft_sdk::{Block, Dir, Props, Registry, State, World, tool_tier};

use crate::arms::{Arm, HubAndArms};
use crate::lib::{BlockState, Pos, TypedWorld};

pub static GANTRY_FRAME: Block = Block::named("gantry_frame");
pub static LASHED_FRAME: Block = Block::named("lashed_frame");

/// A frame block of poles: the block, its model, and the item it drops when broken.
struct Poles {
    block: &'static Block,
    model: &'static LazyLock<HubAndArms>,
    drops: &'static str,
}

/// Log poles lashed with string: the log comes back, the string is lost.
static POLES: [Poles; 1] = [Poles {
    block: &LASHED_FRAME,
    model: &crate::arms::LASHED_FRAME,
    drops: "base:oak_log",
}];

pub fn register(r: &mut Registry) {
    r.block(&GANTRY_FRAME)
        .texture("gantry_frame")
        .cutout()
        .hardness(1.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .drops_nothing()
        .no_item();
    for poles in &POLES {
        register_poles(r, poles);
    }
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
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .drops(poles.drops, 1)
        .no_item();
}

/// The hub with a pole toward every side it joins, lashed where poles meet at an angle or end;
/// a pole running straight through shows bark. Its boxes are what it collides with, too.
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

/// The model's names for the bark and for the lashing.
const POLE: &str = "pole";
const KNOT: &str = "knot";

/// Whether a frame block joining toward `joins` is a pole running straight through it: two
/// arms, opposite each other.
fn runs_straight(joins: impl Fn(Dir) -> bool) -> bool {
    let arms: Vec<Dir> = Dir::ALL.into_iter().filter(|&dir| joins(dir)).collect();
    matches!(arms[..], [a, b] if a.opposite() == b)
}

/// Puts a frame block of `frame` at `cell`, joined to the frame blocks beside it.
pub fn put(world: &World, cell: Pos, frame: &Block) {
    world.set_block_typed(cell, BlockState::of(frame));
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
