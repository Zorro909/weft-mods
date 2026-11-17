//! The shaker screen: the sieve's driven counterpart. Rubble and slag come in by tube or by
//! hand, and with engine energy its tray shakes each one through the sieve's own tables
//! (`rubble::sifting`). What it sifts out leaves by its front, the side that faced the player
//! who placed it, into a tube or a machine there; a timber tube can pull it out too.
//!
//! Its shape is the Blockbench model `assets/models/block/shaker_screen.json`, a frame on a
//! catch pan; the tray is `assets/models/shaker_screen_tray.geo.json` (`tray`).

mod tray;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, Item, ItemStack, Menu, Registry, S,
    Slots, Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::rubble::sifting::Pile;

pub static SHAKER_SCREEN: Block = Block::named("shaker_screen");
static SHAKER_MENU: Menu = Menu::named("shaker_screen");
static RUBBLE: Item = Item::named("rubble");

/// Its slots: Rubble to sift, then what it sifted out.
const INPUT: (u32, u32) = (3, 1);
const OUTPUT: (u32, u32) = (3, 3);
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 100;
const STORES: i64 = 2_000;

value! {
    /// A shaker screen's state: its `tray` (an entity), the energy `spent` on the Rubble or slag
    /// it sifts, which stays in its input until it is sifted, its `status`, and what that load
    /// turned up once paid for (`sifted`, see `work`), kept until it has room in the output.
    #[derive(Default)]
    pub struct Shaker {
        pub tray: u64,
        pub spent: i64,
        pub status: u32,
        pub sifted: u32,
    }
}
static SHAKER: Component<Shaker> = Component::named("cogwork:shaker");

/// What the shaker screen is doing; its screen says `cogwork.shaker_screen.<name>`. Kept in
/// `Shaker::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Sifting,
    /// What it sifted out has no room in its output.
    Full,
}

impl Shaker {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SHAKER,
        Shaker::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&SHAKER, work::tick);
    let (input, output) = (INPUT.0 * INPUT.1, OUTPUT.0 * OUTPUT.1);
    r.block(&SHAKER_SCREEN)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("shaker_screen")
        // The tray too, so aiming at it aims at the block.
        .hitbox([0.0; 3], [16.0, 15.0, 16.0])
        .hardness(3.0)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .entity(move |entity| {
            entity
                .attach(&SHAKER)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(input + output)
        })
        .opens(&SHAKER_MENU)
        .tab(&crate::TAB);
    r.menu(&SHAKER_MENU)
        .group(
            "input",
            Slots::grid(INPUT.0, INPUT.1)
                .accepts(&format!("{} {}", RUBBLE.name(), crate::steel::SLAG.name()))
                .holder(),
        )
        .group("output", Slots::grid(OUTPUT.0, OUTPUT.1).output().holder())
        .player_inventory()
        .screen(screen);
    tray::register(r);
}

/// Puts `stack` from a tube into the shaker screen `at`: Rubble and slag into its input, nothing
/// else. None for what is no shaker screen.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &SHAKER)?;
    if Pile::of(stack.item).is_none() {
        return Some(stack);
    }
    let input = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(input, stack, simulate))
}

/// The shaker screen's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Shaker Screen                                                   |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, output
/// | 1200 / 2000 energy                        [ ][ ][ ]             |
/// | Using 20 energy a tick                    [ ][ ][ ]             |  drawn last tick
/// | Sifting                                                         |  status
/// | Rubble                                                          |
/// | [ ][ ][ ]                                                       |  Rubble to sift
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                crate::machine::meter_line(ui);
                ui.text_bind(|view, out| {
                    let shaker = view.menu_component(&SHAKER).unwrap_or_default();
                    let status: &str = shaker.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.shaker_screen.{status}"), &args);
                });
            });
            ui.group("output");
        });
        ui.text("cogwork.shaker_screen.input");
        ui.group("input");
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.shaker_screen.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(
            crate::translated("cogwork.shaker_screen.input"),
            "its input"
        );
    }

    #[test]
    fn what_a_shaker_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Shaker, tray), 0, "its tray");
        assert_eq!(offset_of!(Shaker, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Shaker, status), 16, "its status");
        assert_eq!(offset_of!(Shaker, sifted), 20, "what it turned up");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let shaker = Shaker {
            status: 99,
            ..Shaker::default()
        };
        assert_eq!(shaker.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn the_shaker_screen_has_its_model() {
        assert!(
            crate::has_asset("models/block/shaker_screen.json"),
            "no model"
        );
    }
}
