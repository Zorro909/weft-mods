//! The Bit Magazine: placed against a quarry of any kind, it keeps the quarry's bit slot filled
//! from its own bits, which come in by tube or by hand. When the fitted bit is down to its last
//! use, or the slot is empty, it puts in the first of its bits that the quarry takes (`swap`), and
//! the worn bit goes into its output, where a timber tube or the player takes it out. A quarry it
//! points into never uses a bit's last use (`holds_back`), so a bit waiting for room in the
//! output comes back worn rather than not at all.
//!
//! Its shape is the Blockbench model `assets/models/block/bit_magazine.json`, a steel housing
//! with a feed toward the quarry; the drum of bits inside is an owned entity that turns a quarter
//! at each swap (`work`).

mod swap;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, Dir, EntityKind, Facings, Holder, Holds, ItemStack, Menu, Registry, S,
    Slots, StackData, Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::excavator::bit;
use crate::lib::{Pos, TypedWorld};

pub static BIT_MAGAZINE: Block = Block::named("bit_magazine");
static BIT_MAGAZINE_MENU: Menu = Menu::named("bit_magazine");
static DRUM: EntityKind = EntityKind::named("bit_magazine_drum");
/// The drum's model, its texture and the file of its clip.
const DRUM_MODEL: &str = "bit_magazine_drum";

/// Its slots: fresh bits, then the worn ones it took out.
const BITS: u32 = 4;
const OUTPUT: u32 = 4;
/// The sides a magazine can stand on against a quarry: its feed faces sideways.
const SIDES: [Dir; 4] = [Dir::West, Dir::East, Dir::North, Dir::South];

value! {
    /// A Bit Magazine's state: its `drum` (an entity) and its `status`.
    #[derive(Default)]
    pub struct Magazine {
        pub drum: u64,
        pub status: u32,
    }
}
static MAGAZINE: Component<Magazine> = Component::named("cogwork:magazine");

/// What the magazine is doing or lacks; its screen says `cogwork.bit_magazine.<name>`. Kept in
/// `Magazine::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// No quarry where it points.
    #[default]
    NoQuarry,
    /// The quarry has a bit to dig with, or will have at once.
    Ready,
    /// None of its bits fits the quarry with uses to spare.
    NoBit,
    /// The worn bit has no room in its output.
    Full,
}

impl Magazine {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &MAGAZINE,
        Magazine::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&MAGAZINE, work::tick);
    // Its front, the feed, faces the way the player looked: into the quarry it was put against.
    r.block(&BIT_MAGAZINE)
        .facing(Facings::HORIZONTAL, Toward::Away)
        .model_file("bit_magazine")
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| entity.attach(&MAGAZINE).inventory(BITS + OUTPUT))
        .opens(&BIT_MAGAZINE_MENU)
        .tab(&crate::TAB);
    r.menu(&BIT_MAGAZINE_MENU)
        .group(
            "bits",
            Slots::row(BITS).accepts("#cogwork:drill_bits").holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    crate::owned::kind(r.entity(&DRUM), work::claims)
        .texture(DRUM_MODEL)
        .animations(DRUM_MODEL)
        .size(0.25, 0.25);
    r.set_model(&DRUM, DRUM_MODEL);
}

/// Puts `stack` from a tube into the Bit Magazine `at`: a drill bit with uses to spare into its
/// bits, nothing else, so a worn bit it gave out never comes back round. None for what is no
/// magazine.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &MAGAZINE)?;
    let spare = bit::of(stack).is_some_and(|bit| !swap::held(uses_left(bit, stack)));
    if !spare {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("bits")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// Whether the quarry at `pos` must leave `fitted`, the bit in its slot, unused: a bit at its last
/// use with a Bit Magazine pointing into the quarry, which takes it out once it can.
pub fn holds_back(world: &World, pos: Pos, fitted: ItemStack) -> bool {
    let Some(bit) = bit::of(fitted) else {
        return false;
    };
    swap::held(uses_left(bit, fitted)) && fed(world, pos)
}

/// Whether a Bit Magazine beside `pos` points into it.
fn fed(world: &World, pos: Pos) -> bool {
    SIDES.into_iter().any(|dir| {
        let beside = world.block_typed(pos.toward(dir));
        beside.is(&BIT_MAGAZINE) && crate::machine::front_of(beside) == dir.opposite()
    })
}

/// Uses `stack`, a `bit`, has left.
fn uses_left(bit: &bit::Bit, stack: ItemStack) -> u32 {
    bit.lasts.saturating_sub(stack.wear())
}

/// The Bit Magazine's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Bit Magazine                                                    |
/// | Keeping the quarry's bit slot filled                            |  status
/// | Bits                     Worn bits                              |
/// | [ ][ ][ ][ ]             [ ][ ][ ][ ]                           |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|view, out| {
            let magazine = view.menu_component(&MAGAZINE).unwrap_or_default();
            let status: &str = magazine.status().into();
            let args: [Arg; 0] = [];
            out.set(&format!("cogwork.bit_magazine.{status}"), &args);
        });
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.bit_magazine.bits");
                ui.group("bits");
            });
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.bit_magazine.worn");
                ui.group("output");
            });
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.bit_magazine.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        for label in ["bits", "worn"] {
            let key = format!("cogwork.bit_magazine.{label}");
            assert!(crate::translated(&key), "{key}");
        }
    }

    #[test]
    fn what_a_magazine_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Magazine, drum), 0, "its drum");
        assert_eq!(offset_of!(Magazine, status), 8, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let magazine = Magazine {
            status: 99,
            ..Magazine::default()
        };
        assert_eq!(magazine.status(), Status::NoQuarry, "from a newer cogwork");
    }

    #[test]
    fn the_magazine_has_its_models_clip_and_texture() {
        for file in [
            "models/block/bit_magazine.json".to_string(),
            "models/item/bit_magazine.json".to_string(),
            format!("models/{DRUM_MODEL}.geo.json"),
            format!("animations/{DRUM_MODEL}.animation.json"),
            format!("textures/entity/{DRUM_MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/bit_magazine_drum.animation.json");
        assert!(
            clips.contains("\"animation.bit_magazine_drum.swap\""),
            "the clip"
        );
    }
}
