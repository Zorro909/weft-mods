//! Cogwork's film for the showcase mod, built only with the `showcase` feature:
//! `weft --showcase cogwork:showcase`, or `tools/showcase.sh cogwork:showcase`. A site on flat
//! meadow: excavators at work and one putting up its frame, their output sorted by tubes, an
//! assembly line of two benches started by a lever, a leveler set out with survey markers and
//! the tuning wrench, and an excavator finishing its job. The camera mostly looks north, at the
//! machines' working side.

mod crafting;
mod excavators;
mod finishing;
mod leveling;
mod parts;

use showcase_api::{Ease, Film, Look, Set, crane, dolly, film, flyby, orbit};
use weft_sdk::*;

use self::excavators::{AREA_Z, BUILDER, DEEP, DEEP_DEPTH, SHALLOW, SORTER};
use crate::lib::{BlockState, Pos};

pub fn register(r: &mut Registry) {
    // Block entities only exist once the tick they were placed in is over: everything is
    // stocked and started on cues.
    film(r, "showcase", showcase)
        .on_cue("start", |set| {
            excavators::start(set);
            crafting::start(set);
            finishing::start(set);
        })
        .on_cue("build", excavators::power_builder)
        .on_cue("switch", crafting::switch_on)
        .on_cue("survey", leveling::survey)
        .on_cue("leveler", leveling::place_leveler)
        .on_cue("wrench", leveling::turn_engine)
        .on_cue("finish", finishing::power);
    finishing::register(r);
}

fn showcase(set: &Set) -> Film {
    // Flat ground as far as the camera comes near, the landscape around left as it is.
    set.level(parts::area(Pos::new(-30, 0, -16), Pos::new(42, 20, 30)));
    excavators::build(set);
    crafting::build(set);
    leveling::build(set);
    finishing::build(set);
    let film = Film::new().cue("start");
    let film = opening(film);
    let film = excavating(film);
    let film = sorting(film);
    let film = autocrafting(film);
    let film = surveying(film);
    let film = finishing(film);
    closing(film)
}

/// Where in an excavator's area the camera looks: its middle, `depth` down.
fn area(at: Pos, depth: f64) -> [f64; 3] {
    [at.x as f64 + 0.5, -depth, AREA_Z]
}

/// In from over the meadow to the whole site.
fn opening(film: Film) -> Film {
    let path = [[-46.0, 34.0, 60.0], [-22.0, 22.0, 44.0], [-4.0, 16.0, 34.0]];
    film.title("mod.cogwork.name", 4.0)
        .shot(flyby(&path).look_at([0.0, 0.0, 2.0]).secs(9.0))
}

/// The builder putting up its frame, across to the two digging, and in through the open side
/// of the deeper one's frame, under its gantry, down into the pit.
fn excavating(film: Film) -> Film {
    let frame = [BUILDER.x as f64 + 0.5, 2.0, AREA_Z];
    let (shallow, deep) = (area(SHALLOW, 2.0), area(DEEP, DEEP_DEPTH as f64));
    let across = [[-10.0, 12.0, 18.0], [0.0, 11.0, 16.0], [8.0, 9.0, 12.0]];
    // Over the frame's bottom beam at x 11, below its top at 4, then down past the rim.
    let dive = [
        [8.0, 9.0, 12.0],
        [9.0, 5.0, 5.0],
        [10.0, 3.0, AREA_Z],
        [13.2, 2.0, AREA_Z],
        [14.5, -2.5, AREA_Z],
        [15.0, -5.0, AREA_Z],
    ];
    film.cue("build")
        .title("cogwork.film.frame", 5.0)
        .shot(
            orbit(frame, 15.0)
                .arc(150.0, 100.0)
                .height(9.0, 6.0)
                .look_at(frame)
                .secs(16.0)
                .blend(2.0),
        )
        .title("cogwork.film.dig", 5.0)
        .shot(
            flyby(&across)
                .look(Look::Pan(shallow, deep))
                .secs(8.0)
                .blend(1.5),
        )
        .shot(flyby(&dive).look_at(deep).secs(8.0).blend(1.0))
}

/// Up out of the pit and out past the excavator's front, along the tubes to the sorting tube,
/// and a look into its menu.
fn sorting(film: Film) -> Film {
    let sorter: [f64; 3] = SORTER.center().into();
    let deep = area(DEEP, DEEP_DEPTH as f64);
    let first_tube = [16.5, 0.5, 6.5];
    // Out through the frame's south side west of the excavator's engines.
    let climb = [
        [15.0, -5.0, AREA_Z],
        [13.5, 0.5, 0.5],
        [13.0, 2.6, 4.5],
        [17.5, 2.5, 10.5],
    ];
    film.shot(flyby(&climb).look(Look::Pan(deep, first_tube)).secs(5.0))
        .title("cogwork.film.tubes", 5.0)
        .shot(
            dolly([17.5, 2.5, 10.5], [-3.0, 2.5, 10.5])
                .look(Look::Pan(first_tube, sorter))
                .secs(12.0),
        )
        .title("cogwork.film.sorting", 5.0)
        .shot(
            orbit(sorter, 4.5)
                .arc(50.0, 100.0)
                .height(2.5, 2.0)
                .ease(Ease::In)
                .secs(5.0)
                .blend(1.5),
        )
        .menu("cogwork:sorting_tube", SORTER.into(), 4.5)
        .shot(
            orbit(sorter, 4.5)
                .arc(100.0, 150.0)
                .height(2.0, 1.5)
                .ease(Ease::Out)
                .secs(5.0),
        )
}

