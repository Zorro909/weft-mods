//! The oil engine: a 3 by 3 by 3 cube of parts that burns oil, with water for vapor, for 600
//! energy a tick, fifteen times an ember engine. Its controller, in the middle of its front, checks
//! the cube behind it every tick (`pattern`) and runs it once it is built (`run`): liquid tubes
//! fill its intakes with oil and water, and its drives give the energy to the block they face and
//! to torque shafts. Built, its casing shows one framed panel a face and a rig of two cylinders
//! turns on top (`rig`). It has no heat: it runs while it has oil, water and room for what it
//! makes, so no oil goes to waste.

pub mod fuel;
mod parts;
pub mod pattern;
mod rig;
mod run;

pub use parts::{INTAKE_CAPACITY, OIL_INTAKE};
use pattern::{Fault, Problem};
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, ClientView, Component, Facings, Menu, Registry, S, Toward, Ui, component_flags,
    tool_tier, value,
};

use crate::lib::Pos;
use crate::liquid::LiquidKey;

pub static OIL_ENGINE_CONTROLLER: Block = Block::named("oil_engine_controller");
static OIL_ENGINE_MENU: Menu = Menu::named("oil_engine");

value! {
    /// An oil engine, kept by its controller: its rig (an entity), the energy it holds, its oil
    /// and water in millibuckets, `carry` (half millibuckets of oil taken in and not yet burnt),
    /// its status, `stroke` (anything but 0 while its rig makes a stroke), while it is not
    /// built its fault and the block where (`fault_x`, `fault_y`, `fault_z`), and `checked`, 1
    /// once its first tick has shown its casing as it is (`run`).
    #[derive(Default)]
    pub struct OilEngine {
        pub rig: u64,
        pub energy: i64,
        pub oil: u32,
        pub water: u32,
        pub carry: u32,
        pub status: u32,
        pub fault: u32,
        pub stroke: u32,
        pub fault_x: i32,
        pub fault_y: i32,
        pub fault_z: i32,
        pub checked: u32,
    }
}
static OIL_ENGINE: Component<OilEngine> = Component::named("cogwork:oil_engine");

/// What the engine is doing; its screen says `cogwork.oil_engine.<name>`. Kept in
/// `OilEngine::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// Its cube is not built: `OilEngine::fault` says what is wrong.
    #[default]
    Unbuilt,
    Running,
    NoOil,
    NoWater,
    /// Its energy has nowhere to go.
    Full,
}

impl Status {
    pub fn is_built(self) -> bool {
        self != Status::Unbuilt
    }
}

impl OilEngine {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    pub fn fault(&self) -> Fault {
        Fault::from_repr(self.fault).unwrap_or_default()
    }

    /// Keeps it from running for `problem`.
    pub fn set_unbuilt(&mut self, problem: Problem) {
        self.set_status(Status::Unbuilt);
        self.fault = problem.fault as u32;
        (self.fault_x, self.fault_y, self.fault_z) = (problem.at.x, problem.at.y, problem.at.z);
    }

    fn fault_at(&self) -> Pos {
        Pos::new(self.fault_x, self.fault_y, self.fault_z)
    }

    pub fn is_stroking(&self) -> bool {
        self.stroke != 0
    }
}

/// Whether an engine burns `liquid`: oil, and water for vapor. Its intakes take nothing else.
pub fn burns(liquid: LiquidKey) -> bool {
    liquid == fuel::oil() || liquid == fuel::water()
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &OIL_ENGINE,
        OilEngine::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&OIL_ENGINE, run::tick);
    parts::register(r);
    rig::register(r);
    r.block(&OIL_ENGINE_CONTROLLER)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .prop("oil", 0..=fuel::GAUGE_STEPS)
        .prop("water", 0..=fuel::GAUGE_STEPS)
        .model_file("oil_engine_controller")
        .states(parts::controller_look)
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
        .on_break(|world, change| run::broken(world, change.pos.into(), change.state.into()))
        .entity(|entity| entity.attach(&OIL_ENGINE))
        .opens(&OIL_ENGINE_MENU);
    r.menu(&OIL_ENGINE_MENU).screen(screen);
}

