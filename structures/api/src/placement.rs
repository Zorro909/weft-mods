//! `place`: registers a structure that generation stands on the ground. It runs as one of the
//! calling mod's own decorations.

use std::sync::{Arc, OnceLock};

use weft_sdk::*;

use crate::format::Structure;
use crate::site::{self, Ground, Rules, Site};
use crate::stamp::{self, Resolved};

/// Sites worked out per grid cell; each is shared by every cube the structure reaches.
const CACHED_CELLS: usize = 4096;

/// Places `structure` now and then while the world generates. It is also registered as template
/// `name` of the calling mod, so its blocks are checked when the game loads and
/// `world.place(..)` can put it anywhere.
///
/// ```ignore
/// static COBBLESTONE: Block = Block::named("base:cobblestone");
/// let hut = Structure::decode(include_bytes!("../assets/structures/hut.wst")).expect("hut.wst");
/// structures_api::place(r, "hut", hut)
///     .dimension(&base_api::OVERWORLD)
///     .spacing(256).chance(0.3).biome("#animals")
///     .foundation(&COBBLESTONE).max_slope(5);
/// ```
pub fn place<'a>(registry: &'a mut Registry, name: &str, structure: Structure) -> Placement<'a> {
    Placement {
        registry,
        name: name.to_string(),
        structure,
        dimensions: Vec::new(),
        biome: None,
        foundation: None,
        spacing: 256,
        chance: 0.3,
        max_slope: 6,
        sink: 0,
    }
}

/// Registered when dropped, like the SDK's builders.
pub struct Placement<'a> {
    registry: &'a mut Registry,
    name: String,
    structure: Structure,
    dimensions: Vec<&'static Dimension>,
    biome: Option<&'static Tag>,
    foundation: Option<&'static Block>,
    spacing: i32,
    chance: f32,
    max_slope: i32,
    sink: i32,
}

impl Placement<'_> {
    /// Only in this dimension (call again for more). Without, in every dimension but bare ones.
    pub fn dimension(mut self, dimension: &'static Dimension) -> Self {
        self.dimensions.push(dimension);
        self
    }
    /// One chance per `blocks` x `blocks` cell (256 by default).
    pub fn spacing(mut self, blocks: u32) -> Self {
        self.spacing = blocks.max(1) as i32;
        self
    }
    /// How likely a cell is to get one, 0 to 1 (0.3 by default).
    pub fn chance(mut self, chance: f32) -> Self {
        self.chance = chance;
        self
    }
    /// Only where the biome under its middle has this tag (`"#animals"`).
    pub fn biome(mut self, tag: &str) -> Self {
        let name = tag.strip_prefix('#').unwrap_or(tag).to_string();
        self.biome = Some(Box::leak(Box::new(Tag::named(name.leak()))));
        self
    }
    /// Fills under its bottom layer down to the ground, so it never floats. Without, the gap stays.
    pub fn foundation(mut self, block: &'static Block) -> Self {
        self.foundation = Some(block);
        self
    }
    /// How much the ground under it may rise and fall, in blocks (6 by default): steeper places
    /// get none.
    pub fn max_slope(mut self, blocks: i32) -> Self {
        self.max_slope = blocks;
        self
    }
    /// How far its bottom layer goes into the ground (0 by default: it stands on it).
    pub fn buried(mut self, blocks: i32) -> Self {
        self.sink = blocks;
        self
    }
}

impl Drop for Placement<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let template = Template::generated(self.name.clone());
        register_template(self.registry, template, &self.structure);
        let generator = Arc::new(Generator {
            name: template.name().to_string(),
            rules: Rules {
                salt: salt(template.name()),
                spacing: self.spacing,
                chance: (self.chance.clamp(0.0, 1.0) * 65536.0) as u32,
                max_slope: self.max_slope,
                sink: self.sink,
            },
            structure: std::mem::take(&mut self.structure),
            biome: self.biome,
            foundation: self.foundation,
            resolved: OnceLock::new(),
            sites: ColumnCache::new(CACHED_CELLS),
        });
        let mut decoration = self.registry.decorate(move |g| generator.generate(g));
        for dimension in &self.dimensions {
            decoration = decoration.dimension(dimension);
        }
    }
}

fn register_template(registry: &mut Registry, template: &Template, structure: &Structure) {
    let mut builder = registry.template(template);
    for (pos, block) in structure.blocks() {
        builder = builder.fill(block, pos, pos);
    }
}

/// A grid of its own for each structure, from its full name.
fn salt(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

/// One placed structure, shared by the generation threads.
struct Generator {
    name: String,
    rules: Rules,
    structure: Structure,
    biome: Option<&'static Tag>,
    foundation: Option<&'static Block>,
    /// Ids exist only once the game has loaded, so this is filled at the first cube.
    resolved: OnceLock<Option<Resolved>>,
    sites: ColumnCache<Option<Site>>,
}

impl Generator {
    fn generate(&self, g: &mut Gen) {
        let Some(resolved) = self.resolved.get_or_init(|| self.resolve()) else {
            return;
        };
        let [ox, oy, oz] = g.origin();
        let [sx, _, sz] = resolved.size;
        for cell_z in site::cells(oz, oz + 15, sz, self.rules.spacing) {
            for cell_x in site::cells(ox, ox + 15, sx, self.rules.spacing) {
                let site = self.site(g, [cell_x, cell_z], [sx, sz]);
                if let Some(site) = site.as_ref() {
                    stamp::stamp(resolved, site, [ox, oy, oz], &mut |[x, y, z], state| {
                        g.set(x, y, z, state)
                    });
                }
            }
        }
    }

    fn site(&self, g: &Gen, cell: [i32; 2], footprint: [i32; 2]) -> Arc<Option<Site>> {
        // The cache is keyed by seed: the dimension goes into it so dimensions don't share sites.
        let key = g.seed() ^ noise::mix(g.dimension() as u64 + 1);
        let ground = |x, z| ground(g, x, z);
        let fits_biome = |biome| self.biome.is_none_or(|tag| tag.on_biome(biome));
        self.sites.get(key, cell[0], cell[1], || {
            let site = site::site_in(&self.rules, footprint, g.seed(), cell, &ground, &fits_biome);
            if let Some(site) = &site {
                // `world.structure_near` doesn't know these: this is how to find one.
                debug!(
                    "structure {} at {:?} in dimension {}",
                    self.name,
                    site.corner,
                    g.dimension()
                );
            }
            site
        })
    }

    /// `None` (said once) if a block is unknown; the template check at load should have caught it.
    fn resolve(&self) -> Option<Resolved> {
        let palette: Option<Vec<u32>> = self
            .structure
            .palette
            .iter()
            .map(|name| Block::lookup(name))
            .collect();
        let (Some(palette), Some(air)) = (palette, Block::lookup("core:air")) else {
            error!(
                "structure {}: a block in it is unknown, so it is not placed",
                self.name
            );
            return None;
        };
        let foundation = self
            .foundation
            .map(|block| block.id())
            .filter(|&id| id != u32::MAX);
        let states = self
            .structure
            .indices
            .iter()
            .map(|&i| palette[i as usize])
            .collect();
        Some(Resolved {
            size: self.structure.size.map(|v| v as i32),
            states,
            air,
            foundation,
        })
    }
}

/// `None` where nothing may stand: no terrain, or a cave took the surface.
fn ground(g: &Gen, x: i32, z: i32) -> Option<Ground> {
    let column = g.column(x, z);
    (column.height != i32::MIN && !g.surface_carved(x, z)).then_some(Ground {
        height: column.height,
        biome: column.biome,
    })
}
