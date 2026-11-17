//! Quarries: each builds a frame around its area, then a drill on a gantry across the frame
//! mines the area out layer by layer from the top of the frame down as deep as its kind and the
//! world's settings let it (`kind`, `depth`), handing everything it mines to the tubes and chests
//! beside it. The Timber and Iron Quarries and the Steel Excavator dig with a drill bit (`bit`)
//! and build their frames of logs and string, of iron ingots or of steel ingots; the Gantry
//! Excavator, block `excavator`, builds its frame of composite panels and digs with no bit or a
//! diamond-tipped one. A Bit Magazine beside a quarry keeps its bit slot filled
//! (`bit_magazine`).

mod area;
pub mod bit;
mod depth;
mod dig;
mod frames;
mod framing;
pub mod kind;
mod lube;
mod materials;
mod mine;
mod power;
mod rig;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Holder, Holds, ItemStack, Registry, S, Slots, Ui, Vec3, World,
    abi, component_flags, value,
};

pub static EXCAVATOR: Block = Block::named("excavator");
pub static TIMBER_QUARRY: Block = Block::named("timber_quarry");
pub static IRON_QUARRY: Block = Block::named("iron_quarry");
pub static STEEL_EXCAVATOR: Block = Block::named("steel_excavator");
pub use area::Frame;
use depth::Floor;
pub use frames::{COMPOSITE_FRAME, GANTRY_FRAME, IRON_FRAME, LASHED_FRAME, STEEL_FRAME};
use kind::{KINDS, Kind};
pub use lube::CAPACITY as LUBRICANT_CAPACITY;
pub use materials::taken;

use crate::lib::{Pos, TypedWorld};
use crate::loading::Cubes;

value! {
    /// An excavator's work. It puts up `frame`, then mines inside it. `step` counts the frame
    /// blocks put up, `cursor` is the next block to mine, `spent` the energy put into the job
    /// under way, and `waited` the ticks the cursor's block has been unloaded. `head` is where
    /// the drill's tip is, in the world. `refused` counts the tries running at breaking what
    /// stands where the next frame block goes that were refused. `paid_ahead` counts the frame
    /// blocks the materials already taken pay for (`materials`). `lube_skipped` is 1 while the
    /// last block mined with lubricant skipped wearing the bit (`lube::wears`).
    #[derive(Default)]
    pub struct Dig {
        pub stage: u32,
        pub status: u32,
        // Laid out as the `lo` and `hi` it replaced, so saved excavators keep their frame.
        pub frame: Frame,
        pub cursor: Pos,
        pub step: u32,
        pub spent: i64,
        // In `f32`s, as it has always been saved: `head()` reads it.
        head: [f32; 3],
        pub waited: u32,
        // Last, so excavators saved before it load with it 0.
        pub refused: u32,
        // The padding of the 80 bytes excavators were saved in until `paid_ahead` came: never
        // read, as those saves may hold anything there.
        saved_padding: u32,
        // Past the old 80 bytes, so excavators saved before it load with it 0.
        pub paid_ahead: u32,
        // Last, so excavators saved before it load with it 0.
        pub lube_skipped: u32,
    }
}
pub static DIG: Component<Dig> = Component::named("cogwork:dig");

/// How far an excavator has got. Kept in `Dig::stage` as its number, so the order is part of
/// the save format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Stage {
    /// Placed, its area not planned yet.
    #[default]
    New,
    Frame,
    Mine,
    Done,
}

/// What the excavator is doing; its screen says `cogwork.excavator.<name>`. Kept in
/// `Dig::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Frame,
    Mining,
    Full,
    Done,
    Blocked,
    Jammed,
    /// Out of what its frame is built of.
    NeedsMaterials,
    /// It digs only with a bit, and has none that fits.
    NeedsBit,
}

impl Status {
    /// Whether the excavator shows it is at work: building its frame or mining.
    pub fn is_working(self) -> bool {
        matches!(self, Status::Frame | Status::Mining)
    }
}

impl Dig {
    pub fn stage(&self) -> Stage {
        Stage::from_repr(self.stage).unwrap_or_default()
    }

