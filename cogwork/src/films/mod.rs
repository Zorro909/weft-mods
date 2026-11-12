//! Cogwork's film for the showcase mod, built only with the `showcase` feature:
//! `tools/showcase.sh cogwork:showcase -- --view 32`. A small oil works on the meadow by the
//! lake north of seed 42's spawn (`site`), built into the slope on terraces (`ground`): a pump
//! on the lake and one on an oil seep, which bores down to its field on camera; their lines of
//! liquid tubes into an open hall with the tanks, the oil engine and a row of assembly benches
//! the engine drives; an excavator on the engine's shaft and its sorting tube; a leveler
//! cutting a shoulder of the knoll flat; an excavator putting up its frame; and a cuboid
//! loader, with the lens that shows what it keeps. The camera comes in over the lake, goes
//! east through it all, and back to the loader by the hall. The set is for a fresh world: built
//! a second time in the same one, it lands on what the first left.

mod crafting;
mod engine_bay;
mod ground;
mod hall;
mod jetty;
mod leveling;
mod loading;
mod parts;
mod quarry;
mod seep;
mod signals;
mod site;

use showcase_api::{Ease, Film, Look, Set, dolly, film, flyby, orbit};
use weft_sdk::Registry;

use self::engine_bay::SHAFT_X;
use self::ground::Pad;
use self::parts::{GRASS, GRAVEL};
use self::site::{above, at_site, from_site, on_site};
use crate::lib::Pos;

/// The terraces: the hall's, the digging excavator's beside it, and the frame builder's beyond
/// the knoll.
const PADS: [Pad; 3] = [
    Pad {
        west: -5,
        north: -10,
        east: 25,
        south: 9,
        top: -1,
        surface: &GRAVEL,
    },
    Pad {
        west: 26,
        north: -12,
        east: 41,
        south: 5,
        top: -1,
        surface: &GRAVEL,
    },
    Pad {
        west: 57,
        north: -10,
        east: 71,
        south: 5,
        top: 0,
        surface: &GRASS,
    },
];

/// What the camera looks at, in blocks from the site: the two pumps, the hall's west end, its
/// oil tank, the tubes up into the oil engine and the engine, its shaft under the roof and
/// where that comes down behind the benches, the second bench, the sorting tube, the middles
/// of the two excavators' areas, the loader and the sky over it, the works as a whole, and the
/// shore below them.
const JETTY_PUMP: [f64; 3] = above(jetty::PUMP_AT, 0.0);
const SEEP: [f64; 3] = above(seep::PUMP_AT, 1.0);
const HALL_WEST: [f64; 3] = [hall::WEST as f64, 3.0, 0.0];
const OIL_TANK: [f64; 3] = above(engine_bay::OIL_TANK.lo, 0.7);
const OIL_RISE: [f64; 3] = above(engine_bay::OIL_RISE[1], 0.5);
const OIL_ENGINE: [f64; 3] = above(engine_bay::CHAMBER, 0.3);
const SHAFT: [f64; 3] = above(Pos::new(SHAFT_X, engine_bay::SHAFT_HEIGHT, 0), -0.5);
const SHAFT_FOOT: [f64; 3] = above(Pos::new(SHAFT_X, 1, crafting::SHAFT_Z), 0.0);
const BENCH: [f64; 3] = above(crafting::STONE_COGS, 0.1);
const SORTER: [f64; 3] = above(quarry::SORTER, 0.0);
const PIT: [f64; 3] = above(quarry::PIT, 0.5);
const FRAME: [f64; 3] = above(quarry::RAISED, 0.5);
const LOADER: [f64; 3] = above(loading::LOADER, 0.0);
const OVER_LOADER: [f64; 3] = above(loading::LOADER, 1.3);
const WORKS: [f64; 3] = [22.0, 0.0, -8.0];
const SHORE: [f64; 3] = [18.0, -2.0, -15.0];

