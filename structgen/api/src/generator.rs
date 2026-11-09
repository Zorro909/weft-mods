//! One registered structure while the world generates: its plans, worked out once per region
//! and shared by every cube in it.

use std::sync::{Arc, OnceLock};

use weft_sdk::*;
use weftkit::Pos;

use crate::fit::Columns;
use crate::locate;
use crate::plan::{Plan, PlanError};
use crate::planned::Planned;
use crate::random::Random;
use crate::region::{self, Region};

/// Plans worked out per region; each is shared by every cube in it.
const CACHED_REGIONS: usize = 1024;

/// A region's plan, `None` if it failed. Threads that need it together wait for one of them to
/// work it out.
pub(crate) type Slot = OnceLock<Option<Planned>>;

/// Lays out one region of a structure. It must be pure: it runs on the server and on clients,
/// in any order, and maybe twice at once.
pub(crate) type Planner = dyn Fn(&mut Plan) -> Result<(), PlanError> + Send + Sync;

pub(crate) struct Generator {
    /// Full name, `mymod:village`.
    pub name: String,
    /// Empty for every dimension.
    pub dimensions: Vec<&'static Dimension>,
    /// The side of a region, in blocks.
    pub region: i32,
    planner: Box<Planner>,
    plans: ColumnCache<Slot>,
    /// Where a region's random numbers start: from the name, or a group's.
    salt: u64,
    /// The group it belongs to, and the one whose plans it keeps off.
    group: Option<String>,
    avoids: Option<String>,
}

impl Generator {
    pub fn new(
        name: String,
        region: i32,
        dimensions: Vec<&'static Dimension>,
        planner: Box<Planner>,
    ) -> Generator {
        Generator {
            salt: salt(&name),
            name,
            dimensions,
            region: region.max(1),
            planner,
            plans: ColumnCache::new(CACHED_REGIONS),
            group: None,
            avoids: None,
        }
    }

    /// Draws the same random numbers in a region as every other generator of `group` there.
    pub fn in_group(mut self, group: &str) -> Generator {
        self.salt = salt(group);
        self.group = Some(group.to_string());
        self
    }

    /// Gives up a plan that would touch a plan of `group`'s.
    pub fn avoiding(mut self, group: &str) -> Generator {
        self.avoids = Some(group.to_string());
        self
    }

    /// Whether it generates in dimension `id`.
    pub fn is_in(&self, id: u32) -> bool {
        self.dimensions.is_empty() || self.dimensions.iter().any(|d| d.id() == id)
    }

    /// Registers it for `locate`, and as one of the calling mod's decorations, named like the
    /// structure so another mod can remove it: `r.remove(&Decoration::named("mymod:village"))`.
    pub fn register(self, registry: &mut Registry) {
        let generator = Arc::new(self);
        locate::add(generator.clone());
        let decorate = generator.clone();
        let mut decoration = registry
            .decoration(move |g| decorate.generate(g))
            .named(&generator.name);
        for &dimension in &generator.dimensions {
            decoration = decoration.dimension(dimension);
        }
    }

    fn generate(&self, g: &mut Gen) {
        let (seed, dimension) = (g.seed(), g.dimension());
        let origin = Pos::from(g.origin());
        for iz in region::indices(origin.z, origin.z + 15, self.region) {
            for ix in region::indices(origin.x, origin.x + 15, self.region) {
                let slot = self.planned(seed, dimension, [ix, iz], &|x, z| ground(g, x, z));
                if let Some(Some(planned)) = slot.get() {
                    stamp(planned, g);
                }
            }
        }
    }

    /// The plan of region `index`, worked out on the first call.
    pub fn planned(
        &self,
        seed: u64,
        dimension: u32,
        index: [i32; 2],
        ground: Columns,
    ) -> Arc<Slot> {
        // The cache is keyed by seed: the dimension goes into it so dimensions don't share plans.
        let key = seed ^ noise::mix(dimension as u64 + 1);
        let slot = self.plans.get(key, index[0], index[1], OnceLock::new);
        slot.get_or_init(|| self.make(seed, dimension, index, ground));
        slot
    }

