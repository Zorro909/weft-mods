//! The trip hammer: the forge anvil's driven counterpart, a separate machine. Steel blooms come
//! in by tube or by hand and steam by liquid tube on any side; a steam-lifted hammer strikes the
//! first bloom five times, 10 mB of steam a strike, into a steel ingot (`strike`). The bloom stays
//! in the input until its ingot is in the output, and the ingot leaves by the front, the side
//! with the anvil, into a tube or a machine there.
//!
//! Its shape is the Blockbench model `assets/models/block/trip_hammer.json`, an anvil on a plinth
//! with the hammer's frame and steam cylinder behind it, and `trip_hammer_bloom.json` with a hot
//! bloom on the anvil; the hammer is an owned entity that falls at each strike (`hammer`).

mod hammer;
mod strike;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots, Toward, Ui,
    World, component_flags, tool_tier, value,
};

use crate::liquid::HELD;
use crate::steel::STEEL_BLOOM;

pub static TRIP_HAMMER: Block = Block::named("trip_hammer");
static TRIP_HAMMER_MENU: Menu = Menu::named("trip_hammer");

/// Millibuckets of steam it holds: 100 strikes.
pub const STEAM_CAPACITY: u32 = 1_000;
/// Its slots: blooms to forge, then the ingots.
const INPUT: u32 = 3;
const OUTPUT: u32 = 3;
/// Light a hot bloom on its anvil gives.
const BLOOM_GLOW: u8 = 7;

value! {
    /// A trip hammer's state: its `hammer` (an entity), the `strikes` its bloom has had, the
    /// ticks since its last strike (`timer`) and its status.
    #[derive(Default)]
    pub struct Hammering {
        pub hammer: u64,
        pub strikes: u32,
        pub timer: u32,
        pub status: u32,
    }
}
static HAMMERING: Component<Hammering> = Component::named("cogwork:hammering");

/// What the hammer is doing or lacks; its screen says `cogwork.trip_hammer.<name>`. Kept in
/// `Hammering::status` as its number, so the order is part of the save format: new ones go
/// last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoSteam,
    Striking,
    /// Its ingot has no room in the output.
    Full,
}

impl Hammering {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &HAMMERING,
        Hammering::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&HAMMERING, work::tick);
    r.block(&TRIP_HAMMER)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("trip_hammer")
        .prop("bloom", ["false", "true"])
        .states(|props, state| {
            if props.is("bloom", "true") {
                state.model_file("trip_hammer_bloom").light(BLOOM_GLOW)
            } else {
                state
            }
        })
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&HAMMERING)
                .attach(&HELD)
                .inventory(INPUT + OUTPUT)
        })
        .opens(&TRIP_HAMMER_MENU)
        .tab(&crate::TAB);
    r.menu(&TRIP_HAMMER_MENU)
        .group(
            "input",
            Slots::row(INPUT).accepts(STEEL_BLOOM.name()).holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    hammer::register(r);
}

/// Puts `stack` from a tube into the trip hammer `at`: steel blooms into its input, nothing
/// else. None for what is no trip hammer.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &HAMMERING)?;
    if stack.item != STEEL_BLOOM.id() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The trip hammer's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Trip Hammer                                                     |
/// | Striking                                                        |  status
/// | [####......]                                                    |  the bloom's strikes
/// | [####......]                                                    |  its steam
/// | 500 / 1000 mB of Steam                                          |
/// | Steel blooms        Steel ingots                                |
/// | [ ][ ][ ]           [ ][ ][ ]                                   |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|view, out| {
            let hammering = view.menu_component(&HAMMERING).unwrap_or_default();
            let status: &str = hammering.status().into();
            let args: [Arg; 0] = [];
            out.set(&format!("cogwork.trip_hammer.{status}"), &args);
        });
        ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |view| {
            let hammering = view.menu_component(&HAMMERING).unwrap_or_default();
            hammering.strikes as f32 / strike::STRIKES as f32
        });
        crate::machine::tank_bar(ui, STEAM_CAPACITY);
        ui.row(S::row().gap(10.0), |ui| {
            for group in ["input", "output"] {
                ui.column(S::column().gap(3.0), |ui| {
                    ui.text(&format!("cogwork.trip_hammer.{group}"));
                    ui.group(group);
                });
            }
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_and_label_has_a_name() {
        let statuses = Status::VARIANTS.iter().map(|&status| <&str>::from(status));
        for name in statuses.chain(["input", "output"]) {
            let key = format!("cogwork.trip_hammer.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_trip_hammer_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Hammering, hammer), 0, "its hammer");
        assert_eq!(offset_of!(Hammering, strikes), 8, "its strikes");
        assert_eq!(offset_of!(Hammering, timer), 12, "since the last");
        assert_eq!(offset_of!(Hammering, status), 16, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let hammering = Hammering {
            status: 99,
            ..Hammering::default()
        };
        assert_eq!(hammering.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn it_holds_a_hundred_strikes_of_steam() {
        assert_eq!(
            STEAM_CAPACITY / strike::STEAM_A_STRIKE,
            100,
            "a strike 10 mB"
        );
    }

    #[test]
    fn the_trip_hammer_has_its_models() {
        for file in [
            "models/block/trip_hammer.json",
            "models/block/trip_hammer_bloom.json",
            "models/item/trip_hammer.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
