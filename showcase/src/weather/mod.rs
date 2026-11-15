//! A film's weather in the game: each part of it (`fade`) is a local environment source of its
//! own, put on every frame at the weight film time gives it, with no fade of the engine's, which
//! would run in real seconds. The film's end lifts it at once, as it does screen effects.

mod fade;

pub use fade::{Layer, Part, WeatherFade};
use weft_sdk::{Client, Environment, Vec3};

/// Where film weather applies among the environment's sources: after the world's weather (-900)
/// and a mod's own sources (0 unless they say), so the film's look wins. Its parts follow on
/// in `Part`'s order.
const PRIORITY: i32 = 500;
/// Blocks from the centre of the zone that gives a part less than its full weight to its edge.
const ZONE: f64 = 1.0e6;

/// Shows `layers`, the film's weather now, to a camera at `eye`; the world's own where it is
/// left out.
pub fn show(c: &Client, layers: &[Layer], eye: Vec3) {
    for part in Part::ALL {
        if let Some(layer) = layers.iter().find(|layer| layer.part == part) {
            c.environment(&source(layer, eye));
        } else {
            c.clear_environment(&name(part));
        }
    }
}

/// Lifts the film's weather, as its end does.
pub fn clear(c: &Client) {
    show(c, &[], Vec3::ZERO);
}

fn name(part: Part) -> String {
    let part = match part {
        Part::Under => "under",
        Part::Going => "going",
        Part::Coming => "coming",
    };
    format!("film_weather_{part}")
}

/// The environment source for `layer`, seen from `eye`.
fn source(layer: &Layer, eye: Vec3) -> Environment {
    let weather = &layer.weather;
    let mut source = Environment::new(&name(layer.part)).priority(PRIORITY + layer.part as i32);
    if layer.weight < 1.0 {
        source = weighed(source, layer.weight, eye);
    }
    if let Some(fog) = weather.fog {
        source = source.fog(fog.color, fog.start, fog.end);
    }
    if let Some(color) = weather.sky {
        source = source.sky_color(color);
    }
    if let Some(strength) = weather.sunlight {
        source = source.sunlight(strength);
    }
    if let Some(color) = weather.cloud_color {
        source = source.cloud_color(color);
    }
    if let Some(cover) = weather.clouds {
        source.values.cloud_opacity = cover;
    }
    if let Some(sound) = weather.acoustics {
        source = source.acoustics(sound.cutoff_hz, sound.reverb, sound.volume);
    }
    source
}

/// `source` at `weight` for a camera at `eye`. A source's weight can't be set, only faded to in
/// real seconds or given by a zone: full at its centre, none past its radius, linear between. So
/// it covers a zone of radius `ZONE` whose centre is `1 - weight` of that from the eye.
fn weighed(source: Environment, weight: f32, eye: Vec3) -> Environment {
    let off_centre = (1.0 - weight as f64) * ZONE;
    let centre = [eye.x, eye.y + off_centre, eye.z];
    source.around(centre, 0.0, ZONE as f32)
}
