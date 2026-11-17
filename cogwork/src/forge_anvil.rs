//! The forge anvil and its hammer: steel blooms from the blast furnace are beaten into steel
//! ingots here by hand. Using the anvil with a bloom lays it on the face; five strikes with a
//! forge hammer, which wears a little each, draw it out, and the ingot pops out on top. An empty
//! hand takes a bloom back off. The bloom glows on the face, and draws out flatter after the
//! third strike; each strike rings and throws sparks.
//!
//! Its shapes are the Blockbench models `assets/models/block/forge_anvil.json`, and
//! `forge_anvil_bloom.json` and `forge_anvil_drawn.json` with the bloom on it. What it holds is
//! kept on its item when it is broken (`CARRIED`, and `harvest` for machines).

use strum::{IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Facings, Item, ItemStack, Output, Registry, StackData, SystemBatch,
    Toward, Use, Vec3, World, component_flags, tool_tier, value,
};

use crate::lib::{Pos, TypedWorld};
use crate::steel::{SPARK, STEEL_BLOOM, STEEL_INGOT};

pub static FORGE_ANVIL: Block = Block::named("forge_anvil");
pub static FORGE_HAMMER: Item = Item::named("forge_hammer");

/// Strikes that forge a bloom into an ingot.
pub const STRIKES: u32 = 5;
/// Strikes after which the bloom shows drawn out.
const DRAWN_AT: u32 = 3;
/// Ticks a strike takes; one sooner rings nothing.
const STRIKE_TICKS: u64 = 8;
/// Strikes a hammer lasts: fifty ingots.
const HAMMER_LASTS: u32 = 250;
/// Blocks above the anvil's floor its face is, where sparks fly and the ingot pops out.
const FACE_HEIGHT: f64 = 0.95;
const SPARKS: u32 = 14;
/// Light a hot bloom gives.
const BLOOM_GLOW: u8 = 7;

value! {
    /// A forge anvil's state: the game time it was last `struck_at`, whether a `bloom` lies on it
    /// (1) or not, and the `strikes` it has had.
    #[derive(Default)]
    pub struct Forging {
        pub struck_at: u64,
        pub bloom: u32,
        pub strikes: u32,
    }
}
pub(crate) static FORGING: Component<Forging> = Component::named("cogwork:forging");

/// What a use of the anvil did; shown above the hotbar as `cogwork.forge_anvil.<name>`. Never
/// saved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum Status {
    /// Nothing lies on it to strike or take.
    Empty,
    Laid,
    /// A bloom lies there already.
    Busy,
    Struck,
    Forged,
    Taken,
}

impl Forging {
    fn has_bloom(&self) -> bool {
        self.bloom != 0
    }

    /// Lays a bloom on the face; false while one lies there.
    fn lay(&mut self) -> bool {
        let free = !self.has_bloom();
        if free {
            (self.bloom, self.strikes) = (1, 0);
        }
        free
    }

    /// Strikes the bloom once; the last strike forges it into an ingot and clears the face.
    fn strike(&mut self) -> Status {
        if !self.has_bloom() {
            return Status::Empty;
        }
        self.strikes += 1;
        if self.strikes < STRIKES {
            return Status::Struck;
        }
        (self.bloom, self.strikes) = (0, 0);
        Status::Forged
    }

    /// Takes the bloom off, its strikes lost; whether there was one.
    fn take(&mut self) -> bool {
        let had = self.has_bloom();
        (self.bloom, self.strikes) = (0, 0);
        had
    }

    /// What the face shows: the `bloom` property.
    fn look(&self) -> &'static str {
        match (self.has_bloom(), self.strikes >= DRAWN_AT) {
            (false, _) => "none",
            (true, false) => "hot",
            (true, true) => "drawn",
        }
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &FORGING,
        Forging::default(),
        component_flags::SAVE | component_flags::CARRIED,
    );
    r.system(&FORGING, tick);
    r.block(&FORGE_ANVIL)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("forge_anvil")
        .prop("bloom", ["none", "hot", "drawn"])
        .states(|props, state| match props.get("bloom") {
            "hot" => state.model_file("forge_anvil_bloom").light(BLOOM_GLOW),
            "drawn" => state.model_file("forge_anvil_drawn").light(BLOOM_GLOW),
            _ => state,
        })
        .hitbox([0.0; 3], [16.0, 16.0, 16.0])
        .hardness(5.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .on_use(use_anvil)
        .entity(|entity| entity.attach(&FORGING))
        .tab(&crate::TAB);
    r.item(&FORGE_HAMMER)
        .durability(HAMMER_LASTS)
        .handheld()
        .tab(&crate::TAB);
}

/// Every anvil shows whether a bloom lies on it, and how far drawn.
fn tick(batch: &mut SystemBatch<Forging>) {
    let world = batch.world();
    for (id, forging, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&FORGE_ANVIL) {
            continue;
        }
        let look = forging.look();
        if world.prop(pos, "bloom").is_some_and(|now| now != look) {
            world.prop_set(pos, "bloom", look);
        }
    }
}

