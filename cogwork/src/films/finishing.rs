//! An excavator at the end of its job, east of the other three: its frame up, its pit dug and its
//! drill a few blocks short of the floor of the world. Once its engines are lit it mines the
//! last blocks, or waits out an unloaded floor, and is done; the film waits for that
//! (`FINISHED`).

use std::sync::{Mutex, PoisonError};

use showcase_api::{Set, signal};
use weft_sdk::*;

use super::parts::{engine, fuel};
use crate::excavator::{DIG, EXCAVATOR, FLOOR_Y, Frame, GANTRY_FRAME, Stage, Status};
use crate::lib::{BlockState, Pos};

pub const FINISHER: Pos = Pos::new(32, 0, 4);
/// The signal the film waits for once the engines are lit.
pub const FINISHED: &str = "finished";
/// Blocks left to mine on the floor of the world when the engines are lit.
const LEFT: i32 = 3;
/// How deep the pit is dug where the camera sees it.
const PIT_DEPTH: i32 = 12;

/// The finisher whose end the film waits for: the player watching, and where it stands.
static WATCHED: Mutex<Option<(u64, Pos)>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.world_tick(signal_when_done);
}

/// The excavator, its engines, its frame and its pit.
pub fn build(set: &Set) {
    set.place(FINISHER.into(), EXCAVATOR.with(&[("facing", "south")]));
    for (at, toward) in engines() {
        engine(set, at, toward);
    }
    let world = set.world();
    let pos = Pos::from(set.at(FINISHER.into()));
    let frame = Frame::plan(&world, pos, Dir::South);
    for cell in frame.cells().into_iter().filter(|&c| c != pos) {
        BlockState::of(&GANTRY_FRAME).place(&world, cell);
    }
    let air = Block::lookup("core:air").unwrap_or(0);
    for y in pos.y - PIT_DEPTH..pos.y {
        for z in frame.lo.z + 1..frame.hi.z {
            for x in frame.lo.x + 1..frame.hi.x {
                world.set([x, y, z], air);
            }
        }
    }
}

/// Puts the excavator to mining its last few blocks, frame built and drill down at them.
pub fn start(set: &Set) {
    let world = set.world();
    let pos = Pos::from(set.at(FINISHER.into()));
    let frame = Frame::plan(&world, pos, Dir::South);
    let Some(d) = world.component_mut(world.block_entity(pos.into()), &DIG) else {
        return;
    };
    d.frame = frame;
    d.step = frame.cells().len() as u32;
    d.set_stage(Stage::Mine);
    d.set_status(Status::NoPower);
    d.cursor = near_floor(&frame);
    d.set_head(d.cursor.floor_middle());
}

/// Lights the engines and waits for the excavator to be done.
pub fn power(set: &Set) {
    for (at, _) in engines() {
        fuel(set, at);
    }
    let pos = Pos::from(set.at(FINISHER.into()));
    *WATCHED.lock().unwrap_or_else(PoisonError::into_inner) = Some((set.player(), pos));
}

/// On top, to the east and to the west, all pointing into it.
fn engines() -> [(Pos, Dir); 3] {
    [Dir::Up, Dir::East, Dir::West].map(|side| (FINISHER.toward(side), side.opposite()))
}

/// The block `LEFT` blocks before the last one the excavator mines in `frame`, on the floor of
/// the world.
fn near_floor(frame: &Frame) -> Pos {
    let [across_x, across_z] = frame.inside();
    let mut c = frame.lo.add_x(1).add_z(1).with_y(FLOOR_Y);
    for _ in 0..across_x * across_z - LEFT {
        c = frame.next(c);
    }
    c
}

/// Lets the film go on once the excavator it waits for is done.
fn signal_when_done(w: &World) {
    let mut watched = WATCHED.lock().unwrap_or_else(PoisonError::into_inner);
    let Some((player, pos)) = *watched else {
        return;
    };
    let dig = w.component(w.block_entity(pos.into()), &DIG);
    if dig.is_some_and(|d| d.stage() == Stage::Done) {
        signal(w, player, FINISHED);
        *watched = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drill_starts_a_few_blocks_from_the_end() {
        let frame = Frame::from(([0, 0, 0], [10, 4, 10]));
        let mut c = near_floor(&frame);
        assert_eq!(c.y, FLOOR_Y, "on the floor");
        for _ in 0..LEFT {
            c = frame.next(c);
        }
        assert_eq!(c.y, FLOOR_Y - 1, "past the floor after the last ones");
    }
}
