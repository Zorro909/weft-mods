//! Survey markers: posts that mark out a rectangle for an excavator or a leveler. Three in an L
//! (a corner and one along each axis, at the same height) are enough; using one shows the
//! rectangle. A machine with no markers next to it works a square behind it. How far apart
//! markers may stand is the world's setting `marker_max_area`.

use weft_sdk::{Block, Config, Dir, ItemStack, Output, Particle, Registry, Use, Vec3, World};

use crate::lib::{Area, Pos, TypedWorld};

pub static SURVEY_MARKER: Block = Block::named("survey_marker");
static SURVEY_GLOW: Particle = Particle::named("survey_glow");
/// The two corners, in texels, of the box that is aimed at and bumped into: round the thin post
/// and the wider head on it, which are the Blockbench model `models/block/survey_marker.json`.
const HITBOX: ([f32; 3], [f32; 3]) = ([6.0, 0.0, 6.0], [10.0, 13.0, 10.0]);

/// The world's largest rectangle of markers, as an index into `MAX_AREAS`.
static MAX_AREA: Config<u32> = Config::named("marker_max_area");
/// The choices of `MAX_AREA`: its name, and how far along a line a marker looks for the next
/// one, in blocks. The search starts 2 blocks away: markers side by side would leave an
/// excavator's frame nothing inside.
const MAX_AREAS: [(&str, i32); 4] = [("32", 32), ("64", 64), ("128", 128), ("256", 256)];
/// The choice a new world starts with: 64.
const DEFAULT_MAX_AREA: u32 = 1;
/// The furthest any choice lets markers stand apart.
pub const LARGEST_REACH: i32 = MAX_AREAS[MAX_AREAS.len() - 1].1;

pub fn register(r: &mut Registry) {
    r.block(&SURVEY_MARKER)
        .hitbox(HITBOX.0, HITBOX.1)
        .needs_below("")
        .hardness(0.0)
        .light(4)
        .item_texture("survey_marker")
        .on_use(show)
        .tab(&crate::TAB);
    r.config(&MAX_AREA)
        .choice(DEFAULT_MAX_AREA, &MAX_AREAS.map(|(name, _)| name))
        .world();
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

/// How far apart markers may stand in this world, in blocks.
fn reach() -> i32 {
    reach_of(MAX_AREA.get())
}

/// The reach of the `choice` of `MAX_AREA`; one from a newer cogwork reads as the default.
fn reach_of(choice: u32) -> i32 {
    let (_, reach) = MAX_AREAS
        .get(choice as usize)
        .unwrap_or(&MAX_AREAS[DEFAULT_MAX_AREA as usize]);
    *reach
}

/// The rectangle of markers that `pos` is a corner of, at `pos`'s height.
pub fn rectangle(world: &World, pos: Pos) -> Option<Area> {
    let reach = reach();
    let marker_along = |step: fn(Pos, i32) -> Pos| {
        [1, -1].into_iter().find_map(|sign| {
            (2..=reach)
                .map(|blocks| step(pos, sign * blocks))
                .find(|&at| world.block_typed(at).is(&SURVEY_MARKER))
        })
    };
    let (x, z) = (marker_along(Pos::add_x)?, marker_along(Pos::add_z)?);
    Some(Area::between(pos, Pos { x: x.x, ..z }))
}

/// The rectangle of the markers next to a machine at `pos`, when the machine is outside it, at
/// the markers' height. The machine takes the markers down.
pub fn take_area(world: &World, pos: Pos) -> Option<Area> {
    let area = rectangle(world, beside(world, pos)?)?;
    if area.spans(pos) {
        return None;
    }
    for corner in area.bottom_corners() {
        if world.block_typed(corner).is(&SURVEY_MARKER) {
            world.break_block(corner, ItemStack::EMPTY, 0, true);
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
fn beside(world: &World, pos: Pos) -> Option<Pos> {
    [Dir::West, Dir::East, Dir::North, Dir::South]
        .map(|dir| pos.toward(dir))
        .into_iter()
        .find(|&at| world.block_typed(at).is(&SURVEY_MARKER))
}

fn show(click: &mut Use) -> bool {
    survey(&click.world(), Pos::from(click.pos()));
    true
}

/// Using the marker at `pos`: its rectangle drawn in the air for a few seconds, or a puff from a
/// marker without one.
pub fn survey(world: &World, pos: Pos) {
    static POOF: Particle = Particle::named("core:poof");
    match rectangle(world, pos) {
        Some(area) => {
            outline(world, &area);
            world.play_sound(&crate::sounds::SURVEY, pos.center(), 1.0);
        }
        None => drop(world.spawn_particles(&POOF, pos.center()).count(6)),
    }
}

/// Glowing points along the edges of the rectangle, every half block.
fn outline(world: &World, area: &Area) {
    // A little above the middle of the markers.
    let (lo, hi) = (
        area.lo.center().offset(0.0, 0.1, 0.0),
        area.hi.center().offset(0.0, 0.1, 0.0),
    );
    let mut points = Vec::new();
    let steps = |a: f64, b: f64| (0..=((b - a) * 2.0) as i32).map(move |i| i as f64 * 0.5);
    for along in steps(lo.x, hi.x) {
        let x = lo.x + along;
        points.extend([Vec3 { x, ..lo }, Vec3 { x, z: hi.z, ..lo }]);
    }
    for along in steps(lo.z, hi.z) {
        let z = lo.z + along;
        points.extend([Vec3 { z, ..lo }, Vec3 { z, x: hi.x, ..lo }]);
    }
    for point in points {
        drop(world.spawn_particles(&SURVEY_GLOW, point).count(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_marker_has_its_model() {
        let model = "models/block/survey_marker.json";
        assert!(crate::has_asset(model), "no {model}");
    }

    #[test]
    fn every_largest_area_has_a_name() {
        assert!(
            crate::translated("config.cogwork.marker_max_area"),
            "the setting's name"
        );
        for (name, _) in MAX_AREAS {
            let key = format!("config.cogwork.marker_max_area.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn markers_reach_as_far_as_the_world_allows() {
        assert_eq!(
            reach_of(DEFAULT_MAX_AREA),
            64,
            "64 blocks, as before the setting"
        );
        assert_eq!(reach_of(3), 256, "the largest");
        assert_eq!(reach_of(99), 64, "a choice from a newer cogwork");
    }

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
