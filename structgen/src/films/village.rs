//! The village: round its green, into a cottage on it to open a chest that has something in it,
//! out across the green into its tavern and back across into its church, then out past its post
//! mill and away.

use showcase_api::{Ease, Film, hold, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The village, round its well on the green.
pub const VILLAGE: [f64; 3] = [-2217.0, 72.0, 2765.0];
/// A dormer cottage on the west of the green, its door to the east and the tavern's porch: the
/// building whose chests seed 42 fills in this village.
const COTTAGE: Building = Building {
    corner: Pos::new(-2235, 71, 2771),
    size: [11, 9],
    turn: Turn::Right,
};
/// The chest in the cottage's back corner, past the furnace.
const CHEST: Pos = Pos::new(-2233, 72, 2779);
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

/// Down round the green from the east, under the hill there, by the north to the west; into the
/// cottage (`cottage`) and across the green into the tavern. Round the room to the door, out
/// from under the porch, over the well and round to the church door, in down the nave to the
/// altar; round and out again, up over a cottage and the mill and away to the north-west.
pub fn village(film: Film) -> Film {
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
        (CHURCH.at([5.5, 8.2, -5.5]), [-2229.0, 78.0, 2768.0]),
        ([-2231.0, 86.0, 2764.0], [-2246.0, 80.0, 2766.0]),
        ([-2242.0, 94.0, 2766.0], [-2262.0, 84.0, 2748.0]),
        ([-2254.0, 100.0, 2756.0], MILL),
        ([-2263.0, 106.0, 2742.0], [-2290.0, 90.0, 2690.0]),
        ([-2272.0, 112.0, 2726.0], [-2330.0, 95.0, 2660.0]),
        // On the way the flight goes.
        ([-2280.0, 116.0, 2716.0], [-2340.0, 95.0, 2650.0]),
    ];
    let film = film.title("structgen.film.village", 5.0).shot(
        orbit(VILLAGE, 50.0)
            .arc(360.0, 150.0)
            .radius(50.0, 40.0)
            .height(55.0, 18.0)
            .ease(speeds(12.0, 4.0))
            .secs(14.0),
    );
    cottage(film)
        .title("structgen.film.tavern", 5.0)
        .shot(glide(&in_at_the_door, 5.0).ease(speeds(2.0, 0.5)).bob())
        .shot(turn(in_the_tavern, &round_the_room, 6.0))
        .shot(glide(&across_the_green, 10.0).ease(speeds(1.0, 4.0)))
        .title("structgen.film.church", 4.0)
        .shot(glide(&to_the_altar, 8.0).ease(speeds(4.0, 0.5)))
        .shot(turn(at_the_altar, &round_at_the_altar, 6.0))
        .shot(glide(&out_and_away, 14.0).ease(speeds(1.0, 12.0)))
}

/// Round the south of the cottage from where the orbit left off and down onto the green before
/// its door; in, past the stove to the chest in the back corner, which opens. Round to the door,
/// and out across the green, at a walk, past the well to the tavern's porch.
fn cottage(film: Film) -> Film {
    // Beside the table, the chest in the corner past it.
    let inside = COTTAGE.at([6.5, 2.6, 4.2]);
    let (door, chest) = (COTTAGE.at([5.5, 3.0, 0.0]), COTTAGE.at([8.5, 1.4, 6.5]));
    let way_out = COTTAGE.at([5.5, 2.5, -2.0]);
    // Past the stove and the bench, a like angle apart, so it turns at an even pace.
    let round_to_the_door = [
        chest,
        COTTAGE.at([8.9, 1.8, 4.7]),
        COTTAGE.at([8.8, 2.0, 3.1]),
        COTTAGE.at([7.7, 2.3, 2.0]),
        way_out,
    ];
    // Over the houses south of the cottage and round to the east of it, the door in view.
    let round_to_the_front = [
        (on_orbit(VILLAGE, 40.0, 18.0, 150.0), VILLAGE),
        ([-2243.0, 88.0, 2793.0], door),
        ([-2231.0, 87.0, 2797.0], door),
        ([-2215.0, 87.0, 2793.0], door),
        ([-2207.0, 85.0, 2785.0], door),
        ([-2211.0, 81.0, 2776.0], door),
        (COTTAGE.at([5.5, 5.0, -7.0]), COTTAGE.at([5.5, 2.4, 2.0])),
    ];
    // The door is two blocks high, its lintel three blocks up.
    let in_at_the_door = [
        (COTTAGE.at([5.5, 5.0, -7.0]), COTTAGE.at([5.5, 2.4, 2.0])),
        (COTTAGE.at([5.5, 3.6, -3.5]), COTTAGE.at([5.5, 2.4, 3.0])),
        (COTTAGE.at([5.5, 2.8, -0.5]), COTTAGE.at([5.5, 2.2, 5.0])),
        (COTTAGE.at([5.5, 2.5, 1.5]), COTTAGE.at([7.5, 2.0, 5.5])),
        (COTTAGE.at([6.0, 2.6, 3.0]), chest),
        (inside, chest),
    ];
    // Down the step to the green, a block or two below the floor, and along the path over it,
    // clear of the long grass and flowers, up again to the tavern's.
    let across_the_green = [
        (inside, way_out),
        (COTTAGE.at([5.7, 2.55, 2.8]), COTTAGE.at([5.5, 2.4, -3.0])),
        (COTTAGE.at([5.5, 2.5, 1.5]), COTTAGE.at([5.5, 2.3, -4.0])),
        (COTTAGE.at([5.5, 2.0, -1.0]), [-2214.0, 72.8, 2775.8]),
        ([-2220.0, 72.2, 2776.0], [-2208.0, 73.5, 2775.0]),
        ([-2214.0, 72.8, 2775.8], [-2204.0, 74.2, 2772.0]),
        ([-2208.5, 73.8, 2774.5], porch()),
        ([-2205.0, 74.6, 2771.5], TAVERN.at([5.5, 2.6, 8.0])),
        (porch(), TAVERN.at([5.5, 2.6, 8.0])),
    ];
    film.shot(glide(&round_to_the_front, 12.0).ease(speeds(4.0, 3.0)))
        .title("structgen.film.chest", 5.0)
        .shot(glide(&in_at_the_door, 5.0).ease(speeds(3.0, 0.0)))
        .wait(1.0)
        .menu("base:chest", CHEST, 4.0)
        .shot(hold(inside).look_at(chest).secs(4.0))
        .shot(turn(inside, &round_to_the_door, 5.0).ease(Ease::Smooth))
        .shot(glide(&across_the_green, 10.0).ease(speeds(0.0, 2.0)).bob())
}

/// On the green before the tavern's porch, at a walker's eye height.
fn porch() -> [f64; 3] {
    TAVERN.at([5.5, 2.6, -2.0])
}
