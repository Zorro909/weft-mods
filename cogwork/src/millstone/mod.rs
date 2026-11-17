//! The millstone: a stone runner on a stone bed, turned by hand, which grinds Ore Rubble into
//! its metal's raw ore with water and sand (`grind`). Using it with a water bucket, sand or Ore
//! Rubble puts that in; an empty hand turns the runner (`runner`), and sneaking takes out what
//! it ground, or else the Ore Rubble it holds. Nothing reaches it by tube: it has no slots, and
//! keeps what it holds in its state, which its block takes along when broken.
//!
//! Its shape is the Blockbench model `assets/models/block/millstone.json`; the ring between the
//! runner and the bed's rim shows what it holds (`#grist`).

mod grind;
mod runner;

use strum::{IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Facings, Item, ItemStack, Output, Registry, StackData, SystemBatch, Tag,
    Toward, Use, World, component_flags, tool_tier, value,
};

use crate::ghost::{ItemKey, item_of, key_of};
use crate::lib::{Pos, TypedWorld};
use crate::rubble::{Ore, ore_of, ore_rubble};

pub static MILLSTONE: Block = Block::named("millstone");
static WATER_BUCKET: Item = Item::named("base:water_bucket");
static BUCKET: Item = Item::named("base:bucket");
static SAND: Tag = Tag::named("sand");

value! {
    /// A millstone's state: its `runner` (an entity) and the game time it last `turned_at`; the
    /// raw ore it `ground` (a key, as ghosts keep items) and how many; the Ore Rubble it
    /// `held`, of `ore` at `grade`; millibuckets of `water`, grinds of sand (`grit`), and the
    /// `turns` into the grind under way.
    #[derive(Default)]
    pub struct Mill {
        pub runner: u64,
        pub turned_at: u64,
        pub ground: ItemKey,
        pub ore: Ore,
        pub grade: u32,
        pub held: u32,
        pub ground_count: u32,
        pub water: u32,
        pub grit: u32,
        pub turns: u32,
    }
}
static MILL: Component<Mill> = Component::named("cogwork:mill");

/// What a turn did, or what the millstone lacks for one; shown above the hotbar as
/// `cogwork.millstone.<name>`. Never saved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum Status {
    Empty,
    NoRawOre,
    NoWater,
    NoSand,
    /// The raw ore it ground leaves no room for more.
    Full,
    /// It has no room for what the player holds.
    NoRoom,
    /// It has all it needs: the next turn counts.
    Ready,
    Turning,
    Ground,
}

/// What the ring around the runner shows, by the block's `grist`: the bed's stone, water,
/// sand, or wet sand.
const GRIST: [(&str, &str); 4] = [
    ("dry", "base:gravel"),
    ("water", "base:water_still#3F76E4"),
    ("sand", "base:sand"),
    ("wet", "base:sand#A89878"),
];

pub fn register(r: &mut Registry) {
    r.entity_component(
        &MILL,
        Mill::default(),
        component_flags::SAVE | component_flags::CARRIED,
    );
    r.system(&MILL, tick);
    r.block(&MILLSTONE)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("grist", GRIST.map(|(name, _)| name))
        .states(|props, state| {
            let (_, texture) = GRIST
                .iter()
                .find(|(name, _)| props.is("grist", name))
                .unwrap_or(&GRIST[0]);
            state.model_texture("grist", texture)
        })
        // The runner too, so aiming at it aims at the block.
        .hitbox([1.0, 0.0, 1.0], [15.0, 12.0, 15.0])
        .hardness(2.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .on_use(use_millstone)
        .entity(|entity| entity.attach(&MILL))
        .tab(&crate::TAB);
    runner::register(r);
}

/// Every millstone: its runner where it belongs, and its ring showing what it holds.
fn tick(batch: &mut SystemBatch<Mill>) {
    let world = batch.world();
    for (id, mill, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&MILLSTONE) {
            continue;
        }
        mill.runner = runner::place(&world, id, pos, mill.runner);
        let grist = grist(mill.water > 0, mill.grit > 0);
        if world.prop(pos, "grist") != Some(grist) {
            world.prop_set(pos, "grist", grist);
        }
    }
}

/// The `grist` a millstone shows with water or not, and sand or not.
fn grist(water: bool, sand: bool) -> &'static str {
    match (water, sand) {
        (false, false) => "dry",
        (true, false) => "water",
        (false, true) => "sand",
        (true, true) => "wet",
    }
}

/// A player uses the millstone: what they hold goes in, an empty hand turns it, and sneaking
/// with one takes out what it holds. False passes the click on to what they hold.
fn use_millstone(click: &mut Use) -> bool {
    let world = click.world();
    let pos = Pos::from(click.pos());
    let id = world.block_entity(pos);
    let Some(mut mill) = world.component(id, &MILL) else {
        return false;
    };
    let player = click.player();
    let held = world.inventory(player).get(world.selected_slot(player));
    let used = if !held.is_empty() {
        put_in(&world, player, held, &mut mill)
    } else if click.sneaking() {
        take_out(&world, player, &mut mill)
    } else {
        turn(&world, player, pos, &mut mill)
    };
    if let Some(state) = world.component_mut(id, &MILL) {
        *state = mill;
    }
    used
}

