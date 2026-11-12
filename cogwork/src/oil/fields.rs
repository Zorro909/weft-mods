//! Where oil lies. A field is an engine feature on a grid: the overworld is split into squares
//! `SPACING` blocks wide, and one in `SQUARES_TO_A_FIELD` of those whose place is dry dirt or
//! sand holds a field. The engine lays each one out once, from the seed and the ground alone,
//! and every cube it reaches takes its part. A field is a ball of oil sources deep underground,
//! and above its middle a seep: a puddle of oil in oil-soaked ground.

use base_api::OVERWORLD;
use weft_sdk::{Feature, Placed, Plan, Registry, abi::stage};

use super::{OIL, OIL_SAND, OILY_DIRT};
use crate::lib::{Area, Pos};

static OIL_FIELD: Feature = Feature::named("oil_field");
/// Base's grass, flowers and canes: a field comes after them, to clear them off its seep.
static BASE_PLANTS: Feature = Feature::named("base:decoration");

/// Blocks along each side of a square of the grid, and the squares there are to one field.
const SPACING: i32 = 96;
const SQUARES_TO_A_FIELD: f32 = 32.0;
/// The radius of a field's ball, in blocks: 4 holds 389 buckets, 5 holds 739, 6 holds 1,189.
const RADIUS: [i32; 2] = [4, 6];
/// How far under its puddle a field's middle lies. Under ground at sea level the deepest ball
/// ends at -28, above the overworld's floor at -32.
const DEPTH: [i32; 2] = [45, 85];
/// How far from its puddle a seep soaks the ground, counted along x and z together.
const SEEP_REACH: i32 = 2;
/// How far above or below the puddle a seep still soaks the ground: a cliff beside it stays dry.
const SEEP_STEP: i32 = 3;
/// How high above soaked ground plants are taken away: a sugar cane's height.
const PLANT_HEIGHT: i32 = 3;
/// Ground under the sea stays as it is: base's sea level.
const SEA: i32 = 63;
/// What a field's ball takes: everything but the air of a cave.
const NOT_AIR: &str = "!core:air";
const AIR: u32 = 0;

pub fn register(r: &mut Registry) {
    r.feature(&OIL_FIELD)
        .dimension(&OVERWORLD)
        .spacing(SPACING)
        .chance(1.0 / SQUARES_TO_A_FIELD)
        .ground("#dirt #sand")
        .stage(stage::DECORATION)
        .after(&BASE_PLANTS)
        .reach(RADIUS[1])
        .reach_y(SEEP_STEP + PLANT_HEIGHT, DEPTH[1] + RADIUS[1] + 1)
        .plan(plan);
}

/// One field, as its start's random bits make it: the radius of its ball, how far under the
/// puddle the ball's middle lies, and the bits that shape its seep.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Field {
    radius: i32,
    depth: i32,
    seep: u64,
}

impl Field {
    /// Each pick takes a byte of its own, the seep the upper half. Their places are part of the
    /// world: moving one moves every field's oil.
    fn of(bits: u64) -> Field {
        let pick = |shift: u32, [low, high]: [i32; 2]| {
            low + (bits >> shift & 0xff) as i32 % (high - low + 1)
        };
        Field {
            radius: pick(0, RADIUS),
            depth: pick(8, DEPTH),
            seep: bits >> 32,
        }
    }

    /// Its ball round `middle`, as rows along x: one box each instead of a block at a time.
    fn rows(&self, middle: Pos) -> impl Iterator<Item = Area> + use<> {
        // A radius and a bit, so the ball has no lone blocks sticking out of its sides.
        let within = self.radius * self.radius + self.radius;
        let reach = -self.radius..=self.radius;
        reach
            .clone()
            .flat_map(move |dy| reach.clone().map(move |dz| (dy, dz)))
            .filter_map(move |(dy, dz)| {
                let half = (within - dy * dy - dz * dz).checked_isqrt()?;
                let row = middle.add_y(dy).add_z(dz);
                Some(Area::between(row.add_x(-half), row.add_x(half)))
            })
    }

    /// Whether the ground `dx`, `dz` from its puddle is soaked: the puddle and the blocks next
    /// to it always, the rest of the seep by its bits.
    fn soaks(&self, dx: i32, dz: i32) -> bool {
        let reach = dx.abs() + dz.abs();
        if reach > SEEP_REACH {
            return false;
        }
        // One bit for each block within reach.
        let bit = ((dx + SEEP_REACH) * (2 * SEEP_REACH + 1) + dz + SEEP_REACH) as u64;
        reach <= 1 || self.seep >> bit & 1 == 1
    }

