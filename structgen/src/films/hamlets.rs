//! The hamlets the journey starts and ends in: in from the meadow over the first, into a
//! cottage there and out again; down from the mountains into the last, into a cottage, out and
//! up and away.

use showcase_api::{Film, flyby, orbit};
use structgen_api::Turn;

use super::building::Building;
use super::{glide, speeds, turn};

/// The first hamlet, round its well in a shallow valley.
const FIRST: [f64; 3] = [4168.0, 74.0, -205.0];
/// A crofter's cottage there, its door to the north.
const CROFT: Building = Building {
    corner: [4167, 75, -206],
    size: [9, 7],
    turn: Turn::None,
};
/// The last hamlet, at the foot of the mountains.
const LAST: [f64; 3] = [3772.0, 72.0, -2178.0];
/// Over it, which the closing orbit looks at to keep the land beyond in view.
const OVER_LAST: [f64; 3] = [3772.0, 84.0, -2178.0];
/// A cottage with a loft there, its door to the north, facing the well.
const DORMER: Building = Building {
    corner: [3761, 75, -2173],
    size: [11, 9],
    turn: Turn::None,
};

/// In from the north and round the hamlet to the west; between the well and a cottage to the
/// crofter's door and in. Round past the bed and the stove to the door, out, and up between the
/// cottage and the farm, heading south.
pub fn first(film: Film) -> Film {
    // Down from the north, turning into the way the orbit then goes on.
    let in_from = [
        [4130.0, 150.0, -345.0],
        [4165.0, 118.0, -285.0],
        [4158.66, 97.52, -257.42],
        // Where the orbit starts.
        [4150.02, 95.0, -249.51],
    ];
    let inside = CROFT.at([4.5, 2.6, 3.0]);
    let (bed, stove) = (CROFT.at([2.0, 2.0, 4.6]), CROFT.at([7.0, 2.0, 4.6]));
    let door = CROFT.at([4.5, 2.6, -1.0]);
    // The orbit ends west of the hamlet; a birch stands just north of the door, so the way in
    // comes from the west, south of it.
    let in_at_the_door = [
        ([4138.46, 84.0, -210.21], FIRST),
        ([4148.0, 81.0, -214.5], CROFT.at([4.5, 2.0, -2.0])),
        ([4161.0, 79.5, -214.5], CROFT.at([4.5, 2.2, 1.0])),
        // Over the tall grass before the door, under its lintel three blocks up.
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
        (CROFT.at([4.5, 3.4, -1.5]), CROFT.at([5.5, 3.0, -7.0])),
        ([4175.0, 82.0, -211.0], [4185.0, 80.0, -218.0]),
        ([4182.0, 88.0, -209.0], [4195.0, 82.0, -203.0]),
        ([4187.0, 96.0, -198.0], [4192.0, 85.0, -175.0]),
        ([4184.0, 104.0, -182.0], [4180.0, 95.0, -120.0]),
        // On the way the flight goes.
        ([4183.0, 106.6, -176.7], [4180.0, 95.0, -120.0]),
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
                .arc(248.0, 190.0)
                .radius(48.0, 30.0)
                .height(21.0, 10.0)
                .secs(9.0)
                .ease(speeds(8.0, 0.0)),
        )
        .title("structgen.film.inside", 5.0)
        .shot(glide(&in_at_the_door, 9.0).ease(speeds(0.0, 0.5)))
        .shot(turn(inside, &round_the_room, 8.0))
        .shot(glide(&out_and_up, 11.0).ease(speeds(1.0, 10.0)))
}

/// Down from the mountains, north of the well, to the cottage's door and in. Round past the
/// table, the stove and the ladder to the door, out, and up and away, looking back at the
/// hamlet; then round it, rising, away from the sinkhole to its north-east.
pub fn last(film: Film) -> Film {
    let inside = DORMER.at([5.5, 2.6, 3.5]);
    let table = DORMER.at([8.5, 2.2, 2.5]);
    let door = DORMER.at([5.5, 2.6, -1.0]);
    let in_at_the_door = [
        ([3800.0, 110.0, -2140.0], LAST),
        ([3797.9, 108.3, -2144.2], LAST),
        ([3792.0, 95.0, -2178.0], LAST),
        ([3782.0, 90.0, -2196.0], [3768.0, 76.0, -2180.0]),
        ([3768.0, 84.0, -2193.0], DORMER.at([5.5, 2.0, 0.0])),
        (DORMER.at([5.5, 4.5, -8.0]), DORMER.at([5.5, 2.4, 3.0])),
        (DORMER.at([5.5, 3.2, -2.5]), DORMER.at([5.5, 2.4, 6.0])),
        (DORMER.at([5.5, 2.6, 1.5]), DORMER.at([5.5, 2.4, 8.0])),
        (inside, DORMER.at([6.5, 2.3, 8.0])),
    ];
    let round_the_room = [
        DORMER.at([6.5, 2.3, 8.0]),
        DORMER.at([8.5, 2.2, 5.0]),
        table,
        DORMER.at([7.0, 2.4, -1.0]),
        door,
    ];
    let out_and_up = [
        (inside, door),
        (DORMER.at([5.5, 2.6, 1.0]), DORMER.at([5.5, 3.0, -6.0])),
        (DORMER.at([5.5, 3.2, -1.5]), DORMER.at([7.0, 4.0, -10.0])),
        (DORMER.at([5.5, 5.5, -5.0]), [3780.0, 78.0, -2195.0]),
        ([3764.0, 86.0, -2190.0], OVER_LAST),
        // Into the way the orbit then goes on.
        ([3758.05, 89.99, -2200.79], OVER_LAST),
        ([3759.32, 92.0, -2205.19], OVER_LAST),
    ];
    film.shot(glide(&in_at_the_door, 14.0).ease(speeds(15.0, 0.5)))
        .shot(turn(inside, &round_the_room, 6.0))
        .shot(glide(&out_and_up, 10.0).ease(speeds(1.0, 8.0)))
        .title("structgen.film.end", 7.0)
        .shot(
            orbit(LAST, 30.0)
                .arc(245.0, 345.0)
                .radius(30.0, 90.0)
                .height(20.0, 55.0)
                .look_at(OVER_LAST)
                .ease(speeds(8.0, 3.0))
                .secs(12.0),
        )
}
