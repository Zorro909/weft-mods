//! Where oil lies. The world is split into regions `REGION` blocks square, and each has at most
//! one field, decided from the world's seed and the region alone: every cube and every client
//! work out the same fields, whatever order they are made in. A field is a ball of oil sources
//! deep underground, and above its middle a seep: a puddle of oil in oil-soaked ground.

use twox_hash::XxHash3_64;
use weft_sdk::{Gen, Tag};

use super::{OIL, OIL_SAND, OILY_DIRT};
use crate::lib::Pos;

/// Blocks along each side of a region, and the share of regions with a field, in 256ths.
const REGION: i32 = 96;
const CHANCE: u64 = 128;
/// The radius of a field's ball, in blocks: 3 holds 179 buckets, 5 holds 739.
const RADIUS_LOW: i32 = 3;
const RADIUS_HIGH: i32 = 5;
/// Heights a field's middle lies at: its ball stays above the overworld's floor at -32 and well
/// under the ground.
const MIDDLE_LOW: i32 = -24;
const MIDDLE_HIGH: i32 = 20;
/// How far from its middle a seep soaks the ground, counted along x and z together.
const SEEP_REACH: i32 = 2;
/// Blocks along each side of a cube of the world, which a decoration draws one at a time.
const CUBE: i32 = 16;
/// Seeps under the sea are left out: base's sea level.
const SEA: i32 = 63;
/// Mixed into the seed, so fields don't line up with what other mods draw from it.
const SALT: u64 = 0x6f69_6c5f_6669_656c;

/// One field: the middle of its ball, its radius, and the bits that shape its seep.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Field {
    pub middle: Pos,
    pub radius: i32,
    seep: u64,
}

impl Field {
    /// The field of region (`region_x`, `region_z`) in the world of `seed`, if it has one.
    pub fn in_region(seed: u64, region_x: i32, region_z: i32) -> Option<Field> {
        let mut bytes = [0u8; 24];
        bytes[..8].copy_from_slice(&(seed ^ SALT).to_le_bytes());
        bytes[8..12].copy_from_slice(&region_x.to_le_bytes());
        bytes[12..16].copy_from_slice(&region_z.to_le_bytes());
        let bits = XxHash3_64::oneshot(&bytes);
        if bits & 0xff >= CHANCE {
            return None;
        }
        // The lowest byte decided whether there is a field; each pick below takes bits of its own.
        let pick = |shift: u32, low: i32, high: i32| {
            low + ((bits >> shift) % (high - low + 1) as u64) as i32
        };
        let middle = Pos {
            x: region_x * REGION + pick(8, 0, REGION - 1),
            y: pick(24, MIDDLE_LOW, MIDDLE_HIGH),
            z: region_z * REGION + pick(16, 0, REGION - 1),
        };
        let radius = pick(32, RADIUS_LOW, RADIUS_HIGH);
        Some(Field {
            middle,
            radius,
            seep: bits >> 40,
        })
    }

    /// Whether `pos` is in its ball of oil.
    pub fn holds(&self, pos: Pos) -> bool {
        let [dx, dy, dz] = [
            pos.x - self.middle.x,
            pos.y - self.middle.y,
            pos.z - self.middle.z,
        ];
        // A radius and a bit, so the ball has no lone blocks sticking out of its sides.
        dx * dx + dy * dy + dz * dz <= self.radius * self.radius + self.radius
    }

    /// Whether the ground `dx`, `dz` from its middle is soaked: the middle, which is the puddle,
    /// always, the rest of the seep by its bits.
    pub fn soaks(&self, dx: i32, dz: i32) -> bool {
        let reach = dx.abs() + dz.abs();
        if reach == 0 {
            return true;
        }
        if reach > SEEP_REACH {
            return false;
        }
        // One bit for each block within reach.
        let bit = ((dx + SEEP_REACH) * (2 * SEEP_REACH + 1) + dz + SEEP_REACH) as u64;
        reach == 1 || self.seep >> bit & 1 == 1
    }
}

/// The fields whose ball or seep may reach into the cube at `origin`.
pub fn near(seed: u64, origin: Pos) -> Vec<Field> {
    let reach = RADIUS_HIGH.max(SEEP_REACH);
    let regions =
        |low: i32| (low - reach).div_euclid(REGION)..=(low + CUBE - 1 + reach).div_euclid(REGION);
    let mut fields = Vec::new();
    for region_x in regions(origin.x) {
        for region_z in regions(origin.z) {
            fields.extend(Field::in_region(seed, region_x, region_z));
        }
    }
    fields
}

/// The decoration: the oil and the seeps of every field that reaches into this cube.
pub fn decorate(cube: &mut Gen) {
    let origin = Pos::from(cube.origin());
    for field in near(cube.seed(), origin) {
        fill_ball(cube, origin, &field);
        soak(cube, origin, &field);
    }
}

