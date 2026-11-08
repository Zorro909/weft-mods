//! Puts every structure compiled into the mod into base's overworld, all the same way.

use structgen_api::{Foundation, Structure};
use weft_sdk::*;

/// One chance per cell of this many blocks square.
const SPACING: u32 = 192;
/// How likely a cell is to get the structure.
const CHANCE: f32 = 0.3;
/// Biomes on land with room to build: plains, forests, taiga, snowy plains and mountains.
const BIOME: &str = "#animals";
/// How much the ground under a structure may rise and fall.
const MAX_SLOPE: i32 = 5;

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
        structgen_api::place(r, name, structure)
            .dimension(&base_api::OVERWORLD)
            .spacing(SPACING)
            .chance(CHANCE)
            .biome(BIOME)
            .foundation(Foundation::Ground)
            .max_slope(MAX_SLOPE);
    }
}
