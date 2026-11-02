//! The gantry players see over a working excavator: a carriage riding on two beams across the
//! frame, and a drill on a pipe of shaft pieces reaching down from it to the block being mined.
//! They are entities that only show where the drill is; each one leaves when its excavator is
//! gone.

use weft_sdk::*;

use super::{Dig, stage};

static CARRIAGE: Entity = Entity::named("drill_carriage");
static BIT: Entity = Entity::named("drill_bit");
static SHAFT: Entity = Entity::named("drill_shaft");
static BEAM: Entity = Entity::named("gantry_beam");

value! {
    /// The rig's entities: carriage, bit, then shaft pieces, then beams along x and along z.
    pub struct Rig { pub ids: [u64; SLOTS] }
}
static RIG: Component<Rig> = Component::named("cogwork:rig");

const SHAFTS: usize = 40;
const BEAMS: usize = 19;
const SLOTS: usize = 2 + SHAFTS + 2 * BEAMS;
/// Blocks one shaft or beam piece is long.
const PIECE: f64 = 4.0;
/// Blocks the bit stands above where it is placed (its tip reaches a quarter below).
const BIT_HEIGHT: f64 = 0.75;

pub fn register(r: &mut Registry) {
    r.entity_component(&RIG, Rig { ids: [0; SLOTS] }, component_flags::SAVE);
    r.attach(&Entity::named("excavator"), &RIG);
    let part = |origin: [f32; 3], size: [f32; 3]| {
        Part::new("body", [0.0; 3]).cube(origin, size, [0.0, 0.0])
    };
    let pieces: [(&'static Entity, Vec<Part>, [u32; 2]); 4] = [
        (
            &CARRIAGE,
            vec![part([-7.0, 0.0, -7.0], [14.0, 8.0, 14.0])],
            [64, 32],
        ),
        (
            &BIT,
            // `bit` turns about its own axis (`animations/drill_bit.animation.json`), the tip
            // with it.
            vec![
                Part::new("bit", [0.0; 3]).cube([-4.0, 0.0, -4.0], [8.0, 12.0, 8.0], [0.0, 0.0]),
                Part::new("tip", [0.0; 3])
                    .cube([-2.0, -4.0, -2.0], [4.0, 4.0, 4.0], [32.0, 0.0])
                    .child_of(0),
            ],
            [64, 32],
        ),
        (
            &SHAFT,
            vec![part([-2.0, 0.0, -2.0], [4.0, 64.0, 4.0])],
            [32, 80],
        ),
        (
            &BEAM,
            vec![part([-32.0, 0.0, -2.0], [64.0, 4.0, 4.0])],
            [144, 16],
        ),
    ];
    for (kind, parts, uv) in pieces {
        crate::owned::kind(r.entity(kind), claims)
            .model(Model::from(parts).uv(uv[0], uv[1]))
            .size(0.5, 0.5);
    }
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

/// Where each piece goes now: by slot, its kind, place and turn.
fn layout(d: &Dig) -> Vec<(usize, &'static Entity, [f64; 3], f32)> {
    let [x, y, z] = d.head.map(|v| v as f64);
    let top = d.hi[1] as f64 + 0.25;
    // On top of the block it drills, only its tip in it; never above the carriage.
    let bit = (y + 1.0).min(top - BIT_HEIGHT);
    let mut out = vec![
        (0, &CARRIAGE, [x, top, z], 0.0),
        (1, &BIT, [x, bit, z], 0.0),
    ];
    // Shaft pieces hang from the carriage down to the bit; the lowest reaches into the ground.
    let reach = top - (bit + BIT_HEIGHT);
    let shafts = ((reach / PIECE).ceil().max(0.0) as usize).min(SHAFTS);
    out.extend((0..shafts).map(|k| (2 + k, &SHAFT, [x, top - PIECE * (k + 1) as f64, z], 0.0)));
    let beam_y = top + 0.125;
    let (x0, x1) = (d.lo[0] as f64 + 0.5, d.hi[0] as f64 + 0.5);
    let (z0, z1) = (d.lo[2] as f64 + 0.5, d.hi[2] as f64 + 0.5);
    for (k, c) in spans(x0, x1).into_iter().enumerate() {
        out.push((2 + SHAFTS + k, &BEAM, [c, beam_y, z], 0.0));
    }
    for (k, c) in spans(z0, z1).into_iter().enumerate() {
        out.push((2 + SHAFTS + BEAMS + k, &BEAM, [x, beam_y, c], 90.0));
    }
    out
}

/// Centers of beam pieces covering `a`..`b`, the last one flush with `b`.
fn spans(a: f64, b: f64) -> Vec<f64> {
    let n = (((b - a) / PIECE).ceil() as usize).clamp(1, BEAMS);
    (0..n)
        .map(|k| (a + PIECE / 2.0 + PIECE * k as f64).min(b - PIECE / 2.0))
        .collect()
}

/// Puts the rig where the drill is, while the excavator mines; takes it away otherwise.
pub fn update(w: &World, owner: u64, d: &Dig) {
    let Some(mut rig) = w.component(owner, &RIG).copied() else {
        return;
    };
    let wanted = if d.stage == stage::MINE {
        layout(d)
    } else {
        Vec::new()
    };
    let mut used = [false; SLOTS];
    for (slot, kind, pos, yaw) in wanted {
        used[slot] = true;
        rig.ids[slot] = crate::owned::place(w, owner, rig.ids[slot], kind, pos, yaw);
    }
    for (slot, id) in rig.ids.iter_mut().enumerate() {
        if !used[slot] && *id != 0 {
            w.remove(*id);
            *id = 0;
        }
    }
    if let Some(r) = w.component_mut(owner, &RIG) {
        *r = rig;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_replaced_piece_is_no_longer_the_rigs() {
        let mut rig = Rig { ids: [0; SLOTS] };
        rig.ids[1] = 7;
        assert!(rig.holds(7), "the bit it placed");
        rig.ids[1] = 9;
        assert!(!rig.holds(7), "the bit it lost and replaced");
    }

    #[test]
    fn beam_pieces_cover_the_span_and_end_flush() {
        let centers = spans(0.5, 10.5);
        assert_eq!(centers.len(), 3, "ten blocks take three pieces");
        assert_eq!(centers[0] - PIECE / 2.0, 0.5, "first starts at the start");
        assert_eq!(centers[2] + PIECE / 2.0, 10.5, "last ends at the end");
    }

    #[test]
    fn shafts_reach_from_the_carriage_down_to_the_bit() {
        let d = Dig {
            stage: stage::MINE,
            hi: [10, 8, 10],
            head: [5.5, -3.0, 5.5],
            ..Dig::default()
        };
        let shafts: Vec<f64> = layout(&d)
            .into_iter()
            .filter(|p| std::ptr::eq(p.1, &SHAFT))
            .map(|p| p.2[1])
            .collect();
        let lowest = shafts.iter().copied().fold(f64::MAX, f64::min);
        assert!(lowest <= -2.0 + BIT_HEIGHT, "down to the top of the bit");
        assert!(
            shafts.iter().all(|y| y + PIECE <= 8.25 + 1e-9),
            "none above the carriage"
        );
    }

    #[test]
    fn the_bit_stands_on_the_block_it_drills() {
        let d = Dig {
            stage: stage::MINE,
            hi: [10, 8, 10],
            head: [5.5, -3.0, 5.5],
            ..Dig::default()
        };
        let bit = layout(&d)
            .into_iter()
            .find(|p| std::ptr::eq(p.1, &BIT))
            .map(|p| p.2[1]);
        assert_eq!(bit, Some(-2.0), "its foot on top of the block at -3");
    }
}
