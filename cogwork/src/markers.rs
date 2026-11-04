//! Survey markers: posts that mark out a rectangle for an excavator or a leveler. Three in an L
//! (a corner and one along each axis, at the same height) are enough; using one shows the
//! rectangle. A machine with no markers next to it works a square behind it.

use weft_sdk::*;

use crate::lib::{Area, BlockState, FloatPos, ModelBox, Pos};

pub static SURVEY_MARKER: Block = Block::named("survey_marker");
static SURVEY_GLOW: Particle = Particle::named("survey_glow");
/// A marker: a thin post with a wider head on top, and the box round both that is aimed at and
/// bumped into.
const POST: ModelBox = ModelBox::new([7.0, 0.0, 7.0], [9.0, 9.0, 9.0]);
const HEAD: ModelBox = ModelBox::new([6.0, 9.0, 6.0], [10.0, 13.0, 10.0]);
const HITBOX: ModelBox = ModelBox::new([6.0, 0.0, 6.0], [10.0, 13.0, 10.0]);

/// How far along a line a marker looks for the next one. It starts 2 blocks away: markers side
/// by side would leave an excavator's frame nothing inside.
const REACH: i32 = 64;

pub fn register(r: &mut Registry) {
    r.block(&SURVEY_MARKER)
        .texture("survey_marker")
        .model_box(POST.lo, POST.hi)
        .model_box(HEAD.lo, HEAD.hi)
        .hitbox(HITBOX.lo, HITBOX.hi)
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

/// The rectangle of markers that `pos` is a corner of, at `pos`'s height.
pub fn rectangle(w: &World, pos: Pos) -> Option<Area> {
    let marker_along = |step: fn(Pos, i32) -> Pos| {
        [1, -1].into_iter().find_map(|sign| {
            (2..=REACH)
                .map(|k| step(pos, sign * k))
                .find(|&p| BlockState::at(w, p).is(&SURVEY_MARKER))
        })
    };
    let (x, z) = (marker_along(Pos::add_x)?, marker_along(Pos::add_z)?);
    Some(Area::between(pos, Pos { x: x.x, ..z }))
}

/// The rectangle of the markers next to a machine at `pos`, when the machine is outside it, at
/// the markers' height. The machine takes the markers down.
pub fn take_area(w: &World, pos: Pos) -> Option<Area> {
    let area = rectangle(w, beside(w, pos)?)?;
    if area.spans(pos) {
        return None;
    }
    for corner in area.corners() {
        if BlockState::at(w, corner).is(&SURVEY_MARKER) {
            w.break_block(corner.into(), ItemStack::EMPTY, 0, true);
        }
    }
    Some(area)
}

/// A square of `size` blocks right behind a machine at `pos` facing `front`, `margin` more on
/// every side, at `pos`'s height.
pub fn square_behind(pos: Pos, front: Dir, size: i32, margin: i32) -> Area {
    // Across is any horizontal way at right angles: the square is the same either way round.
    let across = if front == Dir::West || front == Dir::East {
        Dir::North
    } else {
        Dir::West
    };
    let back = front.opposite();
    let half = size / 2 + margin;
    let corner = |depth: i32, sideways: i32| pos.toward_by(back, depth).toward_by(across, sideways);
    Area::between(corner(1, -half), corner(size + 2 * margin, half))
}

/// A marker next to `pos`, if any, on the four sides.
fn beside(w: &World, pos: Pos) -> Option<Pos> {
    [Dir::West, Dir::East, Dir::North, Dir::South]
        .map(|d| pos.toward(d))
        .into_iter()
        .find(|&p| BlockState::at(w, p).is(&SURVEY_MARKER))
}

/// Using a marker draws its rectangle in the air for a few seconds; a marker without one puffs.
fn show(u: &mut Use) -> bool {
    static POOF: Particle = Particle::named("core:poof");
    let w = u.world();
    let pos = Pos::from(u.pos());
    match rectangle(&w, pos) {
        Some(area) => {
            outline(&w, &area);
            w.play_sound(&crate::sounds::SURVEY, pos.center().into(), 1.0);
        }
        None => drop(w.spawn_particles(&POOF, pos.center().into()).count(6)),
    }
    true
}

/// Glowing points along the edges of the rectangle, every half block.
fn outline(w: &World, area: &Area) {
    // A little above the middle of the markers.
    let (lo, hi) = (area.lo.center().add_y(0.1), area.hi.center().add_y(0.1));
    let mut points = Vec::new();
    let steps = |a: f64, b: f64| (0..=((b - a) * 2.0) as i32).map(move |i| i as f64 * 0.5);
    for along in steps(lo.x, hi.x) {
        points.extend([lo.add_x(along), FloatPos { z: hi.z, ..lo }.add_x(along)]);
    }
    for along in steps(lo.z, hi.z) {
        points.extend([lo.add_z(along), FloatPos { x: hi.x, ..lo }.add_z(along)]);
    }
    for p in points {
        drop(w.spawn_particles(&SURVEY_GLOW, p.into()).count(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(lo: [i32; 3], hi: [i32; 3]) -> Area {
        Area::between(lo.into(), hi.into())
    }

    #[test]
    fn the_square_starts_right_behind_the_machine() {
        let pos = Pos::new(0, 4, 10);
        assert_eq!(
            square_behind(pos, Dir::South, 9, 1),
            area([-5, 4, -1], [5, 4, 9]),
            "11 wide, from one block behind"
        );
        assert_eq!(
            square_behind(pos, Dir::East, 9, 0),
            area([-9, 4, 6], [-1, 4, 14]),
            "9 by 9 to the west"
        );
    }
}
