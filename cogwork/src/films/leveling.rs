//! Leveling on the knoll south of the digger's terrace, plus the seismic thumper set up beside
//! three survey markers on Pad 2's south-west apron. On the knoll, three survey markers in an L
//! mark a square astride the crest, a leveler goes down next to the corner marker, the tuning
//! wrench turns two ember engines into it, and its screen opens while it cuts and fills. East of
//! the knoll, the powered seismic thumper stands ready beside its own marked area, and its screen
//! opens before the camera continues to the composite-frame Gantry Excavator.

use showcase_api::{Film, Look, Set, dolly, orbit};
use weft_sdk::Block;
use weft_sdk::{Dir, Item, ItemStack, Output};

use super::camera::{between, on_orbit, speeds};
use super::ground::ground_top;
use super::parts::{GRASS, engine, fuel};
use super::site::{above, lowered, lowered_block};
use crate::leveler::LEVELER;
use crate::lib::{BlockState, Pos};
use crate::markers::SURVEY_MARKER;
use crate::seismic_thumper::{SEISMIC_THUMPER, SURVEY, Status, survey};

static DIRT: Block = Block::named("base:dirt");

/// The height the markers and the leveler stand at (site coordinates): matches the average ground
/// across the knoll square so cut and fill balance with no plinths or dirt cliffs.
const HEIGHT: i32 = -2;
/// The marked square, x 41 to 50 and z 9 to 18 (10 by 10): its corner marker, and the two along it.
const CORNER: Pos = Pos::new(50, HEIGHT, 9);
const MARKERS: [Pos; 3] = [CORNER, Pos::new(41, HEIGHT, 9), Pos::new(50, HEIGHT, 18)];
/// The leveler, east of the corner marker facing east, flanked north and south by its engines.
const LEVELER_AT: Pos = Pos::new(51, HEIGHT, 9);
const ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(51, HEIGHT, 8), Dir::South),
    (Pos::new(51, HEIGHT, 10), Dir::North),
];
/// The middle of the square, at the height it is made flat at (site coordinates).
pub const MIDDLE: [f64; 3] = [45.5, HEIGHT as f64, 13.5];

/// Seismic thumper on Pad 2's south-west apron (works coordinates), moved 2+ blocks clear of the
/// composite frame corner at (59, 1, 2), with its ember engines and three survey markers.
pub const THUMPER_AT: Pos = Pos::new(57, 1, 5);
const THUMPER_ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(58, 1, 5), Dir::West),
    (Pos::new(57, 1, 4), Dir::South),
];
const THUMPER_MARKERS: [Pos; 3] = [Pos::new(57, 1, 6), Pos::new(62, 1, 6), Pos::new(57, 1, 8)];

/// Elevated arc around the leveler so the entire marked square shows clearly behind it, plus an
/// eye-level western approach to the seismic thumper well clear of its north and east engines.
const LEVELER_CENTER: [f64; 3] = above(LEVELER_AT, 0.0);
const LEVELER_ARC: [f64; 2] = [-42.0, -12.0];
const LEVELER_OUT: f64 = 6.5;
const LEVELER_UP: [f64; 2] = [3.2, 2.8];
pub const THUMPER_LOOK: [f64; 3] = lowered(above(THUMPER_AT, 0.5));
pub const END_EYE: [f64; 3] = lowered([53.2, 3.0, 5.2]);

pub fn build(set: &Set) {
    for marker in MARKERS {
        earth_bed(set, marker);
        set.place(marker, BlockState::of(&SURVEY_MARKER));
    }
    earth_bed(set, LEVELER_AT);
    for (at, into_leveler) in ENGINES {
        earth_bed(set, at);
        // Pointing away from where the leveler goes, until the wrench turns it.
        engine(set, at, into_leveler.opposite());
    }
}

/// Builds the seismic thumper, its ember engines and its three survey markers on Pad 2 (works
/// coordinates).
pub fn build_thumper(set: &Set) {
    set.place(
        THUMPER_AT,
        SEISMIC_THUMPER.with(&[("facing", "west")]).into(),
    );
    for (at, toward) in THUMPER_ENGINES {
        engine(set, at, toward);
    }
    for marker in THUMPER_MARKERS {
        set.place(marker, BlockState::of(&SURVEY_MARKER));
    }
}

/// Fuels the seismic thumper's engines and starts its real survey of the marked area from t = 0 so
/// its ore bands and oil fields are populated when its screen opens.
pub fn start_thumper(set: &Set) {
    for (at, _) in THUMPER_ENGINES {
        fuel(set, at);
    }
    let world = set.world();
    crate::markers::survey(&world, set.at(THUMPER_MARKERS[0]));
    let pos = set.at(THUMPER_AT);
    let id = world.block_entity(pos);
    let marked = crate::markers::take_area(&world, pos);
    let area = survey::columns(marked, pos);
    if let Some(s) = world.component_mut(id, &SURVEY) {
        s.set_area(area, survey::layers(pos.y));
        s.set_status(Status::Thumping);
    }
}

