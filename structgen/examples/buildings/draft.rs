//! A box of block names being drawn: plain states (`base:glass`) and material slots
//! (`@roof/stair[facing=north]`), with the slots' defaults.
//!
//! x runs east, z south, y up. Fronts face north (z = 0): a settlement turns a building so that
//! side faces its street.

use std::collections::BTreeMap;

use crate::kit::SLOTS;
use crate::structure_file::Structure;

pub const AIR: &str = "core:air";

pub struct Draft {
    size: [usize; 3],
    names: Vec<String>,
    /// Defaults this building gives slots in place of the kit's.
    defaults: BTreeMap<&'static str, &'static str>,
}

impl Draft {
    /// All air to begin with.
    pub fn new(size: [usize; 3]) -> Draft {
        Draft {
            size,
            names: vec![AIR.to_string(); size.iter().product()],
            defaults: BTreeMap::new(),
        }
    }

    /// Fills `slot` with `block` by default in this building: oak walls for a barn.
    pub fn default(&mut self, slot: &'static str, block: &'static str) {
        self.defaults.insert(slot, block);
    }

    pub fn size(&self) -> [usize; 3] {
        self.size
    }

    /// Blocks outside the box are left out, so a roof's overhang can be drawn past an edge.
    pub fn set(&mut self, [x, y, z]: [usize; 3], name: &str) {
        let [sx, sy, sz] = self.size;
        if x < sx && y < sy && z < sz {
            self.names[(y * sz + z) * sx + x] = name.to_string();
        }
    }

    pub fn get(&self, [x, y, z]: [usize; 3]) -> &str {
        let [sx, _, sz] = self.size;
        &self.names[(y * sz + z) * sx + x]
    }

    /// Every block from `from` to `to`, inclusive, corners in any order.
    pub fn fill(&mut self, from: [usize; 3], to: [usize; 3], name: &str) {
        let low = [0, 1, 2].map(|i| from[i].min(to[i]));
        let high = [0, 1, 2].map(|i| from[i].max(to[i]));
        for y in low[1]..=high[1] {
            for z in low[2]..=high[2] {
                for x in low[0]..=high[0] {
                    self.set([x, y, z], name);
                }
            }
        }
    }

    /// The ring of blocks around the edge of layer `y`, from `from` to `to` (x, z).
    pub fn ring(&mut self, y: usize, from: [usize; 2], to: [usize; 2], name: &str) {
        self.fill([from[0], y, from[1]], [to[0], y, from[1]], name);
        self.fill([from[0], y, to[1]], [to[0], y, to[1]], name);
        self.fill([from[0], y, from[1]], [from[0], y, to[1]], name);
        self.fill([to[0], y, from[1]], [to[0], y, to[1]], name);
    }

    /// Layer `y` drawn as a map: one row per z from the north, one character per x from the
    /// west, each standing for the block `legend` gives it. Rows may stop short of the south
    /// edge, but each spans the box from west to east.
    pub fn paint(&mut self, y: usize, rows: &[&str], legend: &[(char, &str)]) {
        let [sx, _, sz] = self.size;
        assert!(rows.len() <= sz, "{} rows in a box {sz} long", rows.len());
        for (z, row) in rows.iter().enumerate() {
            assert_eq!(
                row.chars().count(),
                sx,
                "row {z} of a box {sx} wide: {row:?}"
            );
            for (x, c) in row.chars().enumerate() {
                let Some((_, name)) = legend.iter().find(|(key, _)| *key == c) else {
                    panic!("no block for {c:?} in the legend");
                };
                self.set([x, y, z], name);
            }
        }
    }

    /// The structure, with a default for every slot it uses.
    pub fn done(self) -> Structure {
        let used: Vec<&str> = SLOTS
            .iter()
            .map(|(slot, _)| *slot)
            .filter(|slot| self.uses(slot))
            .collect();
        let slots: Vec<(&str, &str)> = used
            .iter()
            .map(|slot| {
                (
                    *slot,
                    self.defaults
                        .get(slot)
                        .copied()
                        .unwrap_or(kit_default(slot)),
                )
            })
            .collect();
        let size = self.size.map(|v| v as i32);
        Structure::from_names(size, self.names.iter().map(String::as_str), &slots)
            .expect("a draft names one block per cell")
    }

    fn uses(&self, slot: &str) -> bool {
        let plain = format!("@{slot}");
        let with_props = format!("@{slot}[");
        self.names
            .iter()
            .any(|name| *name == plain || name.starts_with(&with_props))
    }
}

fn kit_default(slot: &str) -> &'static str {
    SLOTS
        .iter()
        .find(|(s, _)| *s == slot)
        .map(|(_, block)| *block)
        .expect("only the kit's slots are used")
}
