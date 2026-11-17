//! The driven bellows: the hand bellows' driven counterpart, a separate machine. Fed 20 energy a
//! tick by an engine, a shaft or an axle, a cog on its beam turns and pumps its bag (an owned
//! entity looping the clip `pump`); after a second of feeding without a break it blows into the
//! block its nozzle points at, and a furnace there cooks twice as fast. A feed short of 20 a
//! tick never gets that far, so half the energy can't buy most of the speed.
//!
//! It runs whether or not something is there to blow into: `bellows::driven_into` is what a
//! furnace that needs a draft asks.
//!
//! Time its area missed (unloaded, or the world's time jumped) is paid for at 20 a tick like any
//! other; short of that it stops and winds up again. A furnace catching up on that time before
//! the bellows has paid for it gets no draft: the bellows blows only on a tick it was fed.

use strum::{IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Component, ENERGY, EntityKind, Menu, Output, Registry, S, SystemBatch, Ui, World,
    component_flags, value,
};

use super::DRIVEN_BELLOWS;
use crate::lib::{Pos, TypedWorld};
use crate::machine;

static BAG: EntityKind = EntityKind::named("driven_bellows_bag");
static DRIVEN_MENU: Menu = Menu::named("driven_bellows");

/// The energy it takes a tick while it runs.
const DRAW: i64 = 20;
/// Ticks in a row it must be fed before it blows: a second to get the fire going.
const WIND_UP: u32 = 20;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 100;
const STORES: i64 = 400;
/// The bag's model, its texture and the file of its clip, and the clip.
const MODEL: &str = "driven_bellows_bag";
const PUMP: &str = "pump";
/// Ticks one stroke of the clip takes, and how far into it the bag is squeezed flat: the
/// whoosh and the puffs.
const STROKE_TICKS: u64 = 20;
const SQUEEZED_AT: u64 = 10;
/// Puffs out of the furnace a stroke.
const STROKE_PUFFS: u32 = 2;

value! {
    /// A driven bellows' state: its `bag` (an entity), the game time the bag's clip started
    /// (`stroke_from`), the ticks in a row it has been `fed`, counted up to `WIND_UP`, and the
    /// game time it was last fed (`fed_at`).
    #[derive(Default)]
    pub struct Draft {
        pub bag: u64,
        pub stroke_from: u64,
        pub fed: u32,
        pub fed_at: u64,
    }
}
static DRAFT: Component<Draft> = Component::named("cogwork:draft");

/// What the driven bellows is doing, from the ticks it has been fed; its screen says
/// `cogwork.driven_bellows.<name>`. Never saved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
enum Status {
    NoPower,
    /// Fed, and getting the fire going.
    Starting,
    Blowing,
}

impl Draft {
    /// Counts `ticks` ticks at game time `now`, of which it was fed for `paid`: on when it was
    /// fed for all of them, started over otherwise.
    fn count_ticks(&mut self, ticks: u32, paid: u32, now: u64) {
        if paid < ticks {
            self.fed = 0;
            return;
        }
        self.fed = self.fed.saturating_add(ticks).min(WIND_UP);
        self.fed_at = now;
    }

    /// Whether it blows at game time `now`: wound up, and fed this tick or the one before, as
    /// the furnace may tick before or after it.
    fn blows_at(&self, now: u64) -> bool {
        self.status() == Status::Blowing && now.saturating_sub(self.fed_at) <= 1
    }

    fn status(&self) -> Status {
        match self.fed {
            0 => Status::NoPower,
            fed if fed < WIND_UP => Status::Starting,
            _ => Status::Blowing,
        }
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &DRAFT,
        Draft::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&DRAFT, tick);
    super::block(r.block(&DRIVEN_BELLOWS), "driven_bellows")
        .entity(|entity| {
            entity
                .attach(&DRAFT)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, machine::energy(STORES, MAX_IN))
                .attach(&machine::METER)
        })
        .opens(&DRIVEN_MENU);
    r.menu(&DRIVEN_MENU).screen(screen);
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&BAG), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&BAG, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &DRAFT)
        .is_some_and(|draft| draft.bag == id)
}

/// Whether the driven bellows `id` blows now.
pub fn blowing(world: &World, id: u64) -> bool {
    world
        .component(id, &DRAFT)
        .is_some_and(|draft| draft.blows_at(world.game_time()))
}

/// Every driven bellows: fed this tick or not, and its bag pumping while it is.
fn tick(batch: &mut SystemBatch<Draft>) {
    let world = batch.world();
    for (id, draft, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&DRIVEN_BELLOWS) {
            continue;
        }
        machine::start_meter(&world, id);
        let paid = feed(&world, id, ticks);
        draft.count_ticks(ticks, paid, world.game_time());
        show(&world, id, pos, draft);
    }
}

/// Takes the energy of `ticks` ticks from the store of the driven bellows `id`, a whole tick's
/// or none of it each; the ticks it was paid for.
fn feed(world: &World, id: u64, ticks: u32) -> u32 {
    let paid = world.component_mut(id, &ENERGY).map_or(0, |energy| {
        let paid = ticks_paid(energy.stored, ticks);
        energy.stored -= DRAW * i64::from(paid);
        paid
    });
    machine::note_drawn(world, id, DRAW * i64::from(paid), DRAW * i64::from(ticks));
    paid
}

/// The ticks, of `ticks`, that `stored` energy pays for.
fn ticks_paid(stored: i64, ticks: u32) -> u32 {
    u32::try_from(stored.max(0) / DRAW).map_or(ticks, |affordable| affordable.min(ticks))
}

