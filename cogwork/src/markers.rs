//! Survey markers: posts that mark out a rectangle for an excavator or a leveler. Three in an L
//! (a corner and one along each axis, at the same height) are enough; using one shows the
//! rectangle. A machine with no markers next to it works a square behind it.

use weft_sdk::*;

use crate::pos::Pos;

pub static SURVEY_MARKER: Block = Block::named("survey_marker");
static SURVEY_GLOW: Particle = Particle::named("survey_glow");

/// How far along a line a marker looks for the next one. It starts 2 blocks away: markers side
/// by side would leave an excavator's frame nothing inside.
const REACH: i32 = 64;

pub fn register(r: &mut Registry) {
    r.block(&SURVEY_MARKER)
        .texture("survey_marker")
        .model_box([7.0, 0.0, 7.0], [9.0, 9.0, 9.0])
        .model_box([6.0, 9.0, 6.0], [10.0, 13.0, 10.0])
        .hitbox([6.0, 0.0, 6.0], [10.0, 13.0, 10.0])
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

value! {
    /// A rectangle on the ground, by its lowest and highest corner at the same height.
    #[derive(Default)]
    pub struct Area {
        pub lo: Pos,
        pub hi: Pos,
    }
}

impl Area {
    /// The rectangle with the opposite corners `a` and `b`, at `a`'s height.
    pub fn between(a: Pos, b: Pos) -> Area {
        Area {
            lo: Pos::new(a.x.min(b.x), a.y, a.z.min(b.z)),
            hi: Pos::new(a.x.max(b.x), a.y, a.z.max(b.z)),
        }
    }

    /// Grows the area to take in `p`'s column.
    pub fn grow_to(&mut self, p: Pos) {
        let (lo, hi) = (self.lo, self.hi);
        self.lo = Pos::new(lo.x.min(p.x), lo.y, lo.z.min(p.z));
        self.hi = Pos::new(hi.x.max(p.x), hi.y, hi.z.max(p.z));
    }

    /// Whether `p` lies on, over or under the area, its edges included.
    pub fn spans(&self, p: Pos) -> bool {
        (self.lo.x..=self.hi.x).contains(&p.x) && (self.lo.z..=self.hi.z).contains(&p.z)
    }

    /// Blocks along x and along z, its edges included.
    pub fn size(&self) -> [i32; 2] {
        [self.hi.x - self.lo.x + 1, self.hi.z - self.lo.z + 1]
    }

    fn corners(&self) -> [Pos; 4] {
        let (lo, hi) = (self.lo, self.hi);
        [
            lo,
            Pos { x: hi.x, ..lo },
            Pos { z: hi.z, ..lo },
            hi.with_y(lo.y),
        ]
    }
}

/// The rectangle of markers that `pos` is a corner of, at `pos`'s height.
pub fn rectangle(w: &World, pos: Pos) -> Option<Area> {
    let marker_along = |step: fn(Pos, i32) -> Pos| {
        [1, -1].into_iter().find_map(|sign| {
            (2..=REACH)
                .map(|k| step(pos, sign * k))
                .find(|&p| SURVEY_MARKER.is(w.get(p.into())))
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
        if SURVEY_MARKER.is(w.get(corner.into())) {
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
        .find(|&p| SURVEY_MARKER.is(w.get(p.into())))
}

/// Using a marker draws its rectangle in the air for a few seconds; a marker without one puffs.
fn show(u: &mut Use) -> bool {
    static POOF: Particle = Particle::named("core:poof");
    let w = u.world();
    let pos = Pos::from(u.pos());
    match rectangle(&w, pos) {
        Some(area) => outline(&w, &area),
        None => drop(w.spawn_particles(&POOF, pos.center()).count(6)),
    }
    true
}

/// Glowing points along the edges of the rectangle, every half block.
fn outline(w: &World, area: &Area) {
    let [x0, _, z0] = area.lo.center();
    let [x1, _, z1] = area.hi.center();
    // A little above the middle of the markers.
    let y = area.lo.y as f64 + 0.6;
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

    #[test]
    fn an_area_is_the_same_from_either_pair_of_corners() {
        let a = Area::between(Pos::new(5, 2, -3), Pos::new(-1, 9, 4));
        assert_eq!(
            a,
            area([-1, 2, -3], [5, 2, 4]),
            "at the first corner's height"
        );
        assert_eq!(a.size(), [7, 8], "edges included");
        let corners = a.corners().map(|c| [c.x, c.z]);
        assert_eq!(
            corners,
            [[-1, -3], [5, -3], [-1, 4], [5, 4]],
            "four corners"
        );
        assert!(
            corners.iter().all(|&[x, z]| a.spans(Pos::new(x, 99, z))),
            "spanned"
        );
    }

    #[test]
    fn an_area_grows_to_take_in_a_column() {
        let mut a = area([0, 0, 0], [2, 0, 2]);
        a.grow_to(Pos::new(-3, 7, 5));
        assert_eq!(a, area([-3, 0, 0], [2, 0, 5]), "out both ways, not up");
        assert!(a.spans(Pos::new(-3, 99, 5)), "a corner, at any height");
        assert!(!a.spans(Pos::new(3, 0, 0)), "not past its edge");
    }
}
