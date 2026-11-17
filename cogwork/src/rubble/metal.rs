//! The metal of Ore Rubble, from any mod: an ore block tagged `ores/<m>` is a metal ore when an
//! item is tagged `raw_ores/<m>`, and grinding gives the first such item. The game knows every
//! tag once mods have loaded, so this is asked of it then, never listed in advance.
//!
//! Metals cogwork has a name for are listed with the color they show in; the tooltip names the
//! others plainly, and colors them by their name.

use weft_sdk::{abi, components_of, entries_with, value};

use crate::lib::BlockState;

/// Bytes a metal's name may have; an ore of a longer one drops as usual.
const NAME_LEN: usize = 32;

/// The metals cogwork has a name for (`cogwork.metal.<name>`), and their colors, `0xRRGGBB`.
/// A stack keeps a metal's place here (`Ore::named`), so the order is part of the save format:
/// new ones go last.
const NAMED: [(&str, u32); 12] = [
    ("iron", 0xD8A07A),
    ("gold", 0xF6D23C),
    ("copper", 0xE07A48),
    ("tin", 0xC8D2D6),
    ("silver", 0xE4E8EE),
    ("lead", 0x7C82A0),
    ("zinc", 0xA8BCB2),
    ("nickel", 0xD8D2A0),
    ("aluminum", 0xE8EAEE),
    ("uranium", 0x86C84E),
    ("osmium", 0x90AACC),
    ("platinum", 0xB4D6E4),
];

value! {
    /// The metal of a stack of Ore Rubble (`cogwork:ore`): its name as UTF-8 padded with zeros,
    /// and its place in `NAMED` plus one, 0 for a metal cogwork has no name for, which is what
    /// the tooltip reads, being able to compare only numbers.
    #[derive(Default)]
    pub struct Ore {
        metal: [u8; NAME_LEN],
        pub named: u32,
    }
}

impl Ore {
    /// The ore of `metal`; none for a name longer than a stack keeps.
    pub fn of(metal: &str) -> Option<Ore> {
        if metal.is_empty() || metal.len() > NAME_LEN {
            return None;
        }
        let mut bytes = [0; NAME_LEN];
        bytes[..metal.len()].copy_from_slice(metal.as_bytes());
        let named = NAMED.iter().position(|(name, _)| *name == metal);
        Some(Ore {
            metal: bytes,
            named: named.map_or(0, |place| place as u32 + 1),
        })
    }

    /// The metal's name, `iron`; empty for none or bytes that are not UTF-8.
    pub fn metal(&self) -> &str {
        let len = self.metal.iter().position(|&byte| byte == 0);
        std::str::from_utf8(&self.metal[..len.unwrap_or(NAME_LEN)]).unwrap_or("")
    }

    /// The color its flecks show in, `0xRRGGBB`: a named metal's own, any other's made from its
    /// name, light enough to tell from the stone.
    pub fn tint(&self) -> u32 {
        let metal = self.metal();
        if let Some(&(_, color)) = NAMED.iter().find(|(name, _)| *name == metal) {
            return color;
        }
        let hash = metal.bytes().fold(0x811C_9DC5_u32, |hash, byte| {
            (hash ^ byte as u32).wrapping_mul(0x0100_0193)
        });
        0x808080 | (hash & 0x7F7F7F)
    }

    /// The first item of any mod tagged `raw_ores/<metal>`, which grinding gives.
    pub fn raw_ore(&self) -> Option<u32> {
        let tag = format!("raw_ores/{}", self.metal());
        entries_with(abi::entry_kind::ITEM, &tag).first().copied()
    }
}

/// What a block is to a quarry by its ore tags.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vein<'a> {
    /// An ore of this metal, which comes up as Ore Rubble.
    Metal(&'a str),
    /// An ore with no raw ore (coal, diamond, redstone), which keeps its drops.
    Gem,
    /// No ore at all.
    Rock,
}

