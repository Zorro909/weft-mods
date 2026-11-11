//! The timber liquid tube's pull: liquid out of the tanks it touches, from any level of them, as
//! much as its energy pays for, and no faster than liquid passes between holders.

use weft_sdk::{Dir, ENERGY, World};

use crate::lib::{Pos, TypedWorld};
use crate::liquid::{Holders, MOST_A_TURN, Vessel};
use crate::liquid_tube::CAPACITY;

/// Energy a millibucket costs: a thousand a bucket. A spring engine's 10 a tick pull a bucket in
/// 5 seconds, an ember engine's 40 four fifths of one a second.
const COST: i64 = 1;

/// Pulls into the tube `id` at `pos`, which has `energy`, from the tanks its arms reach, and
/// pays for it. Returns the millibuckets pulled.
pub fn from_tanks(world: &World, id: u64, pos: Pos, energy: i64) -> u32 {
    let Some(tube) = Vessel::at(world, pos) else {
        return 0;
    };
    let mut holders = Holders::outside(*world);
    let block = world.block_typed(pos);
    let mut pulled = 0;
    for dir in Dir::ALL {
        let held = tube.held(&holders);
        if held.amount >= CAPACITY || pulled >= MOST_A_TURN {
            break;
        }
        if block.prop(dir.name()) != Some("true") {
            continue;
        }
        let Some(tank @ Vessel::Tank(_)) = Vessel::at(world, pos.toward(dir)) else {
            continue;
        };
        let source = tank.held(&holders);
        if source.is_empty() || !held.takes(source.liquid, CAPACITY) {
            continue;
        }
        let had = if held.liquid == source.liquid {
            held.amount
        } else {
            0
        };
        let paid = energy - pulled as i64 * COST;
        let amount = amount(paid, CAPACITY - had, source.amount).min(MOST_A_TURN - pulled);
        if amount == 0 {
            continue;
        }
        tank.take(&mut holders, amount);
        tube.put(&mut holders, source.liquid, amount);
        pulled += amount;
    }
    if let Some(store) = world.component_mut(id, &ENERGY) {
        store.stored -= (pulled as i64 * COST).min(store.stored);
    }
    pulled
}

/// How many millibuckets a pull with `energy` takes into `room` from a tank holding `there`.
fn amount(energy: i64, room: u32, there: u32) -> u32 {
    let paid_for = (energy / COST).clamp(0, MOST_A_TURN as i64) as u32;
    paid_for.min(room).min(there)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pull_takes_what_its_energy_pays_for() {
        assert_eq!(amount(10, 1000, 16_000), 10, "a spring engine's tick");
        assert_eq!(amount(40, 1000, 16_000), 40, "an ember engine's");
        assert_eq!(
            amount(2_000, 1000, 16_000),
            MOST_A_TURN,
            "a turn's worth at most"
        );
        assert_eq!(amount(40, 25, 16_000), 25, "as much as there is room for");
        assert_eq!(amount(40, 1000, 5), 5, "as much as the tank holds");
        assert_eq!(amount(0, 1000, 16_000), 0, "nothing without energy");
    }

    #[test]
    fn a_bucket_costs_a_thousand() {
        assert_eq!(crate::liquid::BUCKET as i64 * COST, 1_000);
    }
}
