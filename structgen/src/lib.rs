//! Hamlets, villages and towns in base's overworld, laid out afresh from the land and built of
//! the buildings in `assets/structures/`.

mod buildings;
#[cfg(feature = "showcase")]
mod films;
mod settlement;
/// Public for `examples/`, which write the buildings' files with it.
pub mod structure_file;
mod styles;
mod turn;
mod villages;

use weft_sdk::*;

#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

fn init(r: &mut Registry) {
    buildings::register(r);
    styles::register(r);
    villages::register(r);
    #[cfg(feature = "showcase")]
    films::register(r);
}
