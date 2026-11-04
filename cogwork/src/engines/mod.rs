//! Engines: they make energy and push it into the block their trunk points at (a timber tube,
//! the excavator, any mod's machine), and offer it to `conductor` networks too. The spring
//! engine runs on a redstone signal and never overheats; the ember engine burns fuel, makes
//! four times as much, and stops to cool down when its energy has nowhere to go.

mod model;
mod piston;
mod run;

#[cfg(feature = "showcase")]
pub use model::turn;
use weft_sdk::*;

pub static SPRING_ENGINE: Block = Block::named("spring_engine");
pub static EMBER_ENGINE: Block = Block::named("ember_engine");
static SPRING_MENU: Menu = Menu::named("spring_engine");
static EMBER_MENU: Menu = Menu::named("ember_engine");

value! {
    /// An engine's state: its `kind`, `heat` 0..1000, `overheated` (1 while it cools down), the
    /// ticks of fuel left burning out of `burn_max`, ticks into the piston's stroke, the piston
    /// entity, and `running` (1 while it makes energy). New fields go last: a saved engine
    /// loads with them 0.
    #[derive(Default)]
    pub struct Engine {
        pub kind: u32,
        pub heat: u32,
        pub overheated: u32,
        pub burn: u32,
        pub burn_max: u32,
        pub stroke: u32,
        pub piston: u64,
        pub running: u32,
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

/// A machine with no slots, its screen showing how it runs.
fn spring_engine(r: &mut Registry) {
    // Its block entity keeps its `Engine` state and energy, but it holds no items, and the SDK
    // offers no public block entity without slots: `inventory` adds one only for slots. So it
    // is a machine without slot groups, which tubes and other mods' item movers can put nothing
    // into; the screen is an interim choice that comes with being one. Upstream has been asked
    // how a block should get a block entity. The machine's kind is named like the block, as the
    // block entity before it was, so saved engines keep their state.
    r.machine(&SPRING_ENGINE, &SPRING_MENU)
        .block(|b| {
            model::engine(b, "spring")
                .hardness(1.5)
                .mineable("axe")
                .tab(&crate::TAB)
        })
        .attach_with(
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
        .screen(spring_screen);
}

fn ember_engine(r: &mut Registry) {
    r.machine(&EMBER_ENGINE, &EMBER_MENU)
        .block(|b| {
            model::engine(b, "ember")
                .hardness(3.0)
                .mineable("pickaxe")
                .tab(&crate::TAB)
        })
        .group("fuel", Slots::single().accepts("fuel"))
        .attach_with(
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
                heat_text(ui);
            });
        });
        ui.player_inventory(true);
    });
}

/// The spring engine's screen; it has no slots, so no inventory either:
///
/// ```text
/// +-----------------------------+
/// | Spring Engine               |
/// | [#####.....]                |  stored energy
/// | 500 / 1000 energy           |
/// | Heat 45%                    |
/// | Needs a redstone signal     |  only while it has none
/// +-----------------------------+
/// ```
fn spring_screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.column(S::column().gap(3.0), |ui| {
            crate::machine::power_bar(ui);
            heat_text(ui);
            ui.text_bind(|c, out| {
                let e = c.menu_component(&ENGINE).unwrap_or_default();
                if e.running == 0 {
                    out.set("cogwork.engine.no_signal", &[]);
                }
            });
        });
    });
}

/// How hot the engine whose menu is open is, or that it is cooling down.
fn heat_text(ui: &mut Ui) {
    ui.text_bind(|c, out| {
        let e = c.menu_component(&ENGINE).unwrap_or_default();
        let key = if e.overheated != 0 {
            "cogwork.engine.overheated"
        } else {
            "cogwork.engine.heat"
        };
        out.set(key, &[Arg::Int(e.heat as i64 / 10)]);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_line_of_the_screens_has_a_name() {
        let keys = [
            "cogwork.engine.heat",
            "cogwork.engine.overheated",
            "cogwork.engine.no_signal",
        ];
        for key in keys {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }
}
