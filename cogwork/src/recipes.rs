//! Crafting recipes, on the base game's crafting table. Ingredients go by tag where the base game
//! has one, so other mods' planks, ingots and gems work too.

use base_api::CRAFTING;
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    cogs(r);
    tubes(r);
    machines(r);
}

fn cogs(r: &mut Registry) {
    r.shaped(
        &CRAFTING,
        "wooden_cog",
        1,
        &[" s ", "s s", " s "],
        &[('s', "#stick")],
    );
    for (cog, material, under) in [
        ("stone_cog", "#cobblestone", "#cogs/wood"),
        ("iron_cog", "#ingots/iron", "#cogs/stone"),
        ("gold_cog", "#ingots/gold", "#cogs/iron"),
        ("diamond_cog", "#gems/diamond", "#cogs/gold"),
    ] {
        r.shaped(
            &CRAFTING,
            cog,
            1,
            &[" m ", "mcm", " m "],
            &[('m', material), ('c', under)],
        );
    }
    r.shaped(
        &CRAFTING,
        "tuning_wrench",
        1,
        &["i i", " c ", " i "],
        &[('i', "#ingots/iron"), ('c', "#cogs/stone")],
    );
}

fn tubes(r: &mut Registry) {
    for (tube, wall) in [
        ("stone_tube", "#smooth_stone"),
        ("timber_tube", "#planks"),
        ("gilded_tube", "#ingots/gold"),
        ("sorting_tube", "#gems/diamond"),
        ("directing_tube", "#ingots/iron"),
        ("void_tube", "base:obsidian"),
    ] {
        r.shaped(
            &CRAFTING,
            tube,
            8,
            &["wgw"],
            &[('w', wall), ('g', "base:glass")],
        );
    }
    r.shaped(
        &CRAFTING,
        "torque_shaft",
        8,
        &["grg"],
        &[('g', "#ingots/gold"), ('r', "base:redstone")],
    );
}

fn machines(r: &mut Registry) {
    r.shaped(
        &CRAFTING,
        "spring_engine",
        1,
        &["ppp", " g ", "cPc"],
        &[
            ('p', "#planks"),
            ('g', "base:glass"),
            ('c', "#cogs/wood"),
            ('P', "base:piston"),
        ],
    );
    r.shaped(
        &CRAFTING,
        "ember_engine",
        1,
        &["sss", " g ", "cPc"],
        &[
            ('s', "#cobblestone"),
            ('g', "base:glass"),
            ('c', "#cogs/stone"),
            ('P', "base:piston"),
        ],
    );
    r.shaped(
        &CRAFTING,
        "excavator",
        1,
        &["iri", "gPg", "dcd"],
        &[
            ('i', "#cogs/iron"),
            ('r', "base:redstone"),
            ('g', "#cogs/gold"),
            ('P', "base:diamond_pickaxe"),
            ('d', "#cogs/diamond"),
            ('c', "base:chest"),
        ],
    );
    r.shaped(
        &CRAFTING,
        "assembly_bench",
        1,
        &["iTi", "cCc", "iri"],
        &[
            ('i', "#ingots/iron"),
            ('T', "base:crafting_table"),
            ('c', "#cogs/iron"),
            ('C', "base:chest"),
            ('r', "base:redstone"),
        ],
    );
    r.shaped(
        &CRAFTING,
        "leveler",
        1,
        &["iri", "gSg", "iCi"],
        &[
            ('i', "#ingots/iron"),
            ('r', "base:redstone"),
            ('g', "#cogs/gold"),
            ('S', "#shovels/diamond"),
            ('C', "base:chest"),
        ],
    );
    r.shaped(
        &CRAFTING,
        "survey_marker",
        4,
        &["t", "s"],
        &[('t', "base:redstone_torch"), ('s', "#stick")],
    );
}
