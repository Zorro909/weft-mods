//! Trees and bushes for greens, orchards and gardens: the green between a settlement's
//! buildings. Their bottom layer is the trunk and the ground around it.

use structgen_api::Structure;

use crate::draft::Draft;
use crate::kit::{Building, slot};

pub const BUILDINGS: &[Building] = &[
    ("oak_tree", oak_tree),
    ("oak_tree_large", oak_tree_large),
    ("birch_tree", birch_tree),
    ("fruit_tree", fruit_tree),
    ("bush", bush),
];

/// A trunk 4 high under a round crown 5 across.
fn oak_tree() -> Structure {
    let mut d = tree([5, 7, 5], "base:oak_log[axis=y]", 4);
    crown(&mut d, "base:oak_leaves", [2, 2], [3, 6], &[2, 2, 1, 0]);
    d.done()
}

/// A village green's old oak: a trunk 6 high, branches out to a crown 9 across.
fn oak_tree_large() -> Structure {
    let mut d = tree([9, 10, 9], "base:oak_log[axis=y]", 6);
    for (x, z, axis) in [(3, 4, "x"), (5, 4, "x"), (4, 3, "z"), (4, 5, "z")] {
        d.set([x, 5, z], &format!("base:oak_log[axis={axis}]"));
    }
    crown(
        &mut d,
        "base:oak_leaves",
        [4, 4],
        [4, 9],
        &[3, 4, 4, 3, 2, 1],
    );
    d.done()
}

/// Slim and tall: a trunk 6 high in a narrow crown.
fn birch_tree() -> Structure {
    let mut d = tree([5, 8, 5], "base:birch_log[axis=y]", 6);
    crown(
        &mut d,
        "base:birch_leaves",
        [2, 2],
        [3, 7],
        &[1, 2, 2, 1, 0],
    );
    d.done()
}

/// An orchard's tree: low, a short trunk under a wide flat crown.
fn fruit_tree() -> Structure {
    let mut d = tree([5, 5, 5], "base:oak_log[axis=y]", 2);
    crown(&mut d, "base:oak_leaves", [2, 2], [2, 4], &[2, 2, 1]);
    d.done()
}

/// A round bush of leaves on the ground.
fn bush() -> Structure {
    let mut d = Draft::new([3, 3, 3]);
    d.fill([0, 0, 0], [2, 0, 2], &slot("ground"));
    crown(&mut d, "base:oak_leaves", [1, 1], [1, 2], &[1, 0]);
    d.done()
}

/// The ground under a box of `size` with a trunk of `log` up its middle, `height` high.
fn tree(size: [usize; 3], log: &str, height: usize) -> Draft {
    let mut d = Draft::new(size);
    let middle = [size[0] / 2, size[2] / 2];
    d.fill([0, 0, 0], [size[0] - 1, 0, size[2] - 1], &slot("ground"));
    d.fill(
        [middle[0], 0, middle[1]],
        [middle[0], height, middle[1]],
        log,
    );
    d
}

/// Leaves around column `middle` from height `y[0]` to `y[1]`, each layer a disc of the radius
/// `radii` gives it, the corners left off; the trunk stays where it is.
fn crown(d: &mut Draft, leaves: &str, middle: [usize; 2], y: [usize; 2], radii: &[usize]) {
    for (layer, &radius) in (y[0]..=y[1]).zip(radii) {
        for dx in 0..=radius * 2 {
            for dz in 0..=radius * 2 {
                let (ox, oz) = (dx.abs_diff(radius), dz.abs_diff(radius));
                let corner = radius > 0 && ox == radius && oz == radius;
                let pos = [middle[0] + dx - radius, layer, middle[1] + dz - radius];
                if !corner && d.get(pos) == crate::draft::AIR {
                    d.set(pos, leaves);
                }
            }
        }
    }
}
