//! The flights from one place to the next, high enough to clear the hills and the mountains
//! between, each going on from where the camera came out of the last building and looking down
//! the way ahead, and turning into the orbit round the next place.

use showcase_api::Film;

use super::hamlets::LAST;
use super::town::SQUARE;
use super::village::VILLAGE;
use super::{glide, on_orbit, speeds};

/// North from the first hamlet over the fields, round into the orbit up the east of the village,
/// under the hill there.
pub fn to_the_village(film: Film) -> Film {
    let path = [
        ([-2172.0, 110.0, 3223.0], [-2178.0, 85.0, 3150.0]),
        ([-2170.0, 120.0, 3120.0], [-2166.0, 88.0, 3000.0]),
        ([-2166.0, 128.0, 3010.0], [-2165.0, 92.0, 2880.0]),
        ([-2164.0, 128.0, 2900.0], VILLAGE),
        ([-2165.0, 127.0, 2830.0], VILLAGE),
        // Into the way the orbit then goes on.
        (on_orbit(VILLAGE, 50.0, 55.0, 8.0), VILLAGE),
        (on_orbit(VILLAGE, 50.0, 55.0, 0.0), VILLAGE),
    ];
    film.shot(glide(&path, 10.0).ease(speeds(10.0, 12.0)))
}

/// North-west from the village over the plain and its lakes, past a hill on the right, and round
/// into the orbit up the east of the town.
pub fn to_the_town(film: Film) -> Film {
    let path = [
        ([-2280.0, 116.0, 2716.0], [-2340.0, 95.0, 2650.0]),
        ([-2400.0, 135.0, 2600.0], [-2560.0, 95.0, 2460.0]),
        ([-2560.0, 150.0, 2460.0], [-2720.0, 95.0, 2320.0]),
        ([-2720.0, 155.0, 2320.0], [-2880.0, 100.0, 2180.0]),
        ([-2880.0, 160.0, 2180.0], [-3040.0, 110.0, 2040.0]),
        ([-3040.0, 165.0, 2040.0], [-3200.0, 100.0, 1900.0]),
        ([-3200.0, 160.0, 1900.0], [-3340.0, 90.0, 1760.0]),
        ([-3340.0, 148.0, 1760.0], SQUARE),
        ([-3430.0, 132.0, 1660.0], SQUARE),
        ([-3462.0, 121.0, 1605.0], SQUARE),
        // Into the way the orbit then goes on.
        (on_orbit(SQUARE, 88.0, 50.0, 6.0), SQUARE),
        (on_orbit(SQUARE, 88.0, 50.0, 0.0), SQUARE),
    ];
    film.shot(glide(&path, 22.0).ease(speeds(12.0, 20.0)))
}

/// North from the town over the ridge of the mountains between, its peaks over 150, and down to
/// the last hamlet at their foot, into the orbit up its east side.
pub fn over_the_mountains(film: Film) -> Film {
    let path = [
        ([-3597.0, 125.0, 1548.0], [-3600.0, 105.0, 1450.0]),
        ([-3600.0, 140.0, 1420.0], [-3600.0, 120.0, 1250.0]),
        ([-3600.0, 165.0, 1280.0], [-3598.0, 140.0, 1100.0]),
        ([-3598.0, 185.0, 1140.0], [-3600.0, 145.0, 950.0]),
        ([-3600.0, 190.0, 1000.0], [-3606.0, 115.0, 800.0]),
        ([-3605.0, 175.0, 860.0], [-3612.0, 90.0, 640.0]),
        ([-3612.0, 150.0, 720.0], LAST),
        ([-3616.0, 125.0, 580.0], LAST),
        ([-3610.0, 108.0, 440.0], LAST),
        ([-3606.0, 96.0, 330.0], LAST),
        ([-3606.0, 91.0, 262.0], LAST),
        // Into the way the orbit then goes on.
        (on_orbit(LAST, 32.0, 18.0, 15.0), LAST),
        (on_orbit(LAST, 32.0, 18.0, 0.0), LAST),
    ];
    film.title("structgen.film.mountains", 5.0)
        .shot(glide(&path, 20.0).ease(speeds(10.0, 12.0)))
}