    /// Where its seep soaks the ground, as how far from the puddle along x and z.
    fn seep(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let reach = -SEEP_REACH..=SEEP_REACH;
        reach
            .clone()
            .flat_map(move |dx| reach.clone().map(move |dz| (dx, dz)))
            .filter(|&(dx, dz)| self.soaks(dx, dz))
    }
}

/// Lays out the field that starts at the plan's place, the block above its puddle.
fn plan(p: &mut Plan) {
    let field = Field::of(p.random());
    let puddle = Pos::from(p.pos()).add_y(-1);
    for row in field.rows(puddle.add_y(-field.depth)) {
        p.replace(row.lo, row.hi, NOT_AIR, OIL.id());
    }
    for (dx, dz) in field.seep() {
        soak(p, puddle, dx, dz);
    }
}

/// Soaks the ground `dx`, `dz` from the puddle, the puddle itself into oil, and takes away the
/// plants on it, which the soaked ground no longer holds.
fn soak(p: &mut Plan, puddle: Pos, dx: i32, dz: i32) {
    let (x, z) = (puddle.x + dx, puddle.z + dz);
    let ground = Pos::new(x, p.ground(x, z).height, z);
    if ground.y < SEA || (ground.y - puddle.y).abs() > SEEP_STEP {
        return;
    }
    let is_puddle = dx == 0 && dz == 0;
    let soaked = |soaked_ground: u32| if is_puddle { OIL.id() } else { soaked_ground };
    p.replace(ground, ground, "#dirt", soaked(OILY_DIRT.id()));
    p.replace(ground, ground, "#sand", soaked(OIL_SAND.id()));
    let plants = Area::between(ground.add_y(1), ground.add_y(PLANT_HEIGHT));
    p.replace(plants.lo, plants.hi, "#plantable", AIR);
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIDDLE: Pos = Pos::new(100, -10, 200);

    fn field(radius: i32, seep: u64) -> Field {
        Field {
            radius,
            depth: DEPTH[0],
            seep,
        }
    }

    #[test]
    fn a_field_comes_from_its_bits_alone() {
        let made = Field::of(0x0123_4567_89ab_5a17);
        let expected = Field {
            radius: 6,
            depth: 53,
            seep: 0x0123_4567,
        };
        assert_eq!(made, expected, "the picks keep their places");
    }

    #[test]
    fn every_field_is_within_what_the_feature_may_reach() {
        for bits in (0..4000u64).map(|n| n.wrapping_mul(0x9e37_79b9_7f4a_7c15)) {
            let made = Field::of(bits);
            assert!((RADIUS[0]..=RADIUS[1]).contains(&made.radius), "radius");
            assert!((DEPTH[0]..=DEPTH[1]).contains(&made.depth), "depth");
        }
        let lowest = SEA - DEPTH[1] - RADIUS[1];
        assert!(lowest > -32, "above the floor of the world: {lowest}");
    }

    #[test]
    fn a_ball_holds_the_buckets_the_docs_name() {
        let buckets = |radius: i32| -> i32 {
            let rows = field(radius, 0).rows(MIDDLE);
            rows.map(|row| row.size().iter().product::<i32>()).sum()
        };
        assert_eq!(
            [4, 5, 6].map(buckets),
            [389, 739, 1189],
            "radius 4, 5 and 6"
        );
    }

    #[test]
    fn a_ball_is_rows_along_x_round_its_middle() {
        let rows: Vec<Area> = field(4, 0).rows(MIDDLE).collect();
        for row in &rows {
            assert_eq!(row.size()[1..], [1, 1], "one block high and wide");
            assert_eq!(row.lo.x + row.hi.x, 2 * MIDDLE.x, "as far to either side");
        }
        let through_the_middle = Area::between(MIDDLE.add_x(-4), MIDDLE.add_x(4));
        assert!(rows.contains(&through_the_middle), "its widest row");
        assert!(
            rows.iter().all(|row| row.lo.y >= MIDDLE.y - 4),
            "no lower than its radius"
        );
    }

    #[test]
    fn a_seep_is_its_puddle_and_the_ground_around() {
        let dry = field(4, 0);
        assert!(dry.soaks(0, 0), "the puddle");
        assert!(dry.soaks(1, 0) && dry.soaks(0, -1), "next to it, always");
        assert!(!dry.soaks(2, 0), "further out, by its bits");
        assert!(!dry.soaks(3, 0), "beyond its reach");
        assert_eq!(dry.seep().count(), 5, "the least a seep soaks");
        assert_eq!(field(4, u64::MAX).seep().count(), 13, "and the most");
    }
}
