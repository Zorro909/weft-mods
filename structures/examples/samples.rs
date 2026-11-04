//! Writes the sample structures into `assets/structures/`: `cargo run --example samples`.
//! They are drawn here in code so they can be read and changed; structures of your own are
//! easier to build in the game and save with `/structure save`.
//!
//! x runs east, z south, y up. The fronts face north (z = 0): a hamlet turns a cottage so that
//! side faces its well.

use std::path::Path;

use structures_api::Structure;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/structures");
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("could not create {}: {e}", dir.display()));
    for (name, structure) in [
        ("cottage", cottage()),
        ("watchtower", watchtower()),
        ("well", well()),
        ("wheat_farm", wheat_farm()),
        ("ruined_shrine", ruined_shrine()),
    ] {
        let path = dir.join(format!("{name}.wst"));
        std::fs::write(&path, structure.encode())
            .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
        println!("{}", path.display());
    }
}

/// A box of block names being drawn, all air to begin with.
struct Draft {
    size: [usize; 3],
    names: Vec<&'static str>,
}

impl Draft {
    fn new(size: [usize; 3]) -> Draft {
        Draft {
            size,
            names: vec!["core:air"; size.iter().product()],
        }
    }

    fn set(&mut self, [x, y, z]: [usize; 3], name: &'static str) {
        let [sx, _, sz] = self.size;
        self.names[(y * sz + z) * sx + x] = name;
    }

    /// Every block from `from` to `to`, inclusive.
    fn fill(&mut self, from: [usize; 3], to: [usize; 3], name: &'static str) {
        for y in from[1]..=to[1] {
            for z in from[2]..=to[2] {
                for x in from[0]..=to[0] {
                    self.set([x, y, z], name);
                }
            }
        }
    }

    /// The ring of blocks around the edge of layer `y`, from `from` to `to` (x, z).
    fn ring(&mut self, y: usize, from: [usize; 2], to: [usize; 2], name: &'static str) {
        self.fill([from[0], y, from[1]], [to[0], y, from[1]], name);
        self.fill([from[0], y, to[1]], [to[0], y, to[1]], name);
        self.fill([from[0], y, from[1]], [from[0], y, to[1]], name);
        self.fill([to[0], y, from[1]], [to[0], y, to[1]], name);
    }

    fn done(self) -> Structure {
        Structure::from_names(self.size.map(|v| v as u16), self.names)
            .expect("a draft has one name per block")
    }
}

/// Plank walls on a cobblestone floor, log corners, glass windows, a stepped spruce roof with a
/// brick chimney, and a workbench, furnace and chest inside.
fn cottage() -> Structure {
    let mut d = Draft::new([9, 9, 7]);
    d.fill([0, 0, 0], [8, 0, 6], "base:oak_planks");
    for y in 1..=3 {
        d.ring(y, [0, 0], [8, 6], "base:oak_planks");
    }
    for [x, z] in [[0, 0], [8, 0], [0, 6], [8, 6]] {
        d.fill([x, 1, z], [x, 3, z], "base:oak_log[axis=y]");
    }
    d.fill([4, 1, 0], [4, 2, 0], "core:air");
    for [x, z] in [[2, 0], [6, 0], [2, 6], [6, 6], [0, 3], [8, 3]] {
        d.set([x, 2, z], "base:glass");
    }
    d.set([1, 1, 5], "base:crafting_table");
    d.set([1, 2, 5], "base:torch[facing=up]");
    d.set([2, 1, 5], "base:furnace[facing=north,lit=false]");
    d.set([6, 1, 5], "base:chest[facing=north]");
    d.fill([0, 4, 0], [8, 4, 6], "base:spruce_planks");
    d.set([4, 4, 3], "base:glowstone");
    d.fill([0, 5, 1], [8, 5, 5], "base:spruce_planks");
    d.fill([0, 6, 2], [8, 6, 4], "base:spruce_planks");
    d.fill([0, 7, 3], [8, 7, 3], "base:spruce_log[axis=x]");
    d.fill([7, 4, 5], [7, 8, 5], "base:bricks");
    d.done()
}

