//! The flights from one place to the next, high enough to clear the hills and the mountains
//! between, each going on from where the camera came out of the last building and looking down
//! the way ahead.

use showcase_api::Film;

use super::{glide, speeds};

/// South from the first hamlet, over the ridge to the village in the hills.
pub fn to_the_hills(film: Film) -> Film {
    let path = [
        ([4183.0, 106.6, -176.7], [4180.0, 95.0, -120.0]),
        ([4178.0, 135.0, -60.0], [4180.0, 120.0, 60.0]),
        ([4180.0, 165.0, 60.0], [4185.0, 130.0, 200.0]),
        ([4170.0, 160.0, 190.0], [4194.0, 110.0, 270.0]),
        // Into the way the orbit then goes on.
        ([4178.4, 142.4, 223.6], [4196.0, 108.0, 280.0]),
        ([4192.16, 140.0, 225.13], [4196.0, 108.0, 280.0]),
    ];
    film.shot(glide(&path, 12.0).ease(speeds(10.0, 12.0)))
}

/// North-west from the village over the hills, down past a lake and west to the town, the
/// mountains off to the left.
pub fn to_the_town(film: Film) -> Film {
    let path = [
        ([4196.0, 128.0, 252.0], [4150.0, 130.0, 150.0]),
        ([4194.15, 131.2, 247.4], [4150.0, 130.0, 150.0]),
        ([4100.0, 170.0, 60.0], [4040.0, 140.0, -120.0]),
        ([4000.0, 175.0, -250.0], [3950.0, 140.0, -420.0]),
        ([3900.0, 192.0, -600.0], [3860.0, 150.0, -780.0]),
        ([3840.0, 180.0, -850.0], [3700.0, 110.0, -1000.0]),
        ([3700.0, 140.0, -1020.0], [3560.0, 80.0, -1100.0]),
        // Round into the way the orbit then goes on.
        ([3605.0, 126.0, -1076.0], [3500.0, 70.0, -1140.0]),
        ([3569.67, 120.92, -1082.7], [3500.0, 70.0, -1140.0]),
        ([3560.02, 120.0, -1075.64], [3500.0, 70.0, -1140.0]),
    ];
    film.shot(glide(&path, 20.0).ease(speeds(12.0, 20.0)))
}

/// East out of the town, over a hill and down across the plain to the second village's mill.
pub fn to_the_plain(film: Film) -> Film {
    let mill = [4394.5, 82.0, -1115.5];
    let path = [
        ([3514.0, 90.0, -1128.0], [3600.0, 110.0, -1124.0]),
        ([3620.0, 140.0, -1125.0], [3720.0, 120.0, -1125.0]),
        ([3745.0, 165.0, -1125.0], [3900.0, 110.0, -1118.0]),
        ([3950.0, 130.0, -1115.0], [4150.0, 90.0, -1112.0]),
        ([4200.0, 110.0, -1110.0], mill),
        ([4332.3, 99.25, -1116.75], mill),
        ([4340.0, 98.0, -1115.0], mill),
    ];
    film.shot(glide(&path, 12.0).ease(speeds(10.0, 15.0)))
}

/// North-west from the village up a valley between the mountains, its peaks over 200 on either
/// side, over the ridge at its head and down to the last hamlet at their foot.
pub fn over_the_mountains(film: Film) -> Film {
    let hamlet = [3772.0, 72.0, -2178.0];
    let path = [
        // Where the orbit round the village ended, looking over it, already heading this way.
        ([4492.43, 109.0, -1142.43], [4450.0, 85.0, -1100.0]),
        ([4300.0, 120.0, -1330.0], [4160.0, 110.0, -1480.0]),
        ([4160.0, 145.0, -1480.0], [4050.0, 150.0, -1580.0]),
        ([4085.0, 180.0, -1545.0], [3995.0, 140.0, -1670.0]),
        ([4010.0, 168.0, -1610.0], [3985.0, 145.0, -1760.0]),
        ([3985.0, 170.0, -1730.0], [3995.0, 160.0, -1830.0]),
        ([3995.0, 190.0, -1830.0], [3960.0, 170.0, -1950.0]),
        ([3975.0, 205.0, -1910.0], [3900.0, 140.0, -2040.0]),
        ([3920.0, 180.0, -2000.0], [3820.0, 100.0, -2120.0]),
        ([3860.0, 140.0, -2070.0], hamlet),
        ([3803.3, 112.75, -2133.3], hamlet),
        ([3800.0, 110.0, -2140.0], hamlet),
    ];
    film.title("structgen.film.mountains", 5.0)
        .shot(glide(&path, 17.0).ease(speeds(15.0, 15.0)))
}
