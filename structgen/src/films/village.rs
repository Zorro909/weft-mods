//! The village: round and down onto its green, into its tavern, out across the green into its
//! church, then out past the well and away.

use showcase_api::{Film, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The village, round its well on the green.
pub const VILLAGE: [f64; 3] = [-3871.5, 74.0, -3212.5];
/// Its tavern on the east of the green, its porch to the west, a little up the slope.
const TAVERN: Building = Building {
    corner: Pos::new(-3855, 78, -3225),
    size: [11, 15],
    turn: Turn::Left,
};
/// Its church on the north-west of the green, its door to the east; the smithy's wall stands
/// three blocks before it, so the way in comes round its corner from the green.
const CHURCH: Building = Building {
    corner: Pos::new(-3898, 75, -3233),
    size: [11, 18],
    turn: Turn::Right,
};

/// Round the village from the west by the north, the east and the south, down onto the green
/// and across it into the tavern. Round the room to the door, out from under the porch, over
/// the well and round to the church door, in down the nave to the altar; round and out again,
/// up over the well and away to the south-east.
pub fn village(film: Film) -> Film {
    // Over the west cottage and down onto the green, north of the well.
    let to_the_porch = [
        (on_orbit(VILLAGE, 28.0, 14.0, 520.0), porch()),
        ([-3886.0, 85.0, -3214.0], porch()),
        ([-3876.0, 82.0, -3219.0], porch()),
        ([-3866.0, 81.0, -3219.5], TAVERN.at([5.5, 2.6, 8.0])),
        (porch(), TAVERN.at([5.5, 2.6, 8.0])),
    ];
    let in_the_tavern = TAVERN.at([5.5, 2.6, 8.5]);
    let tables = TAVERN.at([2.5, 2.0, 7.0]);
    // Under the porch roof, which is three blocks up.
    let in_at_the_door = [
        (porch(), TAVERN.at([5.5, 2.6, 8.0])),
        (TAVERN.at([5.5, 2.4, 1.0]), TAVERN.at([5.5, 2.4, 10.0])),
        (TAVERN.at([5.5, 2.4, 3.0]), TAVERN.at([5.5, 2.4, 11.0])),
        (TAVERN.at([5.5, 2.5, 5.5]), TAVERN.at([5.5, 2.4, 13.0])),
        (in_the_tavern, TAVERN.at([5.5, 2.4, 13.0])),
    ];
    let to_the_porch_again = TAVERN.at([5.5, 2.6, 1.0]);
    let round_the_room = [
        TAVERN.at([5.5, 2.4, 13.0]),
        TAVERN.at([2.5, 2.2, 11.0]),
        tables,
        to_the_porch_again,
    ];
    let at_the_altar = CHURCH.at([5.5, 2.8, 7.5]);
    let altar = CHURCH.at([5.5, 2.2, 15.0]);
    let church_door = CHURCH.at([5.5, 2.6, 0.0]);
    // Out from under the porch and up over the well, then down round the smithy's corner.
    let across_the_green = [
        (in_the_tavern, to_the_porch_again),
        (TAVERN.at([5.5, 2.5, 5.5]), TAVERN.at([5.5, 2.4, -3.0])),
        (TAVERN.at([5.5, 2.4, 2.5]), TAVERN.at([5.5, 3.0, -6.0])),
        (TAVERN.at([5.5, 2.6, -0.5]), TAVERN.at([5.5, 4.0, -10.0])),
        (TAVERN.at([5.5, 5.0, -5.0]), [-3872.0, 80.0, -3220.0]),
        ([-3866.0, 85.0, -3221.0], CHURCH.at([7.0, 4.0, -4.0])),
        ([-3872.0, 84.0, -3221.0], CHURCH.at([5.5, 3.0, -1.0])),
        (CHURCH.at([10.0, 5.0, -5.0]), CHURCH.at([5.5, 2.8, 1.0])),
    ];
    // Round the smithy's corner to the door's height, three blocks, then in.
    let to_the_altar = [
        (CHURCH.at([10.0, 5.0, -5.0]), CHURCH.at([5.5, 2.8, 1.0])),
        (CHURCH.at([7.5, 3.8, -2.0]), CHURCH.at([5.5, 2.8, 4.0])),
        (CHURCH.at([5.8, 3.0, -0.6]), CHURCH.at([5.5, 2.7, 8.0])),
        (CHURCH.at([5.5, 2.7, 1.0]), CHURCH.at([5.5, 2.6, 10.0])),
        (CHURCH.at([5.5, 2.6, 2.5]), CHURCH.at([5.5, 2.4, 12.0])),
        (CHURCH.at([5.5, 2.7, 5.0]), altar),
        (at_the_altar, altar),
    ];
    let round_at_the_altar = [altar, CHURCH.at([8.5, 2.6, 7.5]), church_door];
    // Out and round the smithy's corner the other way and away.
    let out_and_away = [
        (at_the_altar, church_door),
        (CHURCH.at([5.5, 2.7, 5.0]), CHURCH.at([5.5, 2.8, -3.0])),
        (CHURCH.at([5.5, 2.6, 2.5]), CHURCH.at([6.5, 3.0, -4.0])),
        (CHURCH.at([5.5, 2.7, 0.4]), CHURCH.at([7.5, 3.5, -4.0])),
        // Up at once over the bank before the door, which stands above the church's floor, and
        // well over the well's roof.
        (CHURCH.at([5.8, 3.6, -0.9]), CHURCH.at([9.5, 5.0, -5.0])),
        (CHURCH.at([7.5, 5.2, -1.8]), CHURCH.at([12.0, 7.0, -5.0])),
        ([-3875.0, 84.0, -3222.5], [-3862.0, 86.0, -3209.0]),
        ([-3869.0, 88.0, -3215.0], [-3852.0, 89.0, -3196.0]),
        ([-3860.0, 93.0, -3205.0], [-3836.0, 94.0, -3176.0]),
        ([-3848.0, 98.0, -3191.0], [-3820.0, 98.0, -3150.0]),
        // On the way the flight goes.
        ([-3836.0, 104.0, -3175.0], [-3800.0, 100.0, -3120.0]),
    ];
    film.title("structgen.film.village", 5.0)
        .shot(
            orbit(VILLAGE, 50.0)
                .arc(200.0, 520.0)
                .radius(50.0, 28.0)
                .height(55.0, 14.0)
                .ease(speeds(12.0, 4.0))
                .secs(18.0),
        )
        .shot(glide(&to_the_porch, 7.0).ease(speeds(4.0, 2.0)))
        .title("structgen.film.tavern", 5.0)
        .shot(glide(&in_at_the_door, 5.0).ease(speeds(2.0, 0.5)).bob())
        .shot(turn(in_the_tavern, &round_the_room, 6.0))
        .shot(glide(&across_the_green, 10.0).ease(speeds(1.0, 3.0)))
        .title("structgen.film.church", 4.0)
        .shot(glide(&to_the_altar, 8.0).ease(speeds(3.0, 0.5)))
        .shot(turn(at_the_altar, &round_at_the_altar, 6.0))
        .shot(glide(&out_and_away, 14.0).ease(speeds(1.0, 12.0)))
}

/// On the green before the tavern's porch, at a walker's eye height.
fn porch() -> [f64; 3] {
    TAVERN.at([5.5, 2.6, -2.0])
}
