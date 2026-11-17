//! Cogwork: item tubes that show what they carry, engines and water wheels that drive them,
//! machines for the big jobs (quarries, an assembly bench and a leveler), cuboid loaders that keep
//! them running with nobody near, oil: pumped up from fields underground, carried in liquid
//! tubes, kept in tanks and burnt in the oil engine, and the rubble quarries bring up, sifted and
//! ground by hand at the sieve and the millstone, bellows that blow a furnace's fire, and steel:
//! coke from the coke oven, blooms from the blast furnace, forged into ingots on the anvil, and
//! steam: boiled over a firebox, run through a steam engine and a steam-lifted trip hammer, and
//! the steel age's ore work: the flotation cell, the clarifier and the hand core drill, and the
//! refinery that distils crude into diesel for the oil engine, lubricant for the quarries, tar
//! for asphalt roads, and gas.
//! Everything goes through `weft_sdk` and `base_api`.

mod arms;
#[cfg(test)]
mod art;
mod asphalt;
mod bellows;
mod bench;
mod blast_furnace;
mod boxes;
mod brickwork;
mod chance;
mod clarifier;
mod cogs;
mod coke_oven;
mod core_drill;
mod engines;
mod excavator;
mod fill;
#[cfg(feature = "showcase")]
mod films;
mod firebox;
mod flotation;
mod forge_anvil;
#[cfg(test)]
mod generated;
mod ghost;
mod grinding_mill;
mod harvest;
mod leveler;
/// The types the whole mod builds on: helpers for points in the world, in `src/lib/`, and block
/// positions, areas, states and typed world calls from `weftkit`.
mod lib {
    pub mod point;

    pub use weftkit::{Area, BlockState, Pos, TypedWorld};
}
mod liquid;
mod liquid_tube;
mod loading;
mod machine;
mod markers;
mod millstone;
mod muddy_water;
mod oil;
mod oil_engine;
mod owned;
mod pump;
mod recipes;
mod refinery;
mod rubble;
mod shaft;
mod shaker_screen;
mod sieve;
mod sounds;
mod steam;
mod steam_engine;
mod steel;
mod store;
mod tank;
mod timber_liquid_tube;
mod trip_hammer;
mod tube;
mod washer;
mod water_wheel;

use weft_sdk::{CreativeTab, Item, Registry, declare_mod};

#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

pub static TAB: CreativeTab = CreativeTab::named("cogwork");

fn init(r: &mut Registry) {
    r.creative_tab(&TAB, &Item::named("iron_cog"));
    owned::register(r);
    machine::register(r);
    sounds::register(r);
    cogs::register(r);
    tube::register(r);
    engines::register(r);
    markers::register(r);
    excavator::register(r);
    bench::register(r);
    ghost::register(r);
    leveler::register(r);
    loading::register(r);
    shaft::register(r);
    oil::register(r);
    liquid::register(r);
    fill::register(r);
    liquid_tube::register(r);
    timber_liquid_tube::register(r);
    tank::register(r);
    pump::register(r);
    oil_engine::register(r);
    rubble::register(r);
    millstone::register(r);
    sieve::register(r);
    shaker_screen::register(r);
    grinding_mill::register(r);
    washer::register(r);
    bellows::register(r);
    steel::register(r);
    coke_oven::register(r);
    blast_furnace::register(r);
    forge_anvil::register(r);
    steam::register(r);
    firebox::register(r);
    steam_engine::register(r);
    trip_hammer::register(r);
    muddy_water::register(r);
    flotation::register(r);
    clarifier::register(r);
    core_drill::register(r);
    refinery::register(r);
    asphalt::register(r);
    water_wheel::register(r);
    recipes::register(r);
    #[cfg(feature = "showcase")]
    films::register(r);
}

/// Whether the mod has the file `path` under `assets/`.
#[cfg(test)]
fn has_asset(path: &str) -> bool {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    assets.join(path).is_file()
}

/// The English text of `key`, empty without one.
#[cfg(test)]
fn lang_line(key: &str) -> String {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang/en_us.lang");
    let text = std::fs::read_to_string(file).expect("cogwork has no en_us.lang");
    let line = text
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(" = "));
    line.unwrap_or_default().to_string()
}

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&dir).expect("cogwork has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}