/// What a block with `tags` is, `has_raw_ore` saying for a metal whether any item is its raw ore.
pub fn vein_of<'a>(
    tags: impl IntoIterator<Item = &'a str>,
    has_raw_ore: impl Fn(&str) -> bool,
) -> Vein<'a> {
    let mut vein = Vein::Rock;
    for metal in tags.into_iter().filter_map(|tag| tag.strip_prefix("ores/")) {
        if has_raw_ore(metal) {
            return Vein::Metal(metal);
        }
        vein = Vein::Gem;
    }
    vein
}

/// What the block `block_state` is to a quarry, asked of the game's tags.
pub fn vein(block_state: BlockState) -> Vein<'static> {
    let tags = components_of(abi::entry_kind::BLOCK, block_state.id())
        .into_iter()
        // A tag has no schema; a component with a value is no tag.
        .filter(|(_, schema)| schema.is_empty())
        .map(|(name, _)| name);
    vein_of(tags, |metal| {
        Ore::of(metal).is_some_and(|ore| ore.raw_ore().is_some())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_metal_keeps_its_name() {
        let ore = Ore::of("iron").expect("iron fits");
        assert_eq!(ore.metal(), "iron", "read back");
        assert_eq!(ore.named, 1, "the first named metal");
        let other = Ore::of("mithril").expect("mithril fits");
        assert_eq!(other.named, 0, "a metal without a name of cogwork's");
    }

    #[test]
    fn a_name_too_long_for_a_stack_is_no_metal() {
        assert!(Ore::of(&"a".repeat(NAME_LEN + 1)).is_none(), "too long");
        assert!(Ore::of(&"a".repeat(NAME_LEN)).is_some(), "just fits");
        assert!(Ore::of("").is_none(), "nothing");
    }

    #[test]
    fn bytes_that_are_not_text_read_as_no_metal() {
        let mut ore = Ore::of("iron").expect("iron fits");
        ore.metal[0] = 0xFF;
        assert_eq!(ore.metal(), "", "a broken save");
    }

    #[test]
    fn every_metal_shows_a_light_color() {
        assert_eq!(
            Ore::of("gold").map(|ore| ore.tint()),
            Some(0xF6D23C),
            "named"
        );
        let made = Ore::of("mithril").map_or(0, |ore| ore.tint());
        assert_eq!(made & 0x808080, 0x808080, "every channel past half");
        assert_eq!(
            Ore::of("mithril").map(|ore| ore.tint()),
            Some(made),
            "the same each time"
        );
    }

    #[test]
    fn named_metals_keep_their_places() {
        assert_eq!(NAMED[0].0, "iron", "saved as 1");
        assert_eq!(NAMED[1].0, "gold", "saved as 2");
    }

    #[test]
    fn every_named_metal_has_a_name_and_a_line_in_the_tooltip() {
        for (place, (metal, _)) in NAMED.iter().enumerate() {
            let key = format!("cogwork.metal.{metal}");
            assert!(crate::translated(&key), "{key} in every lang file");
            let line = format!("{{if:cogwork:ore.named=={}}}{{lang:{key}}}", place + 1);
            assert!(
                crate::lang_line("item.cogwork.ore_rubble.tooltip").contains(&line),
                "the tooltip names {metal}"
            );
        }
    }

    #[test]
    fn a_metal_ore_is_one_with_a_raw_ore() {
        let raw = |metal: &str| metal == "iron";
        assert_eq!(
            vein_of(["ores", "ores/iron"], raw),
            Vein::Metal("iron"),
            "iron ore"
        );
        assert_eq!(vein_of(["ores", "ores/coal"], raw), Vein::Gem, "coal ore");
        assert_eq!(vein_of(["stone"], raw), Vein::Rock, "stone");
        assert_eq!(
            vein_of(["ores/coal", "ores/iron"], raw),
            Vein::Metal("iron"),
            "a metal tag wins over another"
        );
    }
}
