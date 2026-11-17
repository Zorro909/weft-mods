//! What a refinery column makes of crude oil: gas, diesel, lubricant and tar, four Cogwork
//! liquids that flow like oil and come in buckets, and sulfur. Each leaves by the taps on its own
//! layer of the column, tar lowest and gas highest, as they boil off a real column. What one tick
//! of crude gives depends on where the column's temperature stands (`cut`).

use std::sync::LazyLock;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{Block, Item, Registry};

use super::heat::Band;
use crate::liquid::{LiquidKey, key_named};

/// Millibuckets of crude a column distils a tick: a bucket in five seconds.
pub const CRUDE_A_TICK: u32 = 10;
/// Millibuckets of crude that give a sulfur.
pub const CRUDE_A_SULFUR: u32 = 1_000;

/// A product of the column, by the layer of the column its taps are on, from the bottom up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Product {
    Tar = 1,
    Lubricant = 2,
    Diesel = 3,
    Gas = 4,
}

/// Each product's block, bucket and look in the world: fog color, how slowly it flows, how far.
struct Liquid {
    block: Block,
    bucket: Item,
    fog: u32,
    flow_delay: u32,
    drop_off: u32,
}

/// Gas thin and quick, tar as thick as it comes; the order is `Product`'s.
static LIQUIDS: [Liquid; 4] = [
    liquid("tar", "tar_bucket", 0x0a0908, 30, 3),
    liquid("lubricant", "lubricant_bucket", 0x6a5214, 20, 2),
    liquid("diesel", "diesel_bucket", 0x8a6a1c, 10, 2),
    liquid("gas", "gas_bucket", 0xc8c49a, 5, 1),
];

const fn liquid(
    name: &'static str,
    bucket: &'static str,
    fog: u32,
    flow_delay: u32,
    drop_off: u32,
) -> Liquid {
    Liquid {
        block: Block::named(name),
        bucket: Item::named(bucket),
        fog,
        flow_delay,
        drop_off,
    }
}

static BUCKET: Item = Item::named("base:bucket");

impl Product {
    /// The layer of the column its taps are on, counting the heater's 0.
    pub fn layer(self) -> i32 {
        self as i32
    }

    /// The product whose taps are on `layer`, if any are.
    pub fn on_layer(layer: i32) -> Option<Product> {
        Product::from_repr(u32::try_from(layer).ok()?)
    }

    /// Its place among the products, from 0.
    pub fn index(self) -> usize {
        self as usize - 1
    }

    /// The key of its liquid, as holders keep liquids.
    pub fn key(self) -> LiquidKey {
        static KEYS: LazyLock<[LiquidKey; 4]> = LazyLock::new(|| {
            std::array::from_fn(|index| {
                let product = Product::VARIANTS[index];
                key_named(&format!("cogwork:{}", product.name()))
            })
        });
        KEYS[self.index()]
    }

    /// Its name, as its liquid's block is named.
    pub fn name(self) -> &'static str {
        self.into()
    }
}

/// What one tick's crude gives, by product (`Product::index`), and the gas that vents.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Cut {
    pub made: [u32; 4],
    pub vented: u32,
}

/// What `CRUDE_A_TICK` of crude gives a column at `band`: in it, a fifth gas, two fifths
/// diesel, a fifth lubricant and a fifth tar (1,000 mB: 200, 400, 200, 200); too hot, the same
/// but the gas lost; too cold, all tar. A cold column takes none (`None`).
pub fn cut(band: Band) -> Option<Cut> {
    let fifth = CRUDE_A_TICK / 5;
    let in_band = [fifth, fifth, 2 * fifth, fifth];
    match band {
        Band::Cold => None,
        Band::Under => Some(Cut {
            made: [CRUDE_A_TICK, 0, 0, 0],
            vented: 0,
        }),
        Band::In => Some(Cut {
            made: in_band,
            vented: 0,
        }),
        Band::Over => {
            let mut made = in_band;
            let vented = std::mem::take(&mut made[Product::Gas.index()]);
            Some(Cut { made, vented })
        }
    }
}

/// Whether a column at `band` makes sulfur of its crude: tar keeps it.
pub fn gives_sulfur(band: Band) -> bool {
    matches!(band, Band::In | Band::Over)
}

pub fn register(r: &mut Registry) {
    for liquid in &LIQUIDS {
        r.liquid(&liquid.block)
            .block(|block| block.light_opacity(15).drag(0.4))
            .flow_delay(liquid.flow_delay)
            .drop_off(liquid.drop_off)
            .slope_find(2)
            .fog(liquid.fog, 0.375, 1.5)
            .bucket(&liquid.bucket, &BUCKET, &crate::TAB);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_crude_is_made_or_lost_by_rounding() {
        for band in [Band::Under, Band::In, Band::Over] {
            let cut = cut(band).expect("a warm column distils");
            let total: u32 = cut.made.iter().sum::<u32>() + cut.vented;
            assert_eq!(total, CRUDE_A_TICK, "all of the crude at {band:?}");
        }
        assert_eq!(cut(Band::Cold), None, "a cold column takes none");
    }

    #[test]
    fn a_bucket_in_band_gives_the_brief_s_cut() {
        let cut = cut(Band::In).expect("distils");
        let per_bucket = cut.made.map(|made| made * 1_000 / CRUDE_A_TICK);
        assert_eq!(
            per_bucket,
            [200, 200, 400, 200],
            "tar, lubricant, diesel, gas"
        );
    }

    #[test]
    fn too_hot_loses_the_gas_and_too_cold_makes_tar() {
        let hot = cut(Band::Over).expect("distils");
        assert_eq!(hot.made[Product::Gas.index()], 0, "no gas");
        assert_eq!(hot.vented, 2, "vented");
        let cold = cut(Band::Under).expect("distils");
        assert_eq!(cold.made[Product::Tar.index()], CRUDE_A_TICK, "all tar");
        assert!(!gives_sulfur(Band::Under), "tar keeps the sulfur");
    }

    #[test]
    fn each_product_has_its_own_layer() {
        for &product in Product::VARIANTS {
            assert_eq!(
                Product::on_layer(product.layer()),
                Some(product),
                "{product:?}"
            );
        }
        assert_eq!(Product::on_layer(0), None, "the heater's layer");
        assert_eq!(Product::on_layer(5), None, "the condenser's layer");
        assert_eq!(Product::on_layer(-1), None, "below the column");
    }

    #[test]
    fn every_product_has_its_names_and_pictures() {
        for liquid in &LIQUIDS {
            let name = liquid.block.path();
            assert!(
                crate::translated(&format!("block.cogwork.{name}")),
                "{name}"
            );
            let bucket = liquid.bucket.path();
            assert!(
                crate::translated(&format!("item.cogwork.{bucket}")),
                "{bucket}"
            );
            assert!(
                crate::has_asset(&format!("textures/block/{name}_still.png")),
                "{name}"
            );
            assert!(
                crate::has_asset(&format!("textures/item/{bucket}.png")),
                "{bucket}"
            );
        }
        for (liquid, &product) in LIQUIDS.iter().zip(Product::VARIANTS) {
            assert_eq!(liquid.block.path(), product.name(), "in `Product`'s order");
        }
    }
}
