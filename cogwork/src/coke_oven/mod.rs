//! The coke oven: a 3 by 3 by 3 box of coke oven bricks around one hollow block, its door (the
//! controller) in the middle of its front (`brickwork`). It needs no energy: coal put in by tube
//! or by hand cooks, eight at a time (`batch`), into coke, a fuel the blast furnace burns, and
//! sulfur; each coal gives creosote, a new liquid. The coke and sulfur leave by the door's front
//! into a tube there; the creosote is drained by a liquid tube on the door or on a drain, which
//! takes a brick's place anywhere in the shell.
//!
//! Its blocks are plain cubes: the door glows while it cooks, and smoke rises from the top.

mod batch;
mod work;

pub use batch::COKE_TICKS;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, ClientView, Component, Facings, Holder, Holds, Item, ItemStack, Menu, Particle,
    Registry, S, Slots, Tag, Toward, Ui, Use, WithComponents, World, component_flags, tool_tier,
    value,
};

use crate::brickwork::{Fault, Problem, Shape};
use crate::lib::Pos;
use crate::liquid::{HELD, Holders, Vessel, use_bucket};

pub static COKE_OVEN: Block = Block::named("coke_oven");
pub static COKE_OVEN_BRICKS: Block = Block::named("coke_oven_bricks");
pub static COKE_OVEN_DRAIN: Block = Block::named("coke_oven_drain");
pub static CREOSOTE: Block = Block::named("creosote");
static CREOSOTE_BUCKET: Item = Item::named("creosote_bucket");
static BUCKET: Item = Item::named("base:bucket");
pub static COKE: Item = Item::named("coke");
pub static SULFUR: Item = Item::named("sulfur");
static COKE_OVEN_MENU: Menu = Menu::named("coke_oven");
static SMOKE: Particle = Particle::named("coke_oven_smoke");
/// What a coke oven cooks.
static COAL: Tag = Tag::named("gems/coal");

/// Its box: 3 high, the door in the middle layer, the block behind the door hollow.
pub const SHAPE: Shape = Shape {
    height: 3,
    controller_layer: 1,
    hollow_layers: 1..2,
};
/// Millibuckets of creosote the door holds, and a drain.
pub const CREOSOTE_CAPACITY: u32 = 4_000;
pub const DRAIN_CAPACITY: u32 = 1_000;
/// Ticks of burning a coke gives: twice a coal's.
const COKE_BURNS: u32 = 3_200;
/// Its slots: coal, then coke and sulfur.
const INPUT: u32 = 1;
const OUTPUT: u32 = 4;
/// Light the door gives while it cooks.
const GLOW: u8 = 10;

value! {
    /// A coke oven, kept by its door: the ticks its batch has `cooked`, the coal in that batch
    /// (`charge`), the coal coked since its last sulfur (`toward_sulfur`), its status, and
    /// while it is not built its fault and the block where. Its creosote is its `HELD`.
    #[derive(Default)]
    pub struct Coking {
        pub cooked: u32,
        pub charge: u32,
        pub toward_sulfur: u32,
        pub status: u32,
        pub fault: u32,
        pub fault_x: i32,
        pub fault_y: i32,
        pub fault_z: i32,
    }
}
static COKING: Component<Coking> = Component::named("cogwork:coking");

/// What the oven is doing; its screen says `cogwork.coke_oven.<name>`. Kept in
/// `Coking::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// Its box is not built: `Coking::fault` says what is wrong.
    #[default]
    Unbuilt,
    Idle,
    Coking,
    /// Its coke and sulfur have no room in its output.
    Full,
    /// Its creosote has no room in the door.
    NoDrain,
}

impl Coking {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    pub fn fault(&self) -> Fault {
        Fault::from_repr(self.fault).unwrap_or_default()
    }

    /// Keeps it from cooking for `problem`.
    pub fn set_unbuilt(&mut self, problem: Problem) {
        self.set_status(Status::Unbuilt);
        self.fault = problem.fault as u32;
        (self.fault_x, self.fault_y, self.fault_z) = (problem.at.x, problem.at.y, problem.at.z);
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &COKING,
        Coking::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&COKING, work::tick);
    register_blocks(r);
    r.menu(&COKE_OVEN_MENU)
        .group("input", Slots::row(INPUT).accepts("#gems/coal").holder())
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    r.item(&COKE)
        .tag("coke")
        .set(&base_api::FUEL, COKE_BURNS)
        .tab(&crate::TAB);
    r.item(&SULFUR).tag("dusts/sulfur").tab(&crate::TAB);
    r.liquid(&CREOSOTE)
        .block(|block| block.light_opacity(15).drag(0.3))
        .flow_delay(15)
        .drop_off(2)
        .slope_find(2)
        .fog(0x1a0f06, 0.375, 1.5)
        .bucket(&CREOSOTE_BUCKET, &BUCKET, &crate::TAB);
    r.particle(&SMOKE)
        .textures(&["core:generic_3", "core:generic_4", "core:generic_5"])
        .life(2.0, 3.0)
        .size(0.25, 0.4)
        .gravity(-0.08)
        .drag(0.95)
        .color(0x302A26C0, 0x6A646000)
        .translucent();
}

