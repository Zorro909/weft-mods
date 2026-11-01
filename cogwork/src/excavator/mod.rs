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
    /// `head` is where the drill's tip is, in the world.
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
    }
}
pub static DIG: Component<Dig> = Component::named("cogwork:dig");

pub mod stage {
    pub const NEW: u32 = 0;
    pub const FRAME: u32 = 1;
    pub const MINE: u32 = 2;
    pub const DONE: u32 = 3;
}

/// What the screen says, by `cogwork.excavator.<name>`; 0 is idle, before it starts.
pub mod status {
    pub const NO_POWER: u32 = 1;
    pub const FRAME: u32 = 2;
    pub const MINING: u32 = 3;
    pub const FULL: u32 = 4;
    pub const DONE: u32 = 5;
    pub const BLOCKED: u32 = 6;
    pub const NAMES: [&str; 7] = [
        "idle", "no_power", "frame", "mining", "full", "done", "blocked",
    ];
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &DIG,
        Dig::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&DIG, dig::tick);
    r.machine(&EXCAVATOR, &EXCAVATOR_MENU)
        .block(|b| {
            b.facing(Facings::HORIZONTAL, Toward::Player)
                .prop("working", ["false", "true"])
                .states(|p, s| {
                    let mut faces = ["excavator_side"; 6];
                    faces[3] = "excavator_top";
                    faces[p.facing().face() as usize] = if p.is("working", "true") {
                        "excavator_front_on"
                    } else {
                        "excavator_front"
                    };
                    s.faces(faces)
                })
                .hardness(4.0)
                .mineable("pickaxe")
                .sound_group("base:block.stone")
                .tab(&crate::TAB)
        })
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
                crate::power_bar(ui);
                ui.text_bind(|c, out| {
                    let d = c.menu_component(&DIG).unwrap_or_default();
                    let name = status::NAMES.get(d.status as usize).unwrap_or(&"idle");
                    out.set(
                        &format!("cogwork.excavator.{name}"),
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
