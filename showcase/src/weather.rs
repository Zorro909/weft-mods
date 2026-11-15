//! A film's weather in the game. Each `Film::weather` step is a local environment source of its
//! own, named for the step, so that one weather fades out while the next fades in; the film's
//! end clears the last, which then fades out too.

use std::sync::Mutex;

use showcase_api::Weather;
use weft_sdk::{Client, Environment};

use crate::lock;

/// Where film weather applies among the environment's sources: after the world's weather (-900)
/// and a mod's own sources (0 unless they say), so the film's look wins.
const PRIORITY: i32 = 500;

/// The step whose weather is on, if any.
static SHOWN: Mutex<Option<usize>> = Mutex::new(None);

/// Shows the weather that `step` set, or the world's own for `None`. `at_once` puts it on in
/// full without fading in, for a rehearsal's pictures.
pub fn show(c: &Client, weather: Option<&(usize, Weather)>, at_once: bool) {
    let mut shown = lock(&SHOWN);
    let wanted = weather.map(|(step, _)| *step);
    if *shown == wanted {
        return;
    }
    if let Some(step) = shown.take() {
        c.clear_environment(&name(step));
    }
    if let Some((step, weather)) = weather {
        let fade_in = if at_once { 0.0 } else { weather.fade };
        c.environment(&layer(*step, weather, fade_in));
        *shown = Some(*step);
    }
}

/// Lifts the film's weather, as its end does.
pub fn clear(c: &Client) {
    show(c, None, false);
}

fn name(step: usize) -> String {
    format!("film_weather_{step}")
}

/// The environment source for `weather`, set at `step` of the film.
fn layer(step: usize, weather: &Weather, fade_in: f32) -> Environment {
    let mut layer = Environment::new(&name(step))
        .priority(PRIORITY)
        .fade(fade_in, weather.fade);
    if let Some(fog) = weather.fog {
        layer = layer.fog(fog.color, fog.start, fog.end);
    }
    if let Some(color) = weather.sky {
        layer = layer.sky_color(color);
    }
    if let Some(strength) = weather.sunlight {
        layer = layer.sunlight(strength);
    }
    if let Some(color) = weather.cloud_color {
        layer = layer.cloud_color(color);
    }
    if let Some(cover) = weather.clouds {
        layer.values.cloud_opacity = cover;
    }
    if let Some(sound) = weather.acoustics {
        layer = layer.acoustics(sound.cutoff_hz, sound.reverb, sound.volume);
    }
    layer
}
