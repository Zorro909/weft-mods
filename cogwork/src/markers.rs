//! Survey markers: posts that mark out a rectangle for an excavator or a leveler. Three in an L
//! (a corner and one along each axis, at the same height) are enough; using one shows the
//! rectangle. A machine with no markers next to it works a square behind it.

use weft_sdk::*;

pub static SURVEY_MARKER: Block = Block::named("survey_marker");
static SURVEY_GLOW: Particle = Particle::named("survey_glow");

/// How far along a line a marker looks for the next one.
const REACH: i32 = 64;

pub fn register(r: &mut Registry) {
    r.block(&SURVEY_MARKER)
        .texture("survey_marker")
        .model_box([7.0, 0.0, 7.0], [9.0, 9.0, 9.0])
        .model_box([6.0, 9.0, 6.0], [10.0, 13.0, 10.0])
        .hitbox([0.375, 0.0, 0.375], [0.625, 0.8125, 0.625])
        .needs_below("")
        .hardness(0.0)
        .light(4)
        .item_texture("survey_marker")
        .on_use(show)
        .tab(&crate::TAB);
    r.particle(&SURVEY_GLOW)
        .textures(&["core:glint_0"])
        .life(2.5, 3.0)
        .size(0.15, 0.15)
        .gravity(0.0)
        .drag(1.0)
        .color(0xFF4030FF, 0xFF403000)
        .fullbright()
        .translucent();
}

/// The rectangle of markers that `pos` is a corner of: its lowest and highest corner, the same
/// height as `pos`.
pub fn rectangle(w: &World, pos: [i32; 3]) -> Option<([i32; 3], [i32; 3])> {
    let along = |axis: usize| {
        [1, -1].into_iter().find_map(|sign| {
            (2..=REACH).find_map(|k| {
                let mut p = pos;
                p[axis] += sign * k;
                SURVEY_MARKER.is(w.get(p)).then_some(p[axis])
            })
        })
    };
    let (x, z) = (along(0)?, along(2)?);
    Some((
        [pos[0].min(x), pos[1], pos[2].min(z)],
        [pos[0].max(x), pos[1], pos[2].max(z)],
    ))
}

/// The rectangle of the markers next to a machine at `pos`, when the machine is outside it: its
/// lowest and highest corner at the markers' height. The machine takes the markers down.
pub fn take_area(w: &World, pos: [i32; 3]) -> Option<([i32; 3], [i32; 3])> {
    let (lo, hi) = rectangle(w, beside(w, pos)?)?;
    let outside = !(lo[0]..=hi[0]).contains(&pos[0]) || !(lo[2]..=hi[2]).contains(&pos[2]);
    if !outside {
        return None;
    }
    for x in [lo[0], hi[0]] {
        for z in [lo[2], hi[2]] {
            let p = [x, lo[1], z];
            if SURVEY_MARKER.is(w.get(p)) {
                w.break_block(p, ItemStack::EMPTY, 0, true);
            }
        }
    }
    Some((lo, hi))
}

/// A square of `size` blocks right behind a machine at `pos` facing `front`, `margin` more on
/// every side: its lowest and highest corner at `pos`'s height.
pub fn square_behind(pos: [i32; 3], front: Dir, size: i32, margin: i32) -> ([i32; 3], [i32; 3]) {
    // Across is any horizontal way at right angles: the square is the same either way round.
    let across = if front == Dir::West || front == Dir::East {
        Dir::North
    } else {
        Dir::West
    };
    let (back, side) = (front.opposite().offset(), across.offset());
    let half = size / 2 + margin;
    let corner = |depth: i32, across: i32| {
        [
            pos[0] + back[0] * depth + side[0] * across,
            pos[1],
            pos[2] + back[2] * depth + side[2] * across,
        ]
    };
    let (a, b) = (corner(1, -half), corner(size + 2 * margin, half));
    (
        [a[0].min(b[0]), pos[1], a[2].min(b[2])],
        [a[0].max(b[0]), pos[1], a[2].max(b[2])],
    )
}

/// A marker next to `pos`, if any, on the four sides.
fn beside(w: &World, pos: [i32; 3]) -> Option<[i32; 3]> {
    [Dir::West, Dir::East, Dir::North, Dir::South]
        .into_iter()
        .find_map(|d| {
            let o = d.offset();
            let p = [pos[0] + o[0], pos[1], pos[2] + o[2]];
            SURVEY_MARKER.is(w.get(p)).then_some(p)
        })
}

/// Using a marker draws its rectangle in the air for a few seconds; a marker without one puffs.
fn show(u: &mut Use) -> bool {
    static POOF: Particle = Particle::named("core:poof");
    let w = u.world();
    match rectangle(&w, u.pos()) {
        Some((lo, hi)) => outline(&w, lo, hi),
        None => drop(
            w.spawn_particles(&POOF, u.pos().map(|v| v as f64 + 0.5))
                .count(6),
        ),
    }
    true
}

/// Glowing points along the edges of the rectangle, every half block.
fn outline(w: &World, lo: [i32; 3], hi: [i32; 3]) {
    let y = lo[1] as f64 + 0.6;
    let (x0, x1, z0, z1) = (
        lo[0] as f64 + 0.5,
        hi[0] as f64 + 0.5,
        lo[2] as f64 + 0.5,
        hi[2] as f64 + 0.5,
    );
    let mut points = Vec::new();
    let steps = |a: f64, b: f64| (0..=((b - a) * 2.0) as i32).map(move |i| a + i as f64 * 0.5);
    for x in steps(x0, x1) {
        points.extend([[x, y, z0], [x, y, z1]]);
    }
    for z in steps(z0, z1) {
        points.extend([[x0, y, z], [x1, y, z]]);
    }
    for p in points {
        drop(w.spawn_particles(&SURVEY_GLOW, p).count(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_square_starts_right_behind_the_machine() {
        let (lo, hi) = square_behind([0, 4, 10], Dir::South, 9, 1);
        assert_eq!(
            (lo, hi),
            ([-5, 4, -1], [5, 4, 9]),
            "11 wide, from one block behind"
        );
        let (lo, hi) = square_behind([0, 4, 10], Dir::East, 9, 0);
        assert_eq!((lo, hi), ([-9, 4, 6], [-1, 4, 14]), "9 by 9 to the west");
    }
}
