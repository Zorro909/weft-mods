//! The scenario list.

mod basics;
mod devices;
mod extension;
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
        devices::repeater_delay_and_restore(),
        devices::repeater_lock(),
        devices::repeater_pulse_stretch(),
        devices::comparator_modes(),
        devices::comparator_reads_chest(),
        devices::comparator_reads_through_block(),
        devices::buttons_lengths(),
        devices::plate_on_off(),
        extension::ext_emitter_powers_dust(),
        extension::ext_receiver_gets_sides(),
        extension::ext_wire_isolated(),
        extension::ext_connectable_change(),
        extension::ext_dynamic_output(),
        extension::two_dimensions(),
        freeze::restart_before(),
        freeze::restart_after(),
    ]
}
