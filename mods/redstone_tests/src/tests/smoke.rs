use crate::scenario::{Scenario, Step};

/// The harness itself works and the redstone mod is loaded with its default range.
pub fn range_default() -> Scenario {
    Scenario {
        name: "range_default",
        steps: vec![
            Step::Do(|c| c.check_eq(redstone_api::range().max(), 15, "default range")),
            Step::Do(|c| c.set([0, 0, 0], "base:stone")),
            Step::Wait(1),
            Step::Do(|c| c.check_eq(c.get([0, 0, 0]), "base:stone".to_string(), "harness set/get")),
        ],
    }
}
