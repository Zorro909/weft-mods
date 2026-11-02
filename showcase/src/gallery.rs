//! `/showcase gallery <mod>`: every block of a mod on a plinth in a row, the camera turning a
//! little around each in turn under its name, after a look at the whole row and before rising
//! over it. Works for any mod, with films or without.

use showcase_api::{Film, Set, crane, flyby, orbit, send};
use weft_sdk::*;

/// Blocks between two plinths' middles.
const SPACING: i32 = 3;
/// Seconds the camera spends on each block.
const TURN_SECS: f32 = 2.6;
/// What the blocks stand on: the first of these that exists.
const PLINTHS: [&str; 2] = ["base:stone_bricks", "base:stone"];

/// Builds the gallery of `mod_id` at `origin` and plays it for `player`; how many blocks it shows.
pub fn show(world: &World, player: u64, origin: [i32; 3], mod_id: &str) -> usize {
    let blocks = blocks_of(mod_id);
    if !blocks.is_empty() {
        build(&Set::new(*world, player, origin), &blocks);
        send(
            world,
            player,
            &format!("gallery:{mod_id}"),
            origin,
            &film(mod_id, &blocks),
        );
    }
    blocks.len()
}

/// The first state of every block of `mod_id`, and the block's name.
fn blocks_of(mod_id: &str) -> Vec<(u32, &'static str)> {
    let prefix = format!("{mod_id}:");
    let mut out = Vec::new();
    let mut last = u32::MAX;
    for state in 0.. {
        let name = block_name(state);
        if name.is_empty() {
            break;
        }
        let block = block_of(state);
        if block != last && name.starts_with(&prefix) {
            out.push((state, name.split('[').next().unwrap_or(name)));
        }
        last = block;
    }
    out
}

/// The row on flat ground, with room in front for the camera.
fn build(set: &Set, blocks: &[(u32, &str)]) {
    let length = (blocks.len() as i32 - 1) * SPACING;
    set.level([-6, -4], [length + 6, wide(length) as i32 + 3], 16);
    let plinth = PLINTHS.iter().find_map(|name| Block::lookup(name));
    for (i, (state, _)) in blocks.iter().enumerate() {
        let x = i as i32 * SPACING;
        if let Some(plinth) = plinth {
            set.place([x, 0, 0], plinth);
        }
        set.place([x, 1, 0], *state);
    }
}

/// How far in front of the row the camera takes it in whole, for a row `length` long.
fn wide(length: i32) -> f64 {
    length as f64 * 0.4 + 5.0
}

/// The whole row from in front, each block in turn, then up and away.
fn film(mod_id: &str, blocks: &[(u32, &str)]) -> Film {
    let length = ((blocks.len() - 1) as i32 * SPACING) as f64;
    let row = [length / 2.0 + 0.5, 1.5, 0.5];
    let wide = wide(length as i32);
    let mut film = Film::new()
        .title(&format!("mod.{mod_id}.name"), 4.0)
        .shot(crane(row, wide, 90.0, [4.0, 2.0]).secs(5.0));
    for (i, (_, name)) in blocks.iter().enumerate() {
        let block = [(i as i32 * SPACING) as f64 + 0.5, 1.5, 0.5];
        // From the left of the block to its right, the way the camera goes along the row.
        film = film
            .title(&format!("block.{}", name.replace(':', ".")), TURN_SECS)
            .shot(
                orbit(block, 3.2)
                    .arc(115.0, 65.0)
                    .height(1.4, 0.9)
                    .secs(TURN_SECS)
                    .blend(0.7),
            );
    }
    let last = [length + 0.5, 1.5, 0.5];
    let away = [
        [last[0] + 3.0, 3.0, 4.0],
        [row[0] + 4.0, wide * 0.4, wide * 0.8],
        [row[0], wide * 0.6, wide],
    ];
    film.shot(flyby(&away).look_at(row).secs(6.0).blend(1.0))
}
