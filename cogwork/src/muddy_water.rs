//! Muddy water: what a flotation cell leaves of its water once the froth has carried the ore off,
//! fine rock and a film of creosote in it. It is a Cogwork liquid that flows like slow water and
//! comes in a bucket; a clarifier settles it into clear water again.

use std::sync::LazyLock;

use weft_sdk::{Block, Item, Registry};

use crate::liquid::{LiquidKey, key_named};

pub static MUDDY_WATER: Block = Block::named("muddy_water");
static MUDDY_WATER_BUCKET: Item = Item::named("muddy_water_bucket");
static BUCKET: Item = Item::named("base:bucket");

pub fn register(r: &mut Registry) {
    // Murky: a little see-through, it lets a little light through and slows a swimmer more
    // than water does.
    r.liquid(&MUDDY_WATER)
        .block(|block| block.translucent().light_opacity(6).drag(0.6))
        .flow_delay(10)
        .drop_off(1)
        .fog(0x4a3c28, 0.0, 6.0)
        .bucket(&MUDDY_WATER_BUCKET, &BUCKET, &crate::TAB);
}

/// The key of muddy water, as holders keep liquids.
pub fn key() -> LiquidKey {
    static KEY: LazyLock<LiquidKey> = LazyLock::new(|| key_named("cogwork:muddy_water"));
    *KEY
}

#[cfg(test)]
mod tests {
    #[test]
    fn muddy_water_has_its_pictures_and_names() {
        for file in [
            "textures/block/muddy_water_still.png",
            "textures/item/muddy_water_bucket.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
        for key in [
            "block.cogwork.muddy_water",
            "item.cogwork.muddy_water_bucket",
        ] {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }
}