/// Fills what the field's ball takes of the cube with oil, except where caves left air.
fn fill_ball(cube: &mut Gen, origin: Pos, field: &Field) {
    let misses = |low: i32, middle: i32| {
        low > middle + field.radius || low + CUBE - 1 < middle - field.radius
    };
    let middle = field.middle;
    if misses(origin.x, middle.x) || misses(origin.y, middle.y) || misses(origin.z, middle.z) {
        return;
    }
    let oil = OIL.id();
    let side = CUBE as usize;
    for y in 0..side {
        for z in 0..side {
            for x in 0..side {
                let pos = Pos {
                    x: origin.x + x as i32,
                    y: origin.y + y as i32,
                    z: origin.z + z as i32,
                };
                if field.holds(pos) && cube.get(x, y, z) != 0 {
                    cube.set(x, y, z, oil);
                }
            }
        }
    }
}

/// Soaks the ground of the field's seep where it lies in the cube, and puts the puddle in.
fn soak(cube: &mut Gen, origin: Pos, field: &Field) {
    for dx in -SEEP_REACH..=SEEP_REACH {
        for dz in -SEEP_REACH..=SEEP_REACH {
            if !field.soaks(dx, dz) {
                continue;
            }
            let (x, z) = (field.middle.x + dx, field.middle.z + dz);
            let height = cube.column(x, z).height;
            let local = [x - origin.x, height - origin.y, z - origin.z];
            if height < SEA || local.iter().any(|&at| !(0..CUBE).contains(&at)) {
                continue;
            }
            let [lx, ly, lz] = local.map(|at| at as usize);
            let soaked = soaked(cube.get(lx, ly, lz), dx == 0 && dz == 0);
            if let Some(soaked) = soaked {
                cube.set(lx, ly, lz, soaked);
                clear_plant(cube, lx, ly + 1, lz);
            }
        }
    }
}

/// What the ground `ground` becomes in a seep: the puddle in its middle, oil-soaked dirt or
/// sand around it; nothing where it is neither.
fn soaked(ground: u32, middle: bool) -> Option<u32> {
    static DIRT: Tag = Tag::named("dirt");
    static SAND: Tag = Tag::named("sand");
    if DIRT.on_block(ground) {
        Some(if middle { OIL.id() } else { OILY_DIRT.id() })
    } else if SAND.on_block(ground) {
        Some(if middle { OIL.id() } else { OIL_SAND.id() })
    } else {
        None
    }
}

/// Takes away a plant standing at (`x`, `y`, `z`) of the cube, which the soaked ground below
/// no longer holds.
fn clear_plant(cube: &mut Gen, x: usize, y: usize, z: usize) {
    static PLANTABLE: Tag = Tag::named("plantable");
    if y < CUBE as usize && PLANTABLE.on_block(cube.get(x, y, z)) {
        cube.set(x, y, z, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_is_the_same_every_time() {
        let fields: Vec<_> = (0..20).filter_map(|x| Field::in_region(42, x, 3)).collect();
        let again: Vec<_> = (0..20).filter_map(|x| Field::in_region(42, x, 3)).collect();
        assert_eq!(fields, again, "from the seed alone");
        let other: Vec<_> = (0..20).filter_map(|x| Field::in_region(43, x, 3)).collect();
        assert_ne!(fields, other, "another world, other fields");
    }

    #[test]
    fn about_half_the_regions_have_a_field_inside_them() {
        let regions = (-20..20).flat_map(|x| (-20..20).map(move |z| (x, z)));
        let fields: Vec<_> = regions
            .filter_map(|(x, z)| Some(((x, z), Field::in_region(7, x, z)?)))
            .collect();
        assert!(
            (600..1000).contains(&fields.len()),
            "{} of 1600",
            fields.len()
        );
        for ((x, z), field) in fields {
            let region_x = field.middle.x.div_euclid(REGION);
            let region_z = field.middle.z.div_euclid(REGION);
            assert_eq!((region_x, region_z), (x, z), "its middle in its region");
            assert!(
                (RADIUS_LOW..=RADIUS_HIGH).contains(&field.radius),
                "its radius"
            );
            let lowest = field.middle.y - field.radius;
            assert!(lowest > -32, "above the floor of the world: {lowest}");
        }
    }

    #[test]
    fn every_cube_a_field_reaches_finds_it() {
        let field = (0..)
            .find_map(|x| Field::in_region(5, x, 0))
            .expect("a field somewhere along the row");
        let edge = Pos {
            x: field.middle.x + field.radius,
            ..field.middle
        };
        let origin = Pos {
            x: edge.x.div_euclid(CUBE) * CUBE,
            y: edge.y.div_euclid(CUBE) * CUBE,
            z: edge.z.div_euclid(CUBE) * CUBE,
        };
        assert!(field.holds(edge), "the ball reaches its radius");
        assert!(
            near(5, origin).contains(&field),
            "the cube of its edge asks for it"
        );
    }

    #[test]
    fn a_seep_is_its_puddle_and_the_ground_around() {
        let field = Field {
            middle: Pos { x: 0, y: 0, z: 0 },
            radius: 3,
            seep: 0,
        };
        assert!(field.soaks(0, 0), "the puddle");
        assert!(
            field.soaks(1, 0) && field.soaks(0, -1),
            "next to it, always"
        );
        assert!(!field.soaks(2, 0), "further out, by its bits");
        assert!(!field.soaks(3, 0), "beyond its reach");
    }
}
