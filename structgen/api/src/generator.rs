//! One registered structure while the world generates: its plans, worked out once per region
//! and shared by every cube in it.

use std::sync::{Arc, OnceLock};

use weft_sdk::*;
use weftkit::{Area, Pos};

use crate::fit::Columns;
use crate::locate;
use crate::plan::{Plan, PlanError};
use crate::planned::{Planned, widened};
use crate::random::Random;
use crate::region::{self, Region};

/// Blocks kept between a structure and one of higher priority, so a lone cottage doesn't stand
/// against a village's houses.
const ROOM: i32 = 8;
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
    /// Where its plan would come within `ROOM` of the plan of one with a higher priority, it
    /// gives way.
    pub priority: i32,
    planner: Box<Planner>,
    plans: ColumnCache<Slot>,
}

impl Generator {
    pub fn new(
        name: String,
        region: i32,
        dimensions: Vec<&'static Dimension>,
        planner: Box<Planner>,
    ) -> Generator {
        Generator {
            name,
            dimensions,
            region: region.max(1),
            priority: 0,
            planner,
            plans: ColumnCache::new(CACHED_REGIONS),
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Generator {
        self.priority = priority;
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
        let salt = noise::hash(seed ^ salt(&self.name), index[0] as i64, 0, index[1] as i64);
        let mut plan = Plan::new(region, Random::new(salt), ground);
        let result = (self.planner)(&mut plan);
        match plan.finish(result) {
            Ok(parts) => {
                let planned = Planned::new(parts);
                if self.gives_way(seed, dimension, &planned, ground) {
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

    /// Whether `planned` would come within `ROOM` of the plan of a structure with a higher
    /// priority. Theirs have given way in turn where they had to, so only what is really placed
    /// counts.
    fn gives_way(&self, seed: u64, dimension: u32, planned: &Planned, ground: Columns) -> bool {
        let Some(bounds) = planned.bounds() else {
            return false;
        };
        let Area { lo, hi } = widened(bounds, ROOM);
        let higher = locate::all().into_iter();
        let mut higher = higher.filter(|g| g.priority > self.priority && g.is_in(dimension));
        higher.any(|other| {
            let mut regions = region::indices(lo.z, hi.z, other.region)
                .flat_map(|iz| region::indices(lo.x, hi.x, other.region).map(move |ix| [ix, iz]));
            regions.any(|index| {
                let slot = other.planned(seed, dimension, index, ground);
                matches!(slot.get(), Some(Some(theirs)) if planned.near(theirs, ROOM))
            })
        })
    }
}

/// Writes the plan's blocks into the cube, in order, each that names a tag only over a block
/// that has it. They are gathered first: reading the ground's top blocks and writing the cube
/// both go through `g`.
fn stamp(planned: &Planned, g: &mut Gen) {
    let origin = Pos::from(g.origin());
    if !planned.reaches(origin) {
        return;
    }
    let mut blocks = Vec::new();
    let surface = |x, z| g.surface_block(x, z);
    planned.stamp(origin, &surface, &mut |pos, state, only| {
        blocks.push((pos, state, only))
    });
    for ([x, y, z], state, only) in blocks {
        if only.is_none_or(|tag| tag.on_block(g.get(x, y, z))) {
            g.set(x, y, z, state);
        }
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
    use weftkit::BlockState;

    use super::*;

    /// A generator of `priority` whose plan fills from `from` to `to` in the region holding
    /// them; registered for the others to see. Each test plans in a region of its own.
    fn filling(name: &str, priority: i32, from: [i32; 3], to: [i32; 3]) -> Arc<Generator> {
        let planner = move |p: &mut Plan| match p.region().contains(from[0], from[2]) {
            true => p.fill_state(from.into(), to.into(), BlockState::from(5)),
            false => Ok(()),
        };
        let generator = Generator::new(name.to_string(), 512, Vec::new(), Box::new(planner));
        let generator = Arc::new(generator.with_priority(priority));
        locate::add(generator.clone());
        generator
    }

    fn placed(generator: &Generator, region: [i32; 2]) -> bool {
        let none = |_: i32, _: i32| None;
        generator.make(42, 0, region, &none).is_some()
    }

    #[test]
    fn a_plan_gives_way_where_it_would_touch_a_higher_one() {
        filling("m:village", 2, [100, 70, 100], [110, 70, 110]);
        let on_it = filling("m:cottage", 0, [105, 70, 105], [105, 70, 105]);
        let against = filling("m:cottage2", 0, [115, 70, 100], [115, 70, 100]);
        let beside = filling("m:cottage3", 0, [120, 70, 100], [120, 70, 100]);
        let level = filling("m:hamlet", 2, [105, 70, 105], [105, 70, 105]);
        assert!(!placed(&on_it, [0, 0]), "a cottage in the village");
        assert!(!placed(&against, [0, 0]), "a cottage up against it");
        assert!(placed(&beside, [0, 0]), "a cottage with room between");
        assert!(placed(&level, [0, 0]), "the same priority: both stand");
        let round = move |p: &mut Plan| {
            let stone = BlockState::from(5);
            p.fill_state(Pos::new(80, 70, 100), Pos::new(80, 70, 100), stone)?;
            p.fill_state(Pos::new(130, 70, 100), Pos::new(130, 70, 100), stone)
        };
        let round = Generator::new("m:hamlet3".into(), 512, Vec::new(), Box::new(round));
        assert!(
            placed(&round.with_priority(1), [0, 0]),
            "parts either side, none near it"
        );
    }

    #[test]
    fn a_plan_that_gave_way_leaves_the_ground_to_lower_ones() {
        let at = |x| [x, 70, 2600];
        filling("m:town2", 3, at(2600), at(2600));
        let village = filling("m:village2", 2, at(2600), at(2700));
        let hamlet = filling("m:hamlet2", 1, at(2700), at(2700));
        assert!(!placed(&village, [5, 5]), "the town has it");
        assert!(placed(&hamlet, [5, 5]), "no village there after all");
    }
}
