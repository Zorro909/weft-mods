//! The sieve: a wooden frame with a string mesh tray, worked by hand. Using it with Rubble puts
//! one on the mesh; four uses with an empty hand shake the tray (an owned entity with a clip),
//! and the last sifts the Rubble by `rubble::sifting`, the finds popping out on top.
//!
//! Its shape is the Blockbench model `assets/models/block/sieve.json`, and
//! `sieve_loaded.json` with a heap of Rubble on the mesh; the tray is
//! `assets/models/sieve_tray.geo.json`.

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, EntityKind, Item, Output, Registry, StackData, SystemBatch, Use, Vec3,
    World, component_flags, tool_tier, value,
};

use crate::lib::{Pos, TypedWorld};

pub static SIEVE: Block = Block::named("sieve");
static TRAY: EntityKind = EntityKind::named("sieve_tray");
static RUBBLE: Item = Item::named("rubble");

/// Shakes that sift one load.
const SHAKES: u32 = 4;
/// Ticks a shake takes; the next shake counts only after it.
const SHAKE_TICKS: u64 = 10;
/// The tray's model, its texture and the file of its clip, and the clip.
const TRAY_MODEL: &str = "sieve_tray";
const SHAKE: &str = "shake";
/// Blocks above the sieve's floor its finds pop out at: just over the tray.
const POP_HEIGHT: f64 = 1.0;

value! {
    /// A sieve's state: its `tray` (an entity) and the game time it was last `shaken_at`, what
    /// it holds on its mesh (`load`, a `Load`) and the `shakes` it has had.
    #[derive(Default)]
    pub struct Sift {
        pub tray: u64,
        pub shaken_at: u64,
        pub load: u32,
        pub shakes: u32,
    }
}
pub(crate) static SIFT: Component<Sift> = Component::named("cogwork:sift");

/// What lies on a sieve's mesh. Kept in `Sift::load` as its number, so the order is part of the
/// save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Load {
    #[default]
    Nothing,
    Rubble,
}

/// What a use of the sieve did; shown above the hotbar as `cogwork.sieve.<name>`. Never saved.
#[derive(Clone, Copy, PartialEq, Eq, Debug, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum Status {
    Empty,
    Loaded,
    /// Something lies on the mesh already.
    Busy,
    Shaking,
    Sifted,
}

impl Sift {
    pub fn load(&self) -> Load {
        Load::from_repr(self.load).unwrap_or_default()
    }

    /// Puts `load` on the mesh; false while something lies there.
    fn put_on(&mut self, load: Load) -> bool {
        let free = self.load() == Load::Nothing;
        if free {
            (self.load, self.shakes) = (load as u32, 0);
        }
        free
    }

    /// Shakes the tray once; the last shake sifts the load, and leaves the mesh empty.
    fn shake(&mut self) -> Status {
        if self.load() == Load::Nothing {
            return Status::Empty;
        }
        self.shakes += 1;
        if self.shakes < SHAKES {
            return Status::Shaking;
        }
        (self.load, self.shakes) = (Load::Nothing as u32, 0);
        Status::Sifted
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SIFT,
        Sift::default(),
        component_flags::SAVE | component_flags::CARRIED,
    );
    r.system(&SIFT, tick);
    r.block(&SIEVE)
        .grid_placement()
        .prop("loaded", ["false", "true"])
        .states(|props, state| {
            if props.is("loaded", "true") {
                state.model_file("sieve_loaded")
            } else {
                state
            }
        })
        // The tray too, so aiming at it aims at the block.
        .hitbox([0.0; 3], [16.0, 14.0, 16.0])
        .hardness(1.0)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .on_use(use_sieve)
        .entity(|entity| entity.attach(&SIFT))
        .tab(&crate::TAB);
    // Small enough to stay inside the frame, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&TRAY), claims)
        .texture(TRAY_MODEL)
        .animations(TRAY_MODEL)
        .size(0.25, 0.25);
    r.set_model(&TRAY, TRAY_MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &SIFT)
        .is_some_and(|sift| sift.tray == id)
}

