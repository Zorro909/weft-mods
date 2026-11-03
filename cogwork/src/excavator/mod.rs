//! The excavator: it builds a gantry frame around its area, then a drill on the gantry mines the
//! area out layer by layer from the top of the frame down to the floor of the world, handing
//! everything it mines to the tubes and chests beside it.

mod area;
mod dig;
mod mine;
mod power;
mod rig;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::*;

pub static EXCAVATOR: Block = Block::named("excavator");
pub static GANTRY_FRAME: Block = Block::named("gantry_frame");
static EXCAVATOR_MENU: Menu = Menu::named("excavator");

pub use area::Frame;

value! {
    /// An excavator's work. It puts up `frame`, then mines inside it. `step` counts the frame
    /// blocks put up, `cursor` is the next block to mine, `spent` the energy put into the job
    /// under way, and `waited` the ticks the cursor's block has been unloaded. `head` is where
    /// the drill's tip is, in the world. `refused` counts the tries running at breaking what
    /// stands where the next frame block goes that were refused.
    #[derive(Default)]
    pub struct Dig {
        pub stage: u32,
        pub status: u32,
        // Laid out as the `lo` and `hi` it replaced, so saved excavators keep their frame.
        pub frame: Frame,
        pub cursor: [i32; 3],
        pub step: u32,
        pub spent: i64,
        pub head: [f32; 3],
        pub waited: u32,
        // Last, so excavators saved before it load with it 0.
        pub refused: u32,
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
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &DIG,
        Dig::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&DIG, dig::tick);
    r.machine(&EXCAVATOR, &EXCAVATOR_MENU)
        .block(|b| crate::machine::block(b, "excavator").hardness(4.0))
        .group("output", Slots::grid(3, 3).output())
        .attach(&DIG)
        .attach_with(
            &ENERGY,
            Energy {
                stored: 0,
                capacity: 20_000,
                // Any number of engines: standby alone can pass 1000 on a large area.
                max_in: 20_000,
                max_out: 0,
            },
        )
        .screen(screen);
    r.block(&GANTRY_FRAME)
        .texture("gantry_frame")
        .cutout()
        .hardness(1.0)
        .mineable("pickaxe")
        .drops_nothing()
        .no_item();
    rig::register(r);
}

/// The excavator's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Excavator                                                       |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, output
/// | 12000 / 20000 energy                      [ ][ ][ ]             |
/// | Mining layer 42                           [ ][ ][ ]             |  status
/// | Area 9 x 9, standby 30 energy per tick                          |
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::col().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    let status: &str = d.status().into();
                    out.set(
                        &format!("cogwork.excavator.{status}"),
                        &[Arg::Int(d.cursor[1] as i64), Arg::Int(d.frame.standby())],
                    );
                });
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    let [x, z] = d.frame.inside();
                    out.set(
                        "cogwork.excavator.area",
                        &[
                            Arg::Int(x as i64),
                            Arg::Int(z as i64),
                            Arg::Int(d.frame.standby()),
                        ],
                    );
                });
            });
            ui.group("output");
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
            let key = format!("cogwork.excavator.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn an_unknown_saved_number_reads_as_the_default() {
        let d = Dig {
            stage: 99,
            status: 99,
            ..Dig::default()
        };
        assert_eq!(d.stage(), Stage::New, "a stage from a newer cogwork");
        assert_eq!(d.status(), Status::Idle, "a status from a newer cogwork");
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
    }
}
