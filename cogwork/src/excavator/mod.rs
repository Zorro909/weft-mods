//! The excavator: it builds a gantry frame around its area, then a drill on the gantry mines the
//! area out layer by layer from the top of the frame down to the floor of the world, handing
//! everything it mines to the tubes and chests beside it.

mod area;
mod dig;
mod mine;
mod power;
mod rig;

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

/// What the excavator is doing, as its screen says. Kept in `Dig::status` as its number, so
/// the order is part of the save format: new ones go last, in `ALL` too.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
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
    const ALL: [Status; 8] = [
        Status::Idle,
        Status::NoPower,
        Status::Frame,
        Status::Mining,
        Status::Full,
        Status::Done,
        Status::Blocked,
        Status::Jammed,
    ];

    /// The status kept as `raw`; one this version doesn't know reads as `Idle`.
    pub fn from_raw(raw: u32) -> Status {
        Status::ALL
            .get(raw as usize)
            .copied()
            .unwrap_or(Status::Idle)
    }

    fn key(self) -> &'static str {
        match self {
            Status::Idle => "cogwork.excavator.idle",
            Status::NoPower => "cogwork.excavator.no_power",
            Status::Frame => "cogwork.excavator.frame",
            Status::Mining => "cogwork.excavator.mining",
            Status::Full => "cogwork.excavator.full",
            Status::Done => "cogwork.excavator.done",
            Status::Blocked => "cogwork.excavator.blocked",
            Status::Jammed => "cogwork.excavator.jammed",
        }
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

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::col().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    out.set(
                        Status::from_raw(d.status).key(),
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
    fn every_status_is_kept_as_its_place_and_has_a_name() {
        for (i, status) in Status::ALL.into_iter().enumerate() {
            assert_eq!(Status::from_raw(i as u32), status, "{status:?} read back");
            assert_eq!(status as usize, i, "{status:?} in order");
            assert!(
                crate::translated(status.key()),
                "{status:?} in every lang file"
            );
        }
        assert_eq!(Status::from_raw(99), Status::Idle, "unknown is idle");
    }
}
