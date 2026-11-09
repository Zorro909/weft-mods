//! The gantry players see over a working excavator: a carriage riding on two beams across the
//! frame, and a drill on a pipe of shaft pieces reaching down from it to the block being mined.
//! They are entities that only show where the drill is; each one leaves when its excavator is
//! gone.

use weft_sdk::*;

use super::{Dig, Frame, Stage};
use crate::lib::ModelBox;

static CARRIAGE: EntityKind = EntityKind::named("drill_carriage");
static BIT: EntityKind = EntityKind::named("drill_bit");
static SHAFT: EntityKind = EntityKind::named("drill_shaft");
static BEAM: EntityKind = EntityKind::named("gantry_beam");
/// A block of beam, to end a beam flush with the frame where a whole piece would stick out.
static BEAM_END: EntityKind = EntityKind::named("gantry_beam_end");

value! {
    /// The rig's entities: carriage, bit, then shaft pieces, then beams along x and along z.
    pub struct Rig { pub ids: [u64; SLOTS] }
}
pub static RIG: Component<Rig> = Component::named("cogwork:rig");

const SHAFTS: usize = 40;
/// Whole beam pieces on each axis: enough for the 64 blocks survey markers reach.
const LONG_BEAMS: usize = 16;
/// Beam pieces on each axis, the whole ones then up to three ends.
const BEAMS: usize = LONG_BEAMS + 3;
/// Where in `Rig::ids` each run of pieces starts, and how many there are in all.
const CARRIAGE_SLOT: usize = 0;
const BIT_SLOT: usize = 1;
const SHAFT_SLOTS: usize = 2;
const X_BEAM_SLOTS: usize = SHAFT_SLOTS + SHAFTS;
const Z_BEAM_SLOTS: usize = X_BEAM_SLOTS + BEAMS;
const SLOTS: usize = Z_BEAM_SLOTS + BEAMS;
/// Blocks one shaft or beam piece is long.
const PIECE: f64 = 4.0;
/// Blocks the bit stands above where it is placed (its tip reaches a quarter below).
const BIT_HEIGHT: f64 = 0.75;

pub fn register(r: &mut Registry) {
    r.entity_component(&RIG, Rig { ids: [0; SLOTS] }, component_flags::SAVE);
    let body = |lo, size| vec![ModelBox::sized(lo, size).part("body", [0.0, 0.0])];
    let carriage = body([-7.0, 0.0, -7.0], [14.0, 8.0, 14.0]);
    piece(r, &CARRIAGE, carriage, [64, 32]);
    piece(r, &BIT, bit_parts(), [64, 32]);
    // Hangs below its place, which is its top: the engine lights an entity by the block just
    // above its place, and the lowest piece's foot is down in the ground.
    let shaft = body([-2.0, -64.0, -2.0], [4.0, 64.0, 4.0]);
    piece(r, &SHAFT, shaft, [32, 80]);
    let beam = body([-32.0, 0.0, -2.0], [64.0, 4.0, 4.0]);
    piece(r, &BEAM, beam, [144, 16]);
    // A short piece of the same beam, so it wears the beam's skin.
    let end = body([-8.0, 0.0, -2.0], [16.0, 4.0, 4.0]);
    piece(r, &BEAM_END, end, [144, 16]).texture("gantry_beam");
}

/// The drill's bit and its tip. `bit` turns about its own axis
/// (`animations/drill_bit.animation.json`), the tip with it.
fn bit_parts() -> Vec<ModelPart> {
    let bit = ModelBox::sized([-4.0, 0.0, -4.0], [8.0, 12.0, 8.0]);
    let tip = ModelBox::sized([-2.0, -4.0, -2.0], [4.0, 4.0, 4.0]);
    vec![
        bit.part("bit", [0.0, 0.0]),
        tip.part("tip", [32.0, 0.0]).child_of(0),
    ]
}

fn piece<'a>(
    r: &'a mut Registry,
    kind: &'static EntityKind,
    parts: Vec<ModelPart>,
    uv: [u32; 2],
) -> EntityBuilder<'a> {
    crate::owned::kind(r.entity(kind), claims)
        .model(Model::from(parts).uv_size(uv[0], uv[1]))
        .size(0.5, 0.5)
}