pub fn register(r: &mut Registry) {
    // Block entities only exist once the tick they were placed in is over: everything is
    // stocked and started on cues.
    let build = |set: &Set| from_site(set.origin(), showcase(&on_site(set)));
    film(r, "showcase", build)
        .on_cue("start", at_site(start))
        .on_cue("bore", at_site(seep::bore))
        .on_cue("lift", at_site(lift))
        .on_cue("survey", at_site(leveling::survey))
        .on_cue("leveler", at_site(leveling::place_leveler))
        .on_cue("wrench", at_site(leveling::turn_engines))
        .on_cue("frame", at_site(quarry::raise_frame))
        .on_cue("lens", at_site(loading::lens))
        .on_cue("lens_away", at_site(loading::lens_away));
    signals::register(r);
}

/// What runs or waits ready from the film's first second.
fn start(set: &Set) {
    jetty::pump(set);
    engine_bay::stock_oil(set);
    crafting::start(set);
    quarry::start(set);
}

/// The timber liquid tube is started, and the film waits for the oil engine.
fn lift(set: &Set) {
    engine_bay::lift(set);
    signals::watch(set);
}

fn showcase(set: &Set) -> Film {
    ground::terraces(set, &PADS);
    // The lines over the ground first: under the hall's roof they would climb onto it.
    seep::build(set);
    jetty::build(set);
    hall::build(set);
    engine_bay::build(set);
    crafting::build(set);
    quarry::build(set);
    leveling::build(set);
    loading::build(set);
    let film = Film::new().cue("start");
    let film = opening(film);
    let film = water(film);
    let film = boring(film);
    let film = oil_line(film);
    let film = oil_engine(film);
    let film = benches(film);
    let film = sorting(film);
    let film = digging(film);
    let film = knoll(film);
    let film = framing(film);
    let film = loader(film);
    closing(film)
}

/// From `from` blocks a second to `to`: a shot going on from the one before at the speed that
/// one ended at, so the camera doesn't stop between them.
fn speeds(from: f32, to: f32) -> Ease {
    Ease::Speeds { from, to }
}

/// Where an orbit round `center` is at `angle` (0 toward +x, 90 toward +z), `radius` out and
/// `height` up: the shots before and after an orbit meet it there.
fn on_orbit(center: [f64; 3], radius: f64, height: f64, angle: f64) -> [f64; 3] {
    let (sin, cos) = angle.to_radians().sin_cos();
    [
        center[0] + cos * radius,
        center[1] + height,
        center[2] + sin * radius,
    ]
}

/// The value `part` of the way from `ends[0]` to `ends[1]`.
fn between(ends: [f64; 2], part: f64) -> f64 {
    ends[0] + (ends[1] - ends[0]) * part
}

/// The point halfway from `from` to `to`.
fn halfway(from: [f64; 3], to: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|axis| between([from[axis], to[axis]], 0.5))
}

/// Where the flight from the lake leaves the jetty behind, and where it is halfway up the
/// bank.
const PAST_JETTY: [f64; 3] = [-7.0, -3.5, -35.5];
const UP_BANK: [f64; 3] = [-17.0, 2.0, -28.0];

/// In from high over the lake toward the works.
fn opening(film: Film) -> Film {
    let path = [
        [38.0, 27.0, -100.0],
        [27.0, 12.0, -78.0],
        [17.0, -3.0, -61.0],
    ];
    film.title("mod.cogwork.name", 4.0).shot(
        flyby(&path)
            .look(Look::Pan(WORKS, JETTY_PUMP))
            .ease(speeds(5.0, 5.0))
            .secs(8.0),
    )
}

/// Low round the head of the jetty, where a pump stands over the lake.
fn water(film: Film) -> Film {
    let path = [
        [17.0, -3.0, -61.0],
        [7.5, -7.0, -52.5],
        [0.5, -6.5, -44.0],
        PAST_JETTY,
    ];
    film.title("cogwork.film.water", 6.0).shot(
        flyby(&path)
            .look_at(JETTY_PUMP)
            .ease(speeds(5.0, 4.5))
            .secs(7.0),
    )
}

