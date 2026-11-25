//! Cogwork's extended showcase film (`cogwork:showcase`), built only with the `showcase` feature:
//! `tools/showcase.sh cogwork:showcase -- --view 32`.
//!
//! Covers every block, liquid, multiblock and machine screen in progression order from Tier 0 to
//! Tier 5 across two parallel terraced bands anchored at `SITE` in seed 42:
//! - Part 1 (South Valley, S1–S4): timber camp, iron yard, steel & steam foundry, core drill,
//!   Steel Excavator and closed-loop flotation.
//! - Part 2 (North Campus, S5–S11): lake jetty & cuboid loader, oil seep, refinery column &
//!   asphalt, hall diesel lift & oil engine, assembly benches, plastics & acid annex, sorting &
//!   Bit Magazine & deep Gantry Excavator, leveler & seismic thumper, composite-frame Gantry
//!   Excavator, and ultra cuboid loader & cuboid lens finale.

mod camera;
mod crafting;
mod engine_bay;
mod flotation_yard;
mod foundry;
mod ground;
mod hall;
mod iron_yard;
mod jetty;
mod kit;
mod leveling;
mod loading;
mod parts;
mod plastics_bay;
mod quarry;
mod refinery_yard;
mod seep;
mod signals;
mod site;
mod timber_camp;

use showcase_api::{Film, Set, film};
use weft_sdk::Registry;

use self::ground::Pad;
use self::parts::{GRASS, GRAVEL};
use self::site::{PAD_P_TOP, S1_TOP, VALLEY_TOP, at_site, at_works, from_site, on_site, works};

/// The terraces in blocks from the works: four compact South Valley pads stepping down the ridge
/// (S1–S4), Pad R for the refinery, Pad 0 for the hall, Pad P stepped up on the slope south of
/// the hall for plastics and acid, Pad 1 for the deep digger, and Pad 2 for the seismic thumper
/// and composite-frame builder.
const PADS: [Pad; 9] = [
    Pad {
        west: -35,
        north: 25,
        east: -13,
        south: 38,
        top: S1_TOP,
        surface: &GRASS,
    },
    Pad {
        west: -11,
        north: 25,
        east: 15,
        south: 38,
        top: VALLEY_TOP,
        surface: &GRAVEL,
    },
    Pad {
        west: 16,
        north: 27,
        east: 42,
        south: 38,
        top: VALLEY_TOP,
        surface: &GRAVEL,
    },
    Pad {
        west: 43,
        north: 22,
        east: 69,
        south: 38,
        top: VALLEY_TOP,
        surface: &GRAVEL,
    },
    Pad {
        west: -20,
        north: -11,
        east: -6,
        south: 3,
        top: -1,
        surface: &GRAVEL,
    },
    Pad {
        west: -5,
        north: -10,
        east: 25,
        south: 9,
        top: -1,
        surface: &GRAVEL,
    },
    Pad {
        west: 13,
        north: 11,
        east: 27,
        south: 18,
        top: PAD_P_TOP,
        surface: &GRAVEL,
    },
    Pad {
        west: 26,
        north: -12,
        east: 41,
        south: 6,
        top: -1,
        surface: &GRAVEL,
    },
    Pad {
        west: 55,
        north: -10,
        east: 71,
        south: 8,
        top: 0,
        surface: &GRASS,
    },
];

pub fn register(r: &mut Registry) {
    // Block entities only exist once the tick they were placed in is over: everything is
    // stocked and started on cues.
    let build = |set: &Set| from_site(set.origin(), showcase(&on_site(set)));
    film(r, "showcase", build)
        .on_cue("start", at_site(start))
        .on_cue("noon_1", at_site(parts::noon))
        .on_cue("noon_2", at_site(parts::noon))
        .on_cue("noon_3", at_site(parts::noon))
        .on_cue("start_refinery", at_works(refinery_yard::start))
        .on_cue("bore", at_works(seep::bore))
        .on_cue("lift", at_works(lift))
        .on_cue("start_plastics", at_works(plastics_bay::start))
        .on_cue("prep_swap", at_works(quarry::prep_swap))
        .on_cue("survey", at_site(leveling::survey))
        .on_cue("leveler", at_site(leveling::place_leveler))
        .on_cue("wrench", at_site(leveling::turn_engines))
        .on_cue("frame", at_works(quarry::raise_frame))
        .on_cue("lens", at_works(loading::lens))
        .on_cue("lens_away", at_works(loading::lens_away))
        .on_cue("unpack", at_site(kit::unpack));
    signals::register(r);
}

/// Starts Part 1 and the northern water pump and pre-stocked campus machinery at t = 0.
fn start(set: &Set) {
    kit::pack(set);
    jetty::pump(set);
    let works = works(set);
    timber_camp::start(&works);
    iron_yard::start(&works);
    foundry::start(&works);
    flotation_yard::start(&works);
    seep::prefill_line(&works);
    engine_bay::stock_oil(&works);
    crafting::start(&works);
    quarry::start(&works);
    leveling::start_thumper(&works);
}

/// Seals the oil engine controller, lifts diesel into it, and watches for it to start running.
fn lift(set: &Set) {
    engine_bay::lift(set);
    signals::watch(set);
}

fn showcase(set: &Set) -> Film {
    let works = works(set);
    ground::terraces(&works, &PADS);

    // Part 1: South Valley (Tiers 0–3).
    timber_camp::build(&works);
    iron_yard::build(&works);
    foundry::build(&works);
    flotation_yard::build(&works);

    // Part 2: North Campus (Tiers 4–5). Ground lines go down before the hall's roof.
    seep::build(&works);
    jetty::build(set);
    hall::build(&works);
    refinery_yard::build(&works);
    engine_bay::build(&works);
    crafting::build(&works);
    plastics_bay::build(&works);
    quarry::build(&works);
    leveling::build(set);
    leveling::build_thumper(&works);
    loading::build(&works);

    let film = Film::new().cue("start");
    let film = timber_camp::shots(film);
    let film = iron_yard::shots(film);
    let film = foundry::shots(film);
    let film = flotation_yard::shots(film);
    let film = jetty::shots(film);
    let film = seep::shots(film);
    let film = refinery_yard::shots(film);
    let film = engine_bay::shots(film);
    let film = crafting::shots(film);
    let film = plastics_bay::shots(film);
    let film = quarry::sorting_and_digging_shots(film);
    let film = leveling::shots(film);
    let film = quarry::framing_shots(film);
    loading::shots(film)
}
