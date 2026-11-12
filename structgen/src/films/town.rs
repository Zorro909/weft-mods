//! The town: round its market square, in under the coaching inn's arch and through its yard
//! out onto the square, round to the town hall's door and in, and out again over the square.

use showcase_api::{Film, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The market square, the fountain in its middle.
pub const SQUARE: [f64; 3] = [-3556.0, 68.0, 1575.0];
/// The coaching inn on the square's north-west: in from the square through its front, a yard,
/// and an arch out the back.
const INN: Building = Building {
    corner: Pos::new(-3590, 70, 1553),
    size: [17, 17],
    turn: Turn::Back,
};
/// The town hall on the square's north side, its door to the square; a market stall stands
/// before it.
const HALL: Building = Building {
    corner: Pos::new(-3571, 71, 1546),
    size: [21, 18],
    turn: Turn::Back,
};

/// Round the square from the east by the north; down behind the inn, in under its arch and
/// through its yard out onto the square; round to the town hall, in and round its stair tower to
/// the long bench; round and back out over the square, and round to the north.
pub fn town(film: Film) -> Film {
    let under_the_arch = INN.at([7.0, 2.7, 16.0]);
    let through_the_yard = [
        (on_orbit(SQUARE, 48.0, 24.0, 250.0), SQUARE),
        ([-3578.0, 88.0, 1533.0], INN.at([7.0, 3.0, 22.0])),
        ([-3580.5, 82.0, 1540.0], under_the_arch),
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
    // South out of the inn and round to the east and the north, over the stall to the door.
    let round_to_the_hall = [
        (INN.at([3.5, 2.6, 1.0]), INN.at([3.5, 2.8, -6.0])),
        (INN.at([3.5, 3.0, -2.5]), [-3572.0, 75.0, 1580.0]),
        ([-3574.0, 75.5, 1578.0], [-3564.0, 77.0, 1583.0]),
        ([-3568.0, 77.0, 1583.0], [-3560.0, 77.0, 1580.0]),
        ([-3562.0, 77.5, 1582.0], HALL.at([10.5, 3.0, -3.0])),
        (HALL.at([10.5, 5.0, -9.0]), HALL.at([10.5, 2.6, 2.0])),
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
        (outside_the_hall, HALL.at([10.5, 3.0, -10.0])),
        // Over the stall, clear of the house east of it, then round to the west and the north.
        (HALL.at([10.5, 5.5, -9.0]), [-3556.0, 74.0, 1585.0]),
        ([-3561.0, 85.0, 1582.0], [-3568.0, 78.0, 1594.0]),
        ([-3567.0, 95.0, 1591.0], [-3585.0, 84.0, 1590.0]),
        ([-3580.0, 104.0, 1593.0], [-3595.0, 95.0, 1570.0]),
        ([-3592.0, 114.0, 1580.0], [-3596.0, 100.0, 1520.0]),
        ([-3596.0, 122.0, 1560.0], [-3600.0, 105.0, 1450.0]),
        // On the way the flight goes.
        ([-3597.0, 125.0, 1548.0], [-3600.0, 105.0, 1450.0]),
    ];
    film.title("structgen.film.town", 6.0)
        .shot(
            orbit(SQUARE, 88.0)
                .arc(360.0, 250.0)
                .radius(88.0, 48.0)
                .height(50.0, 24.0)
                .ease(speeds(20.0, 6.0))
                .secs(13.0),
        )
        .title("structgen.film.inn", 5.0)
        .shot(glide(&through_the_yard, 12.0).ease(speeds(6.0, 1.5)))
        .shot(glide(&round_to_the_hall, 10.0).ease(speeds(1.5, 1.5)))
        .title("structgen.film.hall", 5.0)
        .shot(glide(&round_the_tower, 11.0).ease(speeds(1.5, 0.5)))
        .shot(turn(
            at_the_bench,
            &[HALL.at([10.0, 2.3, 16.0]), bench, tower, back_round],
            7.0,
        ))
        .shot(glide(&out_and_up, 16.0).ease(speeds(1.0, 10.0)))
}
