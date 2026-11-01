//! Cogwork: item tubes that show what they carry, engines that drive them, and an excavator
//! that digs out an area layer by layer. Everything goes through `weft_sdk` and `base_api`.

mod cogs;
mod engines;
mod excavator;
mod markers;
mod recipes;
mod shaft;
mod store;
mod tube;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

pub static TAB: CreativeTab = CreativeTab::named("cogwork");

fn init(r: &mut Registry) {
    r.creative_tab(&TAB, &Item::named("iron_cog"));
    cogs::register(r);
    tube::register(r);
    engines::register(r);
    markers::register(r);
    excavator::register(r);
    shaft::register(r);
    recipes::register(r);
}

/// `a / b`, 0 for nothing out of nothing.
pub fn ratio(a: u32, b: u32) -> f32 {
    if b == 0 { 0.0 } else { a as f32 / b as f32 }
}

/// The energy of the machine whose menu is open: a bar and the numbers.
pub fn power_bar(ui: &mut Ui) {
    ui.progress_bind("cogwork:power", S::default().size(80.0, 8.0), |c| {
        c.menu_component(&ENERGY).map_or(0.0, |e| {
            if e.capacity > 0 {
                e.stored as f32 / e.capacity as f32
            } else {
                0.0
            }
        })
    });
    ui.text_bind(|c, out| {
        let e = c.menu_component(&ENERGY).unwrap_or_default();
        out.set(
            "cogwork.energy",
            &[Arg::Int(e.stored), Arg::Int(e.capacity)],
        );
    });
}
