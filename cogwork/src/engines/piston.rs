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

/// Ticks of one stroke, up and back down.
pub const STROKE_TICKS: u32 = 18;
/// Texels the piston rises at the top of its stroke.
const LIFT: f64 = 6.0;

pub fn register(r: &mut Registry) {
    // A 16 by 4 by 16 plate centered on the pivot, which is 2 texels above the entity's feet.
    let plate = || {
        Part::new("plate", [0.0, 2.0, 0.0]).cube([-8.0, -2.0, -8.0], [16.0, 4.0, 16.0], [0.0, 0.0])
    };
    for k in 0..2 {
        let models = [
            (&FLAT[k], plate()),
            (
                &STANDING[k],
                plate().rotation(std::f32::consts::FRAC_PI_2, 0.0, 0.0),
            ),
        ];
        for (entity, part) in models {
            // Small enough to stay inside the engine, so aiming at it aims at the block.
            crate::owned::kind(r.entity(entity))
                .model(Model::from(vec![part]).uv(64, 20))
                .texture(TEXTURES[k])
                .size(0.25, 0.25);
        }
    }
}

/// Texels the piston stands above its rest at `stroke` ticks into it: up and back down.
pub fn lift(stroke: u32) -> f64 {
    let half = STROKE_TICKS / 2;
    let up = if stroke <= half {
        stroke
    } else {
        STROKE_TICKS - stroke
    };
    LIFT * up as f64 / half as f64
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
        assert_eq!(lift(STROKE_TICKS / 2), LIFT, "top halfway through");
        assert_eq!(lift(3), lift(STROKE_TICKS - 3), "down the way it came up");
        assert!(lift(1) < lift(2), "rising");
    }
}