/// A stone tower with a ladder up to a railed lookout under a roof, torches on the posts.
fn watchtower() -> Structure {
    let mut d = Draft::new([7, 16, 7]);
    d.fill([0, 0, 0], [6, 0, 6], "base:cobblestone");
    d.fill([1, 0, 1], [5, 0, 5], "base:stone_bricks");
    for y in 1..=10 {
        for (x, z) in ring_cells([1, 1], [5, 5]) {
            d.set([x, y, z], weathered(x + 3 * y + 5 * z));
        }
    }
    d.fill([3, 1, 1], [3, 2, 1], "core:air");
    for [x, z] in [[3, 1], [3, 5], [1, 3], [5, 3]] {
        d.set([x, 6, z], "core:air");
    }
    d.fill([0, 11, 0], [6, 11, 6], "base:spruce_planks");
    d.fill([3, 1, 4], [3, 11, 4], "base:ladder[facing=north]");
    for [x, z] in [[0, 0], [6, 0], [0, 6], [6, 6]] {
        d.fill([x, 12, z], [x, 13, z], "base:spruce_log[axis=y]");
    }
    for (x, z) in ring_cells([0, 0], [6, 6]).filter(|(x, z)| (x + z) % 2 == 1) {
        d.set([x, 12, z], "base:oak_planks");
    }
    for [x, z] in [[1, 1], [5, 1], [1, 5], [5, 5]] {
        d.set([x, 12, z], "base:torch[facing=up]");
    }
    d.fill([0, 14, 0], [6, 14, 6], "base:spruce_planks");
    d.fill([2, 15, 2], [4, 15, 4], "base:spruce_planks");
    d.done()
}

/// A cobblestone well with water in it, log posts and a peaked roof.
fn well() -> Structure {
    let mut d = Draft::new([5, 6, 5]);
    for y in 0..=1 {
        d.ring(y, [0, 0], [4, 4], "base:cobblestone");
        d.fill([1, y, 1], [3, y, 3], "base:water");
    }
    for [x, z] in [[0, 0], [4, 0], [0, 4], [4, 4]] {
        d.fill([x, 2, z], [x, 3, z], "base:oak_log[axis=y]");
    }
    d.fill([0, 4, 0], [4, 4, 4], "base:spruce_planks");
    d.fill([0, 5, 2], [4, 5, 2], "base:spruce_log[axis=x]");
    d.done()
}

/// Ripe wheat on farmland around a water channel, framed by logs with torches at the corners.
fn wheat_farm() -> Structure {
    let mut d = Draft::new([9, 2, 7]);
    d.fill([0, 0, 0], [8, 0, 6], "base:farmland");
    d.fill([1, 1, 1], [7, 1, 5], "base:wheat[age=7]");
    d.fill([1, 0, 3], [7, 0, 3], "base:water");
    d.fill([1, 1, 3], [7, 1, 3], "core:air");
    d.fill([0, 0, 0], [8, 0, 0], "base:spruce_log[axis=x]");
    d.fill([0, 0, 6], [8, 0, 6], "base:spruce_log[axis=x]");
    d.fill([0, 0, 1], [0, 0, 5], "base:spruce_log[axis=z]");
    d.fill([8, 0, 1], [8, 0, 5], "base:spruce_log[axis=z]");
    for [x, z] in [[0, 0], [8, 0], [0, 6], [8, 6]] {
        d.set([x, 1, z], "base:torch[facing=up]");
    }
    d.done()
}

/// A worn stone platform with broken pillars of different heights, rubble, and a glowing
/// stone on a pedestal in the middle.
fn ruined_shrine() -> Structure {
    let mut d = Draft::new([7, 6, 7]);
    for z in 0..7 {
        for x in 0..7 {
            let worn = (x * 5 + z * 3) % 11 == 0;
            d.set(
                [x, 0, z],
                if worn {
                    "base:gravel"
                } else {
                    weathered(x + 2 * z)
                },
            );
        }
    }
    for ([x, z], height) in [([1, 1], 5), ([5, 1], 3), ([1, 5], 4), ([5, 5], 2)] {
        for y in 1..=height {
            d.set([x, y, z], weathered(x + 3 * y + z));
        }
    }
    d.fill([1, 5, 2], [1, 5, 3], "base:stone_bricks");
    d.set([3, 1, 3], "base:stone_bricks");
    d.set([3, 2, 3], "base:glowstone");
    for [x, z] in [[2, 5], [6, 3], [0, 2], [4, 0]] {
        d.set([x, 1, z], "base:mossy_cobblestone");
    }
    d.done()
}

/// The cells around the edge of a rectangle, from `from` to `to` (x, z).
fn ring_cells(from: [usize; 2], to: [usize; 2]) -> impl Iterator<Item = (usize, usize)> {
    (from[1]..=to[1])
        .flat_map(move |z| (from[0]..=to[0]).map(move |x| (x, z)))
        .filter(move |&(x, z)| x == from[0] || x == to[0] || z == from[1] || z == to[1])
}

/// Stone bricks with moss and cobblestone mixed in, the same for the same `n`.
fn weathered(n: usize) -> &'static str {
    match n % 7 {
        0 | 4 => "base:mossy_cobblestone",
        2 => "base:cobblestone",
        _ => "base:stone_bricks",
    }
}
