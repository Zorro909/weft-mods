//! The leveler: makes an area flat at its own height, or at the height of the survey markers
//! next to it. It clears what stands above that height into its output, then fills the holes
//! below it with blocks from its stock.

mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
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

pub fn register(r: &mut Registry) {
    r.entity_component(
        &LEVEL,
        Level::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&LEVEL, work::tick);
    r.machine(&LEVELER, &LEVELER_MENU)
        .block(|b| crate::machine::block(b, "leveler").hardness(4.0))
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

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::col().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                ui.text_bind(|c, out| {
                    let l = c.menu_component(&LEVEL).unwrap_or_default();
                    let status: &str = Status::from_repr(l.status).unwrap_or_default().into();
                    out.set(
                        &format!("cogwork.leveler.{status}"),
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
}
