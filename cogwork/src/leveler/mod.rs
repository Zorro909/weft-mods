//! The leveler: makes an area flat at its own height, or at the height of the survey markers
//! next to it. It clears what stands above that height into its output, then fills the holes
//! below it with blocks from its stock.

mod work;

use weft_sdk::*;

pub static LEVELER: Block = Block::named("leveler");
static LEVELER_MENU: Menu = Menu::named("leveler");

value! {
    /// A leveler's work: the area from `lo` to `hi`, flat at `lo`'s height, and the next block
    /// to look at.
    #[derive(Default)]
    pub struct Level {
        pub stage: u32,
        pub status: u32,
        pub lo: [i32; 3],
        pub hi: [i32; 3],
        pub cursor: [i32; 3],
        pub spent: i64,
    }
}
static LEVEL: Component<Level> = Component::named("cogwork:level");

mod stage {
    pub const NEW: u32 = 0;
    pub const CLEAR: u32 = 1;
    pub const FILL: u32 = 2;
    pub const DONE: u32 = 3;
}

/// What the screen says, by `cogwork.leveler.<name>`; 0 is idle, before it starts.
mod status {
    pub const NO_POWER: u32 = 1;
    pub const CLEARING: u32 = 2;
    pub const FILLING: u32 = 3;
    pub const FULL: u32 = 4;
    pub const NO_BLOCKS: u32 = 5;
    pub const BLOCKED: u32 = 6;
    pub const DONE: u32 = 7;
    pub const NAMES: [&str; 8] = [
        "idle",
        "no_power",
        "clearing",
        "filling",
        "full",
        "no_blocks",
        "blocked",
        "done",
    ];
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &LEVEL,
        Level::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&LEVEL, work::tick);
    r.machine(&LEVELER, &LEVELER_MENU)
        .block(block)
        .group("input", Slots::grid(9, 2))
        .group("output", Slots::grid(3, 3).output())
        .attach(&LEVEL)
        .attach_with(
            &ENERGY,
            Energy {
                stored: 0,
                capacity: 10_000,
                max_in: 1_000,
                max_out: 0,
            },
        )
        .screen(screen);
}

fn block(b: BlockBuilder<'_>) -> BlockBuilder<'_> {
    b.facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|p, s| {
            let mut faces = ["leveler_side"; 6];
            faces[3] = "leveler_top";
            faces[p.facing().face() as usize] = if p.is("working", "true") {
                "leveler_front_on"
            } else {
                "leveler_front"
            };
            s.faces(faces)
        })
        .hardness(4.0)
        .mineable("pickaxe")
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::col().gap(3.0), |ui| {
                crate::power_bar(ui);
                ui.text_bind(|c, out| {
                    let l = c.menu_component(&LEVEL).unwrap_or_default();
                    let name = status::NAMES.get(l.status as usize).unwrap_or(&"idle");
                    out.set(
                        &format!("cogwork.leveler.{name}"),
                        &[Arg::Int(l.cursor[1] as i64)],
                    );
                });
                ui.text_bind(|c, out| {
                    let l = c.menu_component(&LEVEL).unwrap_or_default();
                    let (x, z) = (l.hi[0] - l.lo[0] + 1, l.hi[2] - l.lo[2] + 1);
                    let size = |v: i32| Arg::Int(if l.stage == stage::NEW { 0 } else { v as i64 });
                    out.set(
                        "cogwork.leveler.area",
                        &[size(x), size(z), Arg::Int(l.lo[1] as i64)],
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
