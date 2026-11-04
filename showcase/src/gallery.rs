//! `/showcase gallery <mod>`: every block of a mod on a plinth in a row, the camera turning a
//! little around each in turn under its name, after a look at the whole row and before rising
//! over it. Works for any mod, with films or without.

use showcase_api::{Area, BlockState, Film, Pos, Set, crane, flyby, orbit, send};
use weft_sdk::*;

/// Blocks between two plinths' middles.
const SPACING: i32 = 3;
/// Seconds the camera spends on each block.
const TURN_SECS: f32 = 2.6;
/// What the blocks stand on: the first of these that exists.
const PLINTHS: [&str; 2] = ["base:stone_bricks", "base:stone"];

/// Builds the gallery of `mod_id` at `origin` and plays it for `player`; how many blocks it shows.
pub fn show(world: &World, player: u64, origin: Pos, mod_id: &str) -> usize {
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
fn blocks_of(mod_id: &str) -> Vec<(BlockState, &'static str)> {
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
            out.push((state.into(), name.split('[').next().unwrap_or(name)));
        }
        last = block;
    }
    out
}

/// The row on flat ground, with room in front for the camera.
fn build(set: &Set, blocks: &[(BlockState, &str)]) {
    let length = row_length(blocks.len());
    let far = Pos::new(length as i32 + 6, 16, wide(length) as i32 + 3);
    set.level(Area::between(Pos::new(-6, 0, -4), far));
    let plinth = PLINTHS.iter().find_map(|name| BlockState::named(name));
    for (i, (state, _)) in blocks.iter().enumerate() {
        let x = i as i32 * SPACING;
        if let Some(plinth) = plinth {
            set.place(Pos::new(x, 0, 0), plinth);
        }
        set.place(Pos::new(x, 1, 0), *state);
    }
}

/// Blocks from the first plinth to the last, for `count` blocks.
fn row_length(count: usize) -> f64 {
    (count.saturating_sub(1) as i32 * SPACING) as f64
}

/// The middle of the block shown `x` blocks along the row.
fn shown_at(x: f64) -> [f64; 3] {
    [x + 0.5, 1.5, 0.5]
}

/// How far in front of the row the camera takes it in whole, for a row `length` long.
fn wide(length: f64) -> f64 {
    length * 0.4 + 5.0
}

/// The whole row from in front, each block in turn, then up and away.
fn film(mod_id: &str, blocks: &[(BlockState, &str)]) -> Film {
    let length = row_length(blocks.len());
    let row = shown_at(length / 2.0);
    let wide = wide(length);
    let mut film = Film::new()
        .title(&format!("mod.{mod_id}.name"), 4.0)
        .shot(crane(row, wide, 90.0, [4.0, 2.0]).secs(5.0));
    for (i, (_, name)) in blocks.iter().enumerate() {
        let block = shown_at((i as i32 * SPACING) as f64);
        // From the left of the block to its right, the way the camera goes along the row.
        let turn = orbit(block, 3.2).arc(115.0, 65.0).height(1.4, 0.9);
        film = film
            .title(&format!("block.{}", name.replace(':', ".")), TURN_SECS)
            .shot(turn.secs(TURN_SECS).blend(0.7));
    }
    let last = shown_at(length);
    let away = [
        [last[0] + 3.0, 3.0, 4.0],
        [row[0] + 4.0, wide * 0.4, wide * 0.8],
        [row[0], wide * 0.6, wide],
    ];
    film.shot(flyby(&away).look_at(row).secs(6.0).blend(1.0))
}
