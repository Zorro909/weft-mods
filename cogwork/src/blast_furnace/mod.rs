//! The blast furnace: a box of fire bricks 3 by 3 and 4 high, hollow up its middle above the
//! hearth, its controller in the middle of the bottom of its front (`brickwork`). A driven
//! bellows blowing into any block of its bottom layer gives it the draft it smelts by: the
//! bellows then draws 80 energy a tick instead of 20 (`bellows`). An iron ingot and a coke, put
//! in by tube or by hand, smelt in 400 ticks into a steel bloom and slag (`smelt`), which leave by
//! the controller's front into a tube there; the bloom is forged into a steel ingot on the forge
//! anvil.
//!
//! Its blocks are plain cubes: the controller's tap hole glows while it smelts, and sparks fly out
//! of its open top.

mod smelt;
mod work;

pub use smelt::SMELT_TICKS;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, BlockBuilder, ClientView, Component, Facings, Holder, Holds, ItemStack, Menu,
    Registry, S, Slots, Tag, Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::brickwork::{Fault, Part, Problem, Shape};
use crate::lib::{BlockState, Pos, TypedWorld};

pub static BLAST_FURNACE: Block = Block::named("blast_furnace");
pub static FIRE_BRICKS: Block = Block::named("fire_bricks");
static BLAST_MENU: Menu = Menu::named("blast_furnace");
static IRON_INGOTS: Tag = Tag::named("ingots/iron");
static COKE: Tag = Tag::named("coke");

/// Its box: 4 high, the controller at the bottom, hollow above the hearth.
pub const SHAPE: Shape = Shape {
    height: 4,
    controller_layer: 0,
    hollow_layers: 1..4,
};
/// Light its tap hole gives while it smelts.
const GLOW: u8 = 13;

value! {
    /// A blast furnace, kept by its controller: the ticks of draft its ingot has had (`heat`),
    /// its status, and while it is not built its fault and the block where.
    #[derive(Default)]
    pub struct Blast {
        pub heat: u32,
        pub status: u32,
        pub fault: u32,
        pub fault_x: i32,
        pub fault_y: i32,
        pub fault_z: i32,
    }
}
static BLAST: Component<Blast> = Component::named("cogwork:blast");

/// What the furnace is doing; its screen says `cogwork.blast_furnace.<name>`. Kept in
/// `Blast::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// Its box is not built: `Blast::fault` says what is wrong.
    #[default]
    Unbuilt,
    /// It lacks an iron ingot or a coke.
    Idle,
    /// No driven bellows blows into it.
    NoDraft,
    Smelting,
    /// Its bloom and slag have no room in the output.
    Full,
}

impl Blast {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    pub fn fault(&self) -> Fault {
        Fault::from_repr(self.fault).unwrap_or_default()
    }

    /// Keeps it from smelting for `problem`.
    pub fn set_unbuilt(&mut self, problem: Problem) {
        self.set_status(Status::Unbuilt);
        self.fault = problem.fault as u32;
        (self.fault_x, self.fault_y, self.fault_z) = (problem.at.x, problem.at.y, problem.at.z);
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &BLAST,
        Blast::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&BLAST, work::tick);
    brick(r.block(&FIRE_BRICKS)).texture("fire_bricks");
    brick(r.block(&BLAST_FURNACE))
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|props, state| {
            let lit = props.is("working", "true");
            let front = if lit {
                "blast_furnace_front_lit"
            } else {
                "blast_furnace_front"
            };
            let state = state.texture("fire_bricks").front(front);
            if lit { state.light(GLOW) } else { state }
        })
        .entity(|entity| entity.attach(&BLAST).inventory(4))
        .opens(&BLAST_MENU);
    r.menu(&BLAST_MENU)
        .group("input", Slots::single().accepts("#ingots/iron").holder())
        .group("fuel", Slots::single().accepts("#coke").holder())
        .group("output", Slots::row(2).output().holder())
        .player_inventory()
        .screen(screen);
}

