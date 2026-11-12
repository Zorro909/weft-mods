//! What an oil engine burns and makes: 2.5 millibuckets of oil and 5 of water a tick for 600
//! energy, so a bucket of oil gives 240,000 and lasts 20 seconds, and a medium oil field (739
//! buckets) keeps one engine running a little over 4 hours. It runs a tick only when it has all
//! it needs and room for what it makes, so it never burns oil for nothing.

use super::OilEngine;
use crate::liquid::{self, LiquidKey};

/// Energy it makes a tick.
pub const MAKES: i64 = 600;
/// Oil it burns a tick, in half millibuckets: 2.5 millibuckets.
const OIL_HALVES: u32 = 5;
/// Water it turns to vapor a tick, in millibuckets.
const WATER: u32 = 5;
/// Energy it holds: 100 ticks of what it makes.
pub const ENERGY_STORED: i64 = 100 * MAKES;
/// Millibuckets of oil, and of water, it holds: 16 buckets each.
pub const LIQUID_STORED: u32 = 16_000;
/// Steps the gauges on its controller show, besides empty.
pub const GAUGE_STEPS: u32 = 4;

/// What keeps a tick from running.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lack {
    Oil,
    Water,
    Room,
}

impl OilEngine {
    /// Runs it up to `ticks` ticks, as long as it has oil, water and room for the energy. Returns
    /// the ticks it ran and, when it stopped short, what it lacked.
    pub fn burn(&mut self, ticks: u32) -> (u32, Option<Lack>) {
        for ran in 0..ticks {
            if let Some(lack) = self.lack() {
                return (ran, Some(lack));
            }
            let taken = OIL_HALVES.saturating_sub(self.carry).div_ceil(2);
            self.oil -= taken;
            self.carry = self.carry + 2 * taken - OIL_HALVES;
            self.water -= WATER;
            self.energy += MAKES;
        }
        (ticks, self.lack())
    }

    /// What it lacks for a tick, if anything.
    fn lack(&self) -> Option<Lack> {
        let oil_needed = OIL_HALVES.saturating_sub(self.carry).div_ceil(2);
        if self.energy + MAKES > ENERGY_STORED {
            Some(Lack::Room)
        } else if self.oil < oil_needed {
            Some(Lack::Oil)
        } else if self.water < WATER {
            Some(Lack::Water)
        } else {
            None
        }
    }

    /// Millibuckets more of oil, then of water, it has room for.
    pub fn room(&self) -> (u32, u32) {
        (
            LIQUID_STORED - self.oil.min(LIQUID_STORED),
            LIQUID_STORED - self.water.min(LIQUID_STORED),
        )
    }
}

/// The liquids it burns: oil, and water for vapor.
pub fn oil() -> LiquidKey {
    liquid::key_named("cogwork:oil")
}

pub fn water() -> LiquidKey {
    liquid::key_named("base:water")
}

/// The step a gauge shows for `amount` millibuckets: none when empty, all when full, at least
/// one for anything.
pub fn gauge(amount: u32) -> u32 {
    crate::fill::level_of(amount, LIQUID_STORED, GAUGE_STEPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stocked() -> OilEngine {
        OilEngine {
            oil: 1000,
            water: 1000,
            ..OilEngine::default()
        }
    }

    #[test]
    fn a_bucket_of_oil_lasts_twenty_seconds() {
        let mut engine = OilEngine {
            oil: 1000,
            water: LIQUID_STORED,
            ..OilEngine::default()
        };
        let mut ran = 0;
        loop {
            engine.energy = 0;
            let (ticks, lack) = engine.burn(50);
            ran += ticks;
            if lack.is_some() {
                assert_eq!(lack, Some(Lack::Oil), "it ran out of oil");
                break;
            }
        }
        assert_eq!(ran, 400, "400 ticks a bucket");
        assert_eq!(400 * MAKES, 240_000, "for 240,000 energy");
    }

    #[test]
    fn it_burns_two_and_a_half_millibuckets_a_tick() {
        let mut engine = stocked();
        assert_eq!(engine.burn(2), (2, None), "two ticks");
        assert_eq!(engine.oil, 995, "five millibuckets of oil");
        assert_eq!(engine.water, 990, "ten of water");
        assert_eq!(engine.energy, 2 * MAKES, "for 1,200 energy");
    }

    #[test]
    fn it_runs_only_with_oil_water_and_room() {
        let mut engine = stocked();
        engine.water = 4;
        assert_eq!(engine.burn(10), (0, Some(Lack::Water)), "no water");
        let mut engine = stocked();
        engine.oil = 2;
        assert_eq!(engine.burn(10), (0, Some(Lack::Oil)), "too little oil");
        let mut engine = stocked();
        engine.energy = ENERGY_STORED - MAKES;
        assert_eq!(engine.burn(10), (1, Some(Lack::Room)), "room for one tick");
        assert_eq!(engine.energy, ENERGY_STORED, "full");
    }

    #[test]
    fn a_gauge_shows_four_steps() {
        assert_eq!(gauge(0), 0, "empty");
        assert_eq!(gauge(10), 1, "a little");
        assert_eq!(gauge(8_000), 2, "half");
        assert_eq!(gauge(LIQUID_STORED), GAUGE_STEPS, "full");
    }
}