/// Keeps the bag of the driven bellows `id` at `pos` in place and pumping while it is fed, with
/// a whoosh each stroke and a puff out of the furnace it blows into.
fn show(world: &World, id: u64, pos: Pos, draft: &mut Draft) {
    let front = machine::front(world, pos);
    let yaw = crate::owned::yaw(front);
    draft.bag = crate::owned::place(world, id, draft.bag, &BAG, pos.floor_middle(), yaw);
    let pumping = draft.status() != Status::NoPower;
    let now = world.game_time();
    if crate::owned::keep_looping(world, id, draft.bag, PUMP, pumping) {
        draft.stroke_from = now;
    }
    let into_stroke = now.saturating_sub(draft.stroke_from) % STROKE_TICKS;
    if !pumping || into_stroke != SQUEEZED_AT {
        return;
    }
    world.play_sound(&crate::sounds::DRIVEN_BELLOWS, pos.center(), 1.0);
    if draft.status() == Status::Blowing {
        super::puff(world, pos.toward(front), STROKE_PUFFS);
    }
}

/// The driven bellows' screen:
///
/// ```text
/// +--------------------------------------------+
/// | Driven Bellows                             |
/// | [#####.....]                               |  stored energy
/// | 200 / 400 energy                           |
/// | Using 20 energy a tick                     |  drawn last tick
/// | Blowing: the furnace cooks twice as fast   |  status
/// +--------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        machine::power_bar(ui);
        machine::meter_line(ui);
        ui.text_bind(|view, out| {
            let draft = view.menu_component(&DRAFT).unwrap_or_default();
            let status: &str = draft.status().into();
            let args = [Arg::Int(DRAW)];
            out.set(&format!("cogwork.driven_bellows.{status}"), &args);
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The status after `ticks` ticks, fed on those `fed` says.
    fn after(ticks: u32, fed: impl Fn(u32) -> bool) -> Status {
        let mut draft = Draft::default();
        for tick in 0..ticks {
            draft.count_ticks(1, u32::from(fed(tick)), u64::from(tick));
        }
        draft.status()
    }

    /// A draft blowing at game time `now`, fed every tick so far.
    fn blowing_at(now: u64) -> Draft {
        let mut draft = Draft::default();
        draft.count_ticks(WIND_UP, WIND_UP, now);
        draft
    }

    #[test]
    fn a_steady_feed_blows_after_a_second() {
        assert_eq!(after(0, |_| true), Status::NoPower, "not fed yet");
        assert_eq!(after(1, |_| true), Status::Starting, "fed once");
        assert_eq!(after(WIND_UP - 1, |_| true), Status::Starting, "nearly");
        assert_eq!(after(WIND_UP, |_| true), Status::Blowing, "a second on");
        assert_eq!(after(500, |_| true), Status::Blowing, "and on");
    }

    #[test]
    fn half_the_energy_never_blows() {
        let every_other = |tick: u32| tick.is_multiple_of(2);
        for ticks in [WIND_UP, 2 * WIND_UP, 501] {
            assert_ne!(after(ticks, every_other), Status::Blowing, "{ticks} ticks");
        }
    }

    #[test]
    fn a_tick_without_energy_starts_it_over() {
        let mut draft = blowing_at(100);
        draft.count_ticks(1, 0, 101);
        assert_eq!(draft.status(), Status::NoPower, "stopped");
        draft.count_ticks(1, 1, 102);
        assert_eq!(draft.status(), Status::Starting, "from the start");
    }

    #[test]
    fn missed_time_it_cannot_pay_for_stops_it() {
        // 6,000 ticks missed with a full store of 400: twenty ticks' worth.
        let paid = ticks_paid(STORES, 6_000);
        assert_eq!(paid, 20, "a tick of 20 each");
        let mut draft = blowing_at(100);
        draft.count_ticks(6_000, paid, 6_100);
        assert!(!draft.blows_at(6_100), "no draft for the missed time");
        assert_eq!(draft.status(), Status::NoPower, "it winds up again");
    }

    #[test]
    fn missed_time_paid_for_keeps_it_blowing() {
        let mut draft = blowing_at(100);
        draft.count_ticks(15, ticks_paid(STORES, 15), 115);
        assert!(draft.blows_at(115), "paid for all 15");
    }

    #[test]
    fn a_furnace_catching_up_first_gets_no_draft() {
        let draft = blowing_at(100);
        assert!(draft.blows_at(100), "the tick it was fed");
        assert!(draft.blows_at(101), "the next, ticking before it");
        assert!(
            !draft.blows_at(6_100),
            "after a jump it hasn't paid for yet"
        );
    }

    #[test]
    fn an_empty_store_pays_for_nothing() {
        assert_eq!(ticks_paid(0, 1), 0);
        assert_eq!(ticks_paid(DRAW - 1, 1), 0);
        assert_eq!(ticks_paid(DRAW, 1), 1);
    }

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.driven_bellows.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_driven_bellows_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Draft, bag), 0, "its bag");
        assert_eq!(offset_of!(Draft, stroke_from), 8, "when its clip started");
        assert_eq!(offset_of!(Draft, fed), 16, "the ticks it was fed");
        assert_eq!(offset_of!(Draft, fed_at), 24, "when it was last fed");
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
        let clips = include_str!("../../assets/animations/driven_bellows_bag.animation.json");
        assert!(
            clips.contains("\"animation.driven_bellows_bag.pump\""),
            "the clip"
        );
        let length = format!("\"animation_length\": {}", STROKE_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a stroke long");
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
