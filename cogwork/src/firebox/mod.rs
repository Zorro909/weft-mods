//! The firebox and the boiler on it: the firebox burns fuel to boil the boiler's water into
//! steam (`burn`), the boiler holds both and lets its steam out of its top into a liquid tube or a
//! steam machine there (`work`). The firebox burns any fuel item from a tube or by hand, a lava
//! bucket included (its empty bucket goes to its output), and lava by liquid tube; it burns only
//! while the boiler on it can boil, so no fuel goes to waste. Water comes into the boiler by
//! liquid tube on any side.
//!
//! The firebox is a cube of fire bricks with an iron door that glows while it burns; the boiler
//! is the Blockbench model `assets/models/block/boiler.json`, a riveted drum (`boiler`).

mod boiler;
mod burn;
mod work;

pub use boiler::{BOILER, WATER_CAPACITY};
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots, Toward, Ui,
    World, abi, component_flags, ratio, tool_tier, value,
};

use crate::liquid::HELD;

pub static FIREBOX: Block = Block::named("firebox");
static FIREBOX_MENU: Menu = Menu::named("firebox");

/// Millibuckets of lava it holds: two buckets.
pub const LAVA_CAPACITY: u32 = 2_000;
/// Its slots: fuel, then the empty buckets of what it burnt.
const FUEL: u32 = 1;
const OUTPUT: u32 = 1;
/// Light its door gives while it burns.
const GLOW: u8 = 12;

value! {
    /// A firebox's state: the ticks of fuel left `burn`ing out of `burn_max`, and its status.
    #[derive(Default)]
    pub struct Fire {
        pub burn: u32,
        pub burn_max: u32,
        pub status: u32,
    }
}
static FIRE: Component<Fire> = Component::named("cogwork:fire");

/// What the firebox is doing or lacks; its screen says `cogwork.firebox.<name>`. Kept in
/// `Fire::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    NoBoiler,
    Boiling,
    NoFuel,
    /// The boiler on it has too little water.
    NoWater,
    /// The boiler's steam has nowhere to go.
    Full,
    /// A lava bucket's empty bucket has no room in its output.
    NoRoom,
}

impl Fire {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &FIRE,
        Fire::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&FIRE, work::tick);
    r.block(&FIREBOX)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|props, state| {
            let lit = props.is("working", "true");
            let front = if lit {
                "firebox_front_lit"
            } else {
                "firebox_front"
            };
            let state = state.texture("fire_bricks").top("steel_block").front(front);
            if lit { state.light(GLOW) } else { state }
        })
        .hardness(2.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| entity.attach(&FIRE).attach(&HELD).inventory(FUEL + OUTPUT))
        .opens(&FIREBOX_MENU)
        .tab(&crate::TAB);
    r.menu(&FIREBOX_MENU)
        .group("fuel", Slots::single().accepts("#fuel").holder())
        .group("output", Slots::single().output().holder())
        .player_inventory()
        .screen(screen);
    boiler::register(r);
}

/// Puts `stack` from a tube into the firebox `at`: fuel into its fuel slot, nothing else. None
/// for what is no firebox.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &FIRE)?;
    if base_api::components::FUEL.on_item(stack.item).is_none() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("fuel")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The firebox's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Firebox                                                         |
/// | Boiling                                                         |  status
/// |  /\    [####......]                       Empty buckets         |  fuel left, its lava
/// | [ ]    1000 / 2000 mB of Lava             [ ]                   |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|view, out| {
            let fire = view.menu_component(&FIRE).unwrap_or_default();
            let status: &str = fire.status().into();
            let args: [Arg; 0] = [];
            out.set(&format!("cogwork.firebox.{status}"), &args);
        });
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(2.0).center(), |ui| {
                ui.progress_bind_dir(
                    "flame",
                    S::default().size(14.0, 14.0),
                    abi::ui_dir::UP,
                    |view| {
                        view.menu_component(&FIRE)
                            .map_or(0.0, |fire| ratio(fire.burn, fire.burn_max))
                    },
                );
                ui.group("fuel");
            });
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::tank_bar(ui, LAVA_CAPACITY);
            });
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.firebox.output");
                ui.group("output");
            });
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
            let key = format!("cogwork.firebox.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(crate::translated("cogwork.firebox.output"), "its output");
    }

    #[test]
    fn what_a_firebox_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Fire, burn), 0, "fuel left");
        assert_eq!(offset_of!(Fire, burn_max), 4, "of how much");
        assert_eq!(offset_of!(Fire, status), 8, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let fire = Fire {
            status: 99,
            ..Fire::default()
        };
        assert_eq!(fire.status(), Status::NoBoiler, "from a newer cogwork");
    }

    #[test]
    fn its_pictures_are_there() {
        for file in [
            "textures/block/firebox_front.png",
            "textures/block/firebox_front_lit.png",
            "textures/block/firebox_front_lit_e.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
