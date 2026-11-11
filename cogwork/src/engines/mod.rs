//! Engines: they make energy and push it into the block their trunk points at (a timber tube,
//! the excavator, any mod's machine), and offer it to `conductor` networks too. The spring
//! engine runs on a redstone signal and never overheats; the ember engine burns fuel, makes
//! four times as much, and stops to cool down when its energy has nowhere to go.

mod model;
mod piston;
mod run;

pub use model::turn;
use strum::FromRepr;
use weft_sdk::*;

pub static SPRING_ENGINE: Block = Block::named("spring_engine");
pub static EMBER_ENGINE: Block = Block::named("ember_engine");
static EMBER_MENU: Menu = Menu::named("ember_engine");

value! {
    /// An engine's state: its kind (`Kind`), `heat` 0..1000, `overheated` (1 while it cools
    /// down), the ticks of fuel left burning out of `burn_max`, `stroke` (anything but 0 while
    /// the piston makes a stroke), and the piston entity.
    #[derive(Default)]
    pub struct Engine {
        pub kind: u32,
        pub heat: u32,
        pub overheated: u32,
        pub burn: u32,
        pub burn_max: u32,
        pub stroke: u32,
        pub piston: u64
    }
}
pub static ENGINE: Component<Engine> = Component::named("cogwork:engine");

/// Which engine it is. Kept in `Engine::kind` as its number, so the order is part of the save
/// format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Kind {
    /// Runs on a redstone signal.
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

/// A block entity without slots or a screen: there is nothing to put in it.
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
        .group("fuel", Slots::single().accepts("#fuel").holder())
        .player_inventory()
        .screen(ember_screen);
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
