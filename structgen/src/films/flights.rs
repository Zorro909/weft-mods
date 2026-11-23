//! The flights from one place to the next, high enough to clear the hills between, each going on
//! from where the camera came out of the last building and looking down the way ahead, and
//! turning into the first shot of the next place.

use showcase_api::Film;

use super::village::VILLAGE;
use super::{glide, on_orbit, speeds};

/// North-east from the hamlet over the meadow, into the orbit up the west of the village on its
/// hillside.
pub fn to_the_village(film: Film) -> Film {
    let path = [
        ([-4008.0, 101.0, -3150.0], [-3960.0, 98.0, -3175.0]),
        ([-3985.0, 110.0, -3168.0], VILLAGE),
        ([-3962.0, 119.0, -3188.0], VILLAGE),
        ([-3940.0, 126.0, -3206.0], VILLAGE),
        // Into the way the orbit then goes on.
        (on_orbit(VILLAGE, 50.0, 55.0, 190.0), VILLAGE),
        (on_orbit(VILLAGE, 50.0, 55.0, 200.0), VILLAGE),
    ];
    film.shot(glide(&path, 9.0).ease(speeds(10.0, 12.0)))
}

/// South-east from the village down the hillside, and round to the south down the west of the
/// town.
pub fn to_the_town(film: Film) -> Film {
    let path = [
        ([-3836.0, 104.0, -3175.0], [-3800.0, 100.0, -3120.0]),
        ([-3822.0, 112.0, -3150.0], [-3790.0, 105.0, -3080.0]),
        ([-3806.0, 122.0, -3118.0], [-3740.0, 90.0, -3010.0]),
        // On the way the flyby round the town goes, looking at its square.
        ([-3795.0, 128.0, -3090.0], [-3702.5, 74.0, -2984.5]),
    ];
    film.title("structgen.film.town", 6.0)
        .shot(glide(&path, 8.0).ease(speeds(12.0, 14.0)))
}
