//! The clarifier: a low round settling tank that turns muddy water back into clear water, slowly
//! and with no energy (`settle`). Muddy water comes in by liquid tube on any side: it is a holder
//! of liquid that takes only muddy water. Nine tenths of it comes back as clear water, which it
//! keeps apart (`Settle::clear`) and lets out of its front into a liquid tube or a machine there,
//! a flotation cell taking it back. Now and then a bucket's mud leaves a clay ball, which goes
//! to a tube or a machine on any other side.
//!
//! It is one block, not a wide multiblock: its pace is a number, not a size, and one block needs
//! no pattern, no formed look and only one holder of liquid. Its shape is the Blockbench model
//! `assets/models/block/clarifier.json`, a riveted ring wall round a sloped floor, and
//! `clarifier_water.json` with water settling in it; the rake turning over the floor is
//! `assets/models/clarifier_rake.geo.json` (`rake`).

mod rake;
mod settle;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Facings, Holds, Menu, Registry, S, Slots, Toward, Ui, component_flags,
    tool_tier, value,
};

use crate::liquid::HELD;

pub static CLARIFIER: Block = Block::named("clarifier");
static CLARIFIER_MENU: Menu = Menu::named("clarifier");

/// Millibuckets of muddy water it holds: four buckets.
pub const MUDDY_CAPACITY: u32 = 4_000;
/// Its slots: the clay it settled out.
const OUTPUT: u32 = 3;

value! {
    /// A clarifier's state: its `rake` (an entity), its `status`, millibuckets of `clear` water
    /// it keeps until they leave, of muddy water `settled` toward the bucket under way, and the
    /// ticks `elapsed` toward the next lot.
    #[derive(Default)]
    pub struct Settle {
        pub rake: u64,
        pub status: u32,
        pub clear: u32,
        pub settled: u32,
        pub elapsed: u32,
    }
}
static SETTLE: Component<Settle> = Component::named("cogwork:settle");

/// What the clarifier is doing or lacks; its screen says `cogwork.clarifier.<name>`. Kept in
/// `Settle::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// No muddy water to settle.
    #[default]
    Idle,
    Settling,
    /// Its clear water has nowhere to go.
    Full,
    /// A clay ball would have no room in its output.
    NoRoom,
}

impl Settle {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SETTLE,
        Settle::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&SETTLE, work::tick);
    r.block(&CLARIFIER)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("clarifier")
        .prop("water", ["false", "true"])
        .states(|props, state| {
            if props.is("water", "true") {
                state.model_file("clarifier_water")
            } else {
                state
            }
        })
        .hardness(2.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&SETTLE)
                .attach(&HELD)
                .attach(&crate::owned::LOOPING)
                .inventory(OUTPUT)
        })
        .opens(&CLARIFIER_MENU)
        .tab(&crate::TAB);
    r.menu(&CLARIFIER_MENU)
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    rake::register(r);
}

/// The clarifier's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Clarifier                                                       |
/// | Settling                                  [ ][ ][ ]             |  status, clay
/// | [####......]                                                    |  its muddy water
/// | 1500 / 4000 mB of Muddy Water                                   |
/// | [##........]                                                    |  its clear water
/// | 270 / 1000 mB of Water                                          |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                ui.text_bind(|view, out| {
                    let settle = view.menu_component(&SETTLE).unwrap_or_default();
                    let status: &str = settle.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.clarifier.{status}"), &args);
                });
                crate::machine::tank_bar(ui, MUDDY_CAPACITY);
                crate::machine::kept_bar(ui, settle::CLEAR_HELD, "base:water", |view| {
                    let settle = view.menu_component(&SETTLE).unwrap_or_default();
                    settle.clear
                });
            });
            ui.group("output");
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.clarifier.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_clarifier_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Settle, rake), 0, "its rake");
        assert_eq!(offset_of!(Settle, status), 8, "its status");
        assert_eq!(offset_of!(Settle, clear), 12, "its clear water");
        assert_eq!(offset_of!(Settle, settled), 16, "the bucket under way");
        assert_eq!(offset_of!(Settle, elapsed), 20, "the lot under way");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let settle = Settle {
            status: 99,
            ..Settle::default()
        };
        assert_eq!(settle.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn the_clarifier_has_its_models_and_water() {
        for file in [
            "models/block/clarifier.json",
            "models/block/clarifier_water.json",
            "models/item/clarifier.json",
            "textures/block/clarifier_water.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
