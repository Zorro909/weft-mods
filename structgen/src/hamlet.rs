//! The hamlet: a well with cottages and wheat farms around it, joined to it by gravel paths. A
//! dynamic structure, built from the same files the single structures use.

use structgen_api::{Biomes, Fit, Foundation, Plan, PlanError, Turn};
use weft_sdk::*;

static WELL: Template = Template::named("well");
static COTTAGE: Template = Template::named("cottage");
static FARM: Template = Template::named("wheat_farm");
static GRAVEL: Block = Block::named("base:gravel");
static ANIMALS: Biomes = Biomes::named("#animals");

/// One hamlet at most per region of this many blocks square.
const REGION: u32 = 384;
const CHANCE: f32 = 0.5;
/// How far the well stays from the region's edges: room for the buildings around it.
const MARGIN: i32 = 64;
/// How far a building's middle may be from the well's, along x and along z.
const REACH: i32 = 40;
/// Spots tried for buildings, and how many buildings at most.
const TRIES: usize = 16;
const MOST_BUILDINGS: usize = 6;
/// How many of the buildings are cottages; the rest are farms.
const COTTAGES: f32 = 0.6;
/// Blocks kept free between two buildings, for the paths.
const GAP: i32 = 3;

pub fn register(r: &mut Registry) {
    structgen_api::plan(r, "hamlet", hamlet)
        .dimension(&base_api::OVERWORLD)
        .region(REGION);
}

fn hamlet(p: &mut Plan) -> Result<(), PlanError> {
    if !p.random().chance(CHANCE) {
        return Ok(());
    }
    let centre = p.spot(MARGIN);
    let Some(well) = well(p, centre)? else {
        return Ok(());
    };
    let mut areas = vec![well];
    for _ in 0..TRIES {
        if areas.len() > MOST_BUILDINGS {
            break;
        }
        if let Some(area) = building(p, centre, &areas)? {
            areas.push(area);
        }
    }
    for building in &areas[1..] {
        path(p, centre, building.middle(), &areas)?;
    }
    Ok(())
}

/// The well with its middle at `centre`, if the ground there suits a hamlet.
fn well(p: &mut Plan, centre: [i32; 2]) -> Result<Option<Area>, PlanError> {
    let Some([sx, _, sz]) = p.size(&WELL, Turn::None) else {
        return Ok(None);
    };
    let corner = [centre[0] - sx / 2, centre[1] - sz / 2];
    let fit = Fit::new().biome(&ANIMALS).max_slope(3);
    let Some(well) = p.fit(&WELL, corner, Turn::None, &fit) else {
        return Ok(None);
    };
    p.structure(&WELL, well, Turn::None)?
        .foundation(Foundation::Ground)
        .clear_above();
    Ok(Some(Area::of(corner, [sx, sz])))
}

/// Tries one building at a random spot around the well: a cottage with its door towards the
/// well, or a farm. `None` where it doesn't fit.
fn building(p: &mut Plan, centre: [i32; 2], areas: &[Area]) -> Result<Option<Area>, PlanError> {
    let dx = p.random().between(-REACH, REACH);
    let dz = p.random().between(-REACH, REACH);
    let (template, turn) = if p.random().chance(COTTAGES) {
        (&COTTAGE, door_towards(-dx, -dz))
    } else {
        let turn = p.random().turn();
        (&FARM, turn)
    };
    let Some([sx, _, sz]) = p.size(template, turn) else {
        return Ok(None);
    };
    let corner = [centre[0] + dx - sx / 2, centre[1] + dz - sz / 2];
    let area = Area::of(corner, [sx, sz]);
    if areas.iter().any(|other| other.near(&area, GAP)) {
        return Ok(None);
    }
    let fit = Fit::new().max_slope(4);
    let Some(corner) = p.fit(template, corner, turn, &fit) else {
        return Ok(None);
    };
    p.structure(template, corner, turn)?
        .foundation(Foundation::Ground)
        .clear_above();
    Ok(Some(area))
}

/// The turn that points a cottage's door, on its north side as saved, along (`dx`, `dz`).
fn door_towards(dx: i32, dz: i32) -> Turn {
    match (dz.abs() >= dx.abs(), dx > 0, dz > 0) {
        (true, _, false) => Turn::None,
        (true, _, true) => Turn::Back,
        (false, true, _) => Turn::Right,
        (false, false, _) => Turn::Left,
    }
}

/// Gravel along the ground from `from` to `to`: first across, then straight in, so it meets a
/// cottage at its door. Not inside any building.
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
        if let Some(ground) = p.ground(x, z) {
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
    fn doors_face_the_well() {
        assert_eq!(door_towards(0, -10), Turn::None, "north, as saved");
        assert_eq!(door_towards(10, 3), Turn::Right, "east");
        assert_eq!(door_towards(-2, 10), Turn::Back, "south");
        assert_eq!(door_towards(-10, 0), Turn::Left, "west");
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
