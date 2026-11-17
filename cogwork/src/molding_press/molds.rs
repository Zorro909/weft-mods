//! The molds of the molding press, one row each: the mold, what a press with it takes from the
//! input and the part it makes. Molds are cut by hand at a crafting table and wear a use a press.

use weft_sdk::{Item, ItemStack, Registry, Tag, WithComponents};

use crate::excavator::kind::{Cost, Material};
use crate::plastics::{COMPOSITE_PANEL, PLASTIC_GEAR, PLASTIC_PELLET, PLASTIC_PIPE, PLASTIC_SHEET};

pub static SHEET_MOLD: Item = Item::named("sheet_mold");
pub static GEAR_MOLD: Item = Item::named("gear_mold");
pub static PIPE_MOLD: Item = Item::named("pipe_mold");
pub static PANEL_MOLD: Item = Item::named("panel_mold");
static STEEL_INGOTS: Tag = Tag::named("ingots/steel");

/// Presses a mold lasts.
pub const MOLD_USES: u32 = 64;

/// A mold, what one press with it takes, and the part it makes.
pub struct Shape {
    pub mold: &'static Item,
    pub takes: &'static [Cost],
    pub makes: &'static Item,
}

/// `count` plastic pellets.
const fn pellets(count: u32) -> Cost {
    Cost {
        material: Material::Item(&PLASTIC_PELLET),
        count,
    }
}

pub static SHAPES: [Shape; 4] = [
    Shape {
        mold: &SHEET_MOLD,
        takes: &[pellets(4)],
        makes: &PLASTIC_SHEET,
    },
    Shape {
        mold: &GEAR_MOLD,
        takes: &[pellets(2)],
        makes: &PLASTIC_GEAR,
    },
    Shape {
        mold: &PIPE_MOLD,
        takes: &[pellets(3)],
        makes: &PLASTIC_PIPE,
    },
    Shape {
        mold: &PANEL_MOLD,
        takes: &[
            Cost {
                material: Material::Item(&PLASTIC_SHEET),
                count: 1,
            },
            Cost {
                material: Material::Tagged(&STEEL_INGOTS),
                count: 1,
            },
        ],
        makes: &COMPOSITE_PANEL,
    },
];

pub fn register(r: &mut Registry) {
    for shape in &SHAPES {
        r.item(shape.mold)
            .durability(MOLD_USES)
            .tag("cogwork:molds")
            .tab(&crate::TAB);
    }
}

/// The shape a press with `mold` in its mold slot makes, if it is a mold.
pub fn shape_of(mold: ItemStack) -> Option<&'static Shape> {
    SHAPES
        .iter()
        .find(|shape| !mold.is_empty() && mold.item == shape.mold.id())
}

/// The items the mold slot lets in, as a slot's `accepts` rule.
pub fn mold_rule() -> String {
    let names = SHAPES.iter().map(|shape| shape.mold.name().to_string());
    names.collect::<Vec<_>>().join(" ")
}

impl Shape {
    /// Whether a press with this mold takes `item` into its input.
    pub fn presses(&self, item: u32) -> bool {
        self.takes.iter().any(|cost| cost.material.is(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pellets each part takes, by its mold.
    fn pellets_for(mold: &Item) -> u32 {
        let shape = SHAPES.iter().find(|shape| std::ptr::eq(shape.mold, mold));
        let takes = shape.expect("a shape of the mold").takes;
        takes.iter().map(|cost| cost.count).sum()
    }

    #[test]
    fn each_part_takes_its_pellets() {
        assert_eq!(pellets_for(&SHEET_MOLD), 4, "a sheet");
        assert_eq!(pellets_for(&GEAR_MOLD), 2, "a gear");
        assert_eq!(pellets_for(&PIPE_MOLD), 3, "a pipe");
        assert_eq!(pellets_for(&PANEL_MOLD), 2, "a sheet and a steel ingot");
    }

    #[test]
    fn every_mold_has_its_name_and_picture() {
        for shape in &SHAPES {
            let name = shape.mold.path();
            assert!(crate::translated(&format!("item.cogwork.{name}")), "{name}");
            let icon = format!("textures/item/{name}.png");
            assert!(crate::has_asset(&icon), "no {icon}");
        }
    }

    #[test]
    fn the_mold_slot_names_every_mold() {
        let rule = mold_rule();
        for name in [":sheet_mold", ":gear_mold", ":pipe_mold", ":panel_mold"] {
            assert!(rule.split(' ').any(|mold| mold.ends_with(name)), "{rule}");
        }
    }
}
