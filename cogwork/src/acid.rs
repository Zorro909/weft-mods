//! Sulfuric acid: what the acid tank makes of sulfur and water, and what the leaching vat leaches
//! Rich Ore Rubble into Pure with. It is a Cogwork liquid that flows like water, carried by any
//! liquid tube and tank, twice as fast by the pressure tube, and it comes in a bucket.

use std::sync::LazyLock;

use weft_sdk::{Block, Item, Registry};

use crate::liquid::{LiquidKey, key_named};

pub static SULFURIC_ACID: Block = Block::named("sulfuric_acid");
static ACID_BUCKET: Item = Item::named("sulfuric_acid_bucket");
static BUCKET: Item = Item::named("base:bucket");

pub fn register(r: &mut Registry) {
    // As water flows: vanilla numbers, a step every 5 ticks and 7 blocks out.
    r.liquid(&SULFURIC_ACID)
        .block(|block| block.translucent().light_opacity(2).drag(0.5))
        .flow_delay(5)
        .drop_off(1)
        .fog(0xb8c040, 0.0, 8.0)
        .bucket(&ACID_BUCKET, &BUCKET, &crate::TAB);
}

/// The key of sulfuric acid, as holders keep liquids.
pub fn key() -> LiquidKey {
    static KEY: LazyLock<LiquidKey> = LazyLock::new(|| key_named("cogwork:sulfuric_acid"));
    *KEY
}

#[cfg(test)]
mod tests {
    #[test]
    fn acid_has_its_pictures_and_names() {
        for file in [
            "textures/block/sulfuric_acid_still.png",
            "textures/item/sulfuric_acid_bucket.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
        for key in [
            "block.cogwork.sulfuric_acid",
            "item.cogwork.sulfuric_acid_bucket",
        ] {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }
}
