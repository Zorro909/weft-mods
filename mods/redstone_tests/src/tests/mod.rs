//! The scenario list.

mod basics;
mod freeze;
mod smoke;

use crate::scenario::Scenario;

pub fn all() -> Vec<Scenario> {
    vec![
        smoke::range_default(),
        basics::lever_dust_lamp(),
        basics::fade_ranges(),
        basics::dust_shapes(),
        basics::dust_at_height_limit(),
        basics::long_line_clock(),
        freeze::torch_clock_runs(),
        freeze::freeze_across_border(),
        freeze::pending_survives_unload(),
        freeze::break_pending_torch(),
        freeze::restart_before(),
        freeze::restart_after(),
    ]
}
