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

/// The block one face further.
pub fn step(pos: [i32; 3], face: u32) -> [i32; 3] {
    let o = Dir::from_face(face).offset();
    [pos[0] + o[0], pos[1] + o[1], pos[2] + o[2]]
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