/// The seep's orbit: from where to where round the pump (north, then west, to south), how far
/// out, and how high above it at its two ends.
const SEEP_ARC: [f64; 2] = [-90.0, -270.0];
const SEEP_OUT: f64 = 7.0;
const SEEP_UP: [f64; 2] = [3.0, 3.4];

/// Up the bank, turning from the jetty by way of the hall to the seep, and half round its
/// derrick: the pump is lit as the camera reaches it, and its screen shows it bore down.
fn boring(film: Film) -> Film {
    let up = |angle: f64| {
        let part = (angle - SEEP_ARC[0]) / (SEEP_ARC[1] - SEEP_ARC[0]);
        between(SEEP_UP, part)
    };
    let north = on_orbit(SEEP, SEEP_OUT, SEEP_UP[0], SEEP_ARC[0]);
    let to_seep = [UP_BANK, [-25.0, 6.5, -22.5], north];
    let round = |from: f64, to: f64, secs: f32| {
        orbit(SEEP, SEEP_OUT)
            .arc(from, to)
            .height(up(from), up(to))
            .ease(speeds(1.2, 1.2))
            .secs(secs)
    };
    film.shot(
        dolly(PAST_JETTY, UP_BANK)
            .look(Look::Pan(JETTY_PUMP, HALL_WEST))
            .ease(speeds(4.5, 4.0))
            .secs(3.0),
    )
    .shot(
        flyby(&to_seep)
            .look(Look::Pan(HALL_WEST, SEEP))
            .ease(speeds(4.0, 1.2))
            .secs(6.0),
    )
    .cue("bore")
    .title("cogwork.film.seep", 5.0)
    .shot(round(SEEP_ARC[0], -114.0, 2.5))
    .menu("cogwork:pump", seep::PUMP_AT, 9.0)
    .shot(round(-114.0, -178.0, 6.5))
    .title("cogwork.film.field", 5.0)
    .shot(round(-178.0, SEEP_ARC[1], 9.4))
}

/// Where the camera comes into the hall at its west end, and where it stops south of the oil
/// tank, the timber liquid tube beside the tank in view.
const IN_HALL: [f64; 3] = [1.0, 2.6, -0.5];
const BY_TANK: [f64; 3] = [3.8, 2.0, -1.2];

/// East beside the oil line, down the slope and in at the hall's west end, up to the tank.
fn oil_line(film: Film) -> Film {
    let south_of_seep = on_orbit(SEEP, SEEP_OUT, SEEP_UP[1], SEEP_ARC[1]);
    let along = [
        south_of_seep,
        [-22.0, 6.5, -6.5],
        [-13.0, 4.2, -1.0],
        [-5.0, 2.8, -0.8],
        IN_HALL,
    ];
    film.title("cogwork.film.liquid_tubes", 5.0)
        .shot(
            flyby(&along)
                .look(Look::Pan(SEEP, OIL_TANK))
                .ease(speeds(1.2, 0.8))
                .secs(10.0),
        )
        .shot(
            dolly(IN_HALL, BY_TANK)
                .look_at(OIL_TANK)
                .ease(speeds(0.8, 0.4))
                .secs(4.0),
        )
}

/// The engine's orbit: how far out, and where on it the camera meets it.
const ENGINE_OUT: f64 = 5.5;
const ENGINE_FROM: f64 = 130.0;

/// The timber liquid tube is started and lifts the oil into the engine; once that runs, round
/// its front and into its screen.
fn oil_engine(film: Film) -> Film {
    let at_engine = on_orbit(OIL_ENGINE, ENGINE_OUT, 0.8, ENGINE_FROM);
    let round = |from: f64, to: f64, up: [f64; 2], secs: f32| {
        orbit(OIL_ENGINE, ENGINE_OUT)
            .arc(from, to)
            .height(up[0], up[1])
            .ease(speeds(0.7, 0.7))
            .secs(secs)
    };
    film.cue("lift")
        .title("cogwork.film.lift", 5.0)
        .shot(
            dolly(BY_TANK, at_engine)
                .look(Look::Pan(OIL_TANK, OIL_RISE))
                .ease(speeds(0.4, 0.7))
                .secs(5.0),
        )
        .wait_for(signals::RUNNING, signals::WAIT_SECS)
        .title("cogwork.film.oil_engine", 5.0)
        .shot(round(ENGINE_FROM, 75.0, [0.8, 2.0], 7.5).blend(1.0))
        .menu("cogwork:oil_engine", engine_bay::CONTROLLER, 5.0)
        .shot(round(75.0, 40.0, [2.0, 2.4], 5.0))
}