/// Puts what `player` holds, `held`, into the millstone, if it takes it.
fn put_in(world: &World, player: u64, held: ItemStack, mill: &mut Mill) -> bool {
    let slot = world.selected_slot(player);
    let used = if held.item == WATER_BUCKET.id() {
        mill.pour()
            .then(|| crate::liquid::hand_over(world, player, slot, held, BUCKET.id()))
    } else if SAND.on_item(held.item) {
        mill.add_sand()
            .then(|| use_up(world, player, slot, held, 1))
    } else if let Some((ore, grade)) = ore_of(&held) {
        let taken = mill.put_in(ore, grade, held.count);
        (taken > 0).then(|| use_up(world, player, slot, held, taken))
    } else {
        return false;
    };
    let status = match used {
        Some(()) => mill.lacks(raw_ore(mill)).unwrap_or(Status::Ready),
        None => Status::NoRoom,
    };
    hint(world, player, status, mill);
    true
}

/// The key of the raw ore the Ore Rubble in `mill` grinds into; none when no mod has one.
fn raw_ore(mill: &Mill) -> ItemKey {
    mill.ore.raw_ore().map_or(crate::ghost::NO_ITEM, key_of)
}

/// Takes `count` of `held` out of `slot` of `player`, who keeps them in creative.
fn use_up(world: &World, player: u64, slot: u32, held: ItemStack, count: u32) {
    if !world.creative(player) {
        world.inventory(player).set(slot, held.shrink(count));
    }
}

/// Hands `player` the raw ore the millstone ground, or else the Ore Rubble it holds.
fn take_out(world: &World, player: u64, mill: &mut Mill) -> bool {
    let stack = if mill.ground_count > 0 {
        let (key, count) = mill.take_ground();
        ItemStack::new(item_of(key), count)
    } else {
        let (ore, grade, count) = mill.take_held();
        ore_rubble(ore, grade, count)
    };
    if stack.is_empty() || stack.item == 0 {
        hint(world, player, Status::Empty, mill);
        return true;
    }
    let left = world.inventory(player).insert(stack);
    if !left.is_empty() {
        let at = world.entity(player).map(|entity| entity.pos);
        world.spawn_item(at.unwrap_or(weft_sdk::Vec3::ZERO), left);
    }
    true
}

/// Turns the runner once for `player`, unless it is still going round from the last turn.
fn turn(world: &World, player: u64, pos: Pos, mill: &mut Mill) -> bool {
    let now = world.game_time();
    if now < mill.turned_at + runner::TURN_TICKS {
        return true;
    }
    let status = mill.turn(raw_ore(mill), crate::chance::roll(world));
    if matches!(status, Status::Turning | Status::Ground) {
        mill.turned_at = now;
        runner::turn(world, mill.runner, pos);
    }
    hint(world, player, status, mill);
    true
}

/// Tells `player` above the hotbar what the millstone did or lacks.
fn hint(world: &World, player: u64, status: Status, mill: &Mill) {
    let name: &str = status.into();
    let key = format!("cogwork.millstone.{name}");
    let args = [Arg::Int(mill.turns as i64), Arg::Int(grind::TURNS as i64)];
    world.hint(&key, &args).to(player);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.millstone.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_mill_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Mill, runner), 0, "its runner");
        assert_eq!(offset_of!(Mill, turned_at), 8, "when it turned");
        assert_eq!(offset_of!(Mill, ground), 16, "the raw ore");
        assert_eq!(offset_of!(Mill, ore), 24, "the ore's metal");
        assert_eq!(offset_of!(Mill, grade), 60, "its grade");
        assert_eq!(offset_of!(Mill, held), 64, "how many");
        assert_eq!(offset_of!(Mill, ground_count), 68, "raw ore ground");
        assert_eq!(offset_of!(Mill, water), 72, "water");
        assert_eq!(offset_of!(Mill, grit), 76, "sand");
        assert_eq!(offset_of!(Mill, turns), 80, "turns");
    }

    #[test]
    fn the_ring_shows_water_and_sand() {
        assert_eq!(grist(false, false), "dry", "nothing");
        assert_eq!(grist(true, false), "water", "water");
        assert_eq!(grist(false, true), "sand", "sand");
        assert_eq!(grist(true, true), "wet", "both");
        let names = GRIST.map(|(name, _)| name);
        assert_eq!(names, ["dry", "water", "sand", "wet"], "a state for each");
    }

    #[test]
    fn the_millstone_has_its_model() {
        assert!(crate::has_asset("models/block/millstone.json"), "the bed");
        let model = include_str!("../../assets/models/block/millstone.json");
        assert!(model.contains("\"#grist\""), "a ring showing what it holds");
    }
}
