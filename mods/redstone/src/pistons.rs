//! Pistons in Weft (spec §9.1): the head, the placeholder holding a moving block's destination,
//! and what a landing move does to blocks and entities. The phases are `redstone_core`'s.

use std::collections::BTreeSet;

use redstone_core::{Pos, step};
use weft_sdk::abi::block_flags;
use weft_sdk::*;

use crate::blocks;

pub static HEAD: Block = Block::named("piston_head");
/// Invisible and solid: nothing is built or walks into a cell a block is moving into.
pub static MOVING: Block = Block::named("moving");
/// What a head needs behind it.
pub const BASE: &str = "redstone:piston_base";

pub fn register(r: &mut Registry) {
    r.block(&HEAD)
        .facing(Facings::ALL, Toward::Face)
        .prop("sticky", ["false", "true"])
        .hardness(0.5)
        .no_item()
        .drops_nothing()
        .states(|p, s| {
            let facing = blocks::core_dir(p.facing());
            let front = if p.is("sticky", "true") { "piston_sticky" } else { "piston_front" };
            let mut faces = ["piston_side"; 6];
            faces[facing.index()] = front;
            faces[facing.opposite().index()] = front;
            // Drawn pushing up: the plate, and the rod back into the base.
            let (plo, phi) = blocks::onto(facing.opposite(), [0.0, 12.0, 0.0], [16.0, 16.0, 16.0]);
            let (rlo, rhi) = blocks::onto(facing.opposite(), [6.0, 0.0, 6.0], [10.0, 12.0, 10.0]);
            s.faces(faces).needs(facing.opposite().index() as u32, &format!("#{BASE}")).model_box(plo, phi).model_box(rlo, rhi)
        });
    r.block(&MOVING)
        .air()
        .flags(block_flags::SOLID, block_flags::REPLACEABLE)
        .collision([0.0; 3], [16.0; 3])
        .unbreakable()
        .no_item()
        .drops_nothing();
}

/// `redstone_core::World::reserve`. Fails past the top or bottom of the world, which reads as
/// air (G1).
pub fn reserve(w: &World, p: Pos) -> bool {
    w.get(p) == 0 && w.set(p, MOVING.id())
}

/// Takes away a placeholder no move will fill.
pub fn unreserve(w: &World, p: Pos) {
    if MOVING.is(w.get(p)) {
        w.set(p, 0);
    }
}

/// `redstone_core::World::head`.
pub fn head(w: &World, p: Pos, head: Option<(redstone_core::Dir, bool)>) {
    match head {
        Some((facing, sticky)) => {
            let facing = Dir::from_face(facing.index() as u32).name();
            w.set(p, HEAD.with(&[("facing", facing), ("sticky", if sticky { "true" } else { "false" })]));
        }
        None => {
            let s = w.get(p);
            if HEAD.is(s) || MOVING.is(s) {
                w.set(p, 0);
            }
        }
    }
}

/// `redstone_core::World::shift`. Blocks move as they are: the mod's own devices get a new block
/// entity, and the `Sim` saves their pending change on it.
pub fn shift(w: &World, to: redstone_core::Dir, moved: &[Pos], broken: &[Pos], head: Option<Pos>) {
    for b in broken {
        if !w.break_block(*b, ItemStack::EMPTY, 0, true) {
            w.set(*b, 0);
        }
    }
    let cells: Vec<Pos> = moved.iter().map(|q| step(*q, to)).chain(head).collect();
    push_entities(w, &cells, to);
    for q in moved {
        let s = w.get(*q);
        // Never lose a block: it leaves only once it is written where it goes.
        if w.set(step(*q, to), s) {
            w.set(*q, 0);
        }
    }
}

/// Moves every entity in `cells` one block `to` (spec §9.1): players by teleport, they move
/// themselves (G10). Entities standing on moving blocks stay where they are.
fn push_entities(w: &World, cells: &[Pos], to: redstone_core::Dir) {
    let by = step([0; 3], to).map(|c| c as f64);
    let mut done = BTreeSet::new();
    for c in cells {
        let center = c.map(|v| v as f64 + 0.5);
        for id in w.near(center, 3.0, None) {
            if done.contains(&id) {
                continue;
            }
            let Some(mut e) = w.entity(id) else { continue };
            if !inside(&e, *c) || w.block_entity(e.pos.map(|v| v.floor() as i32)) == id {
                continue;
            }
            done.insert(id);
            let pos = std::array::from_fn(|i| e.pos[i] + by[i]);
            if e.kind == PLAYER.id() {
                w.teleport(id, None, pos);
            } else {
                e.pos = pos;
                w.set_entity(id, &e);
            }
        }
    }
}

/// Whether the entity's box reaches into the cell at `c`.
fn inside(e: &abi::EntityState, c: Pos) -> bool {
    let half = e.size[0] as f64 / 2.0;
    let lo = [e.pos[0] - half, e.pos[1], e.pos[2] - half];
    let hi = [e.pos[0] + half, e.pos[1] + e.size[1] as f64, e.pos[2] + half];
    (0..3).all(|i| lo[i] < c[i] as f64 + 1.0 && hi[i] > c[i] as f64)
}
