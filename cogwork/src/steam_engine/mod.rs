//! The steam engine: it takes steam by liquid tube on any side, or straight from a boiler under
//! it, and runs on 20 mB a tick for 160 energy (`run`). Like the other engines it pushes its
//! energy into the blocks at both ends of its flywheel's axle, its left and right as seen from its
//! front, and offers it to torque shafts and wooden axles beside it.
//!
//! The block is the Blockbench model `assets/models/block/steam_engine.json`: a bed plate, the
//! cylinder at its front and the flywheel's bearings. The piston, the connecting rod and the
//! flywheel are an owned entity that runs while the engine does (`rig`).

mod rig;
mod run;
mod tick;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Energy, Facings, Menu, Registry, S, Toward, Ui, component_flags,
    tool_tier, value,
};

use crate::liquid::HELD;

pub static STEAM_ENGINE: Block = Block::named("steam_engine");
static STEAM_ENGINE_MENU: Menu = Menu::named("steam_engine");

/// Millibuckets of steam it holds: 100 ticks of running.
pub const STEAM_CAPACITY: u32 = 2_000;
/// The most it gives to torque shafts and axles beside it a tick.
const OFFERS: i64 = 200;

value! {
    /// A steam engine's state: its `rig` (an entity), its status, and the energy it held after
    /// its last tick (`stored_after`), from which its screen tells what it gave out since.
    #[derive(Default)]
    pub struct SteamEngine {
        pub rig: u64,
        pub status: u32,
        pub stored_after: i64,
    }
}
static STEAM_ENGINE_STATE: Component<SteamEngine> = Component::named("cogwork:steam_engine");

/// What the engine is doing or lacks; its screen says `cogwork.steam_engine.<name>`. Kept in
/// `SteamEngine::status` as its number, so the order is part of the save format: new ones go
/// last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    NoSteam,
    Running,
    /// Its energy has nowhere to go.
    Full,
}

impl SteamEngine {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &STEAM_ENGINE_STATE,
        SteamEngine::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&STEAM_ENGINE_STATE, tick::tick);
    r.block(&STEAM_ENGINE)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("steam_engine")
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(3.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            let store = Energy {
                capacity: run::ENERGY_STORED,
                max_out: OFFERS,
                ..Energy::default()
            };
            entity
                .attach(&STEAM_ENGINE_STATE)
                .attach(&HELD)
                .attach(&crate::machine::METER)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, store)
        })
        .opens(&STEAM_ENGINE_MENU)
        .tab(&crate::TAB);
    r.menu(&STEAM_ENGINE_MENU).player_inventory().screen(screen);
    rig::register(r);
}

/// The steam engine's screen, above the player's inventory:
///
/// ```text
/// +-------------------------------------+
/// | Steam Engine                        |
/// | [#####.....]                        |  stored energy
/// | 8000 / 16000 energy                 |
/// | Giving 160 energy a tick            |  what left it last tick
/// | Running                             |  status
/// | [####......]                        |  its steam
/// | 800 / 2000 mB of Steam              |
/// +-------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        crate::machine::power_bar(ui);
        crate::machine::giving_line(ui);
        ui.text_bind(|view, out| {
            let engine = view.menu_component(&STEAM_ENGINE_STATE).unwrap_or_default();
            let status: &str = engine.status().into();
            let args: [Arg; 0] = [];
            out.set(&format!("cogwork.steam_engine.{status}"), &args);
        });
        crate::machine::tank_bar(ui, STEAM_CAPACITY);
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.steam_engine.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_steam_engine_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(SteamEngine, rig), 0, "its rig");
        assert_eq!(offset_of!(SteamEngine, status), 8, "its status");
        assert_eq!(offset_of!(SteamEngine, stored_after), 16, "what it held");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let engine = SteamEngine {
            status: 99,
            ..SteamEngine::default()
        };
        assert_eq!(engine.status(), Status::NoSteam, "from a newer cogwork");
    }

    #[test]
    fn the_steam_engine_has_its_models() {
        for file in [
            "models/block/steam_engine.json",
            "models/item/steam_engine.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