/// How far from the second bench, toward where, and how high above it at first and at last,
/// the camera comes to it.
const BENCH_OUT: f64 = 3.6;
const BENCH_FROM: f64 = -130.0;
const BENCH_UP: [f64; 2] = [2.1, 1.0];

/// On east, the look following the shaft: from the engine up to where it crosses the hall
/// under the roof, down behind the benches, and along their row to the second bench and into
/// its screen.
fn benches(film: Film) -> Film {
    let from_engine = on_orbit(OIL_ENGINE, ENGINE_OUT, 2.4, 40.0);
    let [over_bench, at_bench] = BENCH_UP.map(|up| on_orbit(BENCH, BENCH_OUT, up, BENCH_FROM));
    let under_shaft = halfway(from_engine, over_bench);
    film.title("cogwork.film.shaft", 5.0)
        .shot(
            dolly(from_engine, under_shaft)
                .look(Look::Pan(OIL_ENGINE, SHAFT))
                .ease(speeds(0.7, 0.5))
                .secs(3.0),
        )
        .shot(
            dolly(under_shaft, over_bench)
                .look(Look::Pan(SHAFT, SHAFT_FOOT))
                .ease(speeds(0.5, 0.3))
                .secs(3.0),
        )
        .title("cogwork.film.bench", 5.0)
        .shot(
            dolly(over_bench, at_bench)
                .look(Look::Pan(SHAFT_FOOT, BENCH))
                .ease(speeds(0.3, 0.25))
                .secs(4.0),
        )
        .menu("cogwork:assembly_bench", crafting::STONE_COGS, 5.0)
        .wait(5.0)
}

/// The sorting tube's orbit: how far out and how high above it, so the joint shows over its
/// chests, and where on it the camera meets and leaves it.
const SORTER_OUT: f64 = 3.0;
const SORTER_UP: f64 = 3.0;
const SORTER_ARC: [f64; 2] = [175.0, 105.0];

/// Out at the hall's east end to the sorting tube just beyond, and round it to the south,
/// hardly moving while its screen is open.
fn sorting(film: Film) -> Film {
    let at_bench = on_orbit(BENCH, BENCH_OUT, BENCH_UP[1], BENCH_FROM);
    let at_sorter = on_orbit(SORTER, SORTER_OUT, SORTER_UP, SORTER_ARC[0]);
    let out = [at_bench, [17.5, 2.4, 0.6], [20.0, 2.6, 1.2], at_sorter];
    let round = |from: f64, to: f64, ease: Ease, secs: f32| {
        orbit(SORTER, SORTER_OUT)
            .arc(from, to)
            .height(SORTER_UP, SORTER_UP)
            .ease(ease)
            .secs(secs)
    };
    film.shot(
        flyby(&out)
            .look(Look::Pan(BENCH, SORTER))
            .ease(speeds(0.25, 0.6))
            .secs(5.0),
    )
    .title("cogwork.film.sorting", 5.0)
    .shot(round(SORTER_ARC[0], 150.0, speeds(0.6, 0.15), 2.5))
    .menu("cogwork:sorting_tube", quarry::SORTER, 4.0)
    .shot(round(150.0, 140.0, speeds(0.15, 0.15), 4.0))
    .shot(round(140.0, SORTER_ARC[1], speeds(0.15, 1.0), 2.5))
}

