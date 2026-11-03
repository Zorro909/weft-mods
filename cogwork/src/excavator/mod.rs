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

value! {
    /// An excavator's work. `lo` and `hi` are the corners of its frame; it mines inside it.
    /// `step` counts the frame blocks put up, `cursor` is the next block to mine, `spent` the
    /// energy put into the job under way, and `waited` the ticks the cursor's block has been
    /// unloaded. `head` is where the drill's tip is, in the world. `refused` counts the tries
    /// running at breaking what stands where the next frame block goes that were refused.
    #[derive(Default)]
    pub struct Dig {
        pub stage: u32,
        pub status: u32,
        pub lo: [i32; 3],
        pub hi: [i32; 3],
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

pub mod stage {
    pub const NEW: u32 = 0;
    pub const FRAME: u32 = 1;
    pub const MINE: u32 = 2;
    pub const DONE: u32 = 3;
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

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::col().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    let status: &str = Status::from_repr(d.status).unwrap_or_default().into();
                    out.set(
                        &format!("cogwork.excavator.{status}"),
                        &[
                            Arg::Int(d.cursor[1] as i64),
                            Arg::Int(power::standby(d.lo, d.hi)),
                        ],
                    );
                });
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    let (x, z) = (d.hi[0] - d.lo[0] - 1, d.hi[2] - d.lo[2] - 1);
                    out.set(
                        "cogwork.excavator.area",
                        &[
                            Arg::Int(x.max(0) as i64),
                            Arg::Int(z.max(0) as i64),
                            Arg::Int(power::standby(d.lo, d.hi)),
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
}