/// Round the end of the wall to the lever, which starts the row; along the row while it gets
/// going, then down to the second bench and into its menu.
fn autocrafting(film: Film) -> Film {
    let lever: [f64; 3] = crafting::LEVER.center().into();
    let bench: [f64; 3] = crafting::STONE_COGS.center().into();
    let at_lever = [-12.8, 2.2, 18.9];
    let to_lever = [
        [-9.4, 2.0, 9.75],
        [-14.5, 3.0, 13.0],
        [-15.2, 2.5, 17.5],
        at_lever,
    ];
    let in_front = [bench[0], 2.6, 20.5];
    film.shot(flyby(&to_lever).look_at(lever).secs(6.0).blend(1.5))
        .title("cogwork.film.lever", 3.5)
        .wait(1.0)
        .cue("switch")
        .wait(2.5)
        .title("cogwork.film.bench", 6.0)
        .shot(
            dolly(at_lever, in_front)
                .look(Look::Pan(lever, bench))
                .secs(13.0),
        )
        .shot(crane(bench, 4.0, 90.0, [2.1, 1.0]).secs(4.0).blend(0.5))
        .menu("cogwork:assembly_bench", crafting::STONE_COGS.into(), 5.0)
        .wait(5.0)
}

/// East from the benches to the leveling site: the corner marker shows the rectangle, the
/// leveler goes down and the wrench turns its engine to it; round the hill while it is leveled.
/// Halfway round, the excavator further east is started: the floor of the world is too deep
/// to be loaded, so it waits out its ten seconds for it and is done as the camera gets there.
fn surveying(film: Film) -> Film {
    let bench: [f64; 3] = crafting::STONE_COGS.center().into();
    let site = [13.0, 0.0, 19.5];
    let over = [[-1.5, 2.0, 20.5], [3.5, 3.5, 25.5], [9.0, 4.0, 29.0]];
    film.shot(
        flyby(&over)
            .look(Look::Pan(bench, site))
            .secs(5.0)
            .blend(1.0),
    )
    .title("cogwork.film.survey", 3.0)
    .cue("survey")
    .wait(3.0)
    .cue("leveler")
    .title("cogwork.film.wrench", 2.5)
    .wait(1.5)
    .cue("wrench")
    .wait(1.0)
    .title("cogwork.film.level", 5.0)
    .shot(
        orbit(leveling::MIDDLE, 11.9)
            .arc(118.0, 84.0)
            .height(3.5, 4.75)
            .ease(Ease::In)
            .secs(6.0)
            .blend(1.0),
    )
    .cue("finish")
    .shot(
        orbit(leveling::MIDDLE, 11.9)
            .arc(84.0, 50.0)
            .height(4.75, 6.0)
            .ease(Ease::Out)
            .secs(6.0),
    )
}

/// On east to the excavator at the end of its job, which finishes as the camera gets there, and
/// its screen says it is done.
fn finishing(film: Film) -> Film {
    let toward = [[22.2, 6.5, 27.6], [27.0, 7.0, 21.0], [31.0, 6.5, 15.0]];
    let pit_edge = [32.5, 0.5, 0.0];
    film.shot(
        flyby(&toward)
            .look(Look::Pan(leveling::MIDDLE, pit_edge))
            .secs(5.0),
    )
    .title("cogwork.film.finish", 5.0)
    .wait_for(finishing::FINISHED, 12.0)
    .wait(1.5)
    .menu("cogwork:excavator", finishing::FINISHER.into(), 4.0)
    .wait(4.0)
}

/// Up and away over the whole site and the lake behind.
fn closing(film: Film) -> Film {
    let path = [[31.0, 6.5, 15.0], [22.0, 20.0, 32.0], [6.0, 42.0, 50.0]];
    film.title("cogwork.film.end", 5.0)
        .shot(flyby(&path).look_at([8.0, 0.0, 6.0]).secs(11.0).blend(1.5))
}

// The showcase API has a `Pos` and `BlockState` of its own, laid out alike.

impl From<Pos> for showcase_api::Pos {
    fn from(p: Pos) -> showcase_api::Pos {
        showcase_api::Pos::new(p.x, p.y, p.z)
    }
}

impl From<showcase_api::Pos> for Pos {
    fn from(p: showcase_api::Pos) -> Pos {
        Pos::new(p.x, p.y, p.z)
    }
}

impl From<BlockState> for showcase_api::BlockState {
    fn from(state: BlockState) -> showcase_api::BlockState {
        state.id().into()
    }
}
