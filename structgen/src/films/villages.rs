//! The two villages: one up in the hills, into its tavern and out across the green into its
//! church; one out on the plain, past its mill into its smithy and out round the village.

use showcase_api::{Film, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, speeds, turn};
use crate::turn::Turn;

/// The village in the hills, round its green.
const HILLS: [f64; 3] = [4196.0, 108.0, 280.0];
/// Its tavern, its porch to the east, on the green.
const TAVERN: Building = Building {
    corner: Pos::new(4164, 108, 270),
    size: [11, 15],
    turn: Turn::Right,
};
/// Its church, its door to the north in a wall a block in. The ground north of it lies higher
/// than its floor.
const CHURCH: Building = Building {
    corner: Pos::new(4201, 105, 277),
    size: [11, 18],
    turn: Turn::None,
};
/// The village on the plain.
const PLAIN: [f64; 3] = [4450.0, 73.0, -1100.0];
/// Over it, which the orbit round it looks at to keep the land beyond in view.
const OVER_PLAIN: [f64; 3] = [4450.0, 85.0, -1100.0];
/// Its post mill, at the west end of its road.
const MILL: [f64; 3] = [4394.5, 82.0, -1115.5];
/// Its smithy: the forge open to the south under a stone lintel, the house behind it.
const SMITHY: Building = Building {
    corner: Pos::new(4420, 74, -1127),
    size: [12, 15],
    turn: Turn::Back,
};

/// Round the green from the north to the east; down over the oak on the green onto the
/// tavern's porch and in. Round the room to the door, out from under the porch, over the oak and
/// the well and down to the church door, in down the nave to the altar; round and out again, up
/// and away to the north-west.
pub fn in_the_hills(film: Film) -> Film {
    let in_the_tavern = TAVERN.at([5.5, 2.6, 8.5]);
    let tables = TAVERN.at([2.5, 2.0, 7.0]);
    let porch = TAVERN.at([5.5, 3.8, -2.0]);
    // The oak stands in line with the door: down from over it, so it stays behind the camera.
    let to_the_porch = [
        ([4235.39, 126.0, 273.05], HILLS),
        (TAVERN.at([5.5, 13.0, -15.0]), TAVERN.at([5.5, 3.0, 0.0])),
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
        (TAVERN.at([5.5, 6.0, -5.0]), [4200.0, 112.0, 268.0]),
        ([4195.0, 123.0, 271.0], CHURCH.at([5.5, 6.0, -2.0])),
        ([4206.5, 118.0, 262.0], CHURCH.at([5.5, 6.0, 1.0])),
        (CHURCH.at([5.5, 8.5, -9.0]), tower),
    ];
    // The ground before the door falls to the church's floor only right at its wall, and the
    // door is three blocks high: down the slope, over its grass, to the door's height, then in.
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
        // Up off the slope as soon as the door is behind.
        (CHURCH.at([5.5, 4.3, -1.0]), CHURCH.at([5.5, 6.0, -10.0])),
        (CHURCH.at([5.5, 6.0, -2.8]), CHURCH.at([5.5, 7.0, -12.0])),
        (CHURCH.at([5.5, 8.2, -5.5]), [4196.0, 110.0, 250.0]),
        ([4200.0, 121.0, 262.0], [4170.0, 115.0, 200.0]),
        ([4196.0, 128.0, 252.0], [4150.0, 130.0, 150.0]),
    ];
    film.title("structgen.film.village", 5.0)
        .shot(
            orbit(HILLS, 55.0)
                .arc(266.0, 350.0)
                .radius(55.0, 40.0)
                .height(32.0, 18.0)
                .ease(speeds(12.0, 0.0))
                .secs(10.0),
        )
        .shot(glide(&to_the_porch, 8.0).ease(speeds(0.0, 3.0)))
        .title("structgen.film.tavern", 5.0)
        .shot(glide(&in_at_the_door, 4.0).ease(speeds(3.0, 0.5)))
        .shot(turn(in_the_tavern, &round_the_room, 6.0))
        .shot(glide(&across_the_green, 10.0).ease(speeds(1.0, 4.0)))
        .title("structgen.film.church", 4.0)
        .shot(glide(&to_the_altar, 8.0).ease(speeds(4.0, 0.5)))
        .shot(turn(at_the_altar, &round_at_the_altar, 6.0))
        .shot(glide(&out_and_away, 9.0).ease(speeds(1.0, 12.0)))
}

