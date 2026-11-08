//! Engines: they make energy and push it into the block their trunk points at (a timber tube,
//! the excavator, any mod's machine), and offer it to `conductor` networks too. The spring
//! engine runs on a redstone signal and never overheats; the ember engine burns fuel, makes
//! four times as much, and stops to cool down when its energy has nowhere to go.

mod model;
mod piston;
mod run;

pub use model::turn;
use weft_sdk::*;

pub static SPRING_ENGINE: Block = Block::named("spring_engine");
pub static EMBER_ENGINE: Block = Block::named("ember_engine");
static EMBER_MENU: Menu = Menu::named("ember_engine");

value! {
    /// An engine's state: its `kind`, `heat` 0..1000, `overheated` (1 while it cools down), the
    /// ticks of fuel left burning out of `burn_max`, ticks into the piston's stroke, and the
    /// piston entity.
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

pub mod kind {
    pub const SPRING: u32 = 0;
    pub const EMBER: u32 = 1;
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
        .entity(|e| {
            e.attach_with(
                &ENGINE,
                Engine {
                    kind: kind::SPRING,
                    ..Engine::default()
                },
            )
            .attach_with(
                &ENERGY,
                Energy {
                    stored: 0,
                    capacity: 1_000,
                    max_in: 0,
                    max_out: 50,
                },
            )
        });
}

/// Keeps its screen even for the wrench, which turns it while sneaking: tubes find its `fuel`
/// slot through the menu it opens.
fn ember_engine(r: &mut Registry) {
    model::engine(r.block(&EMBER_ENGINE), "ember")
        .hardness(3.0)
        .mined_by("axe", tool_tier::HAND)
        .tab(&crate::TAB)
        .entity(|e| {
            e.attach_with(
                &ENGINE,
                Engine {
                    kind: kind::EMBER,
                    ..Engine::default()
                },
            )
            .attach_with(
                &ENERGY,
                Energy {
                    stored: 0,
                    capacity: 10_000,
                    max_in: 0,
                    max_out: 200,
                },
            )
            .inventory(1)
        })
        .opens(&EMBER_MENU);
    r.menu(&EMBER_MENU)
        .group("fuel", Slots::single().accepts("fuel").holder())
        .player_inventory()
        .screen(ember_screen);
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
                    |c| {
                        c.menu_component(&ENGINE)
                            .map_or(0.0, |e| ratio(e.burn, e.burn_max))
                    },
                );
                ui.group("fuel");
            });
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|c, out| {
                    let e = c.menu_component(&ENGINE).unwrap_or_default();
                    let key = if e.overheated != 0 {
                        "cogwork.engine.overheated"
                    } else {
                        "cogwork.engine.heat"
                    };
                    out.set(key, &[Arg::Int(e.heat as i64 / 10)]);
                });
            });
        });
        ui.player_inventory(true);
    });
}
