//! The spring and ember engines' piston plate rides up and down the trunk. The block's
//! orientation turns the plate and trunk together. The plate is transparent in the middle
//! so its top and bottom faces don't flicker where they meet the trunk.

use weft_sdk::{Holder, World, comps::BLOCK_ANIMATION_PROGRESS};

pub const SPRING_MODEL: &str = "block/spring_piston";
pub const EMBER_MODEL: &str = "block/ember_piston";
/// The stroke clip plays in either direction, with the plate at rest at both ends.
pub const CLIP: &str = "stroke";
/// Ticks the piston takes to rise or fall, and to rest at each end.
const TRAVEL: u32 = 6;
const REST: u32 = 3;
/// Ticks of one stroke including both rests.
const STROKE_TICKS: u32 = 2 * (TRAVEL + REST);

/// Ticks into the stroke all pistons keep time with, at the world's `time`.
fn shared(time: u64) -> u32 {
    (time % STROKE_TICKS as u64) as u32
}

/// Whether a stroke begins at `time`.
fn begins(time: u64) -> bool {
    shared(time) == 0
}

/// Whether a piston making a stroke has just reached the top at `time`: once a stroke.
pub fn at_top(time: u64) -> bool {
    shared(time) == TRAVEL
}

/// Whether the piston makes a stroke at `time`, having made one the tick before or not (`was`).
/// A stroke begins with every shared one while the engine runs, so all pistons rise and fall
/// together, and one begun goes on to its end: a stopped engine finishes its stroke and rests.
pub fn stroking(was: bool, running: bool, time: u64) -> bool {
    if begins(time) { running } else { was }
}

/// Starts a stroke by switching to the other end of the clip. The clip takes 15 ticks
/// in either direction, leaving 3 ticks at rest before the next stroke can start.
pub fn show(world: &World, owner: Holder, stroking: bool, time: u64) {
    if stroking && begins(time) {
        let target = world
            .component(owner, &BLOCK_ANIMATION_PROGRESS)
            .unwrap_or_default();
        world.set_component(owner, &BLOCK_ANIMATION_PROGRESS, 1.0 - target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stroke_begins_only_with_the_shared_one() {
        let start = 5 * STROKE_TICKS as u64;
        assert!(stroking(false, true, start), "running at its start");
        assert!(!stroking(false, true, start + 1), "started late: waits");
        assert!(!stroking(false, false, start), "not running");
        let begun: Vec<u64> = (start..start + 3 * STROKE_TICKS as u64)
            .filter(|&time| begins(time))
            .collect();
        assert_eq!(begun.len(), 3, "once a stroke: {begun:?}");
    }

    #[test]
    fn a_stopped_engine_finishes_its_stroke_and_rests() {
        let start = 2 * STROKE_TICKS as u64;
        let mut stroke = stroking(false, true, start);
        for time in start + 1..start + STROKE_TICKS as u64 {
            stroke = stroking(stroke, false, time);
            assert!(stroke, "still on its way at {time}");
        }
        stroke = stroking(stroke, false, start + STROKE_TICKS as u64);
        assert!(!stroke, "at rest when the next would begin");
    }

    #[test]
    fn the_piston_reaches_the_top_once_a_stroke() {
        let tops: Vec<u64> = (0..2 * STROKE_TICKS as u64)
            .filter(|&time| at_top(time))
            .collect();
        assert_eq!(tops.len(), 2, "once in each of two strokes: {tops:?}");
        assert_eq!(tops[0], TRAVEL as u64, "after rising");
    }

    /// The animated model paths that the two engine blocks register.
    const MODELS: [&str; 2] = [SPRING_MODEL, EMBER_MODEL];

    #[test]
    fn both_directions_through_the_clip_make_the_same_stroke() {
        for model in MODELS {
            let path = assets().join(format!("animations/{model}.animation.json"));
            let json = std::fs::read_to_string(&path).expect("the piston animation exists");
            let clip: serde_json::Value =
                serde_json::from_str(&json).expect("the piston animation is JSON");
            let stroke = &clip["animations"][format!("animation.piston.{CLIP}")];
            let duration = STROKE_TICKS - REST;
            assert_eq!(
                stroke["animation_length"],
                seconds(duration),
                "{model}: duration"
            );
            let positions = &stroke["bones"]["plate"]["position"];
            for (tick, height) in [(0, 0), (TRAVEL, 8), (TRAVEL + REST, 8), (duration, 0)] {
                let time = seconds(tick).to_string();
                let time = if tick == 0 { "0.0" } else { &time };
                assert_eq!(
                    positions[time],
                    serde_json::json!([0, height, 0]),
                    "{model}: the piston is at height {height} at {time}s"
                );
            }
            assert_eq!(
                positions.as_object().unwrap().len(),
                4,
                "{model}: four keys"
            );
        }
    }

    fn seconds(ticks: u32) -> f64 {
        f64::from(ticks) / 20.0
    }

    fn assets() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    /// The texels of the texture `name`, in rows of (red, green, blue, alpha).
    fn texels(name: &str) -> Vec<Vec<[u8; 4]>> {
        let path = assets().join(format!("textures/{name}.png"));
        let file = std::fs::File::open(&path).expect("the texture did not open");
        let decoder = png::Decoder::new(std::io::BufReader::new(file));
        let mut reader = decoder.read_info().expect("the texture is not a PNG");
        let size = reader
            .output_buffer_size()
            .expect("the texture is too large");
        let mut bytes = vec![0; size];
        let frame = reader
            .next_frame(&mut bytes)
            .expect("the texture did not decode");
        assert_eq!(frame.color_type, png::ColorType::Rgba, "{name}: with alpha");
        let row = |texels: &[u8]| texels.chunks(4).map(|t| [t[0], t[1], t[2], t[3]]).collect();
        bytes[..frame.buffer_size()]
            .chunks(frame.line_size)
            .map(row)
            .collect()
    }

    #[test]
    fn the_plate_is_clear_only_where_the_trunk_goes_through() {
        // Where the model's unwrap puts the 16 by 16 top and bottom, and the trunk's 8 by 8 in
        // them.
        let faces = [16, 32];
        let trunk = 4..12;
        for model in MODELS {
            let texels = texels(model);
            for (left, x, y) in faces
                .iter()
                .flat_map(|&left| (0..16).flat_map(move |x| (0..16).map(move |y| (left, x, y))))
            {
                let clear = texels[y][left + x][3] == 0;
                let in_trunk = trunk.contains(&x) && trunk.contains(&y);
                assert_eq!(clear, in_trunk, "{model} at {x}, {y} of the face at {left}");
            }
        }
    }

    #[test]
    fn the_resting_plate_sits_four_texels_above_the_block_floor() {
        for model in MODELS {
            let path = assets().join(format!("models/{model}.geo.json"));
            let json = std::fs::read_to_string(&path).expect("the piston model exists");
            let model: serde_json::Value =
                serde_json::from_str(&json).expect("the piston model is JSON");
            let plate = &model["minecraft:geometry"][0]["bones"][0]["cubes"][0];
            assert_eq!(
                plate["origin"],
                serde_json::json!([-8, 4, -8]),
                "sits on the base"
            );
            assert_eq!(
                plate["size"],
                serde_json::json!([16, 4, 16]),
                "fits its block"
            );
        }
    }
}
