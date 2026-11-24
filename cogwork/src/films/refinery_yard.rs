//! Section 6 (Tier 4b): the refinery yard on Pad R, between the oil seep and the hall. Crude oil
//! arrives into a tank, three firebox-and-boiler pairs feed steam through pressure tubes into the
//! column's heaters, and a timber liquid tube lifts lake water up a vertical riser of sealed
//! liquid tubes into the top condenser. Four taps on the east face draw off tar (beside the
//! asphalt road), lubricant (piped north of the hall to the Gantry Excavator), diesel (piped into
//! the hall's oil engine tank), and gas (piped south of the hall to Pad P's polymerizer).

use showcase_api::{Film, Look, Set, flyby, orbit};
use weft_sdk::Dir;

use super::camera::{on_orbit, speeds};
use super::parts::{
    CHEST, area, engine, fuel, fuel_item, liquid_tubes, prefill, pressure_tubes, sealed_tubes,
    tubes,
};
use super::seep;
use super::site::{PAD_P_TOP, above, lowered, lowered_block};
use crate::asphalt::ASPHALT;
use crate::firebox::{BOILER, FIREBOX};
use crate::lib::{Area, BlockState, Pos};
use crate::refinery::parts::REFINERY_CASING;
use crate::refinery::{REFINERY, REFINERY_CONDENSER, REFINERY_HEATER, REFINERY_TAP};
use crate::tank::LIQUID_TANK;
use crate::timber_liquid_tube::TIMBER_LIQUID_TUBE;

/// Refinery controller and column bounds (3 by 3 by 6, hollow up the middle of layers 1 to 4).
pub const REFINERY_AT: Pos = Pos::new(-11, 0, -4);
const COLUMN: Area = Area {
    lo: Pos::new(-12, 0, -6),
    hi: Pos::new(-10, 5, -4),
};
const HEATERS: [Pos; 2] = [Pos::new(-12, 0, -6), Pos::new(-12, 0, -5)];
const CONDENSER: Pos = Pos::new(-10, 5, -6);
const TAR_TAP: Pos = Pos::new(-10, 1, -4);
const LUBE_TAP: Pos = Pos::new(-10, 2, -6);
const DIESEL_TAP: Pos = Pos::new(-10, 3, -5);
const GAS_TAP: Pos = Pos::new(-10, 4, -4);

/// Crude oil buffer tank where the seep line arrives, and the sulfur chest out the front.
pub const CRUDE_TANK: Area = Area {
    lo: Pos::new(-16, 0, -5),
    hi: Pos::new(-16, 1, -5),
};
const SULFUR_CHEST: Pos = Pos::new(-13, 0, -3);

/// Steam supply: three firebox+boiler pairs and their water tank.
const STEAM_WATER: Area = Area {
    lo: Pos::new(-15, 0, -9),
    hi: Pos::new(-14, 2, -9),
};
const FIREBOXES: [Pos; 3] = [
    Pos::new(-16, 0, -7),
    Pos::new(-15, 0, -7),
    Pos::new(-14, 0, -7),
];
const BOILERS: [Pos; 3] = [
    Pos::new(-16, 1, -7),
    Pos::new(-15, 1, -7),
    Pos::new(-14, 1, -7),
];

/// Cooling water supply: tank fed by the jetty branch, timber liquid tube lift, and sealed riser.
pub const COOLING_TANK: Area = Area {
    lo: Pos::new(-10, 0, -10),
    hi: Pos::new(-9, 1, -9),
};
const WATER_LIFT: Pos = Pos::new(-10, 0, -8);
const WATER_LIFT_ENGINE: Pos = Pos::new(-11, 0, -8);

/// Product buffer tanks on the east side, each separated by air so no two liquids meet.
const TAR_TANK: Area = Area {
    lo: Pos::new(-8, 0, -3),
    hi: Pos::new(-8, 1, -3),
};
const LUBE_TANK: Area = Area {
    lo: Pos::new(-8, 0, -7),
    hi: Pos::new(-8, 2, -7),
};
const GAS_TANK: Area = Area {
    lo: Pos::new(-7, 4, -4),
    hi: Pos::new(-7, 5, -4),
};

/// Refinery orbit geometry and handoff point heading east into the hall.
pub const ORBIT_CENTER: [f64; 3] = lowered([-11.0, 2.5, -5.0]);
const ORBIT_OUT: f64 = 7.4;
const ORBIT_ARC: [f64; 2] = [135.0, 52.0];
const ORBIT_UP: [f64; 2] = [2.2, 1.3];

pub fn end_eye() -> [f64; 3] {
    on_orbit(ORBIT_CENTER, ORBIT_OUT, ORBIT_UP[1], ORBIT_ARC[1])
}

pub fn build(set: &Set) {
    asphalt_road(set);
    column(set);
    crude_and_sulfur(set);
    steam_supply(set);
    cooling_riser(set);
    product_lines(set);
}

