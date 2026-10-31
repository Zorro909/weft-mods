//! The scenario list.

mod smoke;

use crate::scenario::Scenario;

pub fn all() -> Vec<Scenario> {
    vec![smoke::range_default()]
}
