//! A structure as text: each layer a map from above, north at the top, one letter per block,
//! with the key under the last. Reading it finds a wall with a gap, a room without a doorway or
//! a lopsided front before the game is started for pictures.

use std::collections::BTreeMap;

use crate::draft::AIR;
use crate::structure_file::Structure;

/// The letters blocks get, in the order the blocks first come; air is always a dot.
const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const AIR_LETTER: char = '.';
/// What a block gets once the letters are used up.
const NO_LETTER: char = '?';

/// The plan of `structure`, headed by its `name` and size.
pub fn of(name: &str, structure: &Structure) -> String {
    let [sx, sy, sz] = structure.size();
    let key = key_of(structure);
    let mut rows = vec![vec![vec![AIR_LETTER; sx as usize]; sz as usize]; sy as usize];
    for ([x, y, z], block) in structure.blocks() {
        rows[y as usize][z as usize][x as usize] = letter(&key, block);
    }
    let mut lines = vec![format!(
        "{name}: {sx} wide (x, east), {sy} high, {sz} long (z, south); north is up"
    )];
    for (y, layer) in rows.iter().enumerate() {
        lines.push(format!("y = {y}"));
        lines.extend(layer.iter().map(|row| row.iter().collect::<String>()));
    }
    lines.extend(
        key.iter()
            .map(|(block, letter)| format!("{letter}  {block}")),
    );
    lines.join("\n")
}

/// Each block of `structure` without its properties, and its letter.
fn key_of(structure: &Structure) -> BTreeMap<&str, char> {
    let mut key = BTreeMap::new();
    let mut letters = LETTERS.chars();
    for (_, name) in structure.blocks() {
        let block = plain(name);
        if block != AIR && !key.contains_key(block) {
            key.insert(block, letters.next().unwrap_or(NO_LETTER));
        }
    }
    key
}

fn letter(key: &BTreeMap<&str, char>, name: &str) -> char {
    key.get(plain(name)).copied().unwrap_or(AIR_LETTER)
}

/// A cell's block without its properties.
fn plain(name: &str) -> &str {
    name.split('[').next().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft::Draft;

    #[test]
    fn a_plan_maps_each_layer_from_the_north() {
        let mut post = Draft::new([3, 2, 2]);
        post.fill([0, 0, 0], [2, 0, 1], "base:grass");
        post.set([2, 1, 0], "base:oak_log[axis=y]");
        let plan = of("post", &post.done());
        let expected = [
            "post: 3 wide (x, east), 2 high, 2 long (z, south); north is up",
            "y = 0",
            "AAA",
            "AAA",
            "y = 1",
            "..B",
            "...",
            "A  base:grass",
            "B  base:oak_log",
        ];
        assert_eq!(plan, expected.join("\n"));
    }
}