fn register_blocks(r: &mut Registry) {
    brick(r.block(&COKE_OVEN_BRICKS)).texture("coke_oven_bricks");
    brick(r.block(&COKE_OVEN_DRAIN))
        .texture("coke_oven_drain")
        .on_use(use_drain)
        .entity(|entity| entity.attach(&HELD));
    brick(r.block(&COKE_OVEN))
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|props, state| {
            let lit = props.is("working", "true");
            let front = if lit {
                "coke_oven_door_lit"
            } else {
                "coke_oven_door"
            };
            let state = state.texture("coke_oven_bricks").front(front);
            if lit { state.light(GLOW) } else { state }
        })
        .entity(|entity| {
            entity
                .attach(&COKING)
                .attach(&HELD)
                .inventory(INPUT + OUTPUT)
        })
        .opens(&COKE_OVEN_MENU);
}

/// What every block of the oven shares: as hard as bricks, mined with a pickaxe.
fn brick(builder: weft_sdk::BlockBuilder<'_>) -> weft_sdk::BlockBuilder<'_> {
    builder
        .hardness(2.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// Puts `stack` from a tube into the coke oven `at`: coal into its input, nothing else. None
/// for what is no coke oven.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &COKING)?;
    if !COAL.on_item(stack.item) {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The millibuckets of creosote `block` holds for a coke oven, if it is its door or a drain.
pub fn creosote_capacity(block: crate::lib::BlockState) -> Option<u32> {
    if block.is(&COKE_OVEN) {
        Some(CREOSOTE_CAPACITY)
    } else if block.is(&COKE_OVEN_DRAIN) {
        Some(DRAIN_CAPACITY)
    } else {
        None
    }
}

/// An empty bucket used on a drain fills with creosote.
fn use_drain(click: &mut Use) -> bool {
    let world = click.world();
    let Some(drain) = Vessel::at(&world, click.pos().into()) else {
        return false;
    };
    let mut holders = Holders::outside(world);
    use_bucket(&mut holders, &drain, click.player())
}

/// The coke oven's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Coke Oven                                                       |
/// | Coking 8 coal                                                   |  status
/// | [####......]                                                    |  the batch cooked
/// | [####......]                                                    |  its creosote
/// | 2000 / 4000 mB of Creosote                                      |
/// | Coal                Coke and sulfur                             |
/// | [ ]                 [ ][ ][ ][ ]                                |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|view, out| {
            let coking = coking_of(view);
            let (key, args) = status_line(&coking);
            out.set(&key, &args);
        });
        ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |view| {
            coking_of(view).cooked as f32 / COKE_TICKS as f32
        });
        crate::machine::tank_bar(ui, CREOSOTE_CAPACITY);
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.coke_oven.input");
                ui.group("input");
            });
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.coke_oven.output");
                ui.group("output");
            });
        });
        ui.player_inventory(true);
    });
}

/// The status line of `coking`: its lang key and arguments.
fn status_line(coking: &Coking) -> (String, Vec<Arg<'static>>) {
    match coking.status() {
        Status::Unbuilt => {
            let fault: &str = coking.fault().into();
            let place = [coking.fault_x, coking.fault_y, coking.fault_z];
            let args = place.map(|coordinate| Arg::Int(coordinate as i64));
            (format!("cogwork.brickwork.{fault}"), Vec::from(args))
        }
        status => {
            let name: &str = status.into();
            let charge = Arg::Int(coking.charge as i64);
            (format!("cogwork.coke_oven.{name}"), vec![charge])
        }
    }
}

fn coking_of(view: &ClientView) -> Coking {
    view.menu_component(&COKING).unwrap_or_default()
}

/// The part of a coke oven the block at `pos` is, if any.
fn part_at(world: &World, pos: Pos) -> Option<crate::brickwork::Part> {
    use crate::brickwork::Part;
    use crate::lib::TypedWorld;
    let block = world.block_typed(pos);
    let parts = [
        (&COKE_OVEN_BRICKS, Part::Brick),
        (&COKE_OVEN_DRAIN, Part::Port),
        (&COKE_OVEN, Part::Controller),
    ];
    parts
        .into_iter()
        .find(|(kind, _)| block.is(kind))
        .map(|(_, part)| part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_and_fault_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.coke_oven.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        for &fault in Fault::VARIANTS {
            let key = format!("cogwork.brickwork.{}", <&str>::from(fault));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        for key in ["input", "output"] {
            let key = format!("cogwork.coke_oven.{key}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let coking = Coking {
            status: 99,
            fault: 99,
            ..Coking::default()
        };
        assert_eq!(coking.status(), Status::Unbuilt, "a newer status");
        assert_eq!(coking.fault(), Fault::NotPart, "a newer fault");
    }

    #[test]
    fn what_a_coke_oven_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Coking, cooked), 0, "its batch");
        assert_eq!(offset_of!(Coking, toward_sulfur), 8, "toward sulfur");
        assert_eq!(offset_of!(Coking, status), 12, "its status");
        assert_eq!(offset_of!(Coking, fault_x), 20, "where its fault is");
    }

    #[test]
    fn a_batch_of_creosote_fits_the_door() {
        let batch = batch::MOST_AT_ONCE * batch::CREOSOTE_PER_COAL;
        assert_eq!(CREOSOTE_CAPACITY / batch, 2, "two batches undrained");
    }

    #[test]
    fn its_pictures_are_there() {
        for file in [
            "textures/block/coke_oven_bricks.png",
            "textures/block/coke_oven_drain.png",
            "textures/block/coke_oven_door.png",
            "textures/block/coke_oven_door_lit.png",
            "textures/block/coke_oven_door_lit_e.png",
            "textures/block/creosote_still.png",
            "textures/item/creosote_bucket.png",
            "textures/item/coke.png",
            "textures/item/sulfur.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
