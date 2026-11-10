//! The leveler: makes an area flat at its own height, or at the height of the survey markers
//! next to it. It clears what stands above that height into its output, then fills the holes
//! below it with blocks from its stock.

mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::*;

use crate::lib::{Area, Pos};
use crate::loading::Cubes;

pub static LEVELER: Block = Block::named("leveler");
static LEVELER_MENU: Menu = Menu::named("leveler");

value! {
    /// A leveler's work: the `area` it makes flat at its height, the next block to look at, and
    /// the energy put into the job under way.
    #[derive(Default)]
    pub struct Level {
        pub stage: u32,
        pub status: u32,
        // Laid out as the `lo` and `hi` it replaced, so saved levelers keep their area.
        pub area: Area,
        pub cursor: Pos,
        pub spent: i64,
    }
}
static LEVEL: Component<Level> = Component::named("cogwork:level");

/// How far a leveler has got. Kept in `Level::stage` as its number, so the order is part of the
/// save format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Stage {
    /// Placed, its area not planned yet.
    #[default]
    New,
    Clear,
    Fill,
    Done,
}

/// What the leveler is doing; its screen says `cogwork.leveler.<name>`. Kept in
/// `Level::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Clearing,
    Filling,
    Full,
    NoBlocks,
    Blocked,
    Done,
}

impl Status {
    /// Whether the leveler shows it is at work: clearing or filling.
    pub fn is_working(self) -> bool {
        matches!(self, Status::Clearing | Status::Filling)
    }
}

impl Level {
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
}

/// Keeps the cubes the leveler at `pos` needs loaded until it is done, then lets them go: its
/// own and those around it, and those it works in.
fn keep_loaded(world: &World, pos: Pos, level: &Level) {
    match level.stage() {
        Stage::Done => crate::loading::release_machine(world, pos),
        _ => crate::loading::keep_machine(world, pos, work_cubes(level)),
    }
}

/// The cubes it works in, across the whole area: while it clears, those of the layer it
/// clears; while it fills, those of every layer left to fill. Filling ends just below the
/// height clearing ended at, so the cubes there stay rather than going and coming back. None
/// once the cursor has passed the last layer to fill: the stage ends on the next tick.
fn work_cubes(level: &Level) -> Option<Cubes> {
    let (lo, hi, y) = (level.area.lo, level.area.hi, level.cursor.y);
    match level.stage() {
        Stage::Clear => Some(Cubes::spanning(lo.with_y(y), hi.with_y(y))),
        Stage::Fill if y < lo.y => Some(Cubes::spanning(lo.with_y(y), hi.with_y(lo.y - 1))),
        Stage::Fill | Stage::New | Stage::Done => None,
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &LEVEL,
        Level::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&LEVEL, work::tick);
    crate::machine::block(r.block(&LEVELER), "leveler")
        .hardness(4.0)
        .on_break(crate::loading::let_go)
        .entity(|entity| {
            entity
                .attach(&LEVEL)
                .attach_with(&ENERGY, crate::machine::energy(10_000, 1_000))
                // The input, then the output.
                .inventory(18 + 9)
        })
        .opens(&LEVELER_MENU);
    r.menu(&LEVELER_MENU)
        .group("input", Slots::grid(9, 2).holder())
        .group("output", Slots::grid(3, 3).output().holder())
        .player_inventory()
        .screen(screen);
}

/// The leveler's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Leveler                                                         |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, output
/// | 4000 / 10000 energy                       [ ][ ][ ]             |
/// | Clearing layer 72                         [ ][ ][ ]             |  status
/// | Area 9 x 9, flat at height 64                                   |
/// | Stock                                                           |
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]                                     |  blocks to fill with
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]                                     |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|view, out| {
                    let level = view.menu_component(&LEVEL).unwrap_or_default();
                    let status: &str = level.status().into();
                    let layer = level.cursor.y;
                    out.set(
                        &format!("cogwork.leveler.{status}"),
                        &[Arg::Int(layer as i64)],
                    );
                });
                ui.text_bind(|view, out| {
                    let level = view.menu_component(&LEVEL).unwrap_or_default();
                    let planned = level.stage() != Stage::New;
                    let [x, _, z] = if planned { level.area.size() } else { [0; 3] };
                    let height = level.area.lo.y;
                    out.set(
                        "cogwork.leveler.area",
                        &[
                            Arg::Int(x as i64),
                            Arg::Int(z as i64),
                            Arg::Int(height as i64),
                        ],
                    );
                });
            });
            ui.group("output");
        });
        ui.text("cogwork.leveler.stock");
        ui.group("input");
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.leveler.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn the_area_lies_where_the_old_corners_were_saved() {
        use std::mem::offset_of;
        assert_eq!(
            offset_of!(Level, area),
            8,
            "after stage and status, as `lo` was"
        );
        assert_eq!(offset_of!(Level, cursor), 32, "after `lo` and `hi`");
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let level = Level {
            stage: 99,
            status: 99,
            ..Level::default()
        };
        assert_eq!(level.stage(), Stage::New, "a stage from a newer cogwork");
        assert_eq!(
            level.status(),
            Status::Idle,
            "a status from a newer cogwork"
        );
    }
}
