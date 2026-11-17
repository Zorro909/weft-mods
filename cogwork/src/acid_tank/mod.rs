//! The acid tank: a plastic-lined tank that makes sulfuric acid of sulfur (`dusts/sulfur`, from
//! the coke oven and the refinery), water and engine energy (`brew`). Sulfur comes in by tube or
//! by hand. Water it draws itself out of a liquid tube or a tank on any side but its front, and
//! its acid it lets out of its front into a liquid tube or a tank there, never in place of a
//! leftover of another liquid (`liquid::pour_apart`). It keeps both liquids in its own state,
//! which its item carries when it is broken, by hand or by a machine.
//!
//! Its shape is the Blockbench model `assets/models/block/acid_tank.json`: a cream composite tank
//! with steel bands on a steel plinth, a sight glass and an outlet at its front.

mod brew;
mod work;

pub use brew::ACID_HELD;
use brew::WATER_HELD;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, ClientView, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry,
    S, Slots, Toward, Ui, World, component_flags, tool_tier, value,
};

pub static ACID_TANK: Block = Block::named("acid_tank");
static ACID_TANK_MENU: Menu = Menu::named("acid_tank");

/// What its input takes: sulfur of any mod.
const SULFUR: &str = "#dusts/sulfur";
/// Its slots: sulfur.
const INPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 200;
const STORES: i64 = 4_000;

value! {
    /// An acid tank's state: the energy `spent` on the batch under way, whose sulfur stays in its
    /// input until the batch is done, its `status`, and millibuckets of `water` and of `acid` it
    /// holds. Carried on its item when it is broken.
    #[derive(Default)]
    pub struct Brew {
        pub spent: i64,
        pub status: u32,
        pub water: u32,
        pub acid: u32,
    }
}
pub static BREW: Component<Brew> = Component::named("cogwork:brew");

/// What the acid tank is doing or lacks; its screen says `cogwork.acid_tank.<name>`. Kept in
/// `Brew::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    NoSulfur,
    NoWater,
    NoPower,
    Brewing,
    /// A batch's acid has no room in it.
    Full,
}

impl Brew {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &BREW,
        Brew::default(),
        component_flags::SAVE | component_flags::SYNC_MENU | component_flags::CARRIED,
    );
    r.system(&BREW, work::tick);
    r.block(&ACID_TANK)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("acid_tank")
        .hitbox([1.0, 0.0, 1.0], [15.0, 16.0, 15.0])
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&BREW)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT)
        })
        .opens(&ACID_TANK_MENU)
        .tab(&crate::TAB);
    r.menu(&ACID_TANK_MENU)
        .group("input", Slots::row(INPUT).accepts(SULFUR).holder())
        .player_inventory()
        .screen(screen);
}

/// Puts `stack` from a tube into the acid tank `at`: sulfur into its input, nothing else. None
/// for what is no acid tank.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &BREW)?;
    if !work::is_sulfur(stack.item) {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The acid tank's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Acid Tank                                                       |
/// | [#####.....]                                                    |  stored energy
/// | 1200 / 4000 energy                                              |
/// | Using 40 energy a tick                                          |  drawn last tick
/// | Making acid                                                     |  status
/// | [####......]                                                    |  its water
/// | 2000 / 4000 mB of Water                                         |
/// | [######....]                                                    |  its acid
/// | 3000 / 4000 mB of Sulfuric Acid                                 |
/// | Sulfur                                                          |
/// | [ ][ ][ ]                                                       |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        crate::machine::power_bar(ui);
        crate::machine::meter_line(ui);
        ui.text_bind(|view, out| {
            let status: &str = brew(view).status().into();
            let args: [Arg; 0] = [];
            out.set(&format!("cogwork.acid_tank.{status}"), &args);
        });
        crate::machine::kept_bar(ui, WATER_HELD, "base:water", |view| brew(view).water);
        let acid = "cogwork:sulfuric_acid";
        crate::machine::kept_bar(ui, ACID_HELD, acid, |view| brew(view).acid);
        ui.text("cogwork.acid_tank.input");
        ui.group("input");
        ui.player_inventory(true);
    });
}

/// The state of the acid tank whose menu is open.
fn brew(view: &ClientView) -> Brew {
    view.menu_component(&BREW).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.acid_tank.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(crate::translated("cogwork.acid_tank.input"), "a label");
    }

    #[test]
    fn what_an_acid_tank_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Brew, spent), 0, "the energy spent");
        assert_eq!(offset_of!(Brew, status), 8, "its status");
        assert_eq!(offset_of!(Brew, water), 12, "its water");
        assert_eq!(offset_of!(Brew, acid), 16, "its acid");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let brew = Brew {
            status: 99,
            ..Brew::default()
        };
        assert_eq!(brew.status(), Status::NoSulfur, "from a newer cogwork");
    }

    #[test]
    fn the_acid_tank_has_its_models() {
        for file in ["models/block/acid_tank.json", "models/item/acid_tank.json"] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
