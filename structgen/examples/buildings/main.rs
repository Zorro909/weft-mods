//! Writes this mod's buildings into `assets/structures/`: `cargo run --example buildings`, or
//! `cargo run --example buildings -- crofter_cottage longhouse` for some of them. They are drawn
//! here in code, all with the Hearthwood kit (`kit.rs`), so they can be read, changed and kept
//! alike; structures of your own are easier to build in the game and save with
//! `/structure save`. `tools/structure-shots.sh` photographs them in the game.
//!
//! A building is checked before its file is written (`check`): one with a fault is named with
//! what is wrong, its file stays as it was, and the run ends with 1. `--plan` writes no file
//! and prints each building layer by layer instead (`plan`), to read the drawing as text.

mod cells;
mod chapels;
mod check;
mod cottages;
mod draft;
mod farmstead;
mod halls;
mod inns;
mod kit;
mod market;
mod mills;
mod paths;
mod plan;
mod props;
mod smithies;
mod streets;
// The mod's own file reader and writer; each example uses part of it.
#[allow(dead_code)]
#[path = "../../src/structure_file.rs"]
mod structure_file;
mod townhouses;
mod workyards;

use std::path::Path;

use kit::Building;
use structure_file::Structure;

/// Every building: its file name and how it is drawn.
fn buildings() -> Vec<Building> {
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
        paths::BUILDINGS,
        props::BUILDINGS,
        streets::BUILDINGS,
    ]
    .concat()
}

fn main() {
    let mut wanted: Vec<String> = std::env::args().skip(1).collect();
    let plans_only = wanted.iter().any(|arg| arg == "--plan");
    wanted.retain(|arg| arg != "--plan");
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
    let chosen = all
        .into_iter()
        .filter(|(name, _)| wanted.is_empty() || wanted.iter().any(|w| w == name));
    let mut faulty = 0;
    for (name, draw) in chosen {
        let structure = draw();
        let faults = check::faults(&structure);
        for fault in &faults {
            eprintln!("{name}: {fault}");
        }
        faulty += usize::from(!faults.is_empty());
        if plans_only {
            println!("{}", plan::of(name, &structure));
        } else if faults.is_empty() {
            write(name, &structure);
        }
    }
    if faulty > 0 {
        eprintln!("{faulty} buildings have faults; their files stay as they were");
        std::process::exit(1);
    }
}

/// Writes `structure` as `assets/structures/<name>.json` and prints the file's path.
fn write(name: &str, structure: &Structure) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/structures");
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("could not create {}: {e}", dir.display()));
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, structure.to_json())
        .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
    println!("{}", path.display());
}
