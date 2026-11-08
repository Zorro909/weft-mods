//! Writes this mod's buildings into `assets/structures/`: `cargo run --example buildings`, or
//! `cargo run --example buildings -- crofter_cottage longhouse` for some of them. They are drawn
//! here in code, all with the Hearthwood kit (`kit.rs`), so they can be read, changed and kept
//! alike; structures of your own are easier to build in the game and save with
//! `/structure save`. `tools/structure-shots.sh` photographs them in the game.

mod chapels;
mod cottages;
mod draft;
mod farmstead;
mod halls;
mod inns;
mod kit;
mod market;
mod mills;
mod smithies;
mod townhouses;
mod workyards;

use std::path::Path;

use structgen_api::Structure;

/// Every building: its file name and how it is drawn.
fn buildings() -> Vec<(&'static str, fn() -> Structure)> {
    [
        cottages::BUILDINGS,
        townhouses::BUILDINGS,
        farmstead::BUILDINGS,
        inns::BUILDINGS,
        smithies::BUILDINGS,
        market::BUILDINGS,
        halls::BUILDINGS,
        chapels::BUILDINGS,
        mills::BUILDINGS,
        workyards::BUILDINGS,
    ]
    .concat()
}

fn main() {
    let wanted: Vec<String> = std::env::args().skip(1).collect();
    let all = buildings();
    if let Some(unknown) = wanted
        .iter()
        .find(|w| !all.iter().any(|(name, _)| name == w))
    {
        panic!(
            "no building {unknown}; there are {:?}",
            all.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
    }
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/structures");
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("could not create {}: {e}", dir.display()));
    for (name, draw) in all {
        if !wanted.is_empty() && !wanted.iter().any(|w| w == name) {
            continue;
        }
        let path = dir.join(format!("{name}.wst"));
        std::fs::write(&path, draw().encode())
            .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
        println!("{}", path.display());
    }
}
