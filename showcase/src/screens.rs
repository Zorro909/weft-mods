//! A film's screen effects as they change: each post shader's strength going from where it was
//! to where a `Film::screen` step sends it, eased over that step's seconds of film time, so a
//! fade keeps pace with the film through hitches and rehearsals alike.

use showcase_api::Ease;

use crate::rig::math::{eased, lerp};

/// One shader's strength on its way from `from` to `to`, from `start` seconds into the film
/// for `secs`.
#[derive(Clone, Debug, PartialEq)]
struct Ramp {
    shader: String,
    from: f32,
    to: f32,
    start: f32,
    secs: f32,
}

impl Ramp {
    fn at(&self, now: f32) -> f32 {
        if self.secs <= 0.0 {
            return self.to;
        }
        let u = eased(Ease::Smooth, ((now - self.start) / self.secs) as f64);
        lerp(self.from as f64, self.to as f64, u) as f32
    }
}

/// The screen effects of a film, one ramp a shader.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Screens(Vec<Ramp>);

impl Screens {
    /// `shader` going from its strength at `now` to `strength` over `secs`.
    pub fn set(&mut self, shader: &str, strength: f32, secs: f32, now: f32) {
        let from = self.strength(shader, now);
        self.0.retain(|ramp| ramp.shader != shader);
        self.0.push(Ramp {
            shader: shader.to_string(),
            from,
            to: strength.clamp(0.0, 1.0),
            start: now,
            secs,
        });
    }

    /// Every effect showing at `now` and its strength, by shader name.
    pub fn at(&self, now: f32) -> Vec<(String, f32)> {
        (self.0.iter())
            .map(|ramp| (ramp.shader.clone(), ramp.at(now)))
            .filter(|(_, strength)| *strength > 0.0)
            .collect()
    }

    fn strength(&self, shader: &str, now: f32) -> f32 {
        (self.0.iter())
            .find(|ramp| ramp.shader == shader)
            .map_or(0.0, |ramp| ramp.at(now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strength_of(screens: &Screens, now: f32) -> f32 {
        screens.strength("showcase:black", now)
    }

    #[test]
    fn a_fade_goes_from_nothing_to_full_over_its_time() {
        let mut screens = Screens::default();
        screens.set("showcase:black", 1.0, 2.0, 10.0);
        assert_eq!(screens.at(10.0), [], "nothing yet at its start");
        assert!(
            (strength_of(&screens, 11.0) - 0.5).abs() < 1e-6,
            "half at half time"
        );
        assert!(strength_of(&screens, 10.5) < 0.25, "eased in gently");
        assert_eq!(screens.at(12.5), [("showcase:black".to_string(), 1.0)]);
    }

    #[test]
    fn a_fade_back_starts_from_where_the_last_had_got_to() {
        let mut screens = Screens::default();
        screens.set("showcase:black", 1.0, 2.0, 0.0);
        screens.set("showcase:black", 0.0, 2.0, 1.0);
        assert!((strength_of(&screens, 1.0) - 0.5).abs() < 1e-6, "no jump");
        assert_eq!(screens.at(3.0), [], "gone at its end");
    }

    #[test]
    fn no_time_sets_the_strength_at_once() {
        let mut screens = Screens::default();
        screens.set("showcase:warm", 0.6, 0.0, 4.0);
        assert_eq!(screens.at(4.0), [("showcase:warm".to_string(), 0.6)]);
    }

    #[test]
    fn effects_are_kept_apart_and_held_to_full() {
        let mut screens = Screens::default();
        screens.set("showcase:warm", 3.0, 0.0, 0.0);
        screens.set("showcase:black", 1.0, 1.0, 0.0);
        let shown = screens.at(1.0);
        assert_eq!(shown.len(), 2, "both on: {shown:?}");
        assert_eq!(shown[0], ("showcase:warm".to_string(), 1.0), "held to 1");
    }
}
