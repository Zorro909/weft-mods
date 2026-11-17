//! What this mod's machines share: a block whose front lights up while it works, a store of
//! energy put into a job a little every tick, and on their screens the energy bar and a meter of
//! what went through it last tick.

use weft_sdk::{
    Arg, BlockBuilder, Component, Dir, ENERGY, Energy, Facings, Registry, S, Toward, Ui, World,
    component_flags, tool_tier, value,
};

use crate::lib::{BlockState, Pos, TypedWorld};

/// A machine's block, textured `<name>_front` (`<name>_front_on` while it works), `<name>_top`
/// and `<name>_side`, also underneath. Its front faces the player who places it. Any tool breaks
/// it whole; an axe is quickest.
pub fn block<'a>(builder: BlockBuilder<'a>, name: &'static str) -> BlockBuilder<'a> {
    builder
        .grid_placement()
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
            // The front faces north: `facing` turns it.
            state.texture(&side).top(&top).front(&front)
        })
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// The way the front of the machine at `pos` faces.
pub fn front(world: &World, pos: Pos) -> Dir {
    front_of(world.block_typed(pos))
}

/// The way the front of a machine whose block is `block_state` faces.
pub fn front_of(block_state: BlockState) -> Dir {
    block_state
        .prop("facing")
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

value! {
    /// The energy that went through a machine in its last tick: `drawn` from its store, of the
    /// `wanted` it would have drawn with enough there. An engine's `drawn` is what it gave out.
    /// Only shown, never saved.
    #[derive(Default)]
    pub struct Meter {
        pub drawn: i64,
        pub wanted: i64,
    }
}
pub static METER: Component<Meter> = Component::named("cogwork:meter");

impl Meter {
    /// Adds a draw of `drawn` of `wanted` to this tick's.
    fn add(&mut self, drawn: i64, wanted: i64) {
        self.drawn += drawn;
        self.wanted += wanted;
    }

    /// What it wanted and did not get.
    pub fn short(&self) -> i64 {
        (self.wanted - self.drawn).max(0)
    }

    /// The line a machine's screen shows for it: its lang key and arguments.
    fn readout(&self) -> (&'static str, [i64; 2]) {
        let key = match self.short() {
            0 => "cogwork.meter.using",
            _ => "cogwork.meter.short",
        };
        (key, [self.drawn, self.short()])
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(&METER, Meter::default(), component_flags::SYNC_MENU);
}

/// Starts the meter of the machine `id` over, first thing in its tick: what it draws from then
/// on is what its screen shows.
pub fn start_meter(world: &World, id: u64) {
    if let Some(meter) = world.component_mut(id, &METER) {
        *meter = Meter::default();
    }
}

/// Notes on the meter of the machine `id` that it drew `drawn` of the `wanted` energy.
pub fn note_drawn(world: &World, id: u64, drawn: i64, wanted: i64) {
    if let Some(meter) = world.component_mut(id, &METER) {
        meter.add(drawn, wanted);
    }
}

/// Puts up to `rate` of the machine `id`'s stored energy into a job of `cost`, of which `spent`
/// is paid already, and notes it on its meter. The caller starts `spent` over when the job is
/// done.
pub fn pay(world: &World, id: u64, spent: &mut i64, cost: i64, rate: i64) -> Payment {
    let wanted = (cost - *spent).clamp(0, rate);
    let drawn = world.component_mut(id, &ENERGY).map_or(0, |energy| {
        let drawn = energy.stored.min(wanted);
        energy.stored -= drawn;
        drawn
    });
    note_drawn(world, id, drawn, wanted);
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

/// The meter of the machine whose menu is open: "Using 120 energy a tick", and how much it was
/// short of what it wanted.
pub fn meter_line(ui: &mut Ui) {
    ui.text_bind(|view, out| {
        let meter = view.menu_component(&METER).unwrap_or_default();
        let (key, args) = meter.readout();
        out.set(key, &args.map(Arg::Int));
    });
}

/// The meter of the engine whose menu is open: "Sending 600 energy a tick into its drives".
pub fn giving_line(ui: &mut Ui) {
    ui.text_bind(|view, out| {
        let meter = view.menu_component(&METER).unwrap_or_default();
        out.set("cogwork.meter.giving", &[Arg::Int(meter.drawn)]);
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

    #[test]
    fn a_meter_adds_up_the_draws_of_a_tick() {
        let mut meter = Meter::default();
        meter.add(30, 30);
        meter.add(80, 120);
        assert_eq!(meter.drawn, 110, "standby and work together");
        assert_eq!(meter.short(), 40, "the work went short");
    }

    #[test]
    fn a_meter_says_short_only_when_it_was() {
        let fed = Meter {
            drawn: 120,
            wanted: 120,
        };
        assert_eq!(fed.readout(), ("cogwork.meter.using", [120, 0]), "fed");
        let starved = Meter {
            drawn: 90,
            wanted: 120,
        };
        let readout = ("cogwork.meter.short", [90, 30]);
        assert_eq!(starved.readout(), readout, "short 30");
        assert_eq!(Meter::default().readout().0, "cogwork.meter.using", "idle");
    }

    #[test]
    fn every_meter_line_has_a_name() {
        for key in ["using", "short", "giving"] {
            let key = format!("cogwork.meter.{key}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