    pub fn set_stage(&mut self, stage: Stage) {
        self.stage = stage as u32;
    }

    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }

    pub fn head(&self) -> Vec3 {
        Vec3::from(self.head)
    }

    pub fn set_head(&mut self, at: Vec3) {
        self.head = at.to_array().map(|coord| coord as f32);
    }

    /// The cubes of the layer it mines, across the whole frame. None once the cursor has passed
    /// `floor`: it finishes on the next tick.
    fn mining_layer(&self, floor: Floor) -> Option<Cubes> {
        let (lo, hi, y) = (self.frame.lo, self.frame.hi, self.cursor.y);
        (!floor.is_above(y)).then(|| Cubes::spanning(lo.with_y(y), hi.with_y(y)))
    }
}

/// Keeps the cubes the excavator at `pos` needs loaded while it mines, down to `floor`, then
/// lets them go. Building the frame keeps nothing loaded: it takes a player near.
fn keep_loaded(world: &World, pos: Pos, dig: &Dig, floor: Floor) {
    match dig.stage() {
        Stage::Mine => crate::loading::keep_machine(world, pos, dig.mining_layer(floor)),
        _ => crate::loading::release_machine(world, pos),
    }
}

/// Slots of the output, and of the frame materials of a kind that has them.
const OUTPUT_SLOTS: (u32, u32) = (3, 3);
const MATERIAL_SLOTS: (u32, u32) = (3, 2);

pub fn register(r: &mut Registry) {
    r.entity_component(
        &DIG,
        Dig::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&DIG, dig::tick);
    depth::register(r);
    bit::register(r);
    for kind in KINDS {
        quarry(r, kind);
    }
    frames::register(r);
    rig::register(r);
}

/// The block of `kind` and its menu. Its slots are the output, then the bit and the materials
/// where it has them: an excavator saved with only the output keeps it.
fn quarry(r: &mut Registry, kind: &'static Kind) {
    let slots = |(cols, rows): (u32, u32)| cols * rows;
    let bit_slots = u32::from(!kind.bits.is_empty());
    let material_slots = if kind.frame_cost.is_empty() {
        0
    } else {
        slots(MATERIAL_SLOTS)
    };
    let inventory = slots(OUTPUT_SLOTS) + bit_slots + material_slots;
    crate::machine::block(r.block(kind.block), kind.block.path())
        .hardness(kind.hardness)
        .sound_group(kind.sound_group)
        .on_break(crate::loading::let_go)
        .entity(move |entity| {
            entity
                .attach(&DIG)
                .attach(&rig::RIG)
                .attach_with(&ENERGY, power::smallest_store())
                .attach(&crate::machine::METER)
                .attach(&crate::liquid::HELD)
                .inventory(inventory)
        })
        .opens(kind.menu);
    let (cols, rows) = OUTPUT_SLOTS;
    let mut menu = r
        .menu(kind.menu)
        .group("output", Slots::grid(cols, rows).output().holder());
    if bit_slots > 0 {
        let rule = kind.bit_rule();
        menu = menu.group(bit::GROUP, Slots::single().accepts(&rule).holder());
    }
    if material_slots > 0 {
        let (cols, rows) = MATERIAL_SLOTS;
        let rule = kind.material_rule();
        let group = Slots::grid(cols, rows).accepts(&rule).holder();
        menu = menu.group(materials::GROUP, group);
    }
    menu.player_inventory().screen(move |ui| screen(ui, kind));
}

