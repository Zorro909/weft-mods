//! The hamlets the journey starts and ends in: in from the north-east and round the first, into
//! a cottage there and out again; down from the mountains to the last and round it, rising.

use showcase_api::{Film, flyby, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The first hamlet, by its well.
const FIRST: [f64; 3] = [-2186.0, 80.0, 3246.0];
/// A crofter's cottage there, its door to the south; a bush and a birch stand before it.
const CROFT: Building = Building {
    corner: Pos::new(-2201, 80, 3237),
    size: [9, 7],
    turn: Turn::Back,
};
/// The last hamlet, round its well at the foot of the mountains.
pub const LAST: [f64; 3] = [-3638.0, 72.0, 238.0];
/// Over it, which the closing orbit looks at to keep the land beyond in view.
const OVER_LAST: [f64; 3] = [-3638.0, 84.0, 238.0];

/// Down from the north-east and round the hamlet to the south; past the birch to the crofter's
/// door and in. Round past the bed and the stove to the door, out, and up over the longhouses,
/// round to the north.
pub fn first(film: Film) -> Film {
    let in_from = [
        [-2125.0, 140.0, 3130.0],
        [-2138.0, 116.0, 3185.0],
        // Into the way the orbit then goes on.
        on_orbit(FIRST, 48.0, 21.5, -30.0),
        on_orbit(FIRST, 48.0, 21.0, -20.0),
    ];
    let inside = CROFT.at([4.5, 2.6, 3.0]);
    let (bed, stove) = (CROFT.at([2.0, 2.0, 4.6]), CROFT.at([7.0, 2.0, 4.6]));
    let door = CROFT.at([4.5, 2.6, -1.0]);
    let in_at_the_door = [
        (on_orbit(FIRST, 30.0, 10.0, 80.0), FIRST),
        ([-2189.0, 87.5, 3270.0], CROFT.at([4.5, 2.0, -2.0])),
        // East of the birch, west of the longhouse.
        ([-2193.5, 85.5, 3261.0], CROFT.at([4.5, 2.2, 1.0])),
        // Over the bush before the door, under its lintel three blocks up.
        (CROFT.at([4.5, 3.9, -5.5]), CROFT.at([4.5, 2.4, 3.0])),
        (CROFT.at([4.5, 3.6, -2.0]), CROFT.at([4.5, 2.4, 4.0])),
        (CROFT.at([4.5, 2.7, 0.3]), CROFT.at([4.5, 2.4, 5.0])),
        (CROFT.at([4.5, 2.6, 1.5]), CROFT.at([3.0, 2.2, 5.0])),
        (inside, bed),
    ];
    let round_the_room = [
        bed,
        CROFT.at([4.5, 2.2, 6.0]),
        stove,
        CROFT.at([7.5, 2.4, 1.5]),
        door,
    ];
    let out_and_up = [
        (inside, door),
        (CROFT.at([4.5, 2.6, 1.0]), CROFT.at([4.5, 2.4, -4.0])),
        (CROFT.at([4.5, 3.4, -1.5]), [-2194.5, 84.0, 3252.0]),
        ([-2196.0, 86.0, 3250.0], [-2190.0, 82.0, 3258.0]),
        ([-2191.0, 92.0, 3256.0], [-2178.0, 84.0, 3262.0]),
        ([-2180.0, 98.0, 3261.0], [-2166.0, 86.0, 3250.0]),
        ([-2168.0, 104.0, 3251.0], [-2168.0, 90.0, 3215.0]),
        ([-2170.0, 108.0, 3233.0], [-2178.0, 85.0, 3150.0]),
        // On the way the flight goes.
        ([-2172.0, 110.0, 3223.0], [-2178.0, 85.0, 3150.0]),
    ];
    film.title("mod.structgen.name", 4.0)
        .shot(
            flyby(&in_from)
                .look_at(FIRST)
                .secs(10.0)
                .ease(speeds(0.0, 8.0)),
        )
        .title("structgen.film.hamlet", 5.0)
        .shot(
            orbit(FIRST, 48.0)
                .arc(-20.0, 80.0)
                .radius(48.0, 30.0)
                .height(21.0, 10.0)
                .secs(9.0)
                .ease(speeds(8.0, 3.0)),
        )
        .title("structgen.film.inside", 5.0)
        .shot(glide(&in_at_the_door, 9.0).ease(speeds(3.0, 0.5)))
        .shot(turn(inside, &round_the_room, 8.0))
        .shot(glide(&out_and_up, 11.0).ease(speeds(1.0, 10.0)))
}

/// Round the last hamlet from the east by the north to the west, rising and widening, so
/// the mountains it stands under come into view behind it.
pub fn last(film: Film) -> Film {
    film.title("structgen.film.end", 7.0).shot(
        orbit(LAST, 32.0)
            .arc(360.0, 200.0)
            .radius(32.0, 90.0)
            .height(18.0, 55.0)
            .look_at(OVER_LAST)
            .ease(speeds(12.0, 3.0))
            .secs(16.0),
    )
}
