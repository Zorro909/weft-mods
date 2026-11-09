//! `place`: a static structure, one template stood on the ground now and then. It is planned
//! like any other structure, by a planner made from its settings.

use weft_sdk::*;

use crate::biomes::Biomes;
use crate::fit::Fit;
use crate::format::Structure;
use crate::generator::Generator;
use crate::materials::Materials;
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
/// structgen_api::place(r, "hut", hut)
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
        materials: Vec::new(),
        spacing: 256,
        chance: 0.3,
        turns: true,
        priority: 0,
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
    materials: Vec<Materials>,
    spacing: i32,
    chance: f32,
    turns: bool,
    priority: i32,
}

impl Placement<'_> {
    /// One set of blocks its slots may be filled with (call again for more); the seed picks one
    /// for each place. Without, the slots' defaults.
    pub fn materials(mut self, materials: Materials) -> Self {
        self.materials.push(materials);
        self
    }
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
    /// Only where the biome under its middle is one of these, named as anywhere in Weft:
    /// `"#animals"`, `"#forest !base:birch_forest"`.
    pub fn biome(mut self, biomes: &str) -> Self {
        let biomes = Box::leak(Box::new(Biomes::named(biomes.to_string().leak())));
        self.fit = self.fit.biome(biomes);
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
    /// Only where the ground under every column is at least at height `y`: a world's sea level
    /// keeps it out of lakes and the sea. Without, at any height.
    pub fn min_height(mut self, y: i32) -> Self {
        self.fit = self.fit.min_height(y);
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
    /// Not where it would touch a structure with a higher priority; see `Planning::priority`.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl Drop for Placement<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let template = Template::generated(self.name.clone());
        let size = self.structure.size();
        templates::template(self.registry, template, std::mem::take(&mut self.structure));
        let single = Single {
            template,
            size,
            fit: self.fit,
            foundation: self.foundation,
            materials: std::mem::take(&mut self.materials),
            chance: self.chance,
            turns: self.turns,
        };
        // Room for it turned either way, with a block to spare on each side.
        let region = self.spacing.max(size[0].max(size[2]) + 2);
        let dimensions = std::mem::take(&mut self.dimensions);
        let planner = Box::new(move |p: &mut Plan| single.plan(p));
        Generator::new(template.name().to_string(), region, dimensions, planner)
            .with_priority(self.priority)
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
    materials: Vec<Materials>,
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
        let materials = self.materials(p);
        let mut placed = p.structure(self.template, corner, turn)?.clear_above();
        if let Some(materials) = &materials {
            placed = placed.materials(materials);
        }
        if let Some(foundation) = self.foundation {
            placed.foundation(foundation);
        }
        Ok(())
    }

    /// One of its material sets, `None` when it has none.
    fn materials(&self, p: &mut Plan) -> Option<Materials> {
        if self.materials.is_empty() {
            return None;
        }
        let i = p.random().below(self.materials.len() as u64) as usize;
        Some(self.materials[i].clone())
    }
}

#[cfg(test)]
mod tests {
    use weft_sdk::terrain::Column;

    use super::*;
    use crate::random::Random;
    use crate::region::Region;

    static TOWER: Template = Template::named("test:placement_tower");

    fn flat(_: i32, _: i32) -> Option<Column> {
        Some(Column {
            height: 70,
            biome: 0,
        })
    }

    #[test]
    fn a_static_structure_stays_inside_the_smallest_region() {
        let size = [7, 3, 4];
        let tower = Structure::from_names(size.map(|v| v as u16), ["core:air"; 84]);
        templates::add(TOWER.name(), tower.expect("one name per block"));
        let single = Single {
            template: &TOWER,
            size,
            fit: Fit::new(),
            foundation: None,
            materials: Vec::new(),
            chance: 1.0,
            turns: true,
        };
        // What `spacing(1)` makes of it: its longest side and a block to spare on each side.
        let region = Region::at([-3, 5], 9);
        for seed in 0..500 {
            let mut plan = Plan::new(region, Random::new(seed), &flat);
            let result = single.plan(&mut plan);
            let parts = plan.finish(result);
            assert_eq!(
                parts.map(|parts| parts.len()),
                Ok(1),
                "placed whole, seed {seed}"
            );
        }
    }
}
