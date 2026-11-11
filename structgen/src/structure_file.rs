//! The engine's structure file, `assets/structures/<name>.json` (`Weft/docs/worldgen.md`,
//! "Files"): what `/structure save` writes, what the engine makes a template of, what
//! `examples/buildings` draws the buildings into and what this mod reads them back from.
//!
//! Only blocks and slot defaults are read here. A file's entities, block entities, loot and
//! joints stay the engine's: it places them with the template as saved, but not with the
//! turned copies this mod makes (`buildings.rs`).

use std::collections::BTreeMap;

use serde_json::{Value, json};

/// What the engine's files say they are.
const FORMAT: &str = "weft:structure";
/// A cell the file leaves out: placing it keeps what was there.
pub const KEEP: &str = "";
/// The most cells a box read from a file may have, so a stray far block can't make one too big
/// to hold: 256 x 256 x 256.
const MAX_VOLUME: i64 = 1 << 24;

/// A box of blocks: each cell a block state (`base:oak_log[axis=x]`), a material slot
/// (`@roof/stair[facing=north]`) or `KEEP`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Structure {
    size: [i32; 3],
    /// x fastest, then z, then y.
    cells: Vec<String>,
    /// Each slot's default block, by slot name.
    slots: BTreeMap<String, String>,
}

impl Structure {
    /// A box of `size` from each cell's name, x fastest, then z, then y; `None` unless there is
    /// one name per cell.
    pub fn from_names<'a>(
        size: [i32; 3],
        names: impl IntoIterator<Item = &'a str>,
        slots: &[(&str, &str)],
    ) -> Option<Structure> {
        let cells: Vec<String> = names.into_iter().map(str::to_string).collect();
        let volume: usize = size.iter().map(|&side| side.max(0) as usize).product();
        let slots = slots
            .iter()
            .map(|&(slot, block)| (slot.to_string(), block.to_string()))
            .collect();
        (cells.len() == volume).then_some(Structure { size, cells, slots })
    }

    /// Blocks along x, y and z.
    pub fn size(&self) -> [i32; 3] {
        self.size
    }

    /// Each cell's position from the lowest corner and its name, `KEEP` ones left out.
    pub fn blocks(&self) -> impl Iterator<Item = ([i32; 3], &str)> {
        let [sx, _, sz] = self.size;
        let at = move |i: i32| [i % sx, i / (sx * sz), i / sx % sz];
        (self.cells.iter().enumerate())
            .filter(|(_, name)| *name != KEEP)
            .map(move |(i, name)| (at(i as i32), name.as_str()))
    }

    /// Each slot's name and default block.
    pub fn slots(&self) -> impl Iterator<Item = (&str, &str)> {
        (self.slots.iter()).map(|(slot, block)| (slot.as_str(), block.as_str()))
    }

    /// The file, its palette in the order the names first come.
    pub fn to_json(&self) -> String {
        let mut palette: Vec<&str> = Vec::new();
        let blocks: Vec<Value> = self
            .blocks()
            .map(|([x, y, z], name)| {
                let index = palette.iter().position(|p| *p == name).unwrap_or_else(|| {
                    palette.push(name);
                    palette.len() - 1
                });
                json!([x, y, z, index])
            })
            .collect();
        let file = json!({
            "format": FORMAT,
            "size": self.size,
            "palette": palette,
            "blocks": blocks,
            "slots": self.slots,
        });
        file.to_string()
    }

    /// The structure of a file; why not, for one that isn't.
    pub fn from_json(text: &str) -> Result<Structure, String> {
        let file: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let palette: Vec<&str> = list(&file, "palette")
            .iter()
            .map(|name| name.as_str().ok_or("a palette entry is not text"))
            .collect::<Result<_, _>>()?;
        let blocks: Vec<([i32; 3], &str)> = list(&file, "blocks")
            .iter()
            .map(|block| cell(block, &palette))
            .collect::<Result<_, _>>()?;
        let size = blocks.iter().fold([0; 3], |size, (at, _)| {
            [0, 1, 2].map(|i| size[i].max(at[i] + 1))
        });
        let [sx, sy, sz] = size;
        let volume = sx as i64 * sy as i64 * sz as i64;
        if volume > MAX_VOLUME {
            return Err(format!(
                "its box of {sx} x {sy} x {sz} is past {MAX_VOLUME} blocks"
            ));
        }
        let mut cells = vec![KEEP.to_string(); volume as usize];
        for ([x, y, z], name) in blocks {
            cells[((y * sz + z) * sx + x) as usize] = name.to_string();
        }
        let slots = file.get("slots").and_then(Value::as_object);
        let slots = (slots.into_iter().flatten())
            .map(|(slot, block)| (slot.clone(), block.as_str().unwrap_or_default().to_string()))
            .collect();
        Ok(Structure { size, cells, slots })
    }
}

/// The list under `key`, empty without one.
fn list<'a>(file: &'a Value, key: &str) -> &'a [Value] {
    file.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// One `[x, y, z, palette index]` of a file's `blocks`.
fn cell<'a>(block: &Value, palette: &[&'a str]) -> Result<([i32; 3], &'a str), String> {
    let numbers: Option<Vec<i64>> = block
        .as_array()
        .map(|parts| parts.iter().map(Value::as_i64).collect())
        .and_then(|parts: Option<Vec<i64>>| parts);
    let Some(&[x, y, z, index]) = numbers.as_deref() else {
        return Err(format!("block {block} is not [x, y, z, index]"));
    };
    if [x, y, z].iter().any(|&v| !(0..=4096).contains(&v)) {
        return Err(format!("block {block} lies outside the box"));
    }
    let name = palette.get(index as usize).ok_or(format!(
        "block {block} has palette index {index}, past the palette"
    ))?;
    Ok(([x as i32, y as i32, z as i32], name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hut() -> Structure {
        let names = ["@wall", "core:air", "base:glass", "@wall"];
        Structure::from_names([2, 2, 1], names, &[("wall", "base:oak_planks")]).expect("four cells")
    }

    #[test]
    fn a_file_reads_back_as_written() {
        let back = Structure::from_json(&hut().to_json());
        assert_eq!(back, Ok(hut()), "the same cells and slots");
    }

    #[test]
    fn cells_go_x_first_then_z_then_y() {
        let at: Vec<[i32; 3]> = hut().blocks().map(|(at, _)| at).collect();
        assert_eq!(at, [[0, 0, 0], [1, 0, 0], [0, 1, 0], [1, 1, 0]]);
    }

    #[test]
    fn a_cell_left_out_keeps_what_is_there() {
        let file = r#"{"palette": ["base:stone"], "blocks": [[1, 0, 0, 0]]}"#;
        let structure = Structure::from_json(file).expect("a file");
        assert_eq!(structure.size(), [2, 1, 1], "as big as its furthest block");
        let blocks: Vec<_> = structure.blocks().collect();
        assert_eq!(
            blocks,
            [([1, 0, 0], "base:stone")],
            "the left-out cell is not placed"
        );
    }

    #[test]
    fn a_broken_file_says_why() {
        let past = r#"{"palette": [], "blocks": [[0, 0, 0, 3]]}"#;
        assert!(
            Structure::from_json(past).is_err(),
            "an index past the palette"
        );
        assert!(Structure::from_json("PNG").is_err(), "not JSON at all");
        let far = r#"{"palette": ["base:stone"], "blocks": [[4000, 4000, 4000, 0]]}"#;
        assert!(Structure::from_json(far).is_err(), "a box too big to hold");
        let wrong = Structure::from_names([2, 2, 2], ["core:air"], &[]);
        assert_eq!(wrong, None, "too few names");
    }
}
