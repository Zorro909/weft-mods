//! A refinery column's temperature, as warmth above the air round it. Each tick the heater's
//! heat raises it, crude fed in takes some away, and it loses heat to the air in proportion to
//! its warmth, three times as fast while the condenser on top is cooled. So with steady heat,
//! crude and cooling it settles where what comes in is lost again: in its band. Cut the cooling
//! and it settles far above the band; cut the heat and it falls back to the air's warmth.
//!
//! Settled warmth: (heat − crude's) / (loss + cooling), 330 above the air with all three. Every
//! term is per tick and divided by the column's `MASS`, which sets how fast it moves: a column
//! warms into its band in about 6 seconds, and leaves it within one or two once its heat or its
//! cooling is cut, so a heater's and a condenser's stores carry it over a short gap only.

use std::ops::RangeInclusive;

/// Degrees of the air round a column, and of one never heated.
pub const AIR: f32 = 20.0;
/// Degrees the column distils in: below, everything goes to tar; above, the gas vents.
pub const BAND: RangeInclusive<f32> = 320.0..=380.0;
/// Degrees from which it takes crude in at all: below, there is no vapour to distil.
pub const WARM: f32 = 250.0;
/// Degrees the screen's bar reaches.
pub const SCALE: f32 = 500.0;

/// Warmth a tick of heat brings, and a tick of crude fed in takes away.
const HEATING: f32 = 1_200.0;
const CRUDE_COOLS: f32 = 210.0;
/// Share of its warmth lost to the air a tick, and lost more through a cooled condenser.
const LOSS: f32 = 1.0;
const COOLING: f32 = 2.0;
/// What the terms above are divided by: the column's mass, so its warmth moves a little a tick.
const MASS: f32 = 200.0;

/// Where a temperature stands against the band.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Band {
    /// Below `WARM`: it takes no crude.
    Cold,
    /// Warm but below the band: everything goes to tar.
    Under,
    In,
    /// Above the band: the gas vents.
    Over,
}

/// What went into one tick of a column.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tick {
    pub heated: bool,
    pub fed: bool,
    pub cooled: bool,
}

/// The degrees of a column `warmth` above the air.
pub fn degrees(warmth: f32) -> f32 {
    AIR + warmth
}

/// Where `warmth` above the air stands against the band.
pub fn band(warmth: f32) -> Band {
    let degrees = degrees(warmth);
    if degrees < WARM {
        Band::Cold
    } else if degrees < *BAND.start() {
        Band::Under
    } else if degrees <= *BAND.end() {
        Band::In
    } else {
        Band::Over
    }
}

/// The warmth after one `tick` from `warmth`: never below the air's.
pub fn step(warmth: f32, tick: Tick) -> f32 {
    let gained = if tick.heated { HEATING } else { 0.0 };
    let taken = if tick.fed { CRUDE_COOLS } else { 0.0 };
    let lost = warmth * (LOSS + if tick.cooled { COOLING } else { 0.0 });
    (warmth + (gained - taken - lost) / MASS).max(0.0)
}

/// The warmth after `ticks` with nothing going in or out but its loss to the air: a column
/// left alone, or not built.
pub fn left_alone(warmth: f32, ticks: u32) -> f32 {
    let kept = (1.0 - LOSS / MASS).powf(ticks as f32);
    warmth * kept
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUNNING: Tick = Tick {
        heated: true,
        fed: true,
        cooled: true,
    };

    /// The warmth a column settles at with every tick like `tick`.
    fn settles(tick: Tick) -> f32 {
        let gained = if tick.heated { HEATING } else { 0.0 };
        let taken = if tick.fed { CRUDE_COOLS } else { 0.0 };
        let lost = LOSS + if tick.cooled { COOLING } else { 0.0 };
        ((gained - taken) / lost).max(0.0)
    }

    /// The warmth after `ticks` like `tick` from `warmth`.
    fn run(mut warmth: f32, ticks: u32, tick: Tick) -> f32 {
        for _ in 0..ticks {
            warmth = step(warmth, tick);
        }
        warmth
    }

    #[test]
    fn heat_crude_and_cooling_hold_it_in_its_band() {
        assert_eq!(band(settles(RUNNING)), Band::In, "where it settles");
        let warm = WARM - AIR;
        let mut warmth = run(warm, 2_000, RUNNING);
        for _ in 0..10_000 {
            warmth = step(warmth, RUNNING);
            assert_eq!(band(warmth), Band::In, "held at {}", degrees(warmth));
        }
    }

    #[test]
    fn it_reaches_its_band_from_cold_in_seconds() {
        let warming = Tick {
            heated: true,
            ..Tick::default()
        };
        let mut warmth = 0.0;
        let mut ticks = 0;
        while band(warmth) == Band::Cold {
            warmth = step(warmth, warming);
            ticks += 1;
        }
        assert!(ticks < 100, "warm in {ticks} ticks, under 5 seconds");
        let mut under = 0;
        while band(warmth) != Band::In {
            warmth = step(warmth, RUNNING);
            under += 1;
        }
        assert!(
            under < 200,
            "{under} ticks of tar while it warms, under 10 seconds"
        );
    }

    #[test]
    fn without_cooling_it_runs_too_hot() {
        let uncooled = Tick {
            cooled: false,
            ..RUNNING
        };
        assert_eq!(band(settles(uncooled)), Band::Over, "where it settles");
        let held = settles(RUNNING);
        let ticks = (0..).find(|&ticks| band(run(held, ticks, uncooled)) == Band::Over);
        assert!(
            ticks.is_some_and(|ticks| ticks < 40),
            "over in {ticks:?} ticks"
        );
    }

    #[test]
    fn without_heat_it_runs_too_cold_then_stops() {
        let unheated = Tick {
            heated: false,
            ..RUNNING
        };
        let held = settles(RUNNING);
        assert_eq!(band(run(held, 10, unheated)), Band::Under, "tar at first");
        assert_eq!(band(run(held, 200, unheated)), Band::Cold, "then cold");
        assert_eq!(settles(unheated), 0.0, "and down to the air's");
    }

    #[test]
    fn half_the_heat_is_too_little() {
        let mut warmth = settles(RUNNING);
        for tick in 0..2_000 {
            let heated = tick % 2 == 0;
            let fed = band(warmth) != Band::Cold;
            warmth = step(
                warmth,
                Tick {
                    heated,
                    fed,
                    ..RUNNING
                },
            );
        }
        assert_ne!(
            band(warmth),
            Band::In,
            "one boiler's steam: {}",
            degrees(warmth)
        );
    }

    #[test]
    fn heat_without_crude_overshoots_the_band() {
        let idle = Tick {
            fed: false,
            ..RUNNING
        };
        assert_eq!(band(settles(idle)), Band::Over, "no crude to take the heat");
    }

    #[test]
    fn a_column_left_alone_cools_to_the_air() {
        let held = settles(RUNNING);
        assert_eq!(left_alone(held, 0), held, "no time, no change");
        assert!(left_alone(held, 1_200) < 1.0, "a minute later");
        assert!(
            (left_alone(held, 1) - step(held, Tick::default())).abs() < 0.001,
            "as a tick with nothing"
        );
    }

    #[test]
    fn it_never_falls_below_the_air() {
        let fed = Tick {
            fed: true,
            ..Tick::default()
        };
        assert_eq!(step(0.0, fed), 0.0, "crude cools it no further");
    }
}
