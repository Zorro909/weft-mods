//! Steel and what is left on the way to it: the steel bloom the blast furnace makes, which the
//! forge anvil beats into a steel ingot, and the slag that comes with each bloom, which sifts
//! into gravel and now and then a Poor iron Ore Rubble. Steel ingots are tagged as any mod's
//! ingots are, `ingots/steel` and `materials/steel`, so base's recipes pack nine into a block of
//! steel and back.

use weft_sdk::{Block, Item, Particle, Registry, WithComponents, tool_tier};

pub static STEEL_BLOOM: Item = Item::named("steel_bloom");
pub static SLAG: Item = Item::named("slag");
pub static STEEL_INGOT: Item = Item::named("steel_ingot");
pub static STEEL_BLOCK: Block = Block::named("steel_block");
/// Sparks of hot iron: out of a blast furnace's top, and off the anvil at each strike.
pub static SPARK: Particle = Particle::named("forge_spark");

pub fn register(r: &mut Registry) {
    r.item(&STEEL_BLOOM).tab(&crate::TAB);
    r.item(&SLAG).tab(&crate::TAB);
    r.item(&STEEL_INGOT)
        .tag("ingots/steel")
        .tag("materials/steel")
        .tab(&crate::TAB);
    r.block(&STEEL_BLOCK)
        .texture("steel_block")
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::IRON)
        .sound_group("base:block.stone")
        .tag("ore_blocks/steel")
        .tab(&crate::TAB);
    r.particle(&SPARK)
        .textures(&["core:generic_0", "core:generic_1"])
        .life(0.5, 1.0)
        .size(0.08, 0.14)
        .gravity(0.6)
        .drag(0.96)
        .color(0xFFD060FF, 0xD0301000)
        .fullbright();
}

#[cfg(test)]
mod tests {
    #[test]
    fn its_pictures_are_there() {
        for file in [
            "textures/item/steel_bloom.png",
            "textures/item/slag.png",
            "textures/item/steel_ingot.png",
            "textures/block/steel_block.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
