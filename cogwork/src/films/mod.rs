//! Cogwork's film for the showcase mod, built only with the `showcase` feature:
//! `weft --showcase cogwork:showcase`, or `tools/showcase.sh cogwork:showcase`. A site on flat
//! meadow: excavators at work and one putting up its frame, their output sorted by tubes, and
//! an assembly line of two benches.

mod crafting;
mod excavators;
mod parts;

use showcase_api::{Film, Look, Set, crane, dolly, film, flyby, orbit};
use weft_sdk::*;

use self::excavators::{BUILDER, DEEP, DEEP_DEPTH, SHALLOW, SORTER};
use self::parts::middle;

pub fn register(r: &mut Registry) {
    // Block entities only exist once the tick they were placed in is over: everything is
    // stocked and started on cues.
    film(r, "showcase", showcase)
        .on_cue("start", |set| {
            excavators::start(set);
            crafting::start(set);
        })
        .on_cue("build", excavators::power_builder);
}

fn showcase(set: &Set) -> Film {
    // Flat ground as far as the camera comes near, the landscape around left as it is.
    set.level([-30, -16], [30, 30], 20);
    excavators::build(set);
    crafting::build(set);
    let film = Film::new().cue("start");
    let film = opening(film);
    let film = excavating(film);
    let film = sorting(film);
    let film = autocrafting(film);
    closing(film)
}

/// In from over the meadow to the whole site.
fn opening(film: Film) -> Film {
    film.title("mod.cogwork.name", 4.0).shot(
        flyby(&[[-46.0, 34.0, 52.0], [-22.0, 22.0, 36.0], [-4.0, 16.0, 26.0]])
            .look_at([0.0, 0.0, 0.0])
            .secs(9.0),
    )
}

/// The builder putting up its frame, then across to the two digging and down into the deeper
/// pit.
fn excavating(film: Film) -> Film {
    let frame = [BUILDER[0] as f64 + 0.5, 2.0, 2.5];
    let shallow = [SHALLOW[0] as f64 + 0.5, -2.0, 2.5];
    let deep = [DEEP[0] as f64 + 0.5, -(DEEP_DEPTH as f64), 2.5];
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
            flyby(&[[-10.0, 12.0, 16.0], [0.0, 11.0, 14.0], [8.0, 9.0, 10.0]])
                .look(Look::Pan(shallow, deep))
                .secs(8.0)
                .blend(1.5),
        )
        // Down through the open middle of the frame's top, past the gantry, into the pit.
        .shot(
            flyby(&[
                [12.0, 9.0, 6.0],
                [16.0, 7.0, 3.0],
                [15.5, 0.0, 4.0],
                [14.0, -4.0, 6.0],
            ])
            .look_at(deep)
            .secs(9.0)
            .blend(1.0),
        )
}

/// Up out of the pit to where the tubes begin, along them to the sorting tube.
fn sorting(film: Film) -> Film {
    let sorter = middle(SORTER);
    let deep = [DEEP[0] as f64 + 0.5, -(DEEP_DEPTH as f64), 2.5];
    let first_tube = [16.5, 0.5, -7.5];
    let climb = [
        [14.0, -4.0, 6.0],
        [15.0, 6.0, 1.0],
        [17.0, 3.5, -9.0],
        [17.0, 2.5, -11.5],
    ];
    film.shot(flyby(&climb).look(Look::Pan(deep, first_tube)).secs(5.0))
        .title("cogwork.film.tubes", 5.0)
        .shot(
            dolly([17.0, 2.5, -11.5], [-3.0, 2.5, -11.5])
                .look(Look::Pan(first_tube, [-5.5, 0.5, -7.5]))
                .secs(13.0),
        )
        .title("cogwork.film.sorting", 5.0)
        .shot(
            orbit(sorter, 4.5)
                .arc(-30.0, -150.0)
                .height(2.5, 1.5)
                .secs(10.0)
                .blend(1.5),
        )
}

/// Over the excavator to the row of benches, along it, then close on the second one at work.
fn autocrafting(film: Film) -> Film {
    let row = |x: f64| [x, 0.5, crafting::ROW_Z as f64 + 0.5];
    let stone_cogs = middle(crafting::STONE_COGS);
    let over = [[-8.0, 7.0, -7.0], [-12.0, 9.0, 6.0], [-14.0, 2.5, 19.5]];
    film.shot(flyby(&over).look_at(row(-12.5)).secs(6.0).blend(1.5))
        .title("cogwork.film.bench", 6.0)
        .shot(
            dolly([-14.0, 2.5, 19.5], [4.0, 2.5, 19.5])
                .look(Look::Pan(row(-12.5), row(3.5)))
                .secs(16.0),
        )
        .shot(
            crane(stone_cogs, 4.5, 90.0, [3.0, 1.2])
                .secs(7.0)
                .blend(1.5),
        )
}

/// Up and away over the whole site and the lake behind.
fn closing(film: Film) -> Film {
    film.title("cogwork.film.end", 5.0).shot(
        flyby(&[[4.0, 5.0, 22.0], [12.0, 20.0, 34.0], [2.0, 42.0, 46.0]])
            .look_at([0.0, 0.0, 2.0])
            .secs(11.0)
            .blend(1.5),
    )
}