/// The oil engine's screen:
///
/// ```text
/// +------------------------------------------------+
/// | Oil Engine Controller                          |
/// | [#####.....]  12000 / 16000 mB of Oil          |
/// | [###.......]  8000 / 16000 mB of Water         |
/// | [#######...]  42000 / 60000 energy             |
/// | Running: 600 energy a tick                     |  or what keeps it from running
/// +------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        store_bar(ui, "cogwork:oil_engine_oil", "item:cogwork:oil", |engine| {
            engine.oil
        });
        store_bar(
            ui,
            "cogwork:oil_engine_water",
            "item:base:water",
            |engine| engine.water,
        );
        ui.row(S::row().gap(6.0), |ui| {
            ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |view| {
                let engine = engine_of(view);
                engine.energy as f32 / fuel::ENERGY_STORED as f32
            });
            ui.text_bind(|view, out| {
                let stored = Arg::Int(engine_of(view).energy);
                out.set("cogwork.energy", &[stored, Arg::Int(fuel::ENERGY_STORED)]);
            });
        });
        ui.text_bind(|view, out| {
            let engine = engine_of(view);
            let at = engine.fault_at();
            let (key, args) = match engine.status() {
                Status::Unbuilt => {
                    let fault: &str = engine.fault().into();
                    let place = [at.x, at.y, at.z].map(|coordinate| Arg::Int(coordinate as i64));
                    (format!("cogwork.oil_engine.{fault}"), Vec::from(place))
                }
                status => {
                    let name: &str = status.into();
                    let makes = Arg::Int(fuel::MAKES);
                    (format!("cogwork.oil_engine.{name}"), vec![makes])
                }
            };
            out.set(&key, &args);
        });
    });
}

/// A bar of one of the engine's stores of liquid, and how much it holds of the liquid `named`.
fn store_bar(ui: &mut Ui, bar: &str, named: &'static str, amount: fn(&OilEngine) -> u32) {
    ui.row(S::row().gap(6.0), |ui| {
        ui.progress_bind(bar, S::default().size(80.0, 8.0), move |view| {
            amount(&engine_of(view)) as f32 / fuel::LIQUID_STORED as f32
        });
        ui.text_bind(move |view, out| {
            let held = Arg::Int(amount(&engine_of(view)) as i64);
            let capacity = Arg::Int(fuel::LIQUID_STORED as i64);
            out.set("cogwork.liquid.held", &[held, capacity, Arg::Name(named)]);
        });
    });
}

fn engine_of(view: &ClientView) -> OilEngine {
    view.menu_component(&OIL_ENGINE).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_and_fault_has_a_name() {
        let statuses = Status::VARIANTS.iter().map(|&status| <&str>::from(status));
        let faults = Fault::VARIANTS.iter().map(|&fault| <&str>::from(fault));
        for name in statuses.chain(faults) {
            let key = format!("cogwork.oil_engine.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let engine = OilEngine {
            status: 99,
            fault: 99,
            ..OilEngine::default()
        };
        assert_eq!(
            engine.status(),
            Status::Unbuilt,
            "a status from a newer cogwork"
        );
        assert_eq!(
            engine.fault(),
            Fault::NotPart,
            "a fault from a newer cogwork"
        );
    }

    #[test]
    fn the_saved_layout_stays() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(OilEngine, energy), 8, "after its rig");
        assert_eq!(offset_of!(OilEngine, oil), 16, "then its liquids");
        assert_eq!(offset_of!(OilEngine, status), 28, "after the carry");
        assert_eq!(offset_of!(OilEngine, fault_x), 40, "the fault's place");
        assert_eq!(offset_of!(OilEngine, checked), 52, "whether checked last");
    }
}
