//! One registered structure while the world generates: its plans, worked out once per region
//! and shared by every cube in it.

use std::sync::{Arc, OnceLock};

use weft_sdk::*;

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
            planner,
            plans: ColumnCache::new(CACHED_REGIONS),
        }
    }

    /// Registers it as one of the calling mod's decorations, and for `locate`.
    pub fn register(self, registry: &mut Registry) {
        let generator = Arc::new(self);
        locate::add(generator.clone());
        let dimensions = generator.dimensions.clone();
        let mut decoration = registry.decoration(move |g| generator.generate(g));
        for dimension in dimensions {
            decoration = decoration.dimension(dimension);
        }
    }

    fn generate(&self, g: &mut Gen) {
        let (seed, dimension) = (g.seed(), g.dimension());
        let [ox, _, oz] = g.origin().to_array();
        for iz in region::indices(oz, oz + 15, self.region) {
            for ix in region::indices(ox, ox + 15, self.region) {
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
                if let Some(anchor) = planned.anchor() {
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

/// Writes the plan's blocks into the cube. They are gathered first: reading the ground's top
/// blocks and writing the cube both go through `g`.
fn stamp(planned: &Planned, g: &mut Gen) {
    let origin = g.origin().to_array();
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
