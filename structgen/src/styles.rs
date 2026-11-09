//! Named sets of blocks for the material slots of this mod's buildings (see
//! `docs/materials.md`): a brick roof, a spruce frame. Settlements pick them to vary their
//! buildings, and `/structure load` takes them by name.

use structgen_api::Materials;

/// Roof styles: each name and the block the roof is of, its stairs and slabs too.
const ROOFS: &[(&str, &str)] = &[
    ("spruce_roof", "base:spruce_planks"),
    ("oak_roof", "base:oak_planks"),
    ("birch_roof", "base:birch_planks"),
    ("brick_roof", "base:bricks"),
    ("stone_roof", "base:cobblestone"),
];

/// Street styles: each name and its blocks, in the order of `STREET_SLOTS`.
const STREETS: &[(&str, [&str; 5])] = &[
    (
        "cobble_street",
        [
            "base:cobblestone",
            "base:mossy_cobblestone",
            "base:stone_bricks",
            "base:stone_bricks_slab",
            "base:stone_bricks_stairs",
        ],
    ),
    (
        "stone_street",
        [
            "base:stone_bricks",
            "base:cobblestone",
            "base:stone",
            "base:stone_slab",
            "base:stone_stairs",
        ],
    ),
    (
        "brick_street",
        [
            "base:bricks",
            "base:stone_bricks",
            "base:stone_bricks",
            "base:stone_bricks_slab",
            "base:bricks_stairs",
        ],
    ),
    (
        "sandstone_street",
        [
            "base:sandstone",
            "base:sand",
            "base:stone_bricks",
            "base:sandstone_slab",
            "base:sandstone_stairs",
        ],
    ),
];
const STREET_SLOTS: [&str; 5] = [
    "street/paving",
    "street/patch",
    "street/kerb",
    "street/gutter",
    "street/step",
];

/// The other styles: each name and the blocks it puts in which slots.
const OTHERS: &[(&str, &[(&str, &str)])] = &[
    ("oak_frame", &[("frame/log", "base:oak_log")]),
    ("spruce_frame", &[("frame/log", "base:spruce_log")]),
    ("birch_frame", &[("frame/log", "base:birch_log")]),
    ("birch_infill", &[("wall/infill", "base:birch_planks")]),
    ("oak_infill", &[("wall/infill", "base:oak_planks")]),
    ("spruce_infill", &[("wall/infill", "base:spruce_planks")]),
    ("white_infill", &[("wall/infill", "base:wool")]),
    ("cobble_ground", &[("ground", "base:cobblestone")]),
    (
        "gravel_path",
        &[("path/surface", "base:gravel"), ("path/worn", "base:dirt")],
    ),
    (
        "dirt_path",
        &[("path/surface", "base:dirt"), ("path/worn", "base:gravel")],
    ),
];

/// Every style's name.
pub fn names() -> impl Iterator<Item = &'static str> {
    let roofs = ROOFS.iter().map(|(name, _)| *name);
    let streets = STREETS.iter().map(|(name, _)| *name);
    roofs
        .chain(streets)
        .chain(OTHERS.iter().map(|(name, _)| *name))
}

/// The styles named in `names`, each over the one before: `brick_roof,spruce_frame`. `None`
/// for a name no style has.
pub fn named(names: &str) -> Option<Materials> {
    let mut names = names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty());
    names.try_fold(Materials::new(), |materials, name| {
        let known = self::names().any(|style| style == name);
        known.then(|| add(materials, name))
    })
}

/// `materials` with style `name`'s blocks put in their slots; unchanged for a name no style has
/// (`""` for the defaults).
pub fn add(materials: Materials, name: &str) -> Materials {
    blocks(name)
        .into_iter()
        .fold(materials, |materials, (slot, block)| {
            materials.with(slot, &block)
        })
}

/// The blocks of style `name` and their slots, none for a name no style has.
fn blocks(name: &str) -> Vec<(&'static str, String)> {
    if let Some(block) = find(ROOFS, name) {
        return vec![
            ("roof/stair", format!("{block}_stairs")),
            ("roof/slab", format!("{block}_slab")),
            ("roof/block", block.to_string()),
        ];
    }
    if let Some(blocks) = find(STREETS, name) {
        return STREET_SLOTS
            .into_iter()
            .zip(blocks.map(String::from))
            .collect();
    }
    let others = find(OTHERS, name).copied().unwrap_or_default();
    others
        .iter()
        .map(|&(slot, block)| (slot, block.to_string()))
        .collect()
}

/// What `styles` has under `name`.
fn find<T>(styles: &'static [(&str, T)], name: &str) -> Option<&'static T> {
    styles
        .iter()
        .find(|(style, _)| *style == name)
        .map(|(_, blocks)| blocks)
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
        assert_eq!(
            materials.get("roof/slab"),
            Some("base:bricks_slab"),
            "the roof's slabs"
        );
        assert_eq!(named(""), Some(Materials::new()), "none at all");
        assert_eq!(named("gold_roof"), None, "an unknown style");
    }
}
