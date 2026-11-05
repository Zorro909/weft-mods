//! The cuboid lens: held in either hand, it outlines every cube cogwork keeps loaded in the
//! player's dimension, colored by its keeper. The client asks the server now and then while it
//! is held, and draws the last answer every frame.

use std::sync::{Mutex, MutexGuard, PoisonError};

use weft_sdk::*;

use super::{Cubes, Keeper};
use crate::lib::Pos;

pub static CUBOID_LENS: Item = Item::named("cuboid_lens");

/// Client to server: which cubes are kept loaded here? The answer comes on `SHOWN`.
static ASK: Channel<()> = Channel::named("lens_ask");
static SHOWN: Channel<Vec<u8>> = Channel::named("lens_shown");

/// Seconds between asks while the lens is held.
const ASK_EVERY: f32 = 0.5;
/// Blocks each outline is drawn inside its cube, so neighbours' outlines don't fight.
const INSET: f64 = 0.05;
/// Numbers per box in an answer: its lowest and highest cube, then its keeper.
const WORDS: usize = 7;

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
fn answer(p: &Packet, _: ()) {
    let w = p.world();
    let shown = super::shown(w.dimension(), w.game_time());
    p.reply(&SHOWN, &encode(&shown));
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
        for corner in cubes.each() {
            outline(client, corner, color(*keeper));
        }
    }
}

/// The outline of the cube whose first block is `corner`.
fn outline(client: &Client, corner: Pos, color: u32) {
    let (near, far) = (INSET, super::cubes::CUBE as f64 - INSET);
    let at = |d: f64| {
        Vec3::new(
            corner.x as f64 + d,
            corner.y as f64 + d,
            corner.z as f64 + d,
        )
    };
    client.draw_box(None, at(near), at(far), color);
}

/// `0xRRGGBBAA`: cyan for machines kept running by a loader, green for everything, orange for
/// a working excavator or leveler.
fn color(keeper: Keeper) -> u32 {
    match keeper {
        Keeper::Loader => 0x3fd8_e6ff,
        Keeper::LoaderAll => 0x5ee0_5aff,
        Keeper::Machine => 0xff9a_30ff,
    }
}

fn encode(boxes: &[(Cubes, Keeper)]) -> Vec<u8> {
    let words = boxes.iter().flat_map(|(c, keeper)| {
        let (lo, hi) = (c.lo, c.hi);
        [lo.x, lo.y, lo.z, hi.x, hi.y, hi.z, *keeper as i32]
    });
    words.flat_map(i32::to_le_bytes).collect()
}

/// What `encode` wrote; boxes that don't read right are left out.
fn decode(bytes: &[u8]) -> Vec<(Cubes, Keeper)> {
    let (words, _) = bytes.as_chunks::<4>();
    let words: Vec<i32> = words.iter().map(|&b| i32::from_le_bytes(b)).collect();
    let read = |&[x0, y0, z0, x1, y1, z1, keeper]: &[i32; WORDS]| {
        let keeper = Keeper::from_repr(u32::try_from(keeper).ok()?)?;
        let cubes = Cubes {
            lo: Pos::new(x0, y0, z0),
            hi: Pos::new(x1, y1, z1),
        };
        Some((cubes, keeper))
    };
    let (boxes, _) = words.as_chunks::<WORDS>();
    boxes.iter().filter_map(read).collect()
}

fn lock() -> MutexGuard<'static, Drawn> {
    DRAWN.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

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