/// Asphalt avenue along Pad R's south front and along the south side of the hall to Pad P.
fn asphalt_road(set: &Set) {
    set.fill(
        area(Pos::new(-20, -1, 1), Pos::new(-6, -1, 2)),
        BlockState::of(&ASPHALT),
    );
    set.fill(
        area(Pos::new(-6, -1, 3), Pos::new(-5, -1, 9)),
        BlockState::of(&ASPHALT),
    );
    set.fill(
        area(Pos::new(-4, -1, 8), Pos::new(25, -1, 9)),
        BlockState::of(&ASPHALT),
    );
}

/// 3 by 3 by 6 refinery column with heaters, condenser, and four product taps.
fn column(set: &Set) {
    set.fill(COLUMN, BlockState::of(&REFINERY_CASING));
    set.fill(
        area(Pos::new(-11, 1, -5), Pos::new(-11, 4, -5)),
        BlockState::AIR,
    );
    set.place(REFINERY_AT, REFINERY.with(&[("facing", "south")]).into());
    for at in HEATERS {
        set.place(at, BlockState::of(&REFINERY_HEATER));
    }
    set.place(CONDENSER, BlockState::of(&REFINERY_CONDENSER));
    for at in [TAR_TAP, LUBE_TAP, DIESEL_TAP, GAS_TAP] {
        set.place(at, BlockState::of(&REFINERY_TAP));
    }
}

/// Crude oil tank feeding the controller from underneath, and sulfur output chest out the front.
fn crude_and_sulfur(set: &Set) {
    set.fill(CRUDE_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &crude_line());
    tubes(set, Pos::new(-11, 0, -3), Pos::new(-12, 0, -3));
    set.place(SULFUR_CHEST, BlockState::of(&CHEST));
}

fn crude_line() -> Vec<Pos> {
    vec![
        Pos::new(-15, 0, -5),
        Pos::new(-15, 0, -4),
        Pos::new(-14, 0, -4),
        Pos::new(-14, -1, -4),
        Pos::new(-13, -1, -4),
        Pos::new(-12, -1, -4),
        Pos::new(-11, -1, -4),
    ]
}

/// Three firebox+boiler pairs feeding steam through pressure tubes into the column's heaters.
fn steam_supply(set: &Set) {
    set.fill(STEAM_WATER, BlockState::of(&LIQUID_TANK));
    for (&firebox, &boiler) in FIREBOXES.iter().zip(&BOILERS) {
        set.place(firebox, FIREBOX.with(&[("facing", "south")]).into());
        set.place(boiler, BOILER.with(&[("facing", "south")]).into());
    }
    liquid_tubes(set, &boiler_water_line());
    pressure_tubes(set, &steam_pressure_line());
}

fn boiler_water_line() -> Vec<Pos> {
    vec![
        Pos::new(-16, 1, -8),
        Pos::new(-15, 1, -8),
        Pos::new(-14, 1, -8),
    ]
}

fn steam_pressure_line() -> Vec<Pos> {
    vec![
        Pos::new(-16, 2, -7),
        Pos::new(-15, 2, -7),
        Pos::new(-14, 2, -7),
        Pos::new(-13, 2, -7),
        Pos::new(-13, 2, -6),
        Pos::new(-13, 1, -6),
        Pos::new(-13, 0, -6),
        Pos::new(-13, 0, -5),
    ]
}

/// Cooling water tank, timber liquid tube lift, and vertical sealed liquid tube riser to y = 5.
fn cooling_riser(set: &Set) {
    set.fill(COOLING_TANK, BlockState::of(&LIQUID_TANK));
    set.place(WATER_LIFT, BlockState::of(&TIMBER_LIQUID_TUBE));
    engine(set, WATER_LIFT_ENGINE, Dir::East);
    sealed_tubes(set, &sealed_riser());
}

fn sealed_riser() -> Vec<Pos> {
    (0..=5).map(|y| Pos::new(-10, y, -7)).collect()
}

/// Four product tanks and distribution lines: tar, lubricant to DIGGER, diesel to hall, gas to Pad P.
fn product_lines(set: &Set) {
    set.fill(TAR_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &tar_line());

    set.fill(LUBE_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &lubricant_line());

    liquid_tubes(set, &diesel_line());

    set.fill(GAS_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &gas_line());
}

fn tar_line() -> Vec<Pos> {
    vec![Pos::new(-9, 1, -4), Pos::new(-9, 1, -3)]
}

/// Lubricant leaves the tank north to z = -10, bridges over the jetty water line at x = 12, then
/// runs east around DIGGER's frame and enters its top at (33, 1, 2).
pub fn lubricant_line() -> Vec<Pos> {
    let mut blocks = vec![
        Pos::new(-9, 2, -6),
        Pos::new(-9, 2, -7),
        Pos::new(-8, 0, -8),
        Pos::new(-8, 0, -9),
        Pos::new(-8, 0, -10),
    ];
    blocks.extend((-7..=11).map(|x| Pos::new(x, 0, -10)));
    // Bridge two blocks over the lake water line at x = 12 so the two liquids never touch.
    blocks.extend([
        Pos::new(11, 1, -10),
        Pos::new(11, 2, -10),
        Pos::new(12, 2, -10),
        Pos::new(13, 2, -10),
        Pos::new(13, 1, -10),
    ]);
    blocks.extend((13..=39).map(|x| Pos::new(x, 0, -10)));
    blocks.extend((-9..=2).map(|z| Pos::new(39, 0, z)));
    blocks.extend((33..=39).rev().map(|x| Pos::new(x, 1, 2)));
    blocks
}

