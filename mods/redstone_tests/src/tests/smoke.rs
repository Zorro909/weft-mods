use weft_sdk::Item;

use crate::scenario::{Scenario, Step};

/// The harness itself works and the redstone mod is loaded with its default range.
pub fn range_default() -> Scenario {
    Scenario::new(
        "range_default",
        vec![
            Step::Do(|c| c.check_eq(redstone_api::range().max(), 15, "default range")),
            Step::Do(|c| c.set([0, 0, 0], "base:stone")),
            Step::Wait(1),
            Step::Do(|c| c.check_eq(c.get([0, 0, 0]), "base:stone".to_string(), "harness set/get")),
        ],
    )
}

/// The redstone mod's hidden block entities are not creatures: Weft gives every entity a spawn
/// egg unless it says `no_egg`.
pub fn no_spawn_eggs() -> Scenario {
    Scenario::new(
        "no_spawn_eggs",
        vec![Step::Do(|c| {
            for egg in ["redstone:device_spawn_egg", "redstone:plate_spawn_egg"] {
                c.check_eq(Item::lookup(egg), None, egg);
            }
        })],
    )
}
