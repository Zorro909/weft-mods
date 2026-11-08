//! The roofs of the burgher houses that the kit's `gable_roof` doesn't draw, and the gables
//! under them.

use crate::draft::{AIR, Draft};
use crate::kit::{slab, slot, stair};

/// Fills column (`x`, `z`) with `name` from height `y` up to the first block above: a gable
/// under a roof drawn already.
pub fn fill_up(d: &mut Draft, [x, z]: [usize; 2], mut y: usize, name: &str) {
    while y < d.size()[1] && d.get([x, y, z]) == AIR {
        d.set([x, y, z], name);
        y += 1;
    }
}

/// A roof twice as steep as the kit's, its ridge north to south, from `x[0]` to `x[1]` (eaves
/// included, an odd width) the full length `z`, its eaves at height `y`: each step in is a
/// `roof/block` under a `roof/stair`, and a `roof/slab` tops the ridge.
pub fn steep_roof(d: &mut Draft, x: [usize; 2], z: [usize; 2], y: usize) {
    let half = (x[1] - x[0]) / 2;
    for along in z[0]..=z[1] {
        for k in 0..=half {
            let top = y + 2 * k;
            if k > 0 {
                d.set([x[0] + k, top - 1, along], &slot("roof/block"));
                d.set([x[1] - k, top - 1, along], &slot("roof/block"));
            }
            if k == half {
                d.set([x[0] + k, top, along], &slab("roof/slab", "bottom"));
            } else {
                d.set(
                    [x[0] + k, top, along],
                    &stair("roof/stair", "east", "bottom"),
                );
                d.set(
                    [x[1] - k, top, along],
                    &stair("roof/stair", "west", "bottom"),
                );
            }
        }
    }
}

/// Two 45° roofs crossing over a square from 0 to `2 * half` (eaves included), eaves at height
/// `y`: one with its ridge north to south over all of it, one with its ridge east to west over
/// the east half. Gables face north, south and east; valleys run in from the east corners.
pub fn cross_roof(d: &mut Draft, half: usize, y: usize) {
    let end = 2 * half;
    for z in 0..=end {
        for x in 0..=end {
            let up_x = x.min(end - x);
            let up_z = z.min(end - z);
            let (rise, name) = if x <= half || up_x > up_z {
                (up_x, roof_piece(x, half, ["east", "west"], "straight"))
            } else if up_z > up_x {
                (up_z, roof_piece(z, half, ["south", "north"], "straight"))
            } else if z < half {
                (up_x, roof_piece(z, half, ["south", "north"], "inner_right"))
            } else {
                (up_x, roof_piece(z, half, ["south", "north"], "inner_left"))
            };
            d.set([x, y + rise, z], &name);
        }
    }
}

/// The piece of a 45° roof at `at` across it, the ridge at `half`: a slab on the ridge, else a
/// stair facing `facings[0]` before the ridge and `facings[1]` after it, of `shape`.
fn roof_piece(at: usize, half: usize, facings: [&str; 2], shape: &str) -> String {
    if at == half {
        return slab("roof/slab", "bottom");
    }
    let facing = facings[usize::from(at > half)];
    format!("@roof/stair[facing={facing},half=bottom,shape={shape}]")
}