/// The pit's orbit: how far out, how high above its floor's middle, and from where to where:
/// south of it, then west, to north-west, from where the knoll lies straight on.
const PIT_OUT: f64 = 8.0;
const PIT_UP: f64 = 13.0;
const PIT_ARC: [f64; 2] = [108.0, 215.0];

/// Up over the digging excavator and round the west of its pit, looking down into it.
fn digging(film: Film) -> Film {
    let at_sorter = on_orbit(SORTER, SORTER_OUT, SORTER_UP, SORTER_ARC[1]);
    let over_pit = on_orbit(PIT, PIT_OUT, PIT_UP, PIT_ARC[0]);
    film.shot(
        dolly(at_sorter, over_pit)
            .look(Look::Pan(SORTER, PIT))
            .ease(speeds(1.0, 1.5))
            .secs(4.5),
    )
    .title("cogwork.film.dig", 5.0)
    .shot(
        orbit(PIT, PIT_OUT)
            .arc(PIT_ARC[0], PIT_ARC[1])
            .height(PIT_UP, PIT_UP)
            .ease(speeds(1.5, 1.5))
            .secs(10.0),
    )
}

/// The knoll's orbit, round the middle of the marked square: from where to where (west, then
/// north, to north by east), and how far out and how high the camera is at its two ends: it
/// comes in and down toward the leveler.
const KNOLL_ARC: [f64; 2] = [205.0, 300.0];
const KNOLL_OUT: [f64; 2] = [9.5, 9.0];
const KNOLL_UP: [f64; 2] = [6.5, 4.5];

/// How far along the knoll's orbit `angle` is, from 0 to 1.
fn knoll_part(angle: f64) -> f64 {
    (angle - KNOLL_ARC[0]) / (KNOLL_ARC[1] - KNOLL_ARC[0])
}

/// Where the knoll's orbit is at `angle`.
fn on_knoll(angle: f64) -> [f64; 3] {
    let part = knoll_part(angle);
    let (out, up) = (between(KNOLL_OUT, part), between(KNOLL_UP, part));
    on_orbit(leveling::MIDDLE, out, up, angle)
}

/// Over the pit to the knoll and round its north: the corner marker shows the square, the
/// leveler goes down, the wrench turns its engines to it, and it cuts and fills while the
/// camera comes round to it.
fn knoll(film: Film) -> Film {
    let over_pit = on_orbit(PIT, PIT_OUT, PIT_UP, PIT_ARC[1]);
    let round = |from: f64, to: f64, secs: f32| {
        let at_ends = |of: [f64; 2]| [from, to].map(|angle| between(of, knoll_part(angle)));
        let (out, up) = (at_ends(KNOLL_OUT), at_ends(KNOLL_UP));
        orbit(leveling::MIDDLE, out[0])
            .arc(from, to)
            .radius(out[0], out[1])
            .height(up[0], up[1])
            .ease(speeds(0.75, 0.75))
            .secs(secs)
    };
    film.shot(
        dolly(over_pit, on_knoll(KNOLL_ARC[0]))
            .look(Look::Pan(PIT, leveling::MIDDLE))
            .ease(speeds(1.5, 0.75))
            .secs(5.0),
    )
    .title("cogwork.film.survey", 4.0)
    .cue("survey")
    .shot(round(KNOLL_ARC[0], 218.0, 3.0))
    .shot(round(218.0, 231.0, 3.0))
    .cue("leveler")
    .title("cogwork.film.wrench", 4.5)
    .shot(round(231.0, 244.0, 3.0))
    .cue("wrench")
    .shot(round(244.0, 250.0, 1.5))
    .title("cogwork.film.level", 5.0)
    .shot(round(250.0, KNOLL_ARC[1], 11.5))
}

/// The builder's orbit: how far out, how high above its area at its two ends, and from where
/// to where: north-west of it, then west, to south, where the machine itself stands.
const FRAME_OUT: f64 = 10.0;
const FRAME_UP: [f64; 2] = [8.0, 7.0];
const FRAME_ARC: [f64; 2] = [225.0, 95.0];

