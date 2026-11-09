//! The cuboid lens: held in either hand, it outlines every cube cogwork keeps loaded in the
//! player's dimension, colored by its keeper. The client asks the server now and then while it
//! is held, and draws the last answer every frame.

use std::sync::{Mutex, MutexGuard, PoisonError};

use weft_sdk::*;

use super::tickets::{self, Keeper};
use super::{CubePos, Cubes};

pub static CUBOID_LENS: Item = Item::named("cuboid_lens");

/// Client to server: which cubes are kept loaded here? The answer comes on `SHOWN`.
static ASK: Channel<()> = Channel::named("lens_ask");
static SHOWN: Channel<Vec<u8>> = Channel::named("lens_shown");

/// Seconds between asks while the lens is held.
const ASK_EVERY: f32 = 0.5;
/// Blocks each outline is drawn inside its cube, so neighbours' outlines don't fight.
const INSET: f64 = 0.05;

/// On the client: the boxes of the last answer, and the seconds since the last ask.
struct Drawn {
    boxes: Vec<(Cubes, Keeper)>,
    since_ask: f32,
}

static DRAWN: Mutex<Drawn> = Mutex::new(Drawn {
    boxes: Vec::new(),
    since_ask: ASK_EVERY,
});

pub fn register(r: &mut Registry) {
    r.item(&CUBOID_LENS)
        .max_stack(1)
        .handheld()
        .tab(&crate::TAB);
    r.packet(&ASK, answer);
    r.client_packet(&SHOWN, |_, bytes| lock().boxes = decode(&bytes));
    r.client_frame(frame);
}

/// Server side: the boxes kept loaded in the asking player's dimension.
fn answer(packet: &Packet, _: ()) {
    let world = packet.world();
    let shown = tickets::shown(world.dimension(), world.game_time());
    packet.reply(&SHOWN, &encode(&shown));
}

/// Client side: while the lens is held, asks now and then and outlines each cube.
fn frame(client: &Client) {
    let state = client.state();
    let lens = CUBOID_LENS.id();
    let held = [Hand::Main, Hand::Off]
        .into_iter()
        .any(|hand| client.held_item(state.player, hand) == lens);
    let mut drawn = lock();
    if !held {
        drawn.boxes.clear();
        drawn.since_ask = ASK_EVERY;
        return;
    }
    drawn.since_ask += state.dt;
    if drawn.since_ask >= ASK_EVERY {
        drawn.since_ask = 0.0;
        client.send(&ASK, &());
    }
    for (cubes, keeper) in &drawn.boxes {
        for cube in cubes.each() {
            outline(client, cube, color(*keeper));
        }
    }
}

/// The outline of `cube`.
fn outline(client: &Client, cube: CubePos, color: u32) {
    let corner = Vec3::from(cube.first_block());
    let (near, far) = (INSET, super::cubes::CUBE as f64 - INSET);
    let at = |inset: f64| corner.offset(inset, inset, inset);
    client.draw_box(None, at(near), at(far), color);
}

/// Outline colors as `0xRRGGBBAA`. Cyan: a loader keeping machines running.
const LOADER_COLOR: u32 = 0x3fd8_e6ff;
/// Green: a loader keeping everything running.
const LOADER_ALL_COLOR: u32 = 0x5ee0_5aff;
/// Orange: a working excavator or leveler.
const MACHINE_COLOR: u32 = 0xff9a_30ff;

fn color(keeper: Keeper) -> u32 {
    match keeper {
        Keeper::Loader => LOADER_COLOR,
        Keeper::LoaderAll => LOADER_ALL_COLOR,
        Keeper::Machine => MACHINE_COLOR,
    }
}

value! {
    /// One box in an answer: its cubes, and who keeps them as `Keeper`'s number.
    struct Shown {
        cubes: Cubes,
        keeper: u32,
    }
}

impl Shown {
    fn new(cubes: Cubes, keeper: Keeper) -> Shown {
        Shown {
            cubes,
            keeper: keeper as u32,
        }
    }

    /// Its box, if it read right. Its corners are put in order again, as they came over the
    /// network.
    fn read(self) -> Option<(Cubes, Keeper)> {
        let keeper = Keeper::from_repr(self.keeper)?;
        let cubes = Cubes::between(self.cubes.lo(), self.cubes.hi());
        Some((cubes, keeper))
    }
}

fn encode(boxes: &[(Cubes, Keeper)]) -> Vec<u8> {
    let shown = boxes
        .iter()
        .map(|&(cubes, keeper)| Shown::new(cubes, keeper));
    shown
        .flat_map(|shown_box| Payload::encode(&shown_box))
        .collect()
}

/// What `encode` wrote; boxes that don't read right are left out.
fn decode(bytes: &[u8]) -> Vec<(Cubes, Keeper)> {
    let (shown, _) = bytes.as_chunks::<{ size_of::<Shown>() }>();
    let shown = shown.iter().filter_map(|bytes| Shown::decode(bytes));
    shown.filter_map(Shown::read).collect()
}

fn lock() -> MutexGuard<'static, Drawn> {
    DRAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::Pos;

    #[test]
    fn an_answer_reads_as_it_was_written() {
        let boxes = vec![
            (Cubes::around(Pos::new(-40, 70, 8), 3), Keeper::LoaderAll),
            (Cubes::around(Pos::new(5, -60, 5), 0), Keeper::Machine),
        ];
        assert_eq!(decode(&encode(&boxes)), boxes, "both boxes, in order");
    }

    #[test]
    fn a_box_with_an_unknown_keeper_is_left_out() {
        let mut bytes = encode(&[(Cubes::around(Pos::new(0, 0, 0), 0), Keeper::Loader)]);
        bytes[24..28].copy_from_slice(&99i32.to_le_bytes());
        assert!(decode(&bytes).is_empty(), "keeper 99");
        assert!(decode(&[1, 2, 3]).is_empty(), "too short");
    }
}
