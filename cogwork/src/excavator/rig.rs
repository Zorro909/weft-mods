//! The gantry players see over a working quarry: a carriage riding on two beams across the
//! frame, and a drill on a pipe of shaft pieces reaching down from it to the block being mined,
//! with the bit fitted on its end. They are entities that only show where the drill is; each one
//! leaves when its quarry is gone. Their shapes are the Blockbench models of the Gantry's pieces,
//! `assets/models/<piece>.geo.json`, and the bit turns by
//! `assets/animations/drill_bit.animation.json`. Each kind dresses them in its own textures.

use weft_sdk::{Component, EntityKind, Registry, Vec3, World, component_flags, value};

use super::bit::BITS;
use super::kind::{Drill, KINDS};
use super::{Dig, Frame, Stage};

/// A kind's rig: its pieces, each the Gantry's model in textures named like the piece. The
/// shaft's model hangs below its place, which is its top: the engine lights an entity by the
/// block just above its place, and the lowest piece's foot is down in the ground. A beam end is
/// a block of beam, to end a beam flush with the frame where a whole piece would stick out; it
/// wears the beam's texture. The bit is the one shown with no bit fitted, if any.
pub struct RigLook {
    carriage: EntityKind,
    shaft: EntityKind,
    beam: EntityKind,
    beam_end: EntityKind,
    bare_bit: Option<EntityKind>,
}

pub static GANTRY_RIG: RigLook = RigLook {
    carriage: EntityKind::named("drill_carriage"),
    shaft: EntityKind::named("drill_shaft"),
    beam: EntityKind::named("gantry_beam"),
    beam_end: EntityKind::named("gantry_beam_end"),
    bare_bit: Some(EntityKind::named("drill_bit")),
};
pub static TIMBER_RIG: RigLook = RigLook {
    carriage: EntityKind::named("timber_carriage"),
    shaft: EntityKind::named("timber_shaft"),
    beam: EntityKind::named("timber_beam"),
    beam_end: EntityKind::named("timber_beam_end"),
    bare_bit: None,
};
pub static IRON_RIG: RigLook = RigLook {
    carriage: EntityKind::named("iron_carriage"),
    shaft: EntityKind::named("iron_shaft"),
    beam: EntityKind::named("iron_beam"),
    beam_end: EntityKind::named("iron_beam_end"),
    bare_bit: None,
};
pub static STEEL_RIG: RigLook = RigLook {
    carriage: EntityKind::named("steel_carriage"),
    shaft: EntityKind::named("steel_shaft"),
    beam: EntityKind::named("steel_beam"),
    beam_end: EntityKind::named("steel_beam_end"),
    bare_bit: None,
};

/// The Gantry's pieces, whose models every kind's pieces take.
const CARRIAGE_MODEL: &str = "drill_carriage";
const SHAFT_MODEL: &str = "drill_shaft";
const BEAM_MODEL: &str = "gantry_beam";
const BEAM_END_MODEL: &str = "gantry_beam_end";
const BIT_MODEL: &str = "drill_bit";

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
    for rig in distinct(KINDS.iter().map(|kind| kind.rig)) {
        for (kind, model) in rig.pieces() {
            piece(r, kind, model, kind.path());
        }
        piece(r, &rig.beam_end, BEAM_END_MODEL, rig.beam.path());
    }
    for bit in BITS {
        piece(r, &bit.look, BIT_MODEL, bit.look.path());
    }
}