fn diesel_line() -> Vec<Pos> {
    let mut blocks = vec![
        Pos::new(-9, 3, -5),
        Pos::new(-8, 3, -5),
        Pos::new(-7, 3, -5),
        Pos::new(-7, 2, -5),
        Pos::new(-7, 1, -5),
        Pos::new(-7, 0, -5),
    ];
    blocks.extend((-6..=3).map(|x| Pos::new(x, 0, -5)));
    blocks
}

fn gas_line() -> Vec<Pos> {
    // Runs overhead at y = 6 so the camera entering the hall at y = 3..4 passes well underneath,
    // arriving directly atop Pad P's gas tank at (13, PAD_P_TOP + 3, 14).
    let mut blocks = vec![
        Pos::new(-9, 4, -4),
        Pos::new(-8, 4, -4),
        Pos::new(-7, 6, -4),
        Pos::new(-6, 6, -4),
    ];
    blocks.extend((-3..=8).map(|z| Pos::new(-6, 6, z)));
    blocks.extend((-5..=13).map(|x| Pos::new(x, 6, 8)));
    blocks.extend((9..=14).map(|z| Pos::new(13, 6, z)));
    let pad_top_tank = PAD_P_TOP + 2;
    blocks.extend((pad_top_tank + 1..=5).rev().map(|y| Pos::new(13, y, 14)));
    blocks
}

/// Fuels the refinery's boilers and water lift and prefills crude, steam, water and product lines.
/// Fired at the start of S5 (~42 s before S6 arrives) so the column warms into its band naturally.
pub fn start(set: &Set) {
    for firebox in FIREBOXES {
        fuel_item(set, firebox, "cogwork:coke");
    }
    fuel(set, WATER_LIFT_ENGINE);

    prefill(
        set,
        CRUDE_TANK.blocks().chain(crude_line()).chain([REFINERY_AT]),
        "cogwork:oil",
        [3, 4],
    );
    prefill(
        set,
        STEAM_WATER
            .blocks()
            .chain(boiler_water_line())
            .chain(BOILERS),
        "base:water",
        [1, 1],
    );
    prefill(
        set,
        steam_pressure_line().into_iter().chain(HEATERS),
        "cogwork:steam",
        [1, 1],
    );
    prefill(
        set,
        COOLING_TANK
            .blocks()
            .chain([WATER_LIFT])
            .chain(sealed_riser())
            .chain([CONDENSER]),
        "base:water",
        [1, 1],
    );
    prefill(
        set,
        tar_line().into_iter().chain(TAR_TANK.blocks()),
        "cogwork:tar",
        [1, 3],
    );
    prefill(
        set,
        lubricant_line().into_iter().chain(LUBE_TANK.blocks()),
        "cogwork:lubricant",
        [1, 3],
    );
    prefill(set, diesel_line().into_iter(), "cogwork:diesel", [1, 2]);
    prefill(
        set,
        gas_line().into_iter().chain(GAS_TANK.blocks()),
        "cogwork:gas",
        [1, 3],
    );
}

/// Follows the oil line east from the seep to Pad R, then sweeps smoothly round the south of the
/// refinery column, opening its screen and showing the asphalt road and diesel line into the hall.
pub fn shots(film: Film) -> Film {
    let from_seep = seep::end_eye();
    let meet_orbit = on_orbit(ORBIT_CENTER, ORBIT_OUT, ORBIT_UP[0], ORBIT_ARC[0]);
    let approach = [
        from_seep,
        lowered([-24.0, 7.2, -3.5]),
        lowered([-19.5, 6.2, -1.0]),
        meet_orbit,
    ];
    let refinery_look = lowered(above(REFINERY_AT, 1.6));

    film.title("cogwork.film.liquid_tubes", 5.0)
        .shot(
            flyby(&approach)
                .look(Look::Pan(seep::SEEP_LOOK, refinery_look))
                .ease(speeds(1.2, 0.9))
                .secs(7.0),
        )
        .title("cogwork.film.t4_refinery", 5.5)
        .menu("cogwork:refinery", lowered_block(REFINERY_AT), 5.0)
        .shot(
            orbit(ORBIT_CENTER, ORBIT_OUT)
                .arc(ORBIT_ARC[0], 92.0)
                .height(ORBIT_UP[0], 1.75)
                .ease(speeds(0.9, 0.8))
                .secs(7.5),
        )
        .title("cogwork.film.t4_asphalt", 5.0)
        .shot(
            orbit(ORBIT_CENTER, ORBIT_OUT)
                .arc(92.0, ORBIT_ARC[1])
                .height(1.75, ORBIT_UP[1])
                .ease(speeds(0.8, 0.9))
                .secs(7.0),
        )
}
