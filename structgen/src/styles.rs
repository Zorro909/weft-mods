//! Named sets of blocks for the material slots of this mod's buildings (see
//! `docs/materials.md`): a brick roof, a spruce frame. A settlement picks a roof for each
//! building and a frame and infill for all of them. The engine fills a template's slots from
//! one palette, so every mix of a roof, a frame and an infill is a palette of its own,
//! `structgen:styles/brick_roof/spruce_frame`. A town's paving is the paved copies' own default
//! (`buildings.rs`).

use std::collections::HashMap;
use std::sync::OnceLock;

use weft_sdk::*;

/// Roof styles: each name and the block the roof is of, its stairs and slabs too.
const ROOFS: &[(&str, &str)] = &[
    ("spruce_roof", "base:spruce_planks"),
    ("oak_roof", "base:oak_planks"),
    ("birch_roof", "base:birch_planks"),
    ("brick_roof", "base:bricks"),
    ("stone_roof", "base:cobblestone"),
];
/// Frame styles: the logs of the timber frame.
const FRAMES: &[(&str, &str)] = &[
    ("oak_frame", "base:oak_log"),
    ("spruce_frame", "base:spruce_log"),
    ("birch_frame", "base:birch_log"),
];
/// Infill styles: the panels between the frame's beams.
const INFILLS: &[(&str, &str)] = &[
    ("birch_infill", "base:birch_planks"),
    ("oak_infill", "base:oak_planks"),
    ("spruce_infill", "base:spruce_planks"),
    ("white_infill", "base:wool"),
];

/// By name, every palette `register` made.
static PALETTES: OnceLock<HashMap<String, &'static Palette>> = OnceLock::new();

/// A palette for every mix of a roof, a frame and an infill, each or none.
pub fn register(r: &mut Registry) {
    let none = [""];
    let names =
        |styles: &'static [(&str, &str)]| none.into_iter().chain(styles.iter().map(|s| s.0));
    let mut palettes = HashMap::new();
    for roof in names(ROOFS) {
        for frame in names(FRAMES) {
            for infill in names(INFILLS) {
                let styles = [roof, frame, infill];
                let Some(name) = palette_name(&styles) else {
                    continue;
                };
                let palette = Palette::generated(name.clone());
                let mut builder = r.palette(palette);
                for (slot, block) in slots(&styles) {
                    builder = builder.slot(slot, &block);
                }
                drop(builder);
                palettes.insert(name, palette);
            }
        }
    }
    let _ = PALETTES.set(palettes);
}

/// The palette of `styles` (a roof, a frame, an infill, `""` for the defaults); `None` for the
/// defaults alone.
pub fn palette(styles: &[&str]) -> Option<&'static Palette> {
    PALETTES.get()?.get(&palette_name(styles)?).copied()
}

/// `styles/brick_roof/spruce_frame`; `None` when it would say nothing.
fn palette_name(styles: &[&str]) -> Option<String> {
    let parts: Vec<&str> = (styles.iter().copied())
        .filter(|part| !part.is_empty())
        .collect();
    (!parts.is_empty()).then(|| format!("styles/{}", parts.join("/")))
}

/// The slots `styles` fill and their blocks.
fn slots(styles: &[&str]) -> Vec<(&'static str, String)> {
    let mut slots = Vec::new();
    for &style in styles {
        if let Some(block) = find(ROOFS, style) {
            slots.push(("roof/stair", format!("{block}_stairs")));
            slots.push(("roof/slab", format!("{block}_slab")));
            slots.push(("roof/block", block.to_string()));
        }
        if let Some(block) = find(FRAMES, style) {
            slots.push(("frame/log", block.to_string()));
        }
        if let Some(block) = find(INFILLS, style) {
            slots.push(("wall/infill", block.to_string()));
        }
    }
    slots
}

/// The block of `style` among `styles`.
fn find(styles: &[(&str, &'static str)], style: &str) -> Option<&'static str> {
    styles
        .iter()
        .find(|(name, _)| *name == style)
        .map(|(_, block)| *block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_palette_is_named_by_its_styles() {
        let name = palette_name(&["brick_roof", "", "white_infill"]);
        assert_eq!(name.as_deref(), Some("styles/brick_roof/white_infill"));
        assert_eq!(palette_name(&["", "", ""]), None, "the defaults alone");
    }

    #[test]
    fn styles_fill_their_slots() {
        let slots = slots(&["brick_roof", "spruce_frame", ""]);
        let block = |slot: &str| {
            slots
                .iter()
                .find(|(s, _)| *s == slot)
                .map(|(_, b)| b.as_str())
        };
        assert_eq!(block("roof/stair"), Some("base:bricks_stairs"), "the roof");
        assert_eq!(block("roof/slab"), Some("base:bricks_slab"), "its slabs");
        assert_eq!(block("frame/log"), Some("base:spruce_log"), "the frame");
        assert_eq!(block("wall/infill"), None, "the infill's default");
    }
}
