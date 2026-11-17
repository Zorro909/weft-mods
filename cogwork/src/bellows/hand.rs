//! The hand bellows, worked by hand: each use with an empty hand squeezes its bag (an owned
//! entity playing the clip `squeeze`), and the furnace its nozzle points at cooks twice as fast
//! for a minute from then. Pumping again starts the minute over; it doesn't add up.

use strum::{IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Component, EntityKind, Output, Registry, SystemBatch, Use, World, component_flags, value,
};

use super::HAND_BELLOWS;
use crate::lib::{Pos, TypedWorld};

static BAG: EntityKind = EntityKind::named("hand_bellows_bag");

/// Ticks a furnace is blown after one pump: a minute.
const BLOWS_FOR: u64 = 60 * 20;
/// Ticks a squeeze takes; the next pump counts only after it.
const SQUEEZE_TICKS: u64 = 12;
/// The bag's model, its texture and the file of its clip, and the clip.
const MODEL: &str = "hand_bellows_bag";
const SQUEEZE: &str = "squeeze";
/// Puffs out of the furnace on a pump, and the ticks between single ones while it blows.
const PUMP_PUFFS: u32 = 4;
const PUFF_EVERY: u64 = 30;

value! {
    /// A hand bellows' state: its `bag` (an entity), the game time it was last `pumped_at`, and
    /// the game time it blows `until`.
    #[derive(Default)]
    pub struct Pumped {
        pub bag: u64,
        pub pumped_at: u64,
        pub until: u64,
    }
}
static PUMPED: Component<Pumped> = Component::named("cogwork:pumped");

/// What a pump did; shown above the hotbar as `cogwork.hand_bellows.<name>`. Never saved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
enum Status {
    Blowing,
    /// Its nozzle points at nothing that smelts.
    NoFurnace,
}

impl Pumped {
    /// Pumps it at game time `now`; false while the last squeeze is still going.
    fn pump(&mut self, now: u64) -> bool {
        if now < self.pumped_at + SQUEEZE_TICKS {
            return false;
        }
        self.pumped_at = now;
        self.until = now + BLOWS_FOR;
        true
    }

    /// Whether it blows at game time `now`.
    fn blows_at(&self, now: u64) -> bool {
        now < self.until
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(&PUMPED, Pumped::default(), component_flags::SAVE);
    r.system(&PUMPED, tick);
    super::block(r.block(&HAND_BELLOWS), "hand_bellows")
        // The bag and handle too, so aiming at them aims at the block.
        .hitbox([0.0; 3], [16.0, 15.0, 16.0])
        .on_use(pump)
        .entity(|entity| entity.attach(&PUMPED));
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&BAG), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&BAG, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &PUMPED)
        .is_some_and(|pumped| pumped.bag == id)
}

/// Whether the hand bellows `id` blows now.
pub fn blowing(world: &World, id: u64) -> bool {
    world
        .component(id, &PUMPED)
        .is_some_and(|pumped| pumped.blows_at(world.game_time()))
}

/// Every hand bellows: its bag where it belongs, and now and then a puff out of the furnace it
/// blows into.
fn tick(batch: &mut SystemBatch<Pumped>) {
    let world = batch.world();
    let now = world.game_time();
    for (id, pumped, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&HAND_BELLOWS) {
            continue;
        }
        let yaw = crate::owned::yaw(crate::machine::front(&world, pos));
        pumped.bag = crate::owned::place(&world, id, pumped.bag, &BAG, pos.floor_middle(), yaw);
        if pumped.blows_at(now) && now.is_multiple_of(PUFF_EVERY) {
            super::puff(&world, super::target(&world, pos), 1);
        }
    }
}

/// A player uses the hand bellows: an empty hand pumps it. False passes the click on to what
/// they hold.
fn pump(click: &mut Use) -> bool {
    let world = click.world();
    let player = click.player();
    let held = world.inventory(player).get(world.selected_slot(player));
    if !held.is_empty() {
        return false;
    }
    let pos = Pos::from(click.pos());
    let id = world.block_entity(pos);
    let Some(mut pumped) = world.component(id, &PUMPED) else {
        return false;
    };
    if !pumped.pump(world.game_time()) {
        return true;
    }
    world.set_component(id, &PUMPED, pumped);
    world.action_play(pumped.bag, SQUEEZE, false);
    world.play_sound(&crate::sounds::HAND_BELLOWS, pos.center(), 1.0);
    let target = super::target(&world, pos);
    super::puff(&world, target, PUMP_PUFFS);
    let status = if super::smelts(&world, target) {
        Status::Blowing
    } else {
        Status::NoFurnace
    };
    let name: &str = status.into();
    let seconds = Arg::Int((BLOWS_FOR / 20) as i64);
    world
        .hint(&format!("cogwork.hand_bellows.{name}"), &[seconds])
        .to(player);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pump_blows_for_a_minute() {
        let mut pumped = Pumped::default();
        assert!(!pumped.blows_at(100), "not yet pumped");
        assert!(pumped.pump(100), "pumped");
        assert!(pumped.blows_at(100 + 1_199), "the minute's last tick");
        assert!(!pumped.blows_at(100 + 1_200), "a minute on");
    }

    #[test]
    fn a_pump_during_a_squeeze_is_lost_and_a_later_one_starts_the_minute_over() {
        let mut pumped = Pumped::default();
        assert!(pumped.pump(100), "first");
        assert!(!pumped.pump(105), "still squeezing");
        assert_eq!(pumped.until, 1_300, "untouched");
        assert!(pumped.pump(700), "half a minute on");
        assert_eq!(pumped.until, 1_900, "a minute from then, not added on");
    }

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.hand_bellows.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_hand_bellows_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Pumped, bag), 0, "its bag");
        assert_eq!(offset_of!(Pumped, pumped_at), 8, "when it was pumped");
        assert_eq!(offset_of!(Pumped, until), 16, "how long it blows");
    }

    #[test]
    fn the_bag_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/hand_bellows_bag.animation.json");
        assert!(
            clips.contains("\"animation.hand_bellows_bag.squeeze\""),
            "the clip"
        );
        let length = format!("\"animation_length\": {}", SQUEEZE_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a squeeze long");
    }
}
