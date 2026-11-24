//! Leveling on the knoll south of the digger's terrace, plus the seismic thumper set up beside
//! three survey markers on Pad 2's south-west apron. On the knoll, three survey markers in an L
//! mark a square astride the crest, a leveler goes down next to the corner marker, the tuning
//! wrench turns two ember engines into it, and its screen opens while it cuts and fills. East of
//! the knoll, the powered seismic thumper stands ready beside its own marked area, and its screen
//! opens before the camera continues to the composite-frame Gantry Excavator.

use showcase_api::{Film, Look, Set, dolly, flyby, orbit};
use weft_sdk::{Dir, Item, ItemStack, Output};

use super::camera::{between, on_orbit, speeds};
use super::ground::ground_top;
use super::parts::{COBBLESTONE, engine, fuel};
use super::quarry::{PIT_LEFT, PIT_LOOK, PIT_OUT, PIT_UP};
use super::site::{above, lowered, lowered_block};
use crate::leveler::LEVELER;
use crate::lib::{BlockState, Pos};
use crate::markers::SURVEY_MARKER;
use crate::seismic_thumper::SEISMIC_THUMPER;

/// The height the markers and the leveler stand at, in site coordinates.
const HEIGHT: i32 = 0;
/// The marked square, x 42 to 50 and z 9 to 17: its corner marker, and the two along it.
const CORNER: Pos = Pos::new(50, HEIGHT, 9);
const MARKERS: [Pos; 3] = [CORNER, Pos::new(42, HEIGHT, 9), Pos::new(50, HEIGHT, 17)];
/// The leveler, east of the corner marker, and its engines to the east and south.
const LEVELER_AT: Pos = Pos::new(51, HEIGHT, 9);
const ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(52, HEIGHT, 9), Dir::West),
    (Pos::new(51, HEIGHT, 10), Dir::North),
];
/// The middle of the square, at the height it is made flat at (site coordinates).
pub const MIDDLE: [f64; 3] = [46.5, HEIGHT as f64, 13.5];

/// Seismic thumper on Pad 2's south-west apron (in works coordinates), its ember engines, and the
/// three survey markers beside it.
pub const THUMPER_AT: Pos = Pos::new(58, 1, 4);
const THUMPER_ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(59, 1, 4), Dir::West),
    (Pos::new(58, 1, 3), Dir::South),
];
const THUMPER_MARKERS: [Pos; 3] = [Pos::new(58, 1, 5), Pos::new(63, 1, 5), Pos::new(58, 1, 8)];

/// Camera orbit around the knoll and close eye-height framing on the seismic thumper's west front.
const KNOLL_ARC: [f64; 2] = [205.0, 295.0];
const KNOLL_OUT: [f64; 2] = [6.6, 6.2];
const KNOLL_UP: [f64; 2] = [3.6, 3.0];
pub const THUMPER_LOOK: [f64; 3] = lowered(above(THUMPER_AT, 0.5));
pub const END_EYE: [f64; 3] = lowered([55.6, 2.6, 3.0]);

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

/// Fuels the seismic thumper's engines so its store is full when its screen opens.
pub fn start_thumper(set: &Set) {
    for (at, _) in THUMPER_ENGINES {
        fuel(set, at);
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

fn knoll_part(angle: f64) -> f64 {
    (angle - KNOLL_ARC[0]) / (KNOLL_ARC[1] - KNOLL_ARC[0])
}

fn on_knoll(angle: f64) -> [f64; 3] {
    let part = knoll_part(angle);
    let (out, up) = (between(KNOLL_OUT, part), between(KNOLL_UP, part));
    on_orbit(MIDDLE, out, up, angle)
}

/// Section 10a–10b: out of the fade to the knoll (leveling + leveler screen), then east past the
/// seismic thumper and its screen toward the composite-frame Gantry Excavator.
pub fn shots(film: Film) -> Film {
    let over_pit = on_orbit(PIT_LOOK, PIT_OUT, PIT_UP[0], PIT_LEFT);
    let round = |from: f64, to: f64, secs: f32| {
        let at_ends = |of: [f64; 2]| [from, to].map(|angle| between(of, knoll_part(angle)));
        let (out, up) = (at_ends(KNOLL_OUT), at_ends(KNOLL_UP));
        orbit(MIDDLE, out[0])
            .arc(from, to)
            .radius(out[0], out[1])
            .height(up[0], up[1])
            .ease(speeds(0.75, 0.75))
            .secs(secs)
    };
    // Smooth descent clear above the knoll's eastern shoulder down to eye height in front of the
    // seismic thumper on Pad 2.
    let leveler_look = [48.5, 0.2, 11.2];
    let at_thumper = lowered([55.4, 2.5, 4.8]);
    let to_thumper = [
        on_knoll(KNOLL_ARC[1]),
        lowered([51.8, 7.8, 4.6]),
        at_thumper,
    ];

    film.fade_in(1.5)
        .shot(
            dolly(over_pit, on_knoll(KNOLL_ARC[0]))
                .look(Look::Pan(PIT_LOOK, MIDDLE))
                .ease(speeds(1.5, 0.75))
                .secs(5.0),
        )
        .title("cogwork.film.survey", 4.0)
        .cue("survey")
        .shot(round(KNOLL_ARC[0], 218.0, 3.0))
        .shot(round(218.0, 231.0, 3.0))
        .cue("leveler")
        .title("cogwork.film.wrench", 4.0)
        .shot(round(231.0, 244.0, 3.0))
        .cue("wrench")
        .shot(round(244.0, 250.0, 1.5))
        .title("cogwork.film.level", 5.0)
        .shot(round(250.0, 272.0, 5.5).look(Look::Pan(MIDDLE, leveler_look)))
        .menu("cogwork:leveler", LEVELER_AT, 4.0)
        .shot(round(272.0, KNOLL_ARC[1], 6.0).look_at(leveler_look))
        .shot(
            flyby(&to_thumper)
                .look(Look::Pan(leveler_look, THUMPER_LOOK))
                .ease(speeds(0.75, 0.6))
                .secs(6.0),
        )
        .title("cogwork.film.t5_thumper", 5.0)
        .menu("cogwork:seismic_thumper", lowered_block(THUMPER_AT), 4.5)
        .shot(
            dolly(at_thumper, END_EYE)
                .look_at(THUMPER_LOOK)
                .ease(speeds(0.6, 1.0))
                .secs(5.5),
        )
}
