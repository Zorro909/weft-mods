//! How deep excavators dig, and what depth costs. Two world settings say where they stop: by
//! default nowhere, until a layer never loads. Down to 64 blocks below the excavator a block
//! costs its plain price; past that the price climbs with the square of the depth.

use weft_sdk::{Config, Registry};

/// Whether excavators dig on with no lowest layer.
static UNLIMITED: Config<bool> = Config::named("excavator_unlimited_depth");
/// The lowest layer excavators mine when depth is limited, as a height.
static LOWEST_Y: Config<i32> = Config::named("excavator_lowest_y");
/// `LOWEST_Y` in a new world: the floor of vanilla's worlds.
const DEFAULT_LOWEST_Y: i32 = -64;
/// Blocks below the excavator it mines at the plain price.
const PLAIN_DEPTH: i64 = 64;
/// Blocks past `PLAIN_DEPTH` at which a block costs `DEEP_EXTRA` times its plain price more:
/// 384 blocks below the excavator, it costs 4 times as much.
const DEEP_SPAN: f64 = 320.0;
const DEEP_EXTRA: f64 = 3.0;

pub fn register(r: &mut Registry) {
    r.config(&UNLIMITED).bool(true).world();
    r.config(&LOWEST_Y)
        .int(DEFAULT_LOWEST_Y.into(), i32::MIN.into(), 0)
        .world();
}

/// The lowest layer an excavator mines; none when it digs on until a layer never loads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Floor(Option<i32>);

impl Floor {
    /// The world's floor as its settings have it now.
    pub fn now() -> Floor {
        if UNLIMITED.get() {
            Floor(None)
        } else {
            Floor::at(LOWEST_Y.get())
        }
    }

    pub fn at(lowest_y: i32) -> Floor {
        Floor(Some(lowest_y))
    }

    /// Whether the layer `y` lies below the floor, so it is not mined.
    pub fn is_above(self, y: i32) -> bool {
        self.0.is_some_and(|lowest_y| y < lowest_y)
    }
}

/// What a block of plain price `cost` costs `below` blocks under the excavator.
pub fn deep_cost(cost: i64, below: i64) -> i64 {
    let past = (below - PLAIN_DEPTH).max(0) as f64 / DEEP_SPAN;
    // A cast from `f64` saturates: absurd depths cost the most there is, not a wrapped number.
    (cost as f64 * (1.0 + DEEP_EXTRA * past * past)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_settings_have_names() {
        for key in ["excavator_unlimited_depth", "excavator_lowest_y"] {
            let key = format!("config.cogwork.{key}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn a_floor_keeps_its_own_layer() {
        let floor = Floor::at(-64);
        assert!(!floor.is_above(-64), "the lowest layer is mined");
        assert!(floor.is_above(-65), "the one below is not");
        assert!(
            !Floor(None).is_above(i32::MIN),
            "no floor, nothing below it"
        );
    }

    #[test]
    fn depth_costs_more_with_its_square_past_64_blocks() {
        assert_eq!(deep_cost(1000, -5), 1000, "above the excavator");
        assert_eq!(deep_cost(1000, 64), 1000, "64 below at the plain price");
        assert_eq!(
            deep_cost(1000, 224),
            1750,
            "half way to 384: a quarter of the extra"
        );
        assert_eq!(deep_cost(1000, 384), 4000, "4 times at 384 below");
        assert_eq!(deep_cost(1000, i64::MAX), i64::MAX, "the most there is");
    }
}
