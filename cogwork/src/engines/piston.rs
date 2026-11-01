//! An engine's piston: a plate riding up and down its trunk while it runs. It is an owned
//! entity, so the stroke moves it smoothly without the block changing state.

use weft_sdk::*;

use super::{Engine, kind, model};

/// The pistons lying flat (engines facing up or down), then standing (facing sideways), by
/// engine kind.
static FLAT: [Entity; 2] = [
    Entity::named("spring_piston"),
    Entity::named("ember_piston"),
];
static STANDING: [Entity; 2] = [
    Entity::named("spring_piston_standing"),
    Entity::named("ember_piston_standing"),
];
const TEXTURES: [&str; 2] = ["spring_piston", "ember_piston"];

/// Ticks the piston takes to rise or to fall, and to rest at each end: the client eases it
/// toward where it is, so without the rest it would turn back short of the ends.
const TRAVEL: u32 = 6;
const REST: u32 = 3;
/// Ticks of one stroke, up and back down.
pub const STROKE_TICKS: u32 = 2 * (TRAVEL + REST);
/// Texels the piston rises: from on the base plate (4..8) to flush with the trunk's end (12..16).
const LIFT: f64 = 8.0;

pub fn register(r: &mut Registry) {
    for k in 0..2 {
        let models = [(&FLAT[k], plate()), (&STANDING[k], standing_plate())];
        for (entity, part) in models {
            // Small enough to stay inside the engine, so aiming at it aims at the block.
            crate::owned::kind(r.entity(entity))
                .model(Model::from(vec![part]).uv(64, 20))
                .texture(TEXTURES[k])
                .size(0.25, 0.25);
        }
    }
}

/// A 16 by 4 by 16 plate centered on the pivot, which is 2 texels above the entity's feet.
fn plate() -> Part {
    Part::new("plate", [0.0, 2.0, 0.0]).cube([-8.0, -2.0, -8.0], [16.0, 4.0, 16.0], [0.0, 0.0])
}

/// The plate stood on its edge, a quarter turn about x: thin along z.
fn standing_plate() -> Part {
    plate().rotation(90.0, 0.0, 0.0)
}

/// Texels the piston stands above its rest at `stroke` ticks into it: up, a rest, back down, a
/// rest.
pub fn lift(stroke: u32) -> f64 {
    let s = stroke % STROKE_TICKS;
    let up = if s < TRAVEL {
        s
    } else if s < TRAVEL + REST {
        TRAVEL
    } else if s < 2 * TRAVEL + REST {
        2 * TRAVEL + REST - s
    } else {
        0
    };
    LIFT * up as f64 / TRAVEL as f64
}

/// Puts the engine `id`'s piston where its stroke has it.
pub fn show(w: &World, id: u64, pos: [i32; 3], e: &mut Engine) {
    let Some(out) = model::output(w, pos) else {
        return;
    };
    let k = (e.kind == kind::EMBER) as usize;
    let (entity, yaw) = match out {
        Dir::Up | Dir::Down => (&FLAT[k], 0.0),
        Dir::North | Dir::South => (&STANDING[k], 0.0),
        _ => (&STANDING[k], 90.0),
    };
    // At rest the piston's middle is 2 texels from the block's toward the base plate; the
    // stroke lifts it toward the trunk's end.
    let along = (lift(e.stroke) - 2.0) / 16.0;
    let o = out.offset();
    let at: [f64; 3] = std::array::from_fn(|i| pos[i] as f64 + 0.5 + along * o[i] as f64);
    let feet = [at[0], at[1] - 2.0 / 16.0, at[2]];
    e.piston = crate::owned::place(w, id, e.piston, entity, feet, yaw);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_piston_rises_and_falls_once_a_stroke() {
        assert_eq!(lift(0), 0.0, "starts at rest");
        assert!(lift(1) < lift(2), "rising");
        let lifts: Vec<f64> = (0..STROKE_TICKS).map(lift).collect();
        let max = lifts.iter().copied().fold(0.0, f64::max);
        assert_eq!(
            4.0 + max + 4.0,
            16.0,
            "the plate and piston reach the trunk's end"
        );
        let at_top = lifts.iter().filter(|&&l| l == max).count();
        let at_bottom = lifts.iter().filter(|&&l| l == 0.0).count();
        assert!(at_top > 2 && at_bottom > 2, "rests at both ends: {lifts:?}");
    }

    #[test]
    fn the_standing_plate_is_turned_a_quarter() {
        let rest = standing_plate().rest;
        assert!(
            (rest[0] - std::f32::consts::FRAC_PI_2).abs() < 1e-6,
            "a quarter turn about x, got {rest:?} radians"
        );
    }
}
