//! A film's weather as it changes: the weather a `Film::weather` step sets comes in over its
//! `fade` seconds of film time while the one before goes, eased as screen fades are, so that a
//! change starts and ends at the same moments of the film in a rehearsal and in a take.
//!
//! What both weathers set goes straight from one to the other: the old stays in full beneath
//! the new for it. What only the old sets fades back to the world's own. A weather set while
//! another is still coming in starts from that one where it has got to; what was going then
//! goes at once.

use showcase_api::{Ease, Weather};

use crate::rig::math::eased;

/// What a layer of film weather is in a change, from the bottom up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// What the weather going sets that the one coming sets too, beneath it until it is in.
    Under,
    /// What only the weather going sets, fading back to the world's own.
    Going,
    /// The weather coming in.
    Coming,
}

impl Part {
    pub const ALL: [Part; 3] = [Part::Under, Part::Going, Part::Coming];
}

/// A part of the film's weather and how much of it shows, 0 to 1.
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub part: Part,
    pub weather: Weather,
    pub weight: f32,
}

/// The film's weather since its last change.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherFade {
    /// The weather going, and how much of it showed when the change began.
    going: Option<(Weather, f32)>,
    coming: Option<Weather>,
    /// Seconds into the film the change began.
    start: f32,
    /// Seconds of film the change takes.
    secs: f32,
}

impl WeatherFade {
    /// `weather` coming in from `now` over its fade, or for `None` the world's own, the weather
    /// showing going over its fade.
    pub fn set(&mut self, weather: Option<Weather>, now: f32) {
        let going = self.showing(now).filter(|(_, weight)| *weight > 0.0);
        let fading = weather.as_ref().or(going.as_ref().map(|(going, _)| going));
        self.secs = fading.map_or(0.0, |weather| weather.fade);
        self.going = going;
        self.coming = weather;
        self.start = now;
    }

    /// The layers showing at `now`, from the bottom up, each with some weight.
    pub fn at(&self, now: f32) -> Vec<Layer> {
        let progress = self.progress(now);
        let mut layers = Vec::new();
        if let Some((going, shown)) = &self.going
            && progress < 1.0
        {
            let coming = self.coming.clone().unwrap_or_default();
            let under = set_by(going, &coming, true);
            let own = set_by(going, &coming, false);
            layers.push(layer(Part::Under, under, *shown));
            layers.push(layer(Part::Going, own, shown * (1.0 - progress)));
        }
        if let Some(coming) = &self.coming {
            layers.push(layer(Part::Coming, coming.clone(), progress));
        }
        layers.retain(|layer| layer.weight > 0.0 && sets_anything(&layer.weather));
        layers
    }

    /// The weather that shows most at `now`, and its weight.
    fn showing(&self, now: f32) -> Option<(Weather, f32)> {
        let progress = self.progress(now);
        match (&self.coming, &self.going) {
            (Some(coming), _) => Some((coming.clone(), progress)),
            (None, Some((going, shown))) => Some((going.clone(), shown * (1.0 - progress))),
            (None, None) => None,
        }
    }

    /// The share of the change done at `now`, eased: 0 to 1.
    fn progress(&self, now: f32) -> f32 {
        if self.secs <= 0.0 {
            return 1.0;
        }
        eased(Ease::Smooth, ((now - self.start) / self.secs) as f64) as f32
    }
}

fn layer(part: Part, weather: Weather, weight: f32) -> Layer {
    Layer {
        part,
        weather,
        weight,
    }
}

/// What of `weather` that `other` sets too (`shared`), or does not.
fn set_by(weather: &Weather, other: &Weather, shared: bool) -> Weather {
    let keep = |other_sets: bool| other_sets == shared;
    Weather {
        fog: weather.fog.filter(|_| keep(other.fog.is_some())),
        sky: weather.sky.filter(|_| keep(other.sky.is_some())),
        sunlight: weather.sunlight.filter(|_| keep(other.sunlight.is_some())),
        clouds: weather.clouds.filter(|_| keep(other.clouds.is_some())),
        cloud_color: weather
            .cloud_color
            .filter(|_| keep(other.cloud_color.is_some())),
        acoustics: weather
            .acoustics
            .filter(|_| keep(other.acoustics.is_some())),
        fade: weather.fade,
    }
}

