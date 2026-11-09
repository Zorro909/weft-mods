//! The town: round its market square, through the coaching inn's yard, over the church and
//! down to the fountain, into the town hall and out again.

use showcase_api::{Film, orbit};
use structgen_api::{Pos, Turn};

use super::building::Building;
use super::{glide, speeds, turn};

/// The market square, the fountain in its middle.
const SQUARE: [f64; 3] = [3500.0, 70.0, -1140.0];
/// The coaching inn north of the square: a yard behind its west side, out under an arch in
/// the east.
const INN: Building = Building {
    corner: Pos::new(3513, 73, -1173),
    size: [17, 17],
    turn: Turn::Left,
};
/// The town hall on the square's west side, its door to the fountain.
const HALL: Building = Building {
    corner: Pos::new(3479, 71, -1149),
    size: [21, 18],
    turn: Turn::Right,
};

/// Round the square from the south-east to the north; down the alley west of the inn and
/// through its yard, out under its arch; up over the church and down to the fountain, into the
/// town hall and round its stair tower to the long bench; round and back out, over the fountain
/// heading east.
pub fn town(film: Film) -> Film {
    let under_the_arch = INN.at([7.0, 2.7, 16.0]);
    let through_the_yard = [
        ([3495.82, 94.0, -1187.82], SQUARE),
        // On the way the orbit went.
        ([3499.75, 93.45, -1187.31], SQUARE),
        ([3503.0, 90.0, -1189.0], [3507.0, 76.0, -1170.0]),
        ([3507.0, 84.0, -1178.0], [3508.0, 76.0, -1160.0]),
        ([3507.5, 78.0, -1166.0], INN.at([3.5, 2.6, 4.0])),
        ([3509.0, 76.2, -1161.0], INN.at([3.5, 2.6, 8.0])),
        (INN.at([3.5, 2.6, 1.0]), INN.at([3.5, 2.6, 9.0])),
        (INN.at([3.5, 2.6, 6.5]), INN.at([5.0, 2.6, 13.0])),
        (INN.at([3.6, 2.6, 9.6]), INN.at([6.0, 2.6, 15.0])),
        (INN.at([5.5, 2.6, 11.5]), INN.at([7.0, 2.6, 18.0])),
        (INN.at([7.0, 2.6, 14.0]), INN.at([7.0, 2.6, 20.0])),
        (under_the_arch, INN.at([7.0, 2.8, 22.0])),
    ];
    let outside_the_hall = HALL.at([10.5, 3.4, -3.0]);
    // Up from the arch as soon as it is passed: the ground east of the town is grass.
    let over_the_church = [
        (under_the_arch, INN.at([7.0, 2.8, 22.0])),
        (INN.at([7.0, 2.9, 18.0]), INN.at([7.0, 3.2, 22.0])),
        (INN.at([7.0, 5.4, 20.5]), [3548.0, 80.0, -1158.0]),
        ([3538.0, 86.0, -1158.0], [3545.0, 82.0, -1140.0]),
        ([3542.0, 96.0, -1146.0], [3525.0, 78.0, -1128.0]),
        ([3532.0, 101.0, -1132.0], [3505.0, 72.0, -1137.0]),
        ([3518.0, 96.0, -1130.0], HALL.at([10.5, 2.6, 0.0])),
        (HALL.at([10.5, 6.0, -12.0]), HALL.at([10.5, 2.6, 2.0])),
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
        (HALL.at([6.0, 2.8, 6.2]), HALL.at([11.0, 2.7, 5.2])),
        (HALL.at([9.0, 2.7, 5.5]), HALL.at([11.5, 2.6, 2.0])),
        (HALL.at([10.5, 2.6, 4.6]), HALL.at([10.5, 2.6, -2.0])),
        (HALL.at([10.5, 2.6, 3.5]), HALL.at([10.5, 2.6, -3.0])),
        (HALL.at([10.5, 2.6, 1.5]), HALL.at([10.5, 3.0, -8.0])),
        (outside_the_hall, [3510.0, 76.0, -1135.0]),
        ([3506.0, 82.0, -1134.0], [3530.0, 85.0, -1125.0]),
        // Into the way the flight then goes on.
        ([3509.5, 87.9, -1128.1], [3600.0, 110.0, -1124.0]),
        ([3514.0, 90.0, -1128.0], [3600.0, 110.0, -1124.0]),
    ];
    film.title("structgen.film.town", 6.0)
        .shot(
            orbit(SQUARE, 88.0)
                .arc(47.0, 265.0)
                .radius(88.0, 48.0)
                .height(50.0, 24.0)
                .ease(speeds(20.0, 6.0))
                .secs(13.0),
        )
        .title("structgen.film.inn", 5.0)
        .shot(glide(&through_the_yard, 12.0).ease(speeds(6.0, 5.0)))
        .shot(glide(&over_the_church, 12.0).ease(speeds(5.0, 1.5)))
        .title("structgen.film.hall", 5.0)
        .shot(glide(&round_the_tower, 11.0).ease(speeds(1.5, 0.5)))
        .shot(turn(
            at_the_bench,
            &[HALL.at([10.0, 2.3, 16.0]), bench, tower, back_round],
            7.0,
        ))
        .shot(glide(&out_and_up, 13.0).ease(speeds(1.0, 10.0)))
}