/// Each of `looks` once, in order: kinds may share a look.
fn distinct(looks: impl Iterator<Item = &'static RigLook>) -> Vec<&'static RigLook> {
    let mut seen: Vec<&'static RigLook> = Vec::new();
    for look in looks {
        if !seen.iter().any(|known| std::ptr::eq(*known, look)) {
            seen.push(look);
        }
    }
    seen
}

/// A piece of the rig, drawn by the Blockbench model `model` in the texture `texture`. Bits turn.
fn piece(r: &mut Registry, kind: &'static EntityKind, model: &str, texture: &str) {
    let builder = crate::owned::kind(r.entity(kind), claims)
        .size(0.5, 0.5)
        .texture(texture);
    if model == BIT_MODEL {
        drop(builder.animations(BIT_MODEL));
    } else {
        drop(builder);
    }
    r.set_model(kind, model);
}

impl RigLook {
    /// Its pieces with a texture named like them, and the model of each.
    fn pieces(&'static self) -> Vec<(&'static EntityKind, &'static str)> {
        let mut pieces = vec![
            (&self.carriage, CARRIAGE_MODEL),
            (&self.shaft, SHAFT_MODEL),
            (&self.beam, BEAM_MODEL),
        ];
        pieces.extend(self.bare_bit.iter().map(|bit| (bit, BIT_MODEL)));
        pieces
    }
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &RIG)
        .is_some_and(|rig| rig.holds(id))
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

/// Where each piece of `rig` goes now, with the bit `bit` on the end of its shaft, if any.
fn layout(dig: &Dig, rig: &'static RigLook, bit: Option<&'static EntityKind>) -> Vec<Piece> {
    let head = dig.head();
    let Frame { lo, hi } = dig.frame;
    let top = hi.y as f64 + 0.25;
    let carriage = Vec3 { y: top, ..head };
    // On top of the block it drills, only its tip in it; never above the carriage.
    let bit_at = Vec3 {
        y: (head.y + 1.0).min(top - BIT_HEIGHT),
        ..head
    };
    let piece = |slot, kind, at, yaw| Piece {
        slot,
        kind,
        at,
        yaw,
    };
    let mut pieces = vec![piece(CARRIAGE_SLOT, &rig.carriage, carriage, 0.0)];
    if let Some(kind) = bit {
        pieces.push(piece(BIT_SLOT, kind, bit_at, 0.0));
    }
    // Shaft pieces hang from the carriage down to the bit; the lowest reaches into the ground.
    let reach = top - (bit_at.y + BIT_HEIGHT);
    let shafts = ((reach / PIECE).ceil().max(0.0) as usize).min(SHAFTS);
    let below_carriage = |i: usize| carriage.offset(0.0, -PIECE * i as f64, 0.0);
    let shaft = |i| piece(SHAFT_SLOTS + i, &rig.shaft, below_carriage(i), 0.0);
    pieces.extend((0..shafts).map(shaft));
    let beams = carriage.offset(0.0, 0.125, 0.0);
    let (lo, hi) = (lo.center(), hi.center());
    for (i, (whole, x)) in spans(lo.x, hi.x).into_iter().enumerate() {
        let kind = if whole { &rig.beam } else { &rig.beam_end };
        pieces.push(piece(X_BEAM_SLOTS + i, kind, Vec3 { x, ..beams }, 0.0));
    }
    for (i, (whole, z)) in spans(lo.z, hi.z).into_iter().enumerate() {
        let kind = if whole { &rig.beam } else { &rig.beam_end };
        pieces.push(piece(Z_BEAM_SLOTS + i, kind, Vec3 { z, ..beams }, 90.0));
    }
    pieces
}

/// Beam pieces end to end from `a` to `b`, whole ones then ends, by whether whole and center.
/// None overlap: where two did, their faces flickered through each other.
fn spans(a: f64, b: f64) -> Vec<(bool, f64)> {
    let blocks = (b - a).round().max(0.0) as usize;
    let long = (blocks / PIECE as usize).min(LONG_BEAMS);
    let ends = (blocks - long * PIECE as usize).min(BEAMS - LONG_BEAMS);
    let after_long = a + PIECE * long as f64;
    let long = (0..long).map(|i| (true, a + PIECE * (i as f64 + 0.5)));
    let ends = (0..ends).map(|i| (false, after_long + i as f64 + 0.5));
    long.chain(ends).collect()
}

/// Puts the rig of the look `rig` where the drill is, while the quarry `owner` mines with `drill`;
/// takes it away otherwise.
pub fn update(
    world: &World,
    owner: u64,
    dig: &Dig,
    rig_look: &'static RigLook,
    drill: Option<Drill>,
) {
    let Some(mut rig) = world.component(owner, &RIG) else {
        return;
    };
    let bit = match drill.and_then(|drill| drill.bit) {
        Some(bit) => Some(&bit.look),
        None => rig_look.bare_bit.as_ref(),
    };
    let wanted = if dig.stage() == Stage::Mine {
        layout(dig, rig_look, bit)
    } else {
        Vec::new()
    };
    let mut used = [false; SLOTS];
    for piece in wanted {
        let id = &mut rig.ids[piece.slot];
        *id = crate::owned::place(world, owner, *id, piece.kind, piece.at, piece.yaw);
        used[piece.slot] = true;
    }
    for (slot, id) in rig.ids.iter_mut().enumerate() {
        if !used[slot] && *id != 0 {
            world.despawn(*id);
            *id = 0;
        }
    }
    world.set_component(owner, &RIG, rig);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An excavator mining with its drill at `head`, under a frame whose top is at 8.
    fn mining_at(head: Vec3) -> Dig {
        let mut dig = Dig {
            stage: Stage::Mine as u32,
            frame: Frame::from(([0, 0, 0], [10, 8, 10])),
            ..Dig::default()
        };
        dig.set_head(head);
        dig
    }

    /// Where the pieces of `kind` of the gantry `dig` are placed.
    fn placed(dig: &Dig, kind: &'static EntityKind) -> impl Iterator<Item = Vec3> {
        let pieces = layout(dig, &GANTRY_RIG, GANTRY_RIG.bare_bit.as_ref()).into_iter();
        pieces
            .filter(move |piece| std::ptr::eq(piece.kind, kind))
            .map(|piece| piece.at)
    }

    #[test]
    fn every_piece_has_its_model_and_texture() {
        let models = [
            CARRIAGE_MODEL,
            SHAFT_MODEL,
            BEAM_MODEL,
            BEAM_END_MODEL,
            BIT_MODEL,
        ];
        for model in models {
            let file = format!("models/{model}.geo.json");
            assert!(crate::has_asset(&file), "no {file}");
        }
        let rigs = distinct(KINDS.iter().map(|kind| kind.rig));
        let looks = rigs.into_iter().flat_map(|rig| rig.pieces());
        let bits = BITS.iter().map(|bit| (&bit.look, BIT_MODEL));
        for (kind, _) in looks.chain(bits) {
            let texture = format!("textures/entity/{}.png", kind.path());
            assert!(crate::has_asset(&texture), "no {texture}");
        }
        assert!(
            crate::has_asset("animations/drill_bit.animation.json"),
            "the bit's turning"
        );
    }

    #[test]
    fn a_look_two_kinds_share_is_registered_once() {
        let looks = [&GANTRY_RIG, &TIMBER_RIG, &GANTRY_RIG];
        let rigs = distinct(looks.into_iter());
        assert_eq!(rigs.len(), 2, "the gantry's once");
        assert!(std::ptr::eq(rigs[0], &GANTRY_RIG), "in the kinds' order");
    }

    #[test]
    fn a_rig_with_no_bit_fitted_shows_none() {
        let dig = mining_at(Vec3::new(5.5, -3.0, 5.5));
        let pieces = layout(&dig, &TIMBER_RIG, None);
        assert!(pieces.iter().all(|piece| piece.slot != BIT_SLOT), "no bit");
        let shafts = pieces
            .iter()
            .filter(|piece| std::ptr::eq(piece.kind, &TIMBER_RIG.shaft));
        assert!(shafts.count() > 0, "the shaft still reaches down");
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
    fn extents(pieces: &[(bool, f64)]) -> Vec<(f64, f64)> {
        let half = |whole| if whole { PIECE / 2.0 } else { 0.5 };
        pieces
            .iter()
            .map(|&(whole, center)| (center - half(whole), center + half(whole)))
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
        let whole: Vec<bool> = spans(0.5, 10.5).iter().map(|piece| piece.0).collect();
        assert_eq!(whole, [true, true, false, false]);
    }

    #[test]
    fn shafts_reach_from_the_carriage_down_to_the_bit() {
        let dig = mining_at(Vec3::new(5.5, -3.0, 5.5));
        // Each piece's top, where it is placed.
        let shafts: Vec<f64> = placed(&dig, &GANTRY_RIG.shaft).map(|at| at.y).collect();
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
            let dig = mining_at(Vec3::new(5.5, depth, 5.5));
            for at in placed(&dig, &GANTRY_RIG.shaft) {
                let lit_by = (at.y + 0.5).floor();
                assert!(lit_by > depth, "drilling at {depth}: piece at {}", at.y);
            }
        }
    }

    #[test]
    fn the_bit_stands_on_the_block_it_drills() {
        let dig = mining_at(Vec3::new(5.5, -3.0, 5.5));
        let bit: Vec<f64> = placed(
            &dig,
            GANTRY_RIG.bare_bit.as_ref().expect("the gantry's bit"),
        )
        .map(|at| at.y)
        .collect();
        assert_eq!(bit, [-2.0], "its foot on top of the block at -3");
    }
}