fn sets_anything(weather: &Weather) -> bool {
    *weather != Weather::new().fade(weather.fade)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weight_of(fade: &WeatherFade, part: Part, now: f32) -> f32 {
        (fade.at(now).iter())
            .find(|layer| layer.part == part)
            .map_or(0.0, |layer| layer.weight)
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn a_weather_comes_in_over_its_fade_of_film_time() {
        let mut fade = WeatherFade::default();
        fade.set(Some(Weather::dusk().fade(2.0)), 10.0);
        assert_eq!(fade.at(10.0), [], "nothing yet at its start");
        assert!(
            weight_of(&fade, Part::Coming, 10.5) < 0.25,
            "eased in gently"
        );
        let half = weight_of(&fade, Part::Coming, 11.0);
        assert!(close(half, 0.5), "half at half time: {half}");
        assert_eq!(
            fade.at(12.0),
            [layer(Part::Coming, Weather::dusk().fade(2.0), 1.0)],
            "in full at its end"
        );
    }

    #[test]
    fn no_fade_puts_a_weather_on_at_once() {
        let mut fade = WeatherFade::default();
        fade.set(Some(Weather::mist().fade(0.0)), 3.0);
        assert_eq!(weight_of(&fade, Part::Coming, 3.0), 1.0);
    }

    #[test]
    fn what_both_weathers_set_crossfades_without_the_world_between() {
        let dusk = Weather::new().sky(0x6A5878).sunlight(0.4);
        let mist = Weather::new().sky(0xA8B0B4).clouds(0.0).fade(2.0);
        let mut fade = WeatherFade::default();
        fade.set(Some(dusk), 0.0);
        fade.set(Some(mist.clone()), 5.0);
        let layers = fade.at(6.0);
        let under = &layers[0];
        assert_eq!(under.part, Part::Under);
        assert_eq!(
            under.weather,
            Weather::new().sky(0x6A5878),
            "the sky both set"
        );
        assert_eq!(under.weight, 1.0, "in full beneath the mist");
        let going = &layers[1];
        assert_eq!(going.weather, Weather::new().sunlight(0.4), "dusk's own");
        assert!(close(going.weight, 0.5), "half gone halfway");
        assert!(close(layers[2].weight, 0.5), "mist half in");
        assert_eq!(
            fade.at(7.0),
            [layer(Part::Coming, mist, 1.0)],
            "then mist alone"
        );
    }

    #[test]
    fn a_change_takes_the_fade_of_the_weather_coming() {
        let mut fade = WeatherFade::default();
        fade.set(Some(Weather::dusk().fade(4.0)), 0.0);
        fade.set(Some(Weather::mist().fade(1.0)), 10.0);
        assert!(close(weight_of(&fade, Part::Coming, 10.5), 0.5));
        assert_eq!(weight_of(&fade, Part::Under, 11.0), 0.0, "dusk gone in 1 s");
    }

    #[test]
    fn a_weather_set_while_one_comes_in_starts_from_where_it_got_to() {
        let mut fade = WeatherFade::default();
        fade.set(Some(Weather::dusk().fade(2.0)), 0.0);
        fade.set(Some(Weather::mist().fade(2.0)), 1.0);
        let layers = fade.at(1.0);
        assert!(close(layers[0].weight, 0.5), "dusk as far in as it was");
        assert_eq!(layers.last().map(|layer| layer.part), Some(Part::Going));
    }

    #[test]
    fn clearing_fades_the_weather_out_over_its_own_fade() {
        let mut fade = WeatherFade::default();
        fade.set(Some(Weather::underground().fade(3.0)), 0.0);
        fade.set(None, 10.0);
        let going = fade.at(11.5);
        assert_eq!(going.len(), 1, "all of it going: {going:?}");
        assert_eq!(going[0].part, Part::Going);
        assert!(close(going[0].weight, 0.5), "half gone halfway");
        assert_eq!(fade.at(13.0), [], "the world's own at its end");
    }

    #[test]
    fn clearing_with_no_weather_shows_nothing() {
        let mut fade = WeatherFade::default();
        fade.set(None, 1.0);
        assert_eq!(fade.at(1.0), []);
    }
}
