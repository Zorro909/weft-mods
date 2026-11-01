//! `place`: a static structure, one template stood on the ground now and then. It is planned
//! like any other structure, by a planner made from its settings.

use weft_sdk::*;

use crate::fit::Fit;
use crate::format::Structure;
use crate::generator::Generator;
use crate::part::Foundation;
use crate::plan::{Plan, PlanError};
use crate::templates;
use crate::turn::Turn;

/// Places `structure` now and then while the world generates, turned whichever way the seed
/// says. It is also registered as template `name` of the calling mod, so its blocks are checked
/// when the game loads, `world.place(..)` can put it anywhere, and plans can use it.
///
/// ```ignore
/// let hut = Structure::decode(include_bytes!("../assets/structures/hut.wst")).expect("hut.wst");
/// structures_api::place(r, "hut", hut)
///     .dimension(&base_api::OVERWORLD)
///     .spacing(256).chance(0.3).biome("#animals")
///     .foundation(Foundation::Ground).max_slope(5);
/// ```
pub fn place<'a>(registry: &'a mut Registry, name: &str, structure: Structure) -> Placement<'a> {
    Placement {
        registry,
        name: name.to_string(),
        structure,
        dimensions: Vec::new(),
        fit: Fit::new(),
        foundation: None,
        spacing: 256,
        chance: 0.3,
        turns: true,
    }
}

/// Registered when dropped, like the SDK's builders.
pub struct Placement<'a> {
    registry: &'a mut Registry,
    name: String,
    structure: Structure,
    dimensions: Vec<&'static Dimension>,
    fit: Fit,
    foundation: Option<Foundation>,
    spacing: i32,
    chance: f32,
    turns: bool,
}

impl Placement<'_> {
    /// Only in this dimension (call again for more). Without, in every dimension but bare ones.
    pub fn dimension(mut self, dimension: &'static Dimension) -> Self {
        self.dimensions.push(dimension);
        self
    }
    /// One chance per `blocks` x `blocks` region (256 by default), at least its longest side.
    pub fn spacing(mut self, blocks: u32) -> Self {
        self.spacing = blocks.clamp(1, i32::MAX as u32) as i32;
        self
    }
    /// How likely a region is to get one, 0 to 1 (0.3 by default).
    pub fn chance(mut self, chance: f32) -> Self {
        self.chance = chance;
        self
    }
    /// Only where the biome under its middle has this tag (`"#animals"`).
    pub fn biome(mut self, tag: &str) -> Self {
        let name = tag.strip_prefix('#').unwrap_or(tag).to_string();
        self.fit = self.fit.biome(Box::leak(Box::new(Tag::named(name.leak()))));
        self
    }
    /// Fills under its bottom layer down to the ground, so it never floats. Without, the gap stays.
    pub fn foundation(mut self, foundation: Foundation) -> Self {
        self.foundation = Some(foundation);
        self
    }
    /// How much the ground under it may rise and fall, in blocks (6 by default): steeper places
    /// get none.
    pub fn max_slope(mut self, blocks: i32) -> Self {
        self.fit = self.fit.max_slope(blocks);
        self
    }
    /// How much further down its bottom layer goes (0 by default: level with the ground).
    pub fn buried(mut self, blocks: i32) -> Self {
        self.fit = self.fit.buried(blocks);
        self
    }
    /// Always faces the way it was saved, instead of a way the seed picks.
    pub fn keep_facing(mut self) -> Self {
        self.turns = false;
        self
    }
}

impl Drop for Placement<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let template = Template::generated(self.name.clone());
        let size = self.structure.size.map(|v| v as i32);
        templates::template(self.registry, template, std::mem::take(&mut self.structure));
        let single = Single {
            template,
            size,
            fit: self.fit,
            foundation: self.foundation,
            chance: self.chance,
            turns: self.turns,
        };
        // Room for it turned either way, with a block to spare on each side.
        let region = self.spacing.max(size[0].max(size[2]) + 2);
        let dimensions = std::mem::take(&mut self.dimensions);
        let planner = Box::new(move |p: &mut Plan| single.plan(p));
        Generator::new(template.name().to_string(), region, dimensions, planner)
            .register(self.registry);
    }
}

/// The planner of a static structure.
struct Single {
    template: &'static Template,
    /// Unturned.
    size: [i32; 3],
    fit: Fit,
    foundation: Option<Foundation>,
    chance: f32,
    turns: bool,
}

impl Single {
    /// With `chance`, the template on the ground at a random spot of the region, if it fits.
    fn plan(&self, p: &mut Plan) -> Result<(), PlanError> {
        if !p.random().chance(self.chance) {
            return Ok(());
        }
        let turn = if self.turns {
            p.random().turn()
        } else {
            Turn::None
        };
        let [sx, _, sz] = turn.size(self.size);
        let [x, z] = p.spot(sx.max(sz) / 2 + 1);
        let Some(corner) = p.fit(self.template, [x - sx / 2, z - sz / 2], turn, &self.fit) else {
            return Ok(());
        };
        let placed = p.structure(self.template, corner, turn)?.clear_above();
        if let Some(foundation) = self.foundation {
            placed.foundation(foundation);
        }
        Ok(())
    }
}
