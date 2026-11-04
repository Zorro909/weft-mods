//! Crafting recipes, on the base game's crafting table. They are written in
//! `assets/recipes/*.toml`; `build.rs` checks those files and turns them into `RECIPES` when the
//! mod is built.

use base_api::CRAFTING;
use weft_sdk::*;

pub enum Recipe {
    Shaped {
        result: &'static str,
        count: u32,
        /// Rows of letters, a space for an empty cell.
        pattern: &'static [&'static str],
        /// What each letter of the pattern is: an item name or a `#tag`.
        key: &'static [(char, &'static str)],
    },
    // For recipe files to use; none does yet.
    #[allow(dead_code)]
    Shapeless {
        result: &'static str,
        count: u32,
        inputs: &'static [&'static str],
    },
}

/// Every recipe of every file, in the order of the file names.
static RECIPES: &[Recipe] = include!(concat!(env!("OUT_DIR"), "/recipes.rs"));

pub fn register(r: &mut Registry) {
    for recipe in RECIPES {
        match recipe {
            Recipe::Shaped {
                result,
                count,
                pattern,
                key,
            } => drop(r.shaped(&CRAFTING, result, *count, pattern, key)),
            // Crafting takes no time: that is for processing recipes.
            Recipe::Shapeless {
                result,
                count,
                inputs,
            } => drop(r.shapeless(&CRAFTING, result, *count, inputs, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(recipe: &Recipe) -> &'static str {
        match recipe {
            Recipe::Shaped { result, .. } | Recipe::Shapeless { result, .. } => result,
        }
    }

    #[test]
    fn every_item_and_block_with_a_name_can_be_crafted() {
        // The gantry frame is put up by excavators and has no item.
        const NOT_CRAFTED: [&str; 1] = ["gantry_frame"];
        let lang = include_str!("../assets/lang/en_us.lang");
        let named = lang.lines().filter_map(|line| {
            let name = line.split(" = ").next()?;
            let name = name
                .strip_prefix("item.cogwork.")
                .or(name.strip_prefix("block.cogwork."))?;
            (!name.contains('.')).then_some(name)
        });
        for name in named.filter(|name| !NOT_CRAFTED.contains(name)) {
            let recipes = RECIPES.iter().filter(|r| result(r) == name).count();
            assert_eq!(recipes, 1, "{name}: one recipe");
        }
    }

    #[test]
    fn a_recipe_reads_as_its_file_writes_it() {
        let cog = RECIPES.iter().find(|r| result(r) == "wooden_cog");
        let Some(Recipe::Shaped {
            count,
            pattern,
            key,
            ..
        }) = cog
        else {
            panic!("no shaped recipe for wooden_cog");
        };
        assert_eq!(*count, 1, "one if the file says nothing");
        assert_eq!(*pattern, [" s ", "s s", " s "], "the pattern");
        assert_eq!(*key, [('s', "#stick")], "the key");
    }
}
