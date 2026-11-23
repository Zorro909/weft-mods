//! The town, where the journey ends: down past its west side, in under the coaching inn's arch
//! and through its yard out onto the street, across into the town hall and round it, out again
//! and up over the square as the picture fades.

use showcase_api::{Film, flyby, orbit, screen};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The market square, the fountain in its middle.
pub const SQUARE: [f64; 3] = [-3702.5, 74.0, -2984.5];
/// The coaching inn on the west of the main street, south of the square: in from the fields
/// through an arch in its back, a yard, and its front onto the street.
const INN: Building = Building {
    corner: Pos::new(-3728, 72, -2967),
    size: [17, 17],
    turn: Turn::Right,
};
/// The town hall across the street from it, its door to the street.
const HALL: Building = Building {
    corner: Pos::new(-3696, 73, -2985),
    size: [21, 18],
    turn: Turn::Left,
};
/// Seconds the picture takes to fade to black at the end, the camera coming to rest meanwhile.
const FADE_OUT: f32 = 4.0;

/// Down past the town's west side over the lake shore, the square in view; behind the inn, in
/// under its arch and through its yard out onto the street; across to the town hall, in and
/// round its stair tower to the long bench; round and back out, up over the square, and round it
/// rising as the picture fades to black.
pub fn town(film: Film) -> Film {
    let round_the_west = [
        [-3795.0, 128.0, -3090.0],
        [-3808.0, 116.0, -3032.0],
        [-3803.0, 105.0, -2988.0],
        [-3786.0, 97.0, -2962.0],
        // On the way the glide goes.
        [-3762.0, 92.0, -2959.0],
    ];
    let under_the_arch = INN.at([7.0, 2.7, 16.0]);
    let through_the_yard = [
        ([-3762.0, 92.0, -2959.0], SQUARE),
        ([-3748.0, 86.0, -2960.0], INN.at([7.0, 3.0, 22.0])),
        ([-3738.0, 80.0, -2960.0], under_the_arch),
        (INN.at([7.0, 4.0, 22.0]), INN.at([7.0, 2.6, 14.0])),
        (INN.at([7.0, 2.8, 19.0]), INN.at([7.0, 2.6, 12.0])),
        (under_the_arch, INN.at([5.5, 2.6, 11.5])),
        (INN.at([7.0, 2.6, 14.0]), INN.at([3.6, 2.6, 9.6])),
        (INN.at([5.5, 2.6, 11.5]), INN.at([3.5, 2.6, 6.5])),
        (INN.at([3.6, 2.6, 9.6]), INN.at([3.5, 2.6, 2.0])),
        (INN.at([3.5, 2.6, 6.5]), INN.at([3.5, 2.6, -2.0])),
        (INN.at([3.5, 2.6, 1.0]), INN.at([3.5, 2.8, -6.0])),
    ];
    let outside_the_hall = HALL.at([10.5, 3.4, -3.0]);
    // Out of the inn onto the street, left along it and round into the hall's door.
    let across_to_the_hall = [
        (INN.at([3.5, 2.6, 1.0]), INN.at([3.5, 2.8, -6.0])),
        (INN.at([3.5, 3.0, -2.5]), [-3700.0, 76.0, -2968.0]),
        ([-3707.0, 76.0, -2966.5], [-3700.0, 76.0, -2972.0]),
        ([-3704.0, 76.0, -2971.0], HALL.at([10.5, 3.0, -1.0])),
        (HALL.at([10.5, 3.6, -5.0]), HALL.at([10.5, 2.6, 2.0])),
        (outside_the_hall, HALL.at([10.5, 2.6, 6.0])),
    ];
    let at_the_bench = HALL.at([4.5, 3.0, 12.0]);
    let (tower, bench) = (HALL.at([10.5, 2.4, 9.5]), HALL.at([17.0, 2.2, 14.0]));
    let back_round = HALL.at([4.2, 2.8, 8.0]);
    // The door opens onto a passage three blocks wide between the front wall and the tower:
    // along the middle of it, turning only once past the door.
    let round_the_tower = [
        (outside_the_hall, HALL.at([10.5, 2.6, 6.0])),
        (HALL.at([10.5, 2.6, 1.5]), HALL.at([10.0, 2.6, 8.0])),
        (HALL.at([10.5, 2.6, 3.5]), HALL.at([9.0, 2.6, 8.0])),
        (HALL.at([10.5, 2.6, 4.6]), HALL.at([7.5, 2.6, 7.0])),
        (HALL.at([9.0, 2.7, 5.5]), HALL.at([4.0, 2.7, 7.0])),
        (HALL.at([6.0, 2.8, 6.2]), HALL.at([3.5, 2.6, 12.0])),
        (HALL.at([4.0, 2.8, 9.0]), HALL.at([6.0, 2.4, 16.0])),
        (at_the_bench, HALL.at([10.0, 2.3, 16.0])),
    ];
    let out_and_up = [
        (at_the_bench, back_round),
        (HALL.at([4.0, 2.8, 9.0]), HALL.at([6.0, 2.8, 5.0])),
        (HALL.at([6.0, 2.8, 6.2]), HALL.at([11.5, 2.7, 3.5])),
        (HALL.at([9.0, 2.7, 5.5]), HALL.at([11.0, 2.6, 0.5])),
        (HALL.at([10.5, 2.6, 4.6]), HALL.at([10.5, 2.6, -2.0])),
        (HALL.at([10.5, 2.6, 3.5]), HALL.at([10.5, 2.6, -3.0])),
        (HALL.at([10.5, 2.6, 1.5]), HALL.at([10.5, 3.0, -8.0])),
        (outside_the_hall, HALL.at([10.5, 3.0, -10.0])),
        // Up over the street, turning north to the square, into the closing orbit.
        (HALL.at([10.5, 5.0, -5.0]), [-3712.0, 80.0, -2978.0]),
        ([-3702.0, 84.0, -2975.0], [-3709.0, 77.0, -2983.0]),
        ([-3702.3, 92.0, -2973.5], SQUARE),
        (on_orbit(SQUARE, 12.0, 26.0, 85.0), SQUARE),
        (on_orbit(SQUARE, 12.0, 26.0, 90.0), SQUARE),
    ];
    film.shot(
        flyby(&round_the_west)
            .look_at(SQUARE)
            .ease(speeds(14.0, 8.0))
            .secs(15.0),
    )
    .title("structgen.film.inn", 5.0)
    .shot(glide(&through_the_yard, 12.0).ease(speeds(8.0, 1.5)))
    .shot(glide(&across_to_the_hall, 8.0).ease(speeds(1.5, 1.5)))
    .title("structgen.film.hall", 5.0)
    .shot(glide(&round_the_tower, 11.0).ease(speeds(1.5, 0.5)))
    .shot(turn(
        at_the_bench,
        &[HALL.at([10.0, 2.3, 16.0]), bench, tower, back_round],
        7.0,
    ))
    .shot(glide(&out_and_up, 16.0).ease(speeds(1.0, 4.0)))
    .title("structgen.film.end", 7.0)
    .shot(
        orbit(SQUARE, 12.0)
            .arc(90.0, 300.0)
            .radius(12.0, 90.0)
            .height(26.0, 55.0)
            .ease(speeds(4.0, 1.5))
            .secs(16.0),
    )
    .screen(screen::BLACK, 1.0, FADE_OUT)
    .shot(
        orbit(SQUARE, 90.0)
            .arc(300.0, 302.0)
            .radius(90.0, 90.7)
            .height(55.0, 55.5)
            .ease(speeds(1.5, 0.0))
            .secs(FADE_OUT),
    )
    .wait(0.5)
}