/// On east, turning from the leveler to the second excavator as it starts on its frame, and
/// round the west and south of it while the frame goes up.
fn framing(film: Film) -> Film {
    let at_frame = on_orbit(FRAME, FRAME_OUT, FRAME_UP[0], FRAME_ARC[0]);
    film.cue("frame")
        .shot(
            dolly(on_knoll(KNOLL_ARC[1]), at_frame)
                .look(Look::Pan(leveling::MIDDLE, FRAME))
                .ease(speeds(0.75, 1.6))
                .secs(4.0),
        )
        .title("cogwork.film.frame", 5.0)
        .shot(
            orbit(FRAME, FRAME_OUT)
                .arc(FRAME_ARC[0], FRAME_ARC[1])
                .height(FRAME_UP[0], FRAME_UP[1])
                .ease(speeds(1.6, 2.0))
                .secs(12.0),
        )
}

/// Where the camera turns west over the knoll, and where it comes down past the digger's
/// frame toward the loader.
const OVER_KNOLL: [f64; 3] = [52.0, 12.0, 4.0];
const PAST_DIGGER: [f64; 3] = [41.0, 9.0, -11.0];
/// Where the camera watches the loader from, north-east of it, the loader low in the picture
/// so that its screen opens above it: when it gets there, when the screen opens, and when
/// the screen closes.
const BY_LOADER: [[f64; 3]; 3] = [[31.6, 5.2, -12.2], [31.7, 5.4, -12.7], [31.9, 5.8, -13.6]];

/// Back west over the levelled knoll and the pit to the loader on its mast by the hall, with
/// the hall behind it, and into its screen.
fn loader(film: Film) -> Film {
    let from_frame = on_orbit(FRAME, FRAME_OUT, FRAME_UP[1], FRAME_ARC[1]);
    let down = [OVER_KNOLL, PAST_DIGGER, BY_LOADER[0]];
    film.shot(
        dolly(from_frame, OVER_KNOLL)
            .look(Look::Pan(FRAME, PIT))
            .ease(speeds(2.0, 5.0))
            .secs(3.0),
    )
    .shot(
        flyby(&down)
            .look(Look::Pan(PIT, OVER_LOADER))
            .ease(speeds(5.0, 0.2))
            .secs(5.5),
    )
    .title("cogwork.film.loader", 5.0)
    .shot(
        dolly(BY_LOADER[0], BY_LOADER[1])
            .look_at(OVER_LOADER)
            .ease(speeds(0.2, 0.2))
            .secs(1.5),
    )
    .menu("cogwork:ultra_cuboid_loader", loading::LOADER, 4.5)
    .shot(
        dolly(BY_LOADER[1], BY_LOADER[2])
            .look_at(OVER_LOADER)
            .ease(speeds(0.2, 0.3))
            .secs(4.5),
    )
}

/// Up and out over the lake, then east above it with the lens, which shows the cubes kept
/// loaded as a grid through the works; without it again, down to the shore east of the works,
/// to rest on a last look along it all.
fn closing(film: Film) -> Film {
    let over_lake = [37.0, 13.0, -28.0];
    let north_east = [68.0, 27.0, -34.0];
    let with_lens = [over_lake, [50.0, 25.0, -42.0], north_east];
    let to_shore = [north_east, [66.0, 15.0, -27.0], [54.0, 8.5, -22.0]];
    film.shot(
        dolly(BY_LOADER[2], over_lake)
            .look(Look::Pan(OVER_LOADER, LOADER))
            .ease(speeds(0.3, 5.0))
            .secs(4.0),
    )
    .cue("lens")
    .title("cogwork.film.lens", 5.0)
    .shot(
        flyby(&with_lens)
            .look(Look::Pan(LOADER, WORKS))
            .ease(speeds(5.0, 4.5))
            .secs(7.5),
    )
    .cue("lens_away")
    .title("cogwork.film.end", 7.0)
    .shot(
        flyby(&to_shore)
            .look(Look::Pan(WORKS, SHORE))
            .ease(speeds(4.5, 0.3))
            .secs(6.5),
    )
    .wait(1.5)
}