fn claims(w: &World, owner: u64, id: u64) -> bool {
    w.component(owner, &RIG).is_some_and(|rig| rig.holds(id))
}

impl Rig {
    /// Whether `id` is one of the rig's pieces now. One it lost track of while its cube was
    /// unloaded was replaced, and is not.
    fn holds(&self, id: u64) -> bool {
        self.ids.contains(&id)
    }
}

/// One piece of the rig as it should stand now: its slot in `Rig::ids`, its kind, where it is
/// placed and its turn in degrees.
#[derive(Clone, Copy)]
struct Piece {
    slot: usize,
    kind: &'static EntityKind,
    at: Vec3,
    yaw: f32,
}

/// Where each piece goes now.
fn layout(d: &Dig) -> Vec<Piece> {
    let head = d.head();
    let Frame { lo, hi } = d.frame;
    let top = hi.y as f64 + 0.25;
    let carriage = Vec3 { y: top, ..head };
    // On top of the block it drills, only its tip in it; never above the carriage.
    let bit = Vec3 {
        y: (head.y + 1.0).min(top - BIT_HEIGHT),
        ..head
    };
    let piece = |slot, kind, at, yaw| Piece {
        slot,
        kind,
        at,
        yaw,
    };
    let mut pieces = vec![
        piece(CARRIAGE_SLOT, &CARRIAGE, carriage, 0.0),
        piece(BIT_SLOT, &BIT, bit, 0.0),
    ];
    // Shaft pieces hang from the carriage down to the bit; the lowest reaches into the ground.
    let reach = top - (bit.y + BIT_HEIGHT);
    let shafts = ((reach / PIECE).ceil().max(0.0) as usize).min(SHAFTS);
    let below_carriage = |k: usize| carriage.offset(0.0, -PIECE * k as f64, 0.0);
    pieces.extend((0..shafts).map(|k| piece(SHAFT_SLOTS + k, &SHAFT, below_carriage(k), 0.0)));
    let beams = carriage.offset(0.0, 0.125, 0.0);
    let (lo, hi) = (lo.center(), hi.center());
    for (k, (kind, x)) in spans(lo.x, hi.x).into_iter().enumerate() {
        pieces.push(piece(X_BEAM_SLOTS + k, kind, Vec3 { x, ..beams }, 0.0));
    }
    for (k, (kind, z)) in spans(lo.z, hi.z).into_iter().enumerate() {
        pieces.push(piece(Z_BEAM_SLOTS + k, kind, Vec3 { z, ..beams }, 90.0));
    }
    pieces
}

/// Beam pieces end to end from `a` to `b`, whole ones then ends, by kind and center. None
/// overlap: where two did, their faces flickered through each other.
fn spans(a: f64, b: f64) -> Vec<(&'static EntityKind, f64)> {
    let blocks = (b - a).round().max(0.0) as usize;
    let long = (blocks / PIECE as usize).min(LONG_BEAMS);
    let ends = (blocks - long * PIECE as usize).min(BEAMS - LONG_BEAMS);
    let after_long = a + PIECE * long as f64;
    let long = (0..long).map(|k| (&BEAM, a + PIECE * (k as f64 + 0.5)));
    let ends = (0..ends).map(|k| (&BEAM_END, after_long + k as f64 + 0.5));
    long.chain(ends).collect()
}

