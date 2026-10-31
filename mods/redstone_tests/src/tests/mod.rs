//! The scenario list.

mod basics;
mod devices;
mod extension;
mod freeze;
mod pistons;
mod smoke;

use crate::scenario::Scenario;

pub fn all() -> Vec<Scenario> {
    vec![
        smoke::range_default(),
        smoke::no_spawn_eggs(),
        basics::lever_dust_lamp(),
        basics::fade_ranges(),
        basics::dust_shapes(),
        basics::dust_at_height_limit(),
        basics::long_line_clock(),
        basics::dust_sheet_toggles(),
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
        pistons::piston_pushes_line(),
        pistons::piston_limit(),
        pistons::piston_breaks_torch_in_path(),
        pistons::piston_not_powered_from_face(),
        pistons::blocked_piston_extends_when_cleared(),
        pistons::sticky_pull(),
        pistons::sticky_drop_on_one_tick(),
        pistons::pistons_conflict_both_blocked(),
        pistons::piston_pushes_player(),
        pistons::piston_pushes_own_repeater_with_pending(),
        pistons::chest_unpushable(),
        freeze::restart_before(),
        freeze::restart_after(),
    ]
}