    fn make(&self, seed: u64, dimension: u32, index: [i32; 2], ground: Columns) -> Option<Planned> {
        let region = Region::at(index, self.region);
        let salt = noise::hash(seed ^ self.salt, index[0] as i64, 0, index[1] as i64);
        let mut plan = Plan::new(region, Random::new(salt), ground);
        let result = (self.planner)(&mut plan);
        match plan.finish(result) {
            Ok(parts) => {
                let planned = Planned::new(parts);
                if self.crowded(&locate::all(), seed, dimension, &planned, ground) {
                    return None;
                }
                // The log needs the game, which unit tests run without.
                if let Some(anchor) = planned.anchor()
                    && !cfg!(test)
                {
                    debug!(
                        "structure {} at {anchor:?} in dimension {dimension}, {} parts",
                        self.name,
                        planned.len()
                    );
                }
                Some(planned)
            }
            Err(error) => {
                error!(
                    "structure {} in the region from {:?} to {:?}: {error}; nothing of it is placed there",
                    self.name, region.min, region.max
                );
                None
            }
        }
    }
}

impl Generator {
    /// Whether `planned` would touch a plan of the group it avoids, among `others`.
    fn crowded(
        &self,
        others: &[Arc<Generator>],
        seed: u64,
        dimension: u32,
        planned: &Planned,
        ground: Columns,
    ) -> bool {
        let (Some(group), Some((from, to))) = (&self.avoids, planned.bounds()) else {
            return false;
        };
        let ours = others
            .iter()
            .filter(|g| g.group.as_ref() == Some(group) && g.is_in(dimension));
        ours.into_iter().any(|other| {
            let regions = region::indices(from[2], to[2], other.region).flat_map(|iz| {
                region::indices(from[0], to[0], other.region).map(move |ix| [ix, iz])
            });
            regions.into_iter().any(|index| {
                let slot = other.planned(seed, dimension, index, ground);
                matches!(slot.get(), Some(Some(theirs)) if theirs.touches(from, to))
            })
        })
    }
}

/// Writes the plan's blocks into the cube. They are gathered first: reading the ground's top
/// blocks and writing the cube both go through `g`.
fn stamp(planned: &Planned, g: &mut Gen) {
    let origin = Pos::from(g.origin());
    if !planned.reaches(origin) {
        return;
    }
    let mut blocks = Vec::new();
    let surface = |x, z| g.surface_block(x, z);
    planned.stamp(origin, &surface, &mut |pos, state| {
        blocks.push((pos, state))
    });
    for ([x, y, z], state) in blocks {
        g.set(x, y, z, state);
    }
}

/// Randomness of its own for each structure, from its full name.
fn salt(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

/// `None` where nothing may stand: no terrain, or a cave took the surface.
fn ground(g: &Gen, x: i32, z: i32) -> Option<terrain::Column> {
    let column = g.column(x, z);
    (column.height != i32::MIN && !g.surface_carved(x, z)).then_some(column)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// A generator whose planner notes the first number each region draws.
    fn noting(name: &str, drawn: Arc<Mutex<Vec<u64>>>) -> Generator {
        let planner = move |p: &mut Plan| {
            drawn.lock().unwrap().push(p.random().next());
            Ok(())
        };
        Generator::new(name.to_string(), 512, Vec::new(), Box::new(planner))
    }

    #[test]
    fn a_group_draws_the_same_numbers_in_a_region() {
        let drawn = Arc::new(Mutex::new(Vec::new()));
        let none = |_: i32, _: i32| None;
        let hamlet = noting("m:hamlet", drawn.clone()).in_group("m:settlement");
        let town = noting("m:town", drawn.clone()).in_group("m:settlement");
        let alone = noting("m:alone", drawn.clone());
        for generator in [&hamlet, &town, &alone] {
            generator.make(42, 0, [3, -2], &none);
        }
        let drawn = drawn.lock().unwrap();
        assert_eq!(drawn[0], drawn[1], "one group, one roll");
        assert_ne!(drawn[0], drawn[2], "alone, its own roll");
    }

    /// A generator whose plan is one block at `at` in every region.
    fn block_at(name: &str, at: [i32; 3]) -> Generator {
        let planner = move |p: &mut Plan| p.set_state(at, 5);
        Generator::new(name.to_string(), 512, Vec::new(), Box::new(planner))
    }

    #[test]
    fn a_plan_that_would_touch_one_of_the_group_it_avoids_is_given_up() {
        let none = |_: i32, _: i32| None;
        let village = Arc::new(block_at("m:village", [100, 70, 100]).in_group("m:settlement"));
        let others = [village];
        let on_it = block_at("m:cottage", [100, 70, 100]).avoiding("m:settlement");
        let beside = block_at("m:cottage2", [104, 70, 100]).avoiding("m:settlement");
        let planned = |g: &Generator| {
            g.make(42, 0, [0, 0], &none)
                .map(|plan| g.crowded(&others, 42, 0, &plan, &none))
        };
        assert_eq!(planned(&on_it), Some(true), "on the village's block");
        assert_eq!(planned(&beside), Some(false), "a few blocks away");
    }
}
