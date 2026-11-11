//! The pump: a machine on the ground that bores straight down with engine power, leaving a hose
//! behind it, until it reaches a liquid. Then it pumps that liquid up a bucket at a time, taking
//! the source farthest from its hose first, into a tank of its own that liquid tubes empty.
//! What it bores through goes to the chests and tubes beside it.

mod drain;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Dir, ENERGY, Holds, Menu, Registry, S, Slots, Ui, World,
    component_flags, tool_tier, value,
};

use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Held};

pub static PUMP: Block = Block::named("pump");
pub static PUMP_HOSE: Block = Block::named("pump_hose");
static PUMP_MENU: Menu = Menu::named("pump");

/// Millibuckets the pump's own tank holds.
pub const CAPACITY: u32 = 8000;

value! {
    /// A pump's work: its stage, what it is doing, how many blocks of hose hang below it, the
    /// ticks until it may take the next bucket, and the energy put into the job under way.
    #[derive(Default)]
    pub struct Pump {
        pub stage: u32,
        pub status: u32,
        pub depth: u32,
        pub wait: u32,
        pub spent: i64,
    }
}
static PUMP_STATE: Component<Pump> = Component::named("cogwork:pump");

/// How far a pump has got. Kept in `Pump::stage` as its number, so the order is part of the save
/// format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Stage {
    /// Boring down to a liquid.
    #[default]
    Bore,
    /// Taking the liquid its hose reaches.
    Pump,
}

/// What the pump is doing; its screen says `cogwork.pump.<name>`. Kept in `Pump::status` as its
/// number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Boring,
    Pumping,
    NoPower,
    /// Its tank, or the output for what it bores through, has no room.
    Full,
    /// Something it can't bore through, or the floor of the world.
    Blocked,
    /// No more of the liquid within its reach.
    Dry,
}

impl Status {
    /// Whether the pump shows it is at work.
    pub fn is_working(self) -> bool {
        matches!(self, Status::Boring | Status::Pumping)
    }
}

impl Pump {
    pub fn stage(&self) -> Stage {
        Stage::from_repr(self.stage).unwrap_or_default()
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.stage = stage as u32;
    }

    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &PUMP_STATE,
        Pump::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&PUMP_STATE, work::tick);
    r.block(&PUMP)
        .prop("working", ["false", "true"])
        .states(|props, state| {
            let side = if props.is("working", "true") {
                "pump_side_on"
            } else {
                "pump_side"
            };
            state.model_file("pump").model_texture("side", side)
        })
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
        .on_break(|world, change| pull_hose(world, change.pos.into()))
        .entity(|entity| {
            entity
                .attach(&PUMP_STATE)
                .attach(&HELD)
                .attach_with(&ENERGY, crate::machine::energy(10_000, 500))
                .inventory(9)
        })
        .opens(&PUMP_MENU);
    // Unbreakable, so no gap keeps breaking the pump from taking all of it up.
    r.block(&PUMP_HOSE)
        .model_file("pump_hose")
        .unbreakable()
        .sound_group("base:block.stone")
        .no_item()
        .drops_nothing();
    r.menu(&PUMP_MENU)
        .group("output", Slots::grid(3, 3).output().holder())
        .player_inventory()
        .screen(screen);
}

/// Takes the hose of a pump that was at `pos` out of its hole.
fn pull_hose(world: &World, pos: Pos) {
    let mut below = pos.toward(Dir::Down);
    while world.block_typed(below).is(&PUMP_HOSE) {
        world.set_block_typed(below, crate::lib::BlockState::AIR);
        below = below.toward(Dir::Down);
    }
}

/// The pump's screen, above the player's inventory:
///
/// ```text
/// +--------------------------------------------------+
/// | Pump                                             |
/// | [#####.....]                     [ ][ ][ ]       |  stored energy, what it bored out
/// | 4000 / 10000 energy              [ ][ ][ ]       |
/// | Pumping, its hose 54 blocks down [ ][ ][ ]       |  status
/// | [####......]                                     |  its tank
/// | 3000 / 8000 mB of Oil                            |
/// +--------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|view, out| {
                    let pump = view.menu_component(&PUMP_STATE).unwrap_or_default();
                    let status: &str = pump.status().into();
                    let depth = Arg::Int(pump.depth as i64);
                    out.set(&format!("cogwork.pump.{status}"), &[depth]);
                });
                tank_bar(ui);
            });
            ui.group("output");
        });
        ui.player_inventory(true);
    });
}

/// The pump's tank on its screen: a bar, and how much of which liquid.
fn tank_bar(ui: &mut Ui) {
    ui.progress_bind("cogwork:tank", S::default().size(80.0, 8.0), |view| {
        let held = view.menu_component(&HELD).unwrap_or_default();
        held.amount as f32 / CAPACITY as f32
    });
    ui.text_bind(|view, out| {
        let held: Held = view.menu_component(&HELD).unwrap_or_default();
        let amount = Arg::Int(held.amount as i64);
        let capacity = Arg::Int(CAPACITY as i64);
        match crate::liquid::name(held.liquid).filter(|_| !held.is_empty()) {
            Some(liquid) => {
                let name = format!("item:{liquid}");
                out.set("cogwork.liquid.held", &[amount, capacity, Arg::Name(&name)]);
            }
            None => out.set("cogwork.liquid.empty", &[capacity]),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.pump.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let pump = Pump {
            stage: 99,
            status: 99,
            ..Pump::default()
        };
        assert_eq!(pump.stage(), Stage::Bore, "a stage from a newer cogwork");
        assert_eq!(
            pump.status(),
            Status::Boring,
            "a status from a newer cogwork"
        );
    }

    #[test]
    fn the_saved_layout_stays() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Pump, depth), 8, "after stage and status");
        assert_eq!(offset_of!(Pump, spent), 16, "after depth and wait");
    }

    #[test]
    fn its_model_and_textures_are_there() {
        assert!(crate::has_asset("models/block/pump.json"), "the pump");
        assert!(crate::has_asset("models/block/pump_hose.json"), "its hose");
        for texture in ["pump_side", "pump_side_on"] {
            let file = format!("textures/block/{texture}.png");
            assert!(crate::has_asset(&file), "{file}");
        }
    }
}
