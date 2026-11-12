//! Leveling on the knoll between the two excavators, on the land as it lies: three survey
//! markers in an L, set on stone posts to one height since the ground under them is not, mark
//! a square on the knoll's north shoulder. The camera uses the corner marker, a leveler goes
//! down next to it on its own footing, and the tuning wrench turns the two ember engines
//! beside it, which pointed away, into it: it cuts the crest down and fills the low side up.

use showcase_api::Set;
use weft_sdk::{Dir, Item, ItemStack, Output};

use super::ground::ground_top;
use super::parts::{COBBLESTONE, engine, fuel};
use crate::leveler::LEVELER;
use crate::lib::{BlockState, Pos};
use crate::markers::SURVEY_MARKER;

/// The height the markers and the leveler stand at: the ground ends a block below it once the
/// leveler is done.
const HEIGHT: i32 = 4;
/// The marked square, x 44 to 52 and z -6 to 2: its corner marker, and the two along it.
const CORNER: Pos = Pos::new(52, HEIGHT, -6);
const MARKERS: [Pos; 3] = [CORNER, Pos::new(44, HEIGHT, -6), Pos::new(52, HEIGHT, 2)];
/// The leveler, east of the corner marker, and its engines to the east and south.
const LEVELER_AT: Pos = Pos::new(53, HEIGHT, -6);
const ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(54, HEIGHT, -6), Dir::West),
    (Pos::new(53, HEIGHT, -5), Dir::North),
];
/// The middle of the square, at the height it is made flat at.
pub const MIDDLE: [f64; 3] = [48.5, HEIGHT as f64, -1.5];

pub fn build(set: &Set) {
    for marker in MARKERS {
        post(set, marker);
        set.place(marker, BlockState::of(&SURVEY_MARKER));
    }
    post(set, LEVELER_AT);
    for (at, into_leveler) in ENGINES {
        post(set, at);
        // Pointing away from where the leveler goes, until the wrench turns it.
        engine(set, at, into_leveler.opposite());
    }
}

/// Makes a place to stand at `at`: a stone post up from lower ground, a notch in higher.
fn post(set: &Set, at: Pos) {
    let ground = ground_top(set, at.x, at.z);
    for y in ground + 1..at.y {
        set.place(at.with_y(y), BlockState::of(&COBBLESTONE));
    }
    for y in at.y..=ground + 1 {
        set.place(at.with_y(y), BlockState::AIR);
    }
}

/// The camera uses the corner marker, so the square shows, and the engines are lit.
pub fn survey(set: &Set) {
    crate::markers::survey(&set.world(), set.at(CORNER));
    for (at, _) in ENGINES {
        fuel(set, at);
    }
}

/// The leveler goes down next to the corner marker and takes the markers' square.
pub fn place_leveler(set: &Set) {
    set.place(LEVELER_AT, LEVELER.with(&[("facing", "east")]).into());
}

/// Fills the leveler's stock, and the wrench turns both engines into it.
pub fn turn_engines(set: &Set) {
    let world = set.world();
    if let Some(dirt) = Item::lookup("base:dirt") {
        let stock = world.inventory(set.at(LEVELER_AT)).group("input");
        stock.set(0, ItemStack::new(dirt, 64));
        stock.set(1, ItemStack::new(dirt, 64));
    }
    for (at, into_leveler) in ENGINES {
        let at = set.at(at);
        world.prop_set(at, "facing", into_leveler.opposite().name());
        world.play_sound(&crate::sounds::WRENCH, at.center(), 1.0);
    }
}
