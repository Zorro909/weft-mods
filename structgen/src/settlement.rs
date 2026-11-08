//! A settlement laid out around a centre: buildings at random spots near it, most turned to
//! face it, joined to it by gravel paths. What a settlement holds is data (`Settlement`); this
//! file is the planner every kind shares.

use structgen_api::{Biomes, Fit, Foundation, Materials, Plan, PlanError, Turn};
use weft_sdk::*;

use crate::styles;
use crate::worldgen::{OPEN, SEA};

static GRAVEL: Block = Block::named("base:gravel");
static OPEN_LAND: Biomes = Biomes::named(OPEN);

/// Spots tried for each building before it is left out.
const TRIES: usize = 12;
/// Blocks kept free between two buildings, for the paths.
const GAP: i32 = 3;

/// One kind of settlement.
pub struct Settlement {
    /// One per region of this many blocks square, at most.
    pub region: u32,
    pub chance: f32,
    /// How far the centre stays from the region's edges: room for the buildings around it.
    pub margin: i32,
    pub centre: &'static Template,
    /// Its buildings, placed in this order: the ones that matter most first.
    pub wants: &'static [Want],
    /// The roofs its buildings get, one picked for each.
    pub roofs: &'static [&'static str],
    /// The frames and infills it is built with, one of each picked for the whole settlement.
    pub frames: &'static [&'static str],
    pub infills: &'static [&'static str],
}

/// Some buildings of a settlement.
pub struct Want {
    /// One of these for each building, at random.
    pub templates: &'static [&'static Template],
    pub count: usize,
    /// How far its middle is from the centre's: at least `near` along x or z, at most `far`
    /// along both. `near` is at most `far`.
    pub near: i32,
    pub far: i32,
    /// Turned so its front (north as saved) faces the centre; otherwise any way.
    pub faces_centre: bool,
}

/// Lays out one settlement in the region, if the seed and the ground there want one.
pub fn plan(p: &mut Plan, settlement: &Settlement) -> Result<(), PlanError> {
    if !p.random().chance(settlement.chance) {
        return Ok(());
    }
    let centre = p.spot(settlement.margin);
    let Some(middle) = place_centre(p, settlement.centre, centre)? else {
        return Ok(());
    };
    let look = Look::pick(p, settlement);
    let mut areas = vec![middle];
    for want in settlement.wants {
        for _ in 0..want.count {
            let template = want.templates[p.random().below(want.templates.len() as u64) as usize];
            if let Some(area) = building(p, want, template, centre, &areas, &look)? {
                areas.push(area);
            }
        }
    }
    for building in &areas[1..] {
        path(p, centre, building.middle(), &areas)?;
    }
    Ok(())
}

/// The centre with its middle at `centre`, if the ground there suits a settlement.
fn place_centre(
    p: &mut Plan,
    template: &Template,
    centre: [i32; 2],
) -> Result<Option<Area>, PlanError> {
    let Some([sx, _, sz]) = p.size(template, Turn::None) else {
        return Ok(None);
    };
    let corner = [centre[0] - sx / 2, centre[1] - sz / 2];
    let fit = Fit::new().biome(&OPEN_LAND).max_slope(3).min_height(SEA);
    let Some(at) = p.fit(template, corner, Turn::None, &fit) else {
        return Ok(None);
    };
    p.structure(template, at, Turn::None)?
        .foundation(Foundation::Ground)
        .clear_above();
    Ok(Some(Area::of(corner, [sx, sz])))
}

/// Tries `template` at up to `TRIES` spots around the centre. `None` where none suits it.
fn building(
    p: &mut Plan,
    want: &Want,
    template: &Template,
    centre: [i32; 2],
    areas: &[Area],
    look: &Look,
) -> Result<Option<Area>, PlanError> {
    for _ in 0..TRIES {
        let [dx, dz] = offset(p, want.near, want.far);
        let turn = match want.faces_centre {
            true => front_towards(-dx, -dz),
            false => p.random().turn(),
        };
        let Some([sx, _, sz]) = p.size(template, turn) else {
            return Ok(None);
        };
        let corner = [centre[0] + dx - sx / 2, centre[1] + dz - sz / 2];
        let area = Area::of(corner, [sx, sz]);
        if areas.iter().any(|other| other.near(&area, GAP)) {
            continue;
        }
        let fit = Fit::new().biome(&OPEN_LAND).max_slope(4).min_height(SEA);
        let Some(at) = p.fit(template, corner, turn, &fit) else {
            continue;
        };
        let materials = look.for_building(p);
        p.structure(template, at, turn)?
            .materials(&materials)
            .foundation(Foundation::Ground)
            .clear_above();
        return Ok(Some(area));
    }
    Ok(None)
}

/// A random offset at least `near` from the centre along x or z, at most `far` along both.
fn offset(p: &mut Plan, near: i32, far: i32) -> [i32; 2] {
    loop {
        let dx = p.random().between(-far, far);
        let dz = p.random().between(-far, far);
        if dx.abs().max(dz.abs()) >= near {
            return [dx, dz];
        }
    }
}