/// Puts `stack` from a tube into the quarry `at`: what its frame is built of into `materials`,
/// nothing else. Bits go in by hand. None for what is no quarry, or a quarry that takes nothing
/// in.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &DIG)?;
    let kind = Kind::of(block_of(world, at)?)?;
    if !kind.takes_material(stack) {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group(materials::GROUP)?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The block a block entity, or the block, `at` is.
fn block_of(world: &World, at: Holder) -> Option<crate::lib::BlockState> {
    let pos = match at.kind {
        abi::holder::BLOCK => Pos::from(at.pos),
        abi::holder::ENTITY => Pos::from(world.entity(at.entity)?.block_pos()),
        _ => return None,
    };
    Some(world.block_typed(pos))
}

/// A quarry's screen, above the player's inventory; the bit and the materials only on a kind
/// that has them:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Timber Quarry                                                   |
/// | [#####.....]                              [ ][ ][ ]   Bit       |  stored energy, output, bit
/// | 12000 / 20000 energy                      [ ][ ][ ]   [ ]       |
/// | Using 16 energy a tick                    [ ][ ][ ]   Frame     |  drawn last tick, and how short
/// | Mining layer 42                                       [ ][ ][ ] |  status, frame materials
/// | Area 5 x 5, standby 4 energy per tick                 [ ][ ][ ] |
/// | [##........]  200 / 1000 mB of Lubricant                        |  lubricant, if any
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui, kind: &'static Kind) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                crate::machine::meter_line(ui);
                ui.text_bind(move |view, out| {
                    let dig = view.menu_component(&DIG).unwrap_or_default();
                    let status: &str = dig.status().into();
                    let layer = dig.cursor.y;
                    out.set(
                        &format!("cogwork.excavator.{status}"),
                        &[Arg::Int(layer as i64), Arg::Int(kind.standby(&dig.frame))],
                    );
                });
                ui.text_bind(move |view, out| {
                    let dig = view.menu_component(&DIG).unwrap_or_default();
                    let [x, z] = dig.frame.inside();
                    out.set(
                        "cogwork.excavator.area",
                        &[
                            Arg::Int(x as i64),
                            Arg::Int(z as i64),
                            Arg::Int(kind.standby(&dig.frame)),
                        ],
                    );
                });
                crate::machine::tank_bar(ui, LUBRICANT_CAPACITY);
            });
            ui.group("output");
            if !kind.bits.is_empty() || !kind.frame_cost.is_empty() {
                ui.column(S::column().gap(3.0), |ui| supplies(ui, kind));
            }
        });
        ui.player_inventory(true);
    });
}

/// The bit and the frame materials of `kind`, each under its label.
fn supplies(ui: &mut Ui, kind: &Kind) {
    if !kind.bits.is_empty() {
        ui.text("cogwork.excavator.bit_slot");
        ui.group(bit::GROUP);
    }
    if !kind.frame_cost.is_empty() {
        ui.text("cogwork.excavator.materials_slots");
        ui.group(materials::GROUP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.excavator.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let dig = Dig {
            stage: 99,
            status: 99,
            ..Dig::default()
        };
        assert_eq!(dig.stage(), Stage::New, "a stage from a newer cogwork");
        assert_eq!(dig.status(), Status::Idle, "a status from a newer cogwork");
    }

    #[test]
    fn the_frame_lies_where_the_old_corners_were_saved() {
        use std::mem::offset_of;
        assert_eq!(
            offset_of!(Dig, frame),
            8,
            "after stage and status, as `lo` was"
        );
        assert_eq!(offset_of!(Dig, cursor), 32, "after `lo` and `hi`");
        assert_eq!(offset_of!(Dig, refused), 72, "the last field saved before");
        assert_eq!(
            offset_of!(Dig, paid_ahead),
            80,
            "past the 80 bytes saved before, not in their padding"
        );
        assert_eq!(
            offset_of!(Dig, lube_skipped),
            offset_of!(Dig, paid_ahead) + 4,
            "the lubricant's wear last"
        );
    }

    #[test]
    fn no_cube_is_let_go_and_kept_again_over_a_dig() {
        // The frame stands across the cubes from y 0 to 15 and from 16 up.
        let frame = Frame::from(([3, 13, 3], [40, 17, 40]));
        let floor = Floor::at(-64);
        let mut dig = Dig {
            stage: Stage::Mine as u32,
            frame,
            cursor: frame.first(),
            ..Dig::default()
        };
        // What it keeps after each block, as a tick does once the cursor has moved on.
        let mut kept = vec![dig.mining_layer(floor).expect("the top layer")];
        while !floor.is_above(dig.cursor.y) {
            dig.cursor = frame.next(dig.cursor);
            kept.extend(dig.mining_layer(floor));
        }
        assert_eq!(
            kept.last().map(|cubes| cubes.lo().y),
            Some(-4),
            "nothing below the floor"
        );
        assert_eq!(crate::loading::kept_again(kept), None, "from the top down");
    }
}
