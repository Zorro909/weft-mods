//! Cogwork: item tubes that show what they carry, engines that drive them, and an excavator
//! that digs out an area layer by layer. Everything goes through `weft_sdk` and `base_api`.

mod bench;
mod cogs;
mod engines;
mod excavator;
#[cfg(feature = "showcase")]
mod films;
mod ghost;
mod harvest;
mod leveler;
mod markers;
mod owned;
mod recipes;
mod shaft;
mod store;
mod tube;

use weft_sdk::*;

#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

pub static TAB: CreativeTab = CreativeTab::named("cogwork");

fn init(r: &mut Registry) {
    r.creative_tab(&TAB, &Item::named("iron_cog"));
    owned::register(r);
    cogs::register(r);
    tube::register(r);
    engines::register(r);
    markers::register(r);
    excavator::register(r);
    bench::register(r);
    ghost::register(r);
    leveler::register(r);
    shaft::register(r);
    recipes::register(r);
    #[cfg(feature = "showcase")]
    films::register(r);
}

/// `a / b`, 0 for nothing out of nothing.
pub fn ratio(a: u32, b: u32) -> f32 {
    if b == 0 { 0.0 } else { a as f32 / b as f32 }
}

/// Puts up to `rate` of the machine `id`'s stored energy into a job of `cost`, of which `spent`
/// is paid already; returns how much it put in.
pub fn draw(w: &World, id: u64, spent: &mut i64, cost: i64, rate: i64) -> i64 {
    let Some(e) = w.energy_mut(id) else {
        return 0;
    };
    let drawn = e.stored.min(rate).min((cost - *spent).max(0));
    e.stored -= drawn;
    *spent += drawn;
    drawn
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