/// Puts the rig where the drill is, while the excavator mines; takes it away otherwise.
pub fn update(w: &World, owner: u64, d: &Dig) {
    let Some(mut rig) = w.component(owner, &RIG) else {
        return;
    };
    let wanted = if d.stage() == Stage::Mine {
        layout(d)
    } else {
        Vec::new()
    };
    let mut used = [false; SLOTS];
    for piece in wanted {
        let id = &mut rig.ids[piece.slot];
        *id = crate::owned::place(w, owner, *id, piece.kind, piece.at, piece.yaw);
        used[piece.slot] = true;
    }
    for (slot, id) in rig.ids.iter_mut().enumerate() {
        if !used[slot] && *id != 0 {
            w.despawn(*id);
            *id = 0;
        }
    }
    w.set_component(owner, &RIG, rig);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An excavator mining with its drill at `head`, under a frame whose top is at 8.
    fn mining_at(head: Vec3) -> Dig {
        let mut d = Dig {
            stage: Stage::Mine as u32,
            frame: Frame::from(([0, 0, 0], [10, 8, 10])),
            ..Dig::default()
        };
        d.set_head(head);
        d
    }

    /// Where the pieces of `kind` of the excavator `d` are placed.
    fn placed(d: &Dig, kind: &'static EntityKind) -> impl Iterator<Item = Vec3> {
        let pieces = layout(d).into_iter();
        pieces
            .filter(move |piece| std::ptr::eq(piece.kind, kind))
            .map(|piece| piece.at)
    }

    #[test]
    fn a_replaced_piece_is_no_longer_the_rigs() {
        let mut rig = Rig { ids: [0; SLOTS] };
        rig.ids[1] = 7;
        assert!(rig.holds(7), "the bit it placed");
        rig.ids[1] = 9;
        assert!(!rig.holds(7), "the bit it lost and replaced");
    }

    /// Where each piece starts and ends along its beam.
    fn extents(pieces: &[(&'static EntityKind, f64)]) -> Vec<(f64, f64)> {
        let half = |kind| {
            if std::ptr::eq(kind, &BEAM) {
                PIECE / 2.0
            } else {
                0.5
            }
        };
        pieces
            .iter()
            .map(|&(kind, c)| (c - half(kind), c + half(kind)))
            .collect()
    }

    #[test]
    fn beam_pieces_cover_the_span_end_to_end() {
        for blocks in [1, 3, 4, 10, 15, 64] {
            let pieces = extents(&spans(0.5, 0.5 + blocks as f64));
            assert_eq!(
                pieces[0].0, 0.5,
                "{blocks} blocks: first starts at the start"
            );
            for pair in pieces.windows(2) {
                assert_eq!(pair[0].1, pair[1].0, "{blocks} blocks: no overlap, no gap");
            }
            let last = pieces[pieces.len() - 1].1;
            assert_eq!(
                last,
                0.5 + blocks as f64,
                "{blocks} blocks: last ends at the end"
            );
        }
    }

    #[test]
    fn ten_blocks_take_two_pieces_and_two_ends() {
        let whole: Vec<bool> = spans(0.5, 10.5)
            .iter()
            .map(|p| std::ptr::eq(p.0, &BEAM))
            .collect();
        assert_eq!(whole, [true, true, false, false]);
    }

    #[test]
    fn shafts_reach_from_the_carriage_down_to_the_bit() {
        let d = mining_at(Vec3::new(5.5, -3.0, 5.5));
        // Each piece's top, where it is placed.
        let shafts: Vec<f64> = placed(&d, &SHAFT).map(|at| at.y).collect();
        let lowest = shafts.iter().copied().fold(f64::MAX, f64::min);
        assert!(
            lowest - PIECE <= -2.0 + BIT_HEIGHT,
            "down to the top of the bit"
        );
        assert!(
            shafts.iter().all(|&y| y <= 8.25 + 1e-9),
            "none above the carriage"
        );
    }

    #[test]
    fn every_shaft_piece_takes_its_light_from_the_air() {
        // An entity is lit by the block half a block above its place; the drilled block is the
        // highest solid one under the bit.
        for depth in [-3.0, -3.5, -5.0, -6.25] {
            let d = mining_at(Vec3::new(5.5, depth, 5.5));
            for at in placed(&d, &SHAFT) {
                let lit_by = (at.y + 0.5).floor();
                assert!(lit_by > depth, "drilling at {depth}: piece at {}", at.y);
            }
        }
    }

    #[test]
    fn the_bit_stands_on_the_block_it_drills() {
        let d = mining_at(Vec3::new(5.5, -3.0, 5.5));
        let bit: Vec<f64> = placed(&d, &BIT).map(|at| at.y).collect();
        assert_eq!(bit, [-2.0], "its foot on top of the block at -3");
    }
}
