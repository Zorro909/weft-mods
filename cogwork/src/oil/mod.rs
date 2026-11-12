//! Oil: a thick black liquid found in pockets deep underground (`fields`), each under a seep of
//! oil-soaked ground that gives it away. Pockets don't renew: once pumped dry, a field is gone.

mod fields;

use weft_sdk::{Block, Item, Registry, WithComponents, tool_tier};

pub static OIL: Block = Block::named("oil");
static OIL_BUCKET: Item = Item::named("oil_bucket");
static BUCKET: Item = Item::named("base:bucket");
/// The ground of a seep, where it was dirt or grass, and where it was sand.
pub static OILY_DIRT: Block = Block::named("oily_dirt");
pub static OIL_SAND: Block = Block::named("oil_sand");

pub fn register(r: &mut Registry) {
    // Slower and shorter than water, about as lava flows: oil is thick.
    r.liquid(&OIL)
        .block(|block| block.light_opacity(15).drag(0.4).tag("oil"))
        .flow_delay(20)
        .drop_off(2)
        .slope_find(2)
        .fog(0x0a0806, 1.5)
        .bucket(&OIL_BUCKET, &BUCKET, &crate::TAB);
    for (block, texture, tool, sounds) in [
        (&OILY_DIRT, "oily_dirt", "shovel", "base:block.gravel"),
        (&OIL_SAND, "oil_sand", "shovel", "base:block.sand"),
    ] {
        r.block(block)
            .texture(texture)
            .hardness(0.6)
            .mined_by(tool, tool_tier::HAND)
            .sound_group(sounds)
            .tab(&crate::TAB);
    }
    fields::register(r);
}

#[cfg(test)]
mod tests {
    #[test]
    fn its_pictures_are_there() {
        for file in [
            "textures/block/oil_still.png",
            "textures/block/oily_dirt.png",
            "textures/block/oil_sand.png",
            "textures/item/oil_bucket.png",
        ] {
            assert!(crate::has_asset(file), "{file}");
        }
    }
}
