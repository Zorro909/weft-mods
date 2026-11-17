//! The boiler: a riveted iron drum that stands on a firebox. Its water is its `HELD`, a holder of
//! liquid that takes only water, by liquid tube on any side; its steam is kept apart (`Boil`) and
//! goes out of its top. It does nothing by itself: the firebox under it boils it (`work`).

use weft_sdk::{
    Arg, Block, Component, Facings, Menu, Registry, S, Toward, Ui, component_flags, tool_tier,
    value,
};

use super::burn::STEAM_HELD;
use crate::liquid::HELD;

pub static BOILER: Block = Block::named("boiler");
static BOILER_MENU: Menu = Menu::named("boiler");

/// Millibuckets of water it holds: 400 ticks of boiling.
pub const WATER_CAPACITY: u32 = 8_000;

value! {
    /// A boiler's steam, in millibuckets.
    #[derive(Default)]
    pub struct Boil {
        pub steam: u32,
    }
}
pub(super) static BOIL: Component<Boil> = Component::named("cogwork:boil");

pub fn register(r: &mut Registry) {
    r.entity_component(
        &BOIL,
        Boil::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.block(&BOILER)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("boiler")
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| entity.attach(&BOIL).attach(&HELD))
        .opens(&BOILER_MENU)
        .tab(&crate::TAB);
    r.menu(&BOILER_MENU).player_inventory().screen(screen);
}

/// The boiler's screen, above the player's inventory:
///
/// ```text
/// +-------------------------------------+
/// | Boiler                              |
/// | [####......]                        |  its water
/// | 4000 / 8000 mB of Water             |
/// | [##........]                        |  its steam
/// | 800 / 4000 mB of Steam              |
/// +-------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        crate::machine::tank_bar(ui, WATER_CAPACITY);
        ui.progress_bind("cogwork:tank", S::default().size(80.0, 8.0), |view| {
            let boil = view.menu_component(&BOIL).unwrap_or_default();
            boil.steam as f32 / STEAM_HELD as f32
        });
        ui.text_bind(|view, out| {
            let boil = view.menu_component(&BOIL).unwrap_or_default();
            let (amount, most) = (Arg::Int(boil.steam as i64), Arg::Int(STEAM_HELD as i64));
            let steam = Arg::Name("item:cogwork:steam");
            out.set("cogwork.liquid.held", &[amount, most, steam]);
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_boiler_keeps_is_saved_as_laid_out() {
        assert_eq!(std::mem::offset_of!(Boil, steam), 0, "its steam");
    }

    #[test]
    fn it_holds_twice_the_water_of_its_steam() {
        assert_eq!(WATER_CAPACITY, 2 * STEAM_HELD, "8,000 and 4,000 mB");
    }

    #[test]
    fn the_boiler_has_its_models() {
        for file in ["models/block/boiler.json", "models/item/boiler.json"] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
