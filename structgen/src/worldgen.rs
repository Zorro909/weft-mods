//! Puts the structures compiled into the mod into base's overworld: every file becomes a
//! template the settlements build with, and a few buildings also stand alone in the country.

use structgen_api::{Foundation, Structure};
use weft_sdk::*;

use crate::styles;

/// Open land to build on: plains and snowy plains, without the trees of forests, which would
/// stand cut through a building.
pub const OPEN: &str = "#animals !#forest !#mountain";
/// Base's sea level: lakes and the sea fill the ground below it, so buildings stand at least
/// this high.
pub const SEA: i32 = 63;
/// How much the ground under a lone building may rise and fall.
const MAX_SLOPE: i32 = 5;
/// The buildings that also stand alone: each file, one chance per square of this many blocks,
/// and how likely a square is to get one.
const ALONE: &[(&str, u32, f32)] = &[
    ("crofter_cottage", 256, 0.25),
    ("hof", 320, 0.2),
    ("wayside_chapel", 384, 0.2),
    ("post_mill", 384, 0.15),
];
/// The roofs a lone building is given, one picked for each.
const ROOFS: &[&str] = &["spruce_roof", "oak_roof", "birch_roof"];

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

/// The file built into the mod as `name`.
pub fn embedded(name: &str) -> Option<&'static [u8]> {
    EMBEDDED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, file)| *file)
}

/// The names of the files built into the mod.
pub fn embedded_names() -> Vec<&'static str> {
    EMBEDDED.iter().map(|(name, _)| *name).collect()
}

pub fn register(r: &mut Registry) {
    for (name, file) in EMBEDDED {
        let Some(structure) = Structure::decode(file) else {
            error!(
                "assets/structures/{name}.wst is not a structure file of this version, left out"
            );
            continue;
        };
        match ALONE.iter().find(|(alone, _, _)| alone == name) {
            Some(&(_, spacing, chance)) => alone(r, name, structure, spacing, chance),
            None => structgen_api::template(r, Template::generated(name.to_string()), structure),
        }
    }
}

/// Places `structure` alone now and then; it is a template for the settlements too.
fn alone(r: &mut Registry, name: &str, structure: Structure, spacing: u32, chance: f32) {
    let mut placement = structgen_api::place(r, name, structure)
        .dimension(&base_api::OVERWORLD)
        .spacing(spacing)
        .chance(chance)
        .biome(OPEN)
        .min_height(SEA)
        .foundation(Foundation::Ground)
        .max_slope(MAX_SLOPE);
    for roof in ROOFS {
        placement = placement.materials(styles::add(Default::default(), styles::blocks(roof)));
    }
}
