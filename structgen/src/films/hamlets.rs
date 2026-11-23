//! The hamlet the journey starts in: in from over the lake to its south-west and round it, into
//! a cottage there to open a chest that has something in it, and out again, away to the east.

use showcase_api::{Ease, Film, flyby, hold, orbit};
use weftkit::Pos;

use super::building::Building;
use super::{glide, on_orbit, speeds, turn};
use crate::turn::Turn;

/// The hamlet, by its well.
const HAMLET: [f64; 3] = [-4058.5, 70.0, -3136.5];
/// A crofter's cottage north-east of the well, its door to the south onto the lane round the
/// green: the building whose chest seed 42 fills in this hamlet.
const CROFT: Building = Building {
    corner: Pos::new(-4050, 70, -3152),
    size: [9, 7],
    turn: Turn::Back,
};
/// The chest against its back wall, between the bed and the stove.
const CHEST: Pos = Pos::new(-4047, 71, -3150);

/// In over the lake and round the hamlet from the south-west by the south to the east; along
/// the lane to the crofter's door and in. Round past the bed to the chest, which opens; on past
/// the stove to the door, out, and up over the cottages, round to the east.
pub fn first(film: Film) -> Film {
    let in_from = [
        [-4150.0, 120.0, -3060.0],
        [-4120.0, 104.0, -3080.0],
        // Into the way the orbit then goes on.
        on_orbit(HAMLET, 48.0, 21.5, 140.0),
        on_orbit(HAMLET, 48.0, 21.0, 130.0),
    ];
    let inside = CROFT.at([4.5, 2.6, 3.0]);
    let (bed, stove) = (CROFT.at([2.0, 2.0, 4.6]), CROFT.at([7.0, 2.0, 4.6]));
    let chest = CROFT.at([5.5, 1.4, 4.5]);
    let door = CROFT.at([4.5, 2.6, -1.0]);
    // West along the lane before the cottages, turning in to the door.
    let in_at_the_door = [
        (on_orbit(HAMLET, 30.0, 10.0, -10.0), HAMLET),
        ([-4035.0, 77.0, -3139.0], CROFT.at([4.5, 2.6, -6.0])),
        ([-4041.0, 75.0, -3138.0], CROFT.at([4.5, 2.4, -3.0])),
        (CROFT.at([4.5, 3.9, -5.5]), CROFT.at([4.5, 2.4, 3.0])),
        (CROFT.at([4.5, 3.6, -2.0]), CROFT.at([4.5, 2.4, 4.0])),
        (CROFT.at([4.5, 2.7, 0.3]), CROFT.at([4.5, 2.4, 5.0])),
        (CROFT.at([4.5, 2.6, 1.5]), CROFT.at([3.0, 2.2, 5.0])),
        (inside, bed),
    ];
    let to_the_chest = [bed, CROFT.at([4.0, 1.8, 5.6]), chest];
    let to_the_door = [chest, stove, CROFT.at([7.5, 2.4, 1.5]), door];
    let out_and_up = [
        (inside, door),
        (CROFT.at([4.5, 2.6, 1.0]), CROFT.at([4.5, 2.4, -4.0])),
        (CROFT.at([4.5, 3.4, -1.5]), CROFT.at([2.0, 3.0, -8.0])),
        ([-4044.0, 78.0, -3139.0], [-4034.0, 77.0, -3136.0]),
        ([-4039.0, 83.0, -3137.0], [-4024.0, 82.0, -3141.0]),
        ([-4031.0, 89.0, -3139.0], [-4008.0, 88.0, -3150.0]),
        ([-4021.0, 95.0, -3144.0], [-3990.0, 94.0, -3160.0]),
        // On the way the flight goes.
        ([-4008.0, 101.0, -3150.0], [-3960.0, 98.0, -3175.0]),
    ];
    film.title("mod.structgen.name", 4.0)
        .shot(
            flyby(&in_from)
                .look_at(HAMLET)
                .secs(10.0)
                .ease(speeds(0.0, 8.0)),
        )
        .title("structgen.film.hamlet", 5.0)
        .shot(
            orbit(HAMLET, 48.0)
                .arc(130.0, -10.0)
                .radius(48.0, 30.0)
                .height(21.0, 10.0)
                .secs(11.0)
                .ease(speeds(8.0, 3.0)),
        )
        .title("structgen.film.inside", 5.0)
        .shot(glide(&in_at_the_door, 9.0).ease(speeds(3.0, 0.5)))
        .shot(turn(inside, &to_the_chest, 4.0).ease(Ease::Smooth))
        .title("structgen.film.chest", 5.0)
        .wait(1.0)
        .menu("base:chest", CHEST, 4.0)
        .shot(hold(inside).look_at(chest).secs(4.0))
        .shot(turn(inside, &to_the_door, 5.0).ease(Ease::Smooth))
        .shot(glide(&out_and_up, 11.0).ease(speeds(1.0, 10.0)))
}