/// What its blocks share: as hard as bricks, mined with a pickaxe.
fn brick(builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    builder
        .hardness(2.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// Whether `block` is part of a blast furnace: what a driven bellows blowing into it works
/// harder for.
pub fn is_part(block: BlockState) -> bool {
    block.is(&FIRE_BRICKS) || block.is(&BLAST_FURNACE)
}

/// The part of a blast furnace the block at `pos` is, if any.
fn part_at(world: &World, pos: Pos) -> Option<Part> {
    let block = world.block_typed(pos);
    if block.is(&FIRE_BRICKS) {
        Some(Part::Brick)
    } else if block.is(&BLAST_FURNACE) {
        Some(Part::Controller)
    } else {
        None
    }
}

/// Puts `stack` from a tube into the blast furnace `at`: iron ingots into its input, coke into
/// its fuel, nothing else. None for what is no blast furnace.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &BLAST)?;
    let group = if IRON_INGOTS.on_item(stack.item) {
        "input"
    } else if COKE.on_item(stack.item) {
        "fuel"
    } else {
        return Some(stack);
    };
    let slots = world.inventory(at).find_group(group)?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The blast furnace's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Blast Furnace                                                   |
/// | Smelting                                                        |  status
/// | [####......]                                                    |  the bloom's heat
/// | Iron   Coke           Steel bloom and slag                      |
/// | [ ]    [ ]            [ ][ ]                                    |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.text_bind(|view, out| {
            let (key, args) = status_line(&blast_of(view));
            out.set(&key, &args);
        });
        ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |view| {
            blast_of(view).heat as f32 / SMELT_TICKS as f32
        });
        ui.row(S::row().gap(10.0), |ui| {
            for (label, group) in [
                ("cogwork.blast_furnace.input", "input"),
                ("cogwork.blast_furnace.fuel", "fuel"),
                ("cogwork.blast_furnace.output", "output"),
            ] {
                ui.column(S::column().gap(3.0), |ui| {
                    ui.text(label);
                    ui.group(group);
                });
            }
        });
        ui.player_inventory(true);
    });
}

/// The status line of `blast`: its lang key and arguments.
fn status_line(blast: &Blast) -> (String, Vec<Arg<'static>>) {
    match blast.status() {
        Status::Unbuilt => {
            let fault: &str = blast.fault().into();
            let place = [blast.fault_x, blast.fault_y, blast.fault_z];
            let args = place.map(|coordinate| Arg::Int(coordinate as i64));
            (format!("cogwork.brickwork.{fault}"), Vec::from(args))
        }
        status => {
            let name: &str = status.into();
            let draw = Arg::Int(crate::bellows::BLAST_DRAW);
            (format!("cogwork.blast_furnace.{name}"), vec![draw])
        }
    }
}

fn blast_of(view: &ClientView) -> Blast {
    view.menu_component(&BLAST).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_and_label_has_a_name() {
        let statuses = Status::VARIANTS.iter().map(|&status| <&str>::from(status));
        let labels = ["input", "fuel", "output"].into_iter();
        for name in statuses.chain(labels) {
            let key = format!("cogwork.blast_furnace.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let blast = Blast {
            status: 99,
            fault: 99,
            ..Blast::default()
        };
        assert_eq!(blast.status(), Status::Unbuilt, "a newer status");
        assert_eq!(blast.fault(), Fault::NotPart, "a newer fault");
    }

    #[test]
    fn what_a_blast_furnace_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Blast, heat), 0, "its heat");
        assert_eq!(offset_of!(Blast, status), 4, "its status");
        assert_eq!(offset_of!(Blast, fault_x), 12, "where its fault is");
    }

    #[test]
    fn its_pictures_are_there() {
        for file in [
            "textures/block/fire_bricks.png",
            "textures/block/blast_furnace_front.png",
            "textures/block/blast_furnace_front_lit.png",
            "textures/block/blast_furnace_front_lit_e.png",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
