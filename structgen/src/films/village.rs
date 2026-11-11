//! The village: round its green, into its tavern and out across the green into its church, then
//! out past its post mill and away.

use showcase_api::{Film, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The village, round its well on the green.
pub const VILLAGE: [f64; 3] = [-2217.0, 72.0, 2765.0];
/// Its tavern on the east of the green, its porch to the west, in line with the well.
const TAVERN: Building = Building {
    corner: Pos::new(-2199, 72, 2765),
    size: [11, 15],
    turn: Turn::Left,
};
/// Its church on the north of the green, its door to the south.
const CHURCH: Building = Building {
    corner: Pos::new(-2231, 71, 2734),
    size: [11, 18],
    turn: Turn::Back,
};
/// Its post mill, north-west of the green.
const MILL: [f64; 3] = [-2264.5, 85.0, 2726.5];

/// Down round the green from the east, under the hill there, by the north to the west; over a cottage and the well onto the
/// tavern's porch and in. Round the room to the door, out from under the porch, over the well
/// and round to the church door, in down the nave to the altar; round and out again, up over a
/// cottage and the mill and away to the north-west.
pub fn village(film: Film) -> Film {
    let in_the_tavern = TAVERN.at([5.5, 2.6, 8.5]);
    let tables = TAVERN.at([2.5, 2.0, 7.0]);
    let porch = TAVERN.at([5.5, 3.8, -2.0]);
    let to_the_porch = [
        (on_orbit(VILLAGE, 40.0, 18.0, 150.0), VILLAGE),
        ([-2243.0, 89.0, 2784.0], TAVERN.at([5.5, 3.0, -12.0])),
        // Over a cottage, then down over the well, which keeps behind the camera.
        ([-2230.0, 87.0, 2777.0], TAVERN.at([5.5, 3.0, -4.0])),
        (TAVERN.at([5.5, 11.0, -14.0]), TAVERN.at([5.5, 3.0, 0.0])),
        (TAVERN.at([5.5, 7.5, -6.0]), TAVERN.at([5.5, 2.6, 4.0])),
        (porch, TAVERN.at([5.5, 2.6, 8.0])),
    ];
    // Under the porch roof, which is three blocks up.
    let in_at_the_door = [
        (porch, TAVERN.at([5.5, 2.6, 8.0])),
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
    let (tower, altar) = (CHURCH.at([5.5, 14.0, 6.0]), CHURCH.at([5.5, 2.2, 15.0]));
    let church_door = CHURCH.at([5.5, 2.6, 0.0]);
    let across_the_green = [
        (in_the_tavern, to_the_porch_again),
        (TAVERN.at([5.5, 2.5, 5.5]), TAVERN.at([5.5, 2.4, -3.0])),
        (TAVERN.at([5.5, 2.4, 2.5]), TAVERN.at([5.5, 3.0, -6.0])),
        (TAVERN.at([5.5, 2.6, -0.5]), TAVERN.at([5.5, 5.0, -10.0])),
        (TAVERN.at([5.5, 6.0, -5.0]), [-2214.0, 78.0, 2766.0]),
        ([-2212.0, 85.0, 2771.0], [-2224.0, 82.0, 2771.0]),
        ([-2221.0, 89.0, 2773.0], CHURCH.at([5.5, 6.0, -2.0])),
        ([-2225.5, 87.0, 2767.0], tower),
        (CHURCH.at([5.5, 8.5, -9.0]), tower),
    ];
    // Down to the door's height, three blocks, then in.
    let to_the_altar = [
        (CHURCH.at([5.5, 8.5, -9.0]), tower),
        (CHURCH.at([5.5, 6.0, -5.0]), CHURCH.at([5.5, 8.0, 4.0])),
        (CHURCH.at([5.5, 5.2, -2.8]), CHURCH.at([5.5, 3.0, 6.0])),
        (CHURCH.at([5.5, 3.7, -1.2]), CHURCH.at([5.5, 2.8, 8.0])),
        (CHURCH.at([5.5, 2.8, 0.4]), CHURCH.at([5.5, 2.6, 10.0])),
        (CHURCH.at([5.5, 2.6, 2.5]), CHURCH.at([5.5, 2.4, 12.0])),
        (CHURCH.at([5.5, 2.7, 5.0]), altar),
        (at_the_altar, altar),
    ];
    let round_at_the_altar = [altar, CHURCH.at([8.5, 2.6, 7.5]), church_door];
    let out_and_away = [
        (at_the_altar, church_door),
        (CHURCH.at([5.5, 2.7, 5.0]), CHURCH.at([5.5, 2.8, -3.0])),
        (CHURCH.at([5.5, 2.6, 2.5]), CHURCH.at([5.5, 3.2, -6.0])),
        (CHURCH.at([5.5, 2.8, 0.4]), CHURCH.at([5.5, 4.5, -8.0])),
        // Up as soon as the door is behind.
        (CHURCH.at([5.5, 4.3, -1.0]), CHURCH.at([5.5, 6.0, -10.0])),
        (CHURCH.at([5.5, 6.0, -2.8]), CHURCH.at([4.0, 7.0, -12.0])),
        (CHURCH.at([5.5, 8.2, -5.5]), [-2232.0, 78.0, 2768.0]),
        ([-2231.0, 86.0, 2764.0], [-2246.0, 80.0, 2766.0]),
        ([-2242.0, 94.0, 2766.0], [-2262.0, 84.0, 2748.0]),
        ([-2254.0, 100.0, 2756.0], MILL),
        ([-2263.0, 106.0, 2742.0], [-2290.0, 90.0, 2690.0]),
        ([-2272.0, 112.0, 2726.0], [-2330.0, 95.0, 2660.0]),
        // On the way the flight goes.
        ([-2280.0, 116.0, 2716.0], [-2340.0, 95.0, 2650.0]),
    ];
    film.title("structgen.film.village", 5.0)
        .shot(
            orbit(VILLAGE, 50.0)
                .arc(360.0, 150.0)
                .radius(50.0, 40.0)
                .height(55.0, 18.0)
                .ease(speeds(12.0, 4.0))
                .secs(14.0),
        )
        .shot(glide(&to_the_porch, 8.0).ease(speeds(4.0, 3.0)))
        .title("structgen.film.tavern", 5.0)
        .shot(glide(&in_at_the_door, 4.0).ease(speeds(3.0, 0.5)))
        .shot(turn(in_the_tavern, &round_the_room, 6.0))
        .shot(glide(&across_the_green, 10.0).ease(speeds(1.0, 4.0)))
        .title("structgen.film.church", 4.0)
        .shot(glide(&to_the_altar, 8.0).ease(speeds(4.0, 0.5)))
        .shot(turn(at_the_altar, &round_at_the_altar, 6.0))
        .shot(glide(&out_and_away, 12.0).ease(speeds(1.0, 12.0)))
}
