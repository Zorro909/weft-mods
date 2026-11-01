//! `plan`: registers a dynamic structure, laid out by the mod's own planner one region at a
//! time. It runs as one of the calling mod's own decorations.

use weft_sdk::*;

use crate::generator::{Generator, Planner};
use crate::plan::{Plan, PlanError};

/// Region side without `.region(..)`, in blocks.
const DEFAULT_REGION: i32 = 512;

/// Registers structure `name` of the calling mod, laid out region by region by `planner`. Each
/// region gets one plan; everything in it must stay inside the region (`p.region()`), and a
/// plan that reaches out is given up and logged.
///
/// ```ignore
/// static WELL: Template = Template::named("well");
/// fn village(p: &mut Plan) -> Result<(), PlanError> {
///     let [x, z] = p.spot(48);
///     let turn = p.random().turn();
///     let Some(corner) = p.fit(&WELL, [x, z], turn, &Fit::new().max_slope(3)) else { return Ok(()) };
///     p.structure(&WELL, corner, turn)?.foundation(Foundation::Ground).clear_above();
///     Ok(())
/// }
/// structures_api::plan(r, "village", village).dimension(&base_api::OVERWORLD).region(512);
/// ```
pub fn plan<'a>(
    registry: &'a mut Registry,
    name: &str,
    planner: impl Fn(&mut Plan) -> Result<(), PlanError> + Send + Sync + 'static,
) -> Planning<'a> {
    Planning {
        registry,
        name: name.to_string(),
        planner: Some(Box::new(planner)),
        dimensions: Vec::new(),
        region: DEFAULT_REGION,
    }
}

/// Registered when dropped, like the SDK's builders.
pub struct Planning<'a> {
    registry: &'a mut Registry,
    name: String,
    planner: Option<Box<Planner>>,
    dimensions: Vec<&'static Dimension>,
    region: i32,
}

impl Planning<'_> {
    /// Only in this dimension (call again for more). Without, in every dimension but bare ones.
    pub fn dimension(mut self, dimension: &'static Dimension) -> Self {
        self.dimensions.push(dimension);
        self
    }
    /// One plan per `blocks` x `blocks` region (512 by default).
    pub fn region(mut self, blocks: u32) -> Self {
        self.region = blocks.clamp(1, i32::MAX as u32) as i32;
        self
    }
}

impl Drop for Planning<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let Some(planner) = self.planner.take() else {
            return;
        };
        let name = full_name(&self.name);
        let dimensions = std::mem::take(&mut self.dimensions);
        Generator::new(name, self.region, dimensions, planner).register(self.registry);
    }
}

/// `name` with the calling mod's namespace, unless it has one.
pub(crate) fn full_name(name: &str) -> String {
    if name.contains(':') {
        name.to_string()
    } else {
        format!("{}:{name}", mod_id())
    }
}
