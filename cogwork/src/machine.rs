//! What this mod's machines share: a block whose front lights up while it works, a store of
//! energy put into a job a little every tick, and the energy bar on their screens.

use weft_sdk::*;

use crate::lib::Pos;

/// A machine's block, textured `<name>_front` (`<name>_front_on` while it works), `<name>_top`
/// and `<name>_side`, also underneath. Its front faces the player who places it. Any tool breaks
/// it whole; an axe is quickest.
pub fn block<'a>(builder: BlockBuilder<'a>, name: &'static str) -> BlockBuilder<'a> {
    builder
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(move |props, state| {
            let (side, top) = (format!("{name}_side"), format!("{name}_top"));
            let on = if props.is("working", "true") {
                "_on"
            } else {
                ""
            };
            let front = format!("{name}_front{on}");
            // Written facing north: `facing` turns it.
            state.texture(&side).top(&top).front(&front)
        })
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// The way the front of the machine at `pos` faces.
pub fn front(world: &World, pos: Pos) -> Dir {
    world
        .prop(pos, "facing")
        .and_then(Dir::from_name)
        .unwrap_or(Dir::North)
}

/// Lights the front of the machine at `pos` while it works.
pub fn show_working(world: &World, pos: Pos, working: bool) {
    let value = if working { "true" } else { "false" };
    if world.prop(pos, "working").is_some_and(|now| now != value) {
        world.prop_set(pos, "working", value);
    }
}

/// A machine's store of `capacity` energy, empty, taking in up to `max_in` a tick.
pub fn energy(capacity: i64, max_in: i64) -> Energy {
    Energy {
        capacity,
        max_in,
        ..Energy::default()
    }
}

/// How a job stands once a tick's energy has gone into it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Payment {
    Paid,
    /// Energy went in, and more is owed.
    Paying,
    /// More is owed, and there was none to put in.
    NoPower,
}

impl Payment {
    /// With `spent` of `cost` paid, `drawn` of it this tick.
    fn after(spent: i64, cost: i64, drawn: i64) -> Payment {
        if spent >= cost {
            Payment::Paid
        } else if drawn > 0 {
            Payment::Paying
        } else {
            Payment::NoPower
        }
    }
}

/// Puts up to `rate` of the machine `id`'s stored energy into a job of `cost`, of which `spent`
/// is paid already. The caller starts `spent` over when the job is done.
pub fn pay(world: &World, id: u64, spent: &mut i64, cost: i64, rate: i64) -> Payment {
    let owed = (cost - *spent).max(0);
    let drawn = world.component_mut(id, &ENERGY).map_or(0, |energy| {
        let drawn = energy.stored.min(rate).min(owed);
        energy.stored -= drawn;
        drawn
    });
    *spent += drawn;
    Payment::after(*spent, cost, drawn)
}

/// The energy of the machine whose menu is open: a bar and the numbers.
pub fn power_bar(ui: &mut Ui) {
    ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |view| {
        let energy = view.menu_component(&ENERGY).unwrap_or_default();
        energy.stored as f32 / energy.capacity.max(1) as f32
    });
    ui.text_bind(|view, out| {
        let energy = view.menu_component(&ENERGY).unwrap_or_default();
        out.set(
            "cogwork.energy",
            &[Arg::Int(energy.stored), Arg::Int(energy.capacity)],
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_job_is_paid_once_its_cost_is_spent() {
        assert_eq!(Payment::after(60, 100, 30), Payment::Paying, "on its way");
        assert_eq!(Payment::after(60, 100, 0), Payment::NoPower, "run dry");
        assert_eq!(
            Payment::after(100, 100, 40),
            Payment::Paid,
            "the last of it"
        );
        assert_eq!(
            Payment::after(100, 100, 0),
            Payment::Paid,
            "paid before, waiting for room"
        );
    }
}
