//! Cogwork: item tubes that show what they carry, engines that drive them, and machines for the
//! big jobs: an excavator, an assembly bench and a leveler. Everything goes through `weft_sdk`
//! and `base_api`.

mod bench;
mod cogs;
mod engines;
mod excavator;
#[cfg(feature = "showcase")]
mod films;
mod ghost;
mod harvest;
mod leveler;
/// The types the whole mod builds on: block positions, points in the world, block states, areas
/// on the ground and model boxes, in `src/lib/`.
mod lib {
    mod area;
    mod block_state;
    mod float_pos;
    mod model_box;
    mod pos;

    pub use area::Area;
    pub use block_state::BlockState;
    pub use float_pos::FloatPos;
    pub use model_box::ModelBox;
    pub use pos::Pos;
}
mod machine;
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

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&dir).expect("cogwork has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}
