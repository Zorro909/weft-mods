//! Named sets of blocks for the material slots of this mod's buildings (see
//! `docs/materials.md`): a brick roof, a spruce frame. Settlements pick them to vary their
//! buildings, and `/structure load` takes them by name.

use structgen_api::Materials;

/// Each style's name and the blocks it puts in which slots.
pub const STYLES: &[(&str, &[(&str, &str)])] = &[
    (
        "spruce_roof",
        &[
            ("roof/stair", "base:spruce_planks_stairs"),
            ("roof/slab", "base:spruce_planks_slab"),
            ("roof/block", "base:spruce_planks"),
        ],
    ),
    (
        "oak_roof",
        &[
            ("roof/stair", "base:oak_planks_stairs"),
            ("roof/slab", "base:oak_planks_slab"),
            ("roof/block", "base:oak_planks"),
        ],
    ),
    (
        "birch_roof",
        &[
            ("roof/stair", "base:birch_planks_stairs"),
            ("roof/slab", "base:birch_planks_slab"),
            ("roof/block", "base:birch_planks"),
        ],
    ),
    (
        "brick_roof",
        &[
            ("roof/stair", "base:bricks_stairs"),
            ("roof/slab", "base:bricks_slab"),
            ("roof/block", "base:bricks"),
        ],
    ),
    (
        "stone_roof",
        &[
            ("roof/stair", "base:cobblestone_stairs"),
            ("roof/slab", "base:cobblestone_slab"),
            ("roof/block", "base:cobblestone"),
        ],
    ),
    ("oak_frame", &[("frame/log", "base:oak_log")]),
    ("spruce_frame", &[("frame/log", "base:spruce_log")]),
    ("birch_frame", &[("frame/log", "base:birch_log")]),
    ("birch_infill", &[("wall/infill", "base:birch_planks")]),
    ("oak_infill", &[("wall/infill", "base:oak_planks")]),
    ("spruce_infill", &[("wall/infill", "base:spruce_planks")]),
    ("white_infill", &[("wall/infill", "base:wool")]),
];

/// The styles named in `names`, each over the one before: `brick_roof,spruce_frame`. `None`
/// for a name no style has.
pub fn named(names: &str) -> Option<Materials> {
    let mut names = names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty());
    names.try_fold(Materials::new(), |materials, name| {
        let known = STYLES.iter().any(|(style, _)| *style == name);
        known.then(|| add(materials, blocks(name)))
    })
}

/// The blocks of style `name`, none for a name no style has (`""` for the defaults).
pub fn blocks(name: &str) -> &'static [(&'static str, &'static str)] {
    STYLES
        .iter()
        .find(|(style, _)| *style == name)
        .map_or(&[], |(_, blocks)| *blocks)
}

/// `materials` with `blocks` put in their slots.
pub fn add(materials: Materials, blocks: &[(&str, &str)]) -> Materials {
    blocks.iter().fold(materials, |materials, (slot, block)| {
        materials.with(slot, block)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_combine_in_order() {
        let materials = named("brick_roof, spruce_frame").expect("both are styles");
        assert_eq!(
            materials.get("roof/stair"),
            Some("base:bricks_stairs"),
            "the roof"
        );
        assert_eq!(
            materials.get("frame/log"),
            Some("base:spruce_log"),
            "the frame"
        );
        assert_eq!(named(""), Some(Materials::new()), "none at all");
        assert_eq!(named("gold_roof"), None, "an unknown style");
    }
}
