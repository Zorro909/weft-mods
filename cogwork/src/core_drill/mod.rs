//! The core drill: a hand-cranked tripod that tells a prospector what ore lies below it. Each use
//! with an empty hand turns its crank once and drills half a band of 16 layers deeper; sixteen
//! turns reach 128 below (`survey`). What the bit brings up is counted a little every tick, not
//! all at once (`count`), and once it has all been counted the drill reports in chat, for each
//! kind of ore, how many blocks of it lie in each band of the 16 by 16 square around it. A use
//! after that says it again. Moving the drill starts it over.
//!
//! Its shape is the Blockbench model `assets/models/block/core_drill.json`, four steel legs under
//! a gearbox; the crank and the drill rod that turn are `assets/models/core_drill_rod.geo.json`
//! (`rod`).

mod count;
mod rod;
mod survey;

pub use count::{OreKind, ore_kind};
use survey::{BANDS, KINDS, TURNS};
use weft_sdk::{
    Arg, Block, Component, Facings, Output, Registry, Toward, Use, World, component_flags,
    item_name, tool_tier, value,
};

use crate::ghost::{ItemKey, item_of};
use crate::lib::Pos;

pub static CORE_DRILL: Block = Block::named("core_drill");

value! {
    /// A core drill's state: its `rod` (an entity), the game time it last `turned_at`, the
    /// `turns` it has had, the blocks below it `counted`, the player who turned it last
    /// (`driller`) and whether it has `reported` to them. Of each kind of ore it found, the key of
    /// its `ores/<m>` name (`kinds`), the key of the name of the first block of it found
    /// (`shown`), and its `counts` by band, kind after kind. The game time it last `counted_at`,
    /// so it counts once a tick (`machine::first_call_at`).
    pub struct Core {
        pub rod: u64,
        pub turned_at: u64,
        pub turns: u32,
        pub counted: u32,
        pub driller: u64,
        pub reported: u32,
        pub waited: u32,
        pub kinds: [u64; KINDS],
        pub shown: [ItemKey; KINDS],
        pub counts: [u32; KINDS * BANDS],
        pub counted_at: u64,
    }
}
static CORE: Component<Core> = Component::named("cogwork:core");

impl Default for Core {
    fn default() -> Core {
        Core {
            rod: 0,
            turned_at: 0,
            turns: 0,
            counted: 0,
            driller: 0,
            reported: 0,
            waited: 0,
            kinds: [0; KINDS],
            shown: [0; KINDS],
            counts: [0; KINDS * BANDS],
            counted_at: 0,
        }
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(&CORE, Core::default(), component_flags::SAVE);
    r.system(&CORE, count::tick);
    r.block(&CORE_DRILL)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("core_drill")
        // The crank too, so aiming at it aims at the block.
        .hitbox([1.0, 0.0, 1.0], [15.0, 16.0, 15.0])
        .hardness(2.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .on_use(use_drill)
        .on_break(crate::loading::let_go)
        .entity(|entity| entity.attach(&CORE))
        .tab(&crate::TAB);
    rod::register(r);
}

/// A player uses the core drill: with an empty hand it turns, or once it has counted all the way
/// down, or to the floor of the world, says what it found. False passes the click on to what they hold.
fn use_drill(click: &mut Use) -> bool {
    let world = click.world();
    let pos = Pos::from(click.pos());
    let id = world.block_entity(pos);
    let player = click.player();
    let held = world.inventory(player).get(world.selected_slot(player));
    let Some(mut core) = world.component(id, &CORE).filter(|_| held.is_empty()) else {
        return false;
    };
    if core.is_done() {
        report(&world, player, &core);
    } else if core.turns < TURNS {
        turn(&world, player, pos, &mut core);
    } else {
        world.hint("cogwork.core_drill.counting", &[]).to(player);
    }
    if let Some(state) = world.component_mut(id, &CORE) {
        *state = core;
    }
    true
}

/// Turns the crank once for `player`, unless it is still going round from the last turn.
fn turn(world: &World, player: u64, pos: Pos, core: &mut Core) {
    let now = world.game_time();
    if now < core.turned_at + rod::TURN_TICKS {
        return;
    }
    (core.turned_at, core.driller, core.turns) = (now, player, core.turns + 1);
    rod::turn(world, core.rod, pos);
    let args = [
        Arg::Int(survey::depth(core.turns).into()),
        Arg::Int(core.turns.into()),
        Arg::Int(TURNS.into()),
    ];
    world.hint("cogwork.core_drill.turned", &args).to(player);
}

/// Tells `player` in chat what the drill `core` found: a line for each kind of ore, with its
/// blocks in each band from the top.
pub fn report(world: &World, player: u64, core: &Core) {
    let found = core.found();
    if found.is_empty() {
        world.chat("cogwork.core_drill.no_ore", &[]).to(player);
        return;
    }
    world.chat("cogwork.core_drill.report", &[]).to(player);
    for (shown, counts) in found {
        let name = match item_of(shown) {
            0 => None,
            item => Some(format!("item:{}", item_name(item))),
        };
        let mut args = vec![match &name {
            Some(name) => Arg::Name(name),
            None => Arg::Key("cogwork.core_drill.some_ore"),
        }];
        args.extend(counts.map(|count| Arg::Int(count.into())));
        world.chat("cogwork.core_drill.ore", &args).to(player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_it_says_has_a_name() {
        for key in [
            "block.cogwork.core_drill",
            "cogwork.core_drill.turned",
            "cogwork.core_drill.counting",
            "cogwork.core_drill.report",
            "cogwork.core_drill.ore",
            "cogwork.core_drill.no_ore",
            "cogwork.core_drill.some_ore",
        ] {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_core_drill_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Core, rod), 0, "its rod");
        assert_eq!(offset_of!(Core, turned_at), 8, "when it turned");
        assert_eq!(offset_of!(Core, turns), 16, "its turns");
        assert_eq!(offset_of!(Core, counted), 20, "what it counted");
        assert_eq!(offset_of!(Core, driller), 24, "who turned it");
        assert_eq!(offset_of!(Core, reported), 32, "whether it said");
        assert_eq!(offset_of!(Core, waited), 36, "how long it waited");
        assert_eq!(offset_of!(Core, kinds), 40, "the kinds of ore");
        assert_eq!(offset_of!(Core, shown), 104, "their names");
        assert_eq!(offset_of!(Core, counts), 168, "their counts");
        assert_eq!(offset_of!(Core, counted_at), 424, "when it last counted");
    }

    #[test]
    fn the_core_drill_has_its_models() {
        for file in [
            "models/block/core_drill.json",
            "models/item/core_drill.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