/// In from the west past the mill to the smithy's forge, past the anvil and the lava and on
/// into the house; round to the doorway, back out under the lintel and up to the west, and round
/// the whole village.
pub fn on_the_plain(film: Film) -> Film {
    let anvil = SMITHY.at([4.5, 1.6, 4.5]);
    let outside = SMITHY.at([5.5, 3.6, -2.5]);
    let in_the_house = SMITHY.at([6.5, 2.6, 10.0]);
    let doorway = SMITHY.at([5.5, 2.6, 7.0]);
    let to_the_forge = [
        ([4340.0, 98.0, -1115.0], MILL),
        ([4372.0, 90.0, -1098.0], MILL),
        ([4405.0, 82.0, -1100.0], SMITHY.at([4.5, 2.0, 2.0])),
        (SMITHY.at([5.5, 4.5, -6.0]), SMITHY.at([4.5, 2.0, 3.0])),
        (outside, anvil),
    ];
    // Beside the tall grass before the forge, under its lintel three blocks up, past the anvil
    // and the lava and straight through the one-block doorway into the house.
    let into_the_house = [
        (outside, anvil),
        (SMITHY.at([5.5, 3.3, -1.0]), anvil),
        (SMITHY.at([5.5, 2.7, 0.3]), anvil),
        (SMITHY.at([5.5, 2.6, 1.5]), anvil),
        (SMITHY.at([5.5, 2.6, 3.0]), SMITHY.at([5.0, 2.0, 7.0])),
        (SMITHY.at([5.5, 2.6, 5.0]), SMITHY.at([5.5, 2.6, 9.0])),
        (SMITHY.at([5.5, 2.6, 7.5]), SMITHY.at([6.0, 2.4, 12.0])),
        (SMITHY.at([5.5, 2.6, 8.7]), SMITHY.at([6.5, 2.4, 12.0])),
        (in_the_house, SMITHY.at([7.5, 2.4, 13.0])),
    ];
    let round_the_room = [
        SMITHY.at([7.5, 2.4, 13.0]),
        SMITHY.at([9.0, 2.4, 10.0]),
        doorway,
    ];
    let out_and_up = [
        (in_the_house, doorway),
        (SMITHY.at([5.5, 2.6, 8.7]), SMITHY.at([5.5, 2.6, 3.0])),
        (SMITHY.at([5.5, 2.6, 7.5]), SMITHY.at([5.0, 2.6, 3.0])),
        (SMITHY.at([5.5, 2.6, 4.5]), SMITHY.at([5.5, 2.4, -2.0])),
        (SMITHY.at([5.5, 2.6, 2.5]), SMITHY.at([5.5, 2.6, -6.0])),
        (SMITHY.at([5.5, 2.6, 1.2]), SMITHY.at([5.5, 3.0, -8.0])),
        (SMITHY.at([5.5, 2.7, 0.3]), SMITHY.at([5.5, 3.5, -10.0])),
        (SMITHY.at([5.5, 3.4, -1.0]), SMITHY.at([5.5, 4.0, -12.0])),
        (SMITHY.at([5.5, 4.8, -4.5]), SMITHY.at([5.5, 3.0, -16.0])),
        ([4413.0, 90.0, -1103.0], OVER_PLAIN),
        // Into the way the orbit then goes on.
        ([4411.16, 98.59, -1092.17], OVER_PLAIN),
        ([4412.41, 99.0, -1086.32], OVER_PLAIN),
    ];
    film.title("structgen.film.smithy", 5.0)
        .shot(glide(&to_the_forge, 10.0).ease(speeds(15.0, 2.0)))
        .shot(glide(&into_the_house, 9.0).ease(speeds(2.0, 0.5)))
        .shot(turn(in_the_house, &round_the_room, 6.0))
        .shot(glide(&out_and_up, 10.0).ease(speeds(1.0, 10.0)))
        .shot(
            orbit(PLAIN, 40.0)
                .arc(160.0, -45.0)
                .radius(40.0, 60.0)
                .height(26.0, 36.0)
                .look_at(OVER_PLAIN)
                .ease(speeds(10.0, 15.0))
                .secs(10.0),
        )
}
