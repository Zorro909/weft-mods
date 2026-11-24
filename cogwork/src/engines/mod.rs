//! Engines: they make energy and push it into the block their trunk points at (a timber tube,
//! a quarry, any mod's machine), and offer it to `conductor` networks too. The spring engine is
//! wound by hand (`wind`) and runs down while a redstone signal doesn't brake it; it never
//! overheats. The ember engine burns fuel, makes four times as much, and stops to cool down when
//! its energy has nowhere to go. It never burns lava.

mod model;
mod piston;
mod run;
mod wind;

pub use model::turn;
use strum::FromRepr;
use weft_sdk::{
    Arg, Block, Component, ENERGY, Energy, Holder, Holds, Item, ItemStack, Menu, Registry, S,
    Slots, Ui, World, abi, component_flags, ratio, tool_tier, value,
};

pub static SPRING_ENGINE: Block = Block::named("spring_engine");
pub static EMBER_ENGINE: Block = Block::named("ember_engine");
static EMBER_MENU: Menu = Menu::named("ember_engine");
static LAVA_BUCKET: Item = Item::named("base:lava_bucket");
/// What an ember engine's fuel slot takes: any fuel but lava.
const EMBER_FUEL: &str = "#fuel !base:lava_bucket";

value! {
    /// An engine's state: its kind (`Kind`), `heat` 0..1000, `overheated` (1 while it cools
    /// down), the ticks of fuel left burning out of `burn_max`, `stroke` (anything but 0 while
    /// the piston makes a stroke), the piston entity, and the energy `wound` into a spring
    /// engine's spring.
    #[derive(Default)]
    pub struct Engine {
        pub kind: u32,
        pub heat: u32,
        pub overheated: u32,
        pub burn: u32,
        pub burn_max: u32,
        pub stroke: u32,
        pub piston: u64,
        // Last, so engines saved before it load with their spring run down.
        pub wound: u32,
    }
}
pub static ENGINE: Component<Engine> = Component::named("cogwork:engine");

/// Which engine it is. Kept in `Engine::kind` as its number, so the order is part of the save
/// format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Kind {
    /// Runs down a spring wound by hand.
    #[default]
    Spring,
    /// Burns fuel.
    Ember,
}

impl Engine {
    /// An engine of `kind`, cold and at rest.
    fn of(kind: Kind) -> Engine {
        Engine {
            kind: kind as u32,
            ..Engine::default()
        }
    }

    pub fn kind(&self) -> Kind {
        Kind::from_repr(self.kind).unwrap_or_default()
    }

    /// Whether it has stopped to cool down.
    pub fn is_overheated(&self) -> bool {
        self.overheated != 0
    }

    /// Whether its piston is making a stroke.
    pub fn is_stroking(&self) -> bool {
        self.stroke != 0
    }
}

pub use run::push_into;

pub fn register(r: &mut Registry) {
    r.entity_component(
        &ENGINE,
        Engine::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&ENGINE, run::tick_engines);
    piston::register(r);
    spring_engine(r);
    ember_engine(r);
}

/// A block entity without slots or a screen: it is wound with an empty hand.
fn spring_engine(r: &mut Registry) {
    model::engine(r.block(&SPRING_ENGINE), "spring")
        .hardness(1.5)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::TAB)
        .entity(|entity| {
            entity
                .attach_with(&ENGINE, Engine::of(Kind::Spring))
                .attach_with(&ENERGY, gives(1_000, 50))
        });
}

/// Keeps its screen even for the wrench, which turns it while sneaking: tubes find its `fuel`
/// slot through the menu it opens.
fn ember_engine(r: &mut Registry) {
    model::engine(r.block(&EMBER_ENGINE), "ember")
        .hardness(3.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::TAB)
        .entity(|entity| {
            entity
                .attach_with(&ENGINE, Engine::of(Kind::Ember))
                .attach_with(&ENERGY, gives(10_000, 200))
                .inventory(1)
        })
        .opens(&EMBER_MENU);
    r.menu(&EMBER_MENU)
        .group("fuel", Slots::single().accepts(EMBER_FUEL).holder())
        .player_inventory()
        .screen(ember_screen);
}

/// Whether an ember engine burns `stack`: lava it leaves where it is, even a bucket put in its
/// slot before it refused them.
fn burns(stack: ItemStack) -> bool {
    stack.item != LAVA_BUCKET.id()
}

/// Refuses `stack` from a tube when the engine `at` is an ember engine that doesn't burn it.
/// None for anything else, which takes items by the slot names alone.
pub fn insert(world: &World, at: Holder, stack: ItemStack, _simulate: bool) -> Option<ItemStack> {
    let engine = world.component(at, &ENGINE)?;
    (engine.kind() == Kind::Ember && !burns(stack)).then_some(stack)
}

/// An engine's store of `capacity` energy, empty, giving out up to `max_out` a tick to
/// `conductor` networks.
fn gives(capacity: i64, max_out: i64) -> Energy {
    Energy {
        capacity,
        max_out,
        ..Energy::default()
    }
}

/// The ember engine's screen, above the player's inventory:
///
/// ```text
/// +-------------------------------------+
/// | Ember Engine                        |
/// |  /\    [#####.....]                 |  fuel left burning, stored energy
/// | [ ]    5000 / 10000 energy          |  fuel
/// |        Heat 40%                     |  or "Overheated: cooling down"
/// +-------------------------------------+
/// ```
fn ember_screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(2.0).center(), |ui| {
                ui.progress_bind_dir(
                    "flame",
                    S::default().size(14.0, 14.0),
                    abi::ui_dir::UP,
                    |view| {
                        view.menu_component(&ENGINE)
                            .map_or(0.0, |engine| ratio(engine.burn, engine.burn_max))
                    },
                );
                ui.group("fuel");
            });
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|view, out| {
                    let engine = view.menu_component(&ENGINE).unwrap_or_default();
                    let key = if engine.is_overheated() {
                        "cogwork.engine.overheated"
                    } else {
                        "cogwork.engine.heat"
                    };
                    out.set(key, &[Arg::Int(engine.heat as i64 / 10)]);
                });
            });
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spring_is_saved_after_all_that_came_before_it() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Engine, piston), 24, "as it always was");
        assert_eq!(offset_of!(Engine, wound), 32, "the spring after it");
    }

    #[test]
    fn an_unknown_saved_kind_reads_as_the_default() {
        let engine = Engine {
            kind: 99,
            ..Engine::default()
        };
        assert_eq!(engine.kind(), Kind::Spring, "a kind from a newer cogwork");
    }
}