/// The turn that points a building's front, north as saved, along (`dx`, `dz`).
fn front_towards(dx: i32, dz: i32) -> Turn {
    match (dz.abs() >= dx.abs(), dx > 0, dz > 0) {
        (true, _, false) => Turn::None,
        (true, _, true) => Turn::Back,
        (false, true, _) => Turn::Right,
        (false, false, _) => Turn::Left,
    }
}

/// What a settlement is built of: one frame and infill for all of it, roofs per building.
struct Look {
    walls: Materials,
    roofs: &'static [&'static str],
}

impl Look {
    fn pick(p: &mut Plan, settlement: &Settlement) -> Look {
        let frame = styles::blocks(one_of(p, settlement.frames));
        let infill = styles::blocks(one_of(p, settlement.infills));
        let walls = styles::add(styles::add(Materials::new(), frame), infill);
        Look {
            walls,
            roofs: settlement.roofs,
        }
    }

    fn for_building(&self, p: &mut Plan) -> Materials {
        let roof = styles::blocks(one_of(p, self.roofs));
        styles::add(self.walls.clone(), roof)
    }
}

/// One of `names` at random, `""` (the defaults) when there are none.
fn one_of(p: &mut Plan, names: &[&'static str]) -> &'static str {
    match names.len() {
        0 => "",
        n => names[p.random().below(n as u64) as usize],
    }
}

/// Gravel along the ground from `from` to `to`: first across, then straight in, so it meets a
/// building facing the centre at its front. Not inside any building, nor under water.
fn path(p: &mut Plan, from: [i32; 2], to: [i32; 2], areas: &[Area]) -> Result<(), PlanError> {
    let across_x = (to[0] - from[0]).abs() <= (to[1] - from[1]).abs();
    let bend = if across_x {
        [to[0], from[1]]
    } else {
        [from[0], to[1]]
    };
    for [x, z] in line(from, bend).chain(line(bend, to)) {
        if areas.iter().any(|area| area.contains(x, z)) {
            continue;
        }
        if let Some(ground) = p.ground(x, z).filter(|ground| ground.height >= SEA) {
            p.set([x, ground.height, z], &GRAVEL)?;
        }
    }
    Ok(())
}

/// The columns from `from` to `to` along one axis, both included.
fn line(from: [i32; 2], to: [i32; 2]) -> impl Iterator<Item = [i32; 2]> {
    let steps = (to[0] - from[0]).abs().max((to[1] - from[1]).abs());
    let step = [(to[0] - from[0]).signum(), (to[1] - from[1]).signum()];
    (0..=steps).map(move |i| [from[0] + step[0] * i, from[1] + step[1] * i])
}

/// A building's footprint seen from above, both corners included.
#[derive(Clone, Copy, Debug)]
struct Area {
    from: [i32; 2],
    to: [i32; 2],
}

impl Area {
    fn of(corner: [i32; 2], [sx, sz]: [i32; 2]) -> Area {
        Area {
            from: corner,
            to: [corner[0] + sx - 1, corner[1] + sz - 1],
        }
    }

    fn contains(&self, x: i32, z: i32) -> bool {
        (self.from[0]..=self.to[0]).contains(&x) && (self.from[1]..=self.to[1]).contains(&z)
    }

    fn middle(&self) -> [i32; 2] {
        [
            (self.from[0] + self.to[0]) / 2,
            (self.from[1] + self.to[1]) / 2,
        ]
    }

    /// Whether `other` comes within `gap` blocks of it.
    fn near(&self, other: &Area, gap: i32) -> bool {
        (0..2).all(|i| other.from[i] <= self.to[i] + gap && self.from[i] <= other.to[i] + gap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fronts_face_the_centre() {
        assert_eq!(front_towards(0, -10), Turn::None, "north, as saved");
        assert_eq!(front_towards(10, 3), Turn::Right, "east");
        assert_eq!(front_towards(-2, 10), Turn::Back, "south");
        assert_eq!(front_towards(-10, 0), Turn::Left, "west");
    }

    #[test]
    fn buildings_keep_their_gap() {
        let house = Area::of([0, 0], [5, 5]);
        assert!(
            house.near(&Area::of([7, 0], [3, 3]), GAP),
            "two blocks between"
        );
        assert!(
            !house.near(&Area::of([8, 0], [3, 3]), GAP),
            "three blocks between"
        );
        assert!(!house.near(&Area::of([0, -10], [5, 5]), GAP), "far along z");
    }

    #[test]
    fn a_line_includes_both_ends() {
        let cells: Vec<[i32; 2]> = line([2, 5], [-1, 5]).collect();
        assert_eq!(cells, [[2, 5], [1, 5], [0, 5], [-1, 5]], "walking west");
    }
}