/// Every sieve: its tray where it belongs, and a heap on its mesh while it holds a load.
fn tick(batch: &mut SystemBatch<Sift>) {
    let world = batch.world();
    for (id, sift, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&SIEVE) {
            continue;
        }
        sift.tray = crate::owned::place(&world, id, sift.tray, &TRAY, pos.floor_middle(), 0.0);
        let loaded = if sift.load() == Load::Nothing {
            "false"
        } else {
            "true"
        };
        if world.prop(pos, "loaded") != Some(loaded) {
            world.prop_set(pos, "loaded", loaded);
        }
    }
}

/// A player uses the sieve: Rubble goes on its mesh, an empty hand shakes it. False passes the
/// click on to what they hold.
fn use_sieve(click: &mut Use) -> bool {
    let world = click.world();
    let pos = Pos::from(click.pos());
    let id = world.block_entity(pos);
    let Some(mut sift) = world.component(id, &SIFT) else {
        return false;
    };
    let player = click.player();
    let slot = world.selected_slot(player);
    let held = world.inventory(player).get(slot);
    let status = if held.is_empty() {
        shake(&world, pos, &mut sift)
    } else if held.item == RUBBLE.id() {
        if !sift.put_on(Load::Rubble) {
            Status::Busy
        } else {
            if !world.creative(player) {
                world.inventory(player).set(slot, held.shrink(1));
            }
            Status::Loaded
        }
    } else {
        return false;
    };
    if let Some(state) = world.component_mut(id, &SIFT) {
        *state = sift;
    }
    let name: &str = status.into();
    let args = [Arg::Int(sift.shakes as i64), Arg::Int(SHAKES as i64)];
    world
        .hint(&format!("cogwork.sieve.{name}"), &args)
        .to(player);
    true
}

/// Shakes the sieve at `pos` once, unless its tray is still shaking from the last time; the
/// last shake pops out what its load sifts into.
fn shake(world: &World, pos: Pos, sift: &mut Sift) -> Status {
    let now = world.game_time();
    if now < sift.shaken_at + SHAKE_TICKS {
        return Status::Shaking;
    }
    let status = sift.shake();
    if status == Status::Empty {
        return status;
    }
    sift.shaken_at = now;
    world.action_play(sift.tray, SHAKE, false);
    world.play_sound(&crate::sounds::SHAKE, pos.center(), 1.0);
    if status == Status::Sifted {
        let top = pos.floor_middle() + Vec3::new(0.0, POP_HEIGHT, 0.0);
        for stack in crate::rubble::sifting::sift_rubble(world) {
            world.spawn_item(top, stack);
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_shakes_sift_a_load() {
        let mut sift = Sift::default();
        assert_eq!(sift.shake(), Status::Empty, "nothing to shake");
        assert!(sift.put_on(Load::Rubble), "rubble on");
        assert!(!sift.put_on(Load::Rubble), "one at a time");
        let shaken: Vec<Status> = (0..SHAKES).map(|_| sift.shake()).collect();
        let sifted = [
            Status::Shaking,
            Status::Shaking,
            Status::Shaking,
            Status::Sifted,
        ];
        assert_eq!(shaken, sifted, "the fourth sifts it");
        assert_eq!(sift.load(), Load::Nothing, "the mesh empty again");
    }

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.sieve.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_sieve_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Sift, tray), 0, "its tray");
        assert_eq!(offset_of!(Sift, shaken_at), 8, "when it shook");
        assert_eq!(offset_of!(Sift, load), 16, "its load");
        assert_eq!(offset_of!(Sift, shakes), 20, "its shakes");
    }

    #[test]
    fn an_unknown_saved_load_reads_as_nothing() {
        let sift = Sift {
            load: 99,
            ..Sift::default()
        };
        assert_eq!(sift.load(), Load::Nothing, "from a newer cogwork");
    }

    #[test]
    fn the_sieve_has_its_models_clip_and_texture() {
        for file in [
            "models/block/sieve.json".to_string(),
            "models/block/sieve_loaded.json".to_string(),
            format!("models/{TRAY_MODEL}.geo.json"),
            format!("animations/{TRAY_MODEL}.animation.json"),
            format!("textures/entity/{TRAY_MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../assets/animations/sieve_tray.animation.json");
        let length = format!("\"animation_length\": {}", SHAKE_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a shake long");
    }
}