/// A player uses the anvil: a bloom goes on it, a hammer strikes it, an empty hand takes the
/// bloom back. False passes the click on to what they hold.
fn use_anvil(click: &mut Use) -> bool {
    let world = click.world();
    let pos = Pos::from(click.pos());
    let id = world.block_entity(pos);
    let Some(mut forging) = world.component(id, &FORGING) else {
        return false;
    };
    let player = click.player();
    let slot = world.selected_slot(player);
    let held = world.inventory(player).get(slot);
    let status = if held.is_empty() {
        take_back(&world, player, &mut forging)
    } else if held.item == STEEL_BLOOM.id() {
        lay(&world, player, slot, held, &mut forging)
    } else if held.item == FORGE_HAMMER.id() {
        let Some(status) = strike(&world, pos, player, slot, held, &mut forging) else {
            return true;
        };
        status
    } else {
        return false;
    };
    if let Some(state) = world.component_mut(id, &FORGING) {
        *state = forging;
    }
    let name: &str = status.into();
    let args = [Arg::Int(forging.strikes as i64), Arg::Int(STRIKES as i64)];
    world
        .hint(&format!("cogwork.forge_anvil.{name}"), &args)
        .to(player);
    true
}

/// Lays the bloom `player` holds in `slot` on the anvil.
fn lay(world: &World, player: u64, slot: u32, held: ItemStack, forging: &mut Forging) -> Status {
    if !forging.lay() {
        return Status::Busy;
    }
    if !world.creative(player) {
        world.inventory(player).set(slot, held.shrink(1));
    }
    Status::Laid
}

/// Hands `player` the bloom on the anvil back.
fn take_back(world: &World, player: u64, forging: &mut Forging) -> Status {
    if !forging.take() {
        return Status::Empty;
    }
    give(world, player, STEEL_BLOOM.stack(1));
    Status::Taken
}

/// Strikes the anvil at `pos` with the hammer `player` holds in `slot`, wearing it; none while
/// the last strike still rings.
fn strike(
    world: &World,
    pos: Pos,
    player: u64,
    slot: u32,
    held: ItemStack,
    forging: &mut Forging,
) -> Option<Status> {
    let now = world.game_time();
    if now < forging.struck_at + STRIKE_TICKS {
        return None;
    }
    let status = forging.strike();
    if status == Status::Empty {
        return Some(status);
    }
    forging.struck_at = now;
    if !world.creative(player) {
        world.inventory(player).set(slot, held.add_wear(1));
    }
    let face = pos.floor_middle() + Vec3::new(0.0, FACE_HEIGHT, 0.0);
    world.play_sound(&crate::sounds::ANVIL_CLANG, face, 1.0);
    let burst = world.spawn_particles(&SPARK, face).count(SPARKS);
    drop(
        burst
            .spread(0.15)
            .velocity(Vec3::new(0.0, 0.2, 0.0))
            .speed(0.25),
    );
    if status == Status::Forged {
        world.spawn_item(face, STEEL_INGOT.stack(1));
    }
    Some(status)
}

/// Puts `stack` into `player`'s inventory, dropping at their feet what doesn't fit.
fn give(world: &World, player: u64, stack: ItemStack) {
    let left = world.inventory(player).insert(stack);
    if !left.is_empty() {
        let at = world.entity(player).map_or(Vec3::ZERO, |entity| entity.pos);
        world.spawn_item(at, left);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_strikes_forge_a_bloom() {
        let mut forging = Forging::default();
        assert_eq!(forging.strike(), Status::Empty, "nothing to strike");
        assert!(forging.lay(), "a bloom on");
        assert!(!forging.lay(), "one at a time");
        for strike in 1..STRIKES {
            assert_eq!(forging.strike(), Status::Struck, "strike {strike}");
        }
        assert_eq!(forging.strike(), Status::Forged, "the fifth");
        assert_eq!(forging.look(), "none", "the face clear again");
    }

    #[test]
    fn the_bloom_draws_out_after_the_third_strike() {
        let mut forging = Forging::default();
        forging.lay();
        assert_eq!(forging.look(), "hot", "a lump");
        for _ in 0..DRAWN_AT {
            forging.strike();
        }
        assert_eq!(forging.look(), "drawn", "drawn out");
    }

    #[test]
    fn a_bloom_taken_back_loses_its_strikes() {
        let mut forging = Forging::default();
        assert!(!forging.take(), "nothing to take");
        forging.lay();
        forging.strike();
        assert!(forging.take(), "the bloom back");
        forging.lay();
        assert_eq!(forging.strikes, 0, "started over");
    }

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.forge_anvil.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_an_anvil_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Forging, struck_at), 0, "when last struck");
        assert_eq!(offset_of!(Forging, bloom), 8, "its bloom");
        assert_eq!(offset_of!(Forging, strikes), 12, "its strikes");
    }

    #[test]
    fn the_anvil_has_its_models_and_the_hammer_its_picture() {
        for file in [
            "models/block/forge_anvil.json",
            "models/block/forge_anvil_bloom.json",
            "models/block/forge_anvil_drawn.json",
            "models/item/forge_anvil.json",
            "textures/item/forge_hammer.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
