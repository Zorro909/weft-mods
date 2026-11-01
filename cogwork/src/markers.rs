//! Survey markers: posts that mark out a rectangle for an excavator. Three in an L (a corner and
//! one along each axis, at the same height) are enough; using one shows the rectangle.

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
        .collision([6.0, 0.0, 6.0], [10.0, 13.0, 10.0])
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

/// A marker next to `pos`, if any, on the four sides.
pub fn beside(w: &World, pos: [i32; 3]) -> Option<[i32; 3]> {
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
