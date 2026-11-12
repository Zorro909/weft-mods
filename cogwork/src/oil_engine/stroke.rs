//! The rig's stroke, worked out here and written as its clip,
//! `assets/animations/oil_engine_rig.animation.json` (`generated`): the crank turns once, and
//! each piston rides as high as its rod lets it, the rod leaning to keep its top on its crank
//! pin. The pins are half a turn apart, so one piston is down while the other is up. All
//! lengths are in model texels, as `assets/models/oil_engine_rig.geo.json` has them.

use serde_json::{Value, json};

use super::rig::STROKE_TICKS;

/// The crankshaft's height above the rig's feet.
const AXLE: f64 = 18.0;
/// How far a crank pin lies from the shaft.
const THROW: f64 = 2.0;
/// A connecting rod's length, from its crosshead to its pin.
const ROD: f64 = 6.0;

/// A cylinder's bones, and whether its pin starts above the shaft (or below it).
struct Cylinder {
    piston: &'static str,
    rod: &'static str,
    pin_above: bool,
}

const CYLINDERS: [Cylinder; 2] = [
    Cylinder {
        piston: "piston_a",
        rod: "rod_a",
        pin_above: false,
    },
    Cylinder {
        piston: "piston_b",
        rod: "rod_b",
        pin_above: true,
    },
];

impl Cylinder {
    /// 1 when the pin starts above the shaft, -1 below.
    fn side(&self) -> f64 {
        if self.pin_above { 1.0 } else { -1.0 }
    }

    /// The crosshead's height with the crank turned `turn` degrees: under the pin by as much
    /// of the rod as the pin's lean leaves upright.
    fn crosshead(&self, turn: f64) -> f64 {
        let (lean, up) = turn.to_radians().sin_cos();
        let across = THROW * lean;
        AXLE + self.side() * THROW * up - (ROD * ROD - across * across).sqrt()
    }

    /// How far the rod leans, in degrees about the same axis as the crank's turn, to reach its
    /// pin.
    fn rod_lean(&self, turn: f64) -> f64 {
        self.side() * (THROW * turn.to_radians().sin() / ROD).asin().to_degrees()
    }
}

/// The crank's turn at tick `tick` of the stroke, in degrees: a whole turn backward about x,
/// so the flywheel's top goes toward the rig's front.
fn turn_at(tick: u32) -> f64 {
    -360.0 * tick as f64 / STROKE_TICKS as f64
}

/// The clip: a key every tick for each moving bone.
fn clip() -> Value {
    let mut bones = serde_json::Map::new();
    let keys = |value: &dyn Fn(f64) -> [f64; 3]| {
        let each = (0..=STROKE_TICKS).map(|tick| (time(tick), json!(value(turn_at(tick)))));
        Value::Object(each.collect())
    };
    for wheel in ["crank", "flywheel"] {
        bones.insert(
            wheel.into(),
            json!({ "rotation": keys(&|turn: f64| [rounded(turn), 0.0, 0.0]) }),
        );
    }
    for cylinder in &CYLINDERS {
        let rest = cylinder.crosshead(0.0);
        let rise = |turn: f64| [0.0, rounded(cylinder.crosshead(turn) - rest), 0.0];
        let lean = |turn: f64| [rounded(cylinder.rod_lean(turn)), 0.0, 0.0];
        bones.insert(cylinder.piston.into(), json!({ "position": keys(&rise) }));
        bones.insert(cylinder.rod.into(), json!({ "rotation": keys(&lean) }));
    }
    let length = STROKE_TICKS as f64 / 20.0;
    json!({
        "format_version": "1.8.0",
        "animations": {
            "animation.oil_engine_rig.turn": { "animation_length": length, "bones": bones }
        }
    })
}

/// A key's time, seconds at a tick, as a key of the file.
fn time(tick: u32) -> String {
    format!("{:.2}", tick as f64 / 20.0)
}

/// `value` to a thousandth, without a minus on nothing.
fn rounded(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0 + 0.0
}

#[test]
fn the_clip_is_what_is_worked_out_here() {
    let text = serde_json::to_string_pretty(&clip()).expect("a clip as text") + "\n";
    let path = crate::generated::asset("animations/oil_engine_rig.animation.json");
    crate::generated::save_or_check(&path, text.as_bytes());
}

#[test]
fn a_rod_reaches_its_pin_at_every_turn() {
    for cylinder in &CYLINDERS {
        for tick in 0..STROKE_TICKS {
            let turn = turn_at(tick).to_radians();
            // The pin, turned with the crank: from straight above or below the shaft.
            let pin = (
                AXLE + cylinder.side() * THROW * turn.cos(),
                cylinder.side() * THROW * turn.sin(),
            );
            let lean = cylinder.rod_lean(turn_at(tick)).to_radians();
            let top = (
                cylinder.crosshead(turn_at(tick)) + ROD * lean.cos(),
                ROD * lean.sin(),
            );
            let gap = ((pin.0 - top.0).powi(2) + (pin.1 - top.1).powi(2)).sqrt();
            assert!(gap < 1e-9, "{} at tick {tick}: {gap} off", cylinder.rod);
        }
    }
}

#[test]
fn the_model_hangs_its_rods_where_the_crosshead_rests() {
    let model = include_str!("../../assets/models/oil_engine_rig.geo.json");
    assert!(
        model.contains(&format!("\"pivot\": [0, {AXLE}, 0]")),
        "the shaft"
    );
    for (cylinder, across) in CYLINDERS.iter().zip([-7, 7]) {
        let rest = cylinder.crosshead(0.0);
        let pivot = format!("\"pivot\": [{across}, {rest}, 0]");
        assert!(model.contains(&pivot), "{} hangs at {pivot}", cylinder.rod);
    }
}
