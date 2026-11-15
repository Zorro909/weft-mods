//! The haze round a running oil engine: warm exhaust in the air, and the world's sounds muffled
//! but for the engine's low beat, close to it. It is a source of the engine's rig, the entity on
//! its top: the controller's own entity is a block's, which no client hears of, so a zone of its
//! would reach nobody. The haze goes with the rig when the engine is taken apart or broken.

use weft_sdk::{Environment, World};

use super::OilEngine;

/// The source's name on the rig.
const HAZE: &str = "oil_engine_haze";
/// Blocks from the rig: the haze is whole within the first, thins out to none at the second.
const WHOLE_WITHIN: f32 = 8.0;
const GONE_BEYOND: f32 = 20.0;
/// The exhaust's warm brown-grey, a mid tone so it neither glares by day nor lights the night.
const COLOR: u32 = 0x847866;
/// Blocks from the eye where the haze begins, and where it hides everything: what stands round
/// the engine stays clear, a lake 80 blocks off shows through it, and far hills fade into it.
const FOG_START: f32 = 24.0;
const FOG_END: f32 = 384.0;
/// Hz above which sounds are cut near the engine: its beat comes through, the rest is dulled,
/// less than under water (base cuts there at 900).
const MUFFLE_ABOVE: f32 = 2000.0;
/// Real seconds the haze takes to gather as the engine starts, and to clear once it stops.
const GATHERS: f32 = 2.0;
const CLEARS: f32 = 4.0;
/// Leaves a sound setting as it is: reverb and volume.
const UNCHANGED: f32 = f32::NAN;

/// Hazes the air round `engine`'s rig while it runs; lets it clear once it stops.
pub fn show(world: &World, engine: &OilEngine) {
    if let Some(haze) = haze_of(engine) {
        world.environment(engine.rig, &haze);
    } else {
        world.clear_environment(engine.rig, HAZE);
    }
}

/// The haze round `engine`: there while its rig makes strokes, as it does running.
fn haze_of(engine: &OilEngine) -> Option<Environment> {
    engine.is_stroking().then(haze)
}

fn haze() -> Environment {
    Environment::new(HAZE)
        .radius(WHOLE_WITHIN, GONE_BEYOND)
        .fog(COLOR, FOG_START, FOG_END)
        .acoustics(MUFFLE_ABOVE, UNCHANGED, UNCHANGED)
        .fade(GATHERS, CLEARS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blocks from the rig, on the engine's top, to the farthest corner of its 3 by 3 by 3 cube.
    const ENGINE_REACH: f32 = 3.4;

    #[test]
    fn only_an_engine_at_work_hazes_the_air() {
        let stroking = OilEngine {
            stroke: 1,
            ..OilEngine::default()
        };
        assert!(haze_of(&stroking).is_some(), "while its rig strokes");
        let stopped = OilEngine::default();
        assert!(haze_of(&stopped).is_none(), "clear once it stops");
    }

    #[test]
    fn the_haze_blinds_no_one() {
        let values = haze().values;
        assert!(values.fog_start >= 16.0, "the land round it stays clear");
        assert!(values.fog_end >= 256.0, "the distance shows through it");
        assert!(values.lowpass > 900.0, "less muffled than under water");
    }

    #[test]
    fn the_haze_changes_fog_and_sound_alone() {
        let values = haze().values;
        assert!(values.reverb.is_nan(), "a cave's echo stays");
        assert!(values.volume.is_nan(), "as loud as before");
        assert!(values.sunlight.is_nan(), "as light as before");
    }

    #[test]
    fn the_haze_thins_out_round_the_engine() {
        let haze = haze();
        assert!(haze.inner_radius > ENGINE_REACH, "whole beside it");
        assert!(haze.radius > haze.inner_radius, "thinning, not a wall");
    }
}