/// Grades a natural grass/dirt shoulder under `at` (no cobblestone plinths) and clears any higher
/// blocks above `at`.
fn earth_bed(set: &Set, at: Pos) {
    for dz in -1..=1 {
        for dx in -1..=1 {
            let (x, z) = (at.x + dx, at.z + dz);
            let target_top = if dx == 0 && dz == 0 {
                at.y - 1
            } else {
                at.y - 2
            };
            let ground = ground_top(set, x, z);
            if ground < target_top {
                for y in ground..target_top {
                    set.place(Pos::new(x, y, z), BlockState::of(&DIRT));
                }
                set.place(Pos::new(x, target_top, z), BlockState::of(&GRASS));
            }
        }
    }
    let ground = ground_top(set, at.x, at.z);
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

/// Fills the leveler's stock with grass blocks (so the filled ground stays green) and turns both
/// engines into it.
pub fn turn_engines(set: &Set) {
    let world = set.world();
    if let Some(grass) = Item::lookup("base:grass") {
        let stock = world.inventory(set.at(LEVELER_AT)).group("input");
        stock.set(0, ItemStack::new(grass, 64));
        stock.set(1, ItemStack::new(grass, 64));
        stock.set(2, ItemStack::new(grass, 64));
    }
    for (at, into_leveler) in ENGINES {
        let at = set.at(at);
        world.prop_set(at, "facing", into_leveler.opposite().name());
        world.play_sound(&crate::sounds::WRENCH, at.center(), 1.0);
    }
}

fn leveler_part(angle: f64) -> f64 {
    (angle - LEVELER_ARC[0]) / (LEVELER_ARC[1] - LEVELER_ARC[0])
}

fn on_leveler(angle: f64) -> [f64; 3] {
    let up = between(LEVELER_UP, leveler_part(angle));
    on_orbit(LEVELER_CENTER, LEVELER_OUT, up, angle)
}

/// Section 10a–10b: out of the fade to an elevated view of the leveler and its full marked square,
/// opening its screen right while it cuts and fills, then gliding straight to the seismic thumper
/// from the west (clear of its engines and without staring at an empty pad).
pub fn shots(film: Film) -> Film {
    let entry = [49.0, 4.5, 1.0];
    let round = |from: f64, to: f64, secs: f32| {
        let up = [from, to].map(|angle| between(LEVELER_UP, leveler_part(angle)));
        orbit(LEVELER_CENTER, LEVELER_OUT)
            .arc(from, to)
            .height(up[0], up[1])
            .ease(speeds(0.35, 0.35))
            .secs(secs)
    };
    let leveler_look = [48.2, HEIGHT as f64 + 0.4, 11.8];
    let to_thumper = lowered([52.5, 4.2, 8.2]);
    let at_thumper = lowered([53.2, 2.8, 5.8]);

    film.fade_in(1.5)
        .shot(
            dolly(entry, on_leveler(LEVELER_ARC[0]))
                .look(Look::Pan(MIDDLE, leveler_look))
                .ease(speeds(1.5, 0.35))
                .secs(4.5),
        )
        .title("cogwork.film.survey", 3.5)
        .cue("survey")
        .shot(round(LEVELER_ARC[0], -38.0, 3.0).look_at(leveler_look))
        .shot(round(-38.0, -34.0, 2.5).look_at(leveler_look))
        .cue("leveler")
        .title("cogwork.film.wrench", 3.2)
        .shot(round(-34.0, -30.0, 3.5).look_at(leveler_look))
        .cue("wrench")
        .title("cogwork.film.level", 2.8)
        .shot(round(-30.0, -24.0, 3.0).look_at(leveler_look))
        .menu("cogwork:leveler", LEVELER_AT, 3.5)
        .shot(round(-24.0, LEVELER_ARC[1], 4.5).look_at(leveler_look))
        .shot(
            dolly(on_leveler(LEVELER_ARC[1]), to_thumper)
                .look(Look::Pan(leveler_look, THUMPER_LOOK))
                .ease(speeds(0.35, 0.8))
                .secs(4.5),
        )
        .title("cogwork.film.t5_thumper", 3.0)
        .shot(
            dolly(to_thumper, at_thumper)
                .look_at(THUMPER_LOOK)
                .ease(speeds(0.8, 0.5))
                .secs(3.2),
        )
        .menu("cogwork:seismic_thumper", lowered_block(THUMPER_AT), 3.5)
        .shot(
            dolly(at_thumper, END_EYE)
                .look_at(THUMPER_LOOK)
                .ease(speeds(0.5, 1.0))
                .secs(4.0),
        )
}
