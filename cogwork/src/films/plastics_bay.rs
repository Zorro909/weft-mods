//! Section 8 (Tier 5a): plastics, sulfuric acid and ore leaching on Pad P, terraced into the
//! slope south of the hall's east end. All five machines stand in one tight row facing north
//! toward the hall and camera, with chests tucked inline so nothing blocks the lens: a
//! polymerizer, two molding presses, an acid tank brewing sulfuric acid into glowing pressure
//! tubes, and a bubbling leaching vat.

use showcase_api::{Film, Look, Set, dolly, flyby};
use weft_sdk::Dir;

use super::camera::speeds;
use super::crafting;
use super::parts::{
    CHEST, engine, fuel, liquid_tubes, prefill, pressure_tubes, shafts, stock_group, stock_rubble,
    tubes,
};
use super::site::{PAD_P_TOP, above, lowered, lowered_block};
use crate::acid_tank::{ACID_TANK, BREW};
use crate::leaching_vat::{LEACH, LEACHING_VAT};
use crate::lib::{Area, BlockState, Pos};
use crate::molding_press::MOLDING_PRESS;
use crate::polymerizer::POLYMERIZER;
use crate::rubble::Grade;
use crate::tank::LIQUID_TANK;

/// Floor height above Pad P's gravel, in blocks from the works.
const FLOOR: i32 = PAD_P_TOP + 1;

/// Five machines standing along z = 14 facing north, with output chests tucked inline so nothing
/// sits between the camera and the machines.
const POLYMERIZER_AT: Pos = Pos::new(15, FLOOR, 14);
const SHEET_PRESS_AT: Pos = Pos::new(17, FLOOR, 14);
const PANEL_PRESS_AT: Pos = Pos::new(19, FLOOR, 14);
const PANEL_CHEST: Pos = Pos::new(20, FLOOR, 14);
const ACID_TANK_AT: Pos = Pos::new(21, FLOOR, 14);
const LEACH_VAT_AT: Pos = Pos::new(23, FLOOR, 14);
const PURE_CHEST: Pos = Pos::new(24, FLOOR, 14);

/// Local gas buffer tank on the west end and water tank directly behind the acid tank.
const GAS_TANK: Area = Area {
    lo: Pos::new(13, FLOOR, 14),
    hi: Pos::new(13, FLOOR + 1, 14),
};
const ACID_WATER: Area = Area {
    lo: Pos::new(21, FLOOR, 16),
    hi: Pos::new(21, FLOOR + 1, 16),
};

/// Ember engines assisting the torque shaft behind the row.
const ENGINES: [(Pos, Dir); 3] = [
    (Pos::new(15, FLOOR, 16), Dir::North),
    (Pos::new(18, FLOOR, 16), Dir::North),
    (Pos::new(23, FLOOR, 16), Dir::North),
];

/// Where S8 hands the camera off to S9, gliding smoothly north-east over the terrace edge toward
/// the sorting tube on one heading.
pub const END_EYE: [f64; 3] = lowered([24.0, FLOOR as f64 + 1.8, 10.6]);
pub const END_LOOK: [f64; 3] = lowered([27.2, FLOOR as f64 + 0.5, 11.2]);

pub fn build(set: &Set) {
    // Power backbone behind the row, climbing cleanly from the hall's torque shaft up to Pad P.
    shafts(set, Pos::new(14, 0, 6), Pos::new(14, 0, 7));
    shafts(set, Pos::new(14, -1, 7), Pos::new(14, -1, 15));
    shafts(set, Pos::new(14, 0, 15), Pos::new(14, FLOOR, 15));
    shafts(set, Pos::new(15, FLOOR, 15), Pos::new(23, FLOOR, 15));
    for (at, toward) in ENGINES {
        engine(set, at, toward);
    }

    // Polymerizer and gas buffer tank.
    set.fill(GAS_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &[Pos::new(14, FLOOR, 14)]);
    set.place(
        POLYMERIZER_AT,
        POLYMERIZER.with(&[("facing", "north")]).into(),
    );
    tubes(set, Pos::new(15, FLOOR, 13), Pos::new(16, FLOOR, 13));
    tubes(set, Pos::new(16, FLOOR, 14), Pos::new(16, FLOOR, 14));

    // Sheet molding press -> panel molding press -> inline composite panel chest.
    set.place(
        SHEET_PRESS_AT,
        MOLDING_PRESS.with(&[("facing", "north")]).into(),
    );
    tubes(set, Pos::new(17, FLOOR, 13), Pos::new(18, FLOOR, 13));
    tubes(set, Pos::new(18, FLOOR, 14), Pos::new(18, FLOOR, 14));
    set.place(
        PANEL_PRESS_AT,
        MOLDING_PRESS.with(&[("facing", "north")]).into(),
    );
    tubes(set, Pos::new(19, FLOOR, 13), Pos::new(20, FLOOR, 13));
    set.place(PANEL_CHEST, BlockState::of(&CHEST));

    // Acid tank, rear water tank bridged over the shaft, glowing pressure tube, and leaching vat.
    set.fill(ACID_WATER, BlockState::of(&LIQUID_TANK));
    liquid_tubes(
        set,
        &[Pos::new(21, FLOOR + 1, 15), Pos::new(21, FLOOR + 1, 14)],
    );
    set.place(ACID_TANK_AT, ACID_TANK.with(&[("facing", "north")]).into());
    pressure_tubes(set, &acid_line());
    set.place(
        LEACH_VAT_AT,
        LEACHING_VAT.with(&[("facing", "north")]).into(),
    );
    tubes(set, Pos::new(23, FLOOR, 13), Pos::new(24, FLOOR, 13));
    set.place(PURE_CHEST, BlockState::of(&CHEST));
}

fn acid_line() -> Vec<Pos> {
    vec![
        Pos::new(21, FLOOR, 13),
        Pos::new(22, FLOOR, 13),
        Pos::new(22, FLOOR, 14),
    ]
}

/// Fuels Pad P's engines, stocks catalysts, molds, pellets, sheets, steel, sulfur and Rich Ore
/// Rubble, and prefills gas, water and sulfuric acid. Fired during S7's benches shot (~15 s before
/// S8 arrives) so every machine is mid-cycle on camera.
pub fn start(set: &Set) {
    for (at, _) in ENGINES {
        fuel(set, at);
    }
    prefill(
        set,
        GAS_TANK
            .blocks()
            .chain([Pos::new(14, FLOOR, 14), POLYMERIZER_AT]),
        "cogwork:gas",
        [1, 1],
    );
    stock_group(
        set,
        POLYMERIZER_AT,
        "catalyst",
        &[("cogwork:catalyst", 1), ("cogwork:catalyst", 1)],
    );

    stock_group(set, SHEET_PRESS_AT, "mold", &[("cogwork:sheet_mold", 1)]);
    stock_group(
        set,
        SHEET_PRESS_AT,
        "input",
        &[("cogwork:plastic_pellet", 64); 2],
    );

    stock_group(set, PANEL_PRESS_AT, "mold", &[("cogwork:panel_mold", 1)]);
    stock_group(
        set,
        PANEL_PRESS_AT,
        "input",
        &[("cogwork:plastic_sheet", 64), ("cogwork:steel_ingot", 64)],
    );

    prefill(
        set,
        ACID_WATER
            .blocks()
            .chain([Pos::new(21, FLOOR + 1, 15), Pos::new(21, FLOOR + 1, 14)]),
        "base:water",
        [1, 1],
    );
    stock_group(set, ACID_TANK_AT, "input", &[("cogwork:sulfur", 64); 3]);
    prefill(
        set,
        acid_line().into_iter(),
        "cogwork:sulfuric_acid",
        [1, 1],
    );

    let world = set.world();
    if let Some(brew) = world.component_mut(world.block_entity(set.at(ACID_TANK_AT)), &BREW) {
        brew.water = 2_000;
        brew.acid = 1_500;
    }
    if let Some(leach) = world.component_mut(world.block_entity(set.at(LEACH_VAT_AT)), &LEACH) {
        leach.acid = 1_200;
    }
    stock_rubble(set, LEACH_VAT_AT, "input", "iron", Grade::Rich, 3);
}

/// Exits the hall cleanly down the center of the open bay between the x = 14 and x = 18 posts,
/// rising onto the Pad P terrace and gliding close past all five machines.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let poly_look = lowered(above(POLYMERIZER_AT, 0.2));
    let press_look = lowered(above(PANEL_PRESS_AT, 0.2));
    let acid_look = lowered(above(ACID_TANK_AT, 0.2));
    let leach_look = lowered(above(LEACH_VAT_AT, 0.2));

    let exit = [
        crafting::end_eye(),
        lowered([16.2, 4.4, 6.2]),
        lowered([16.2, f + 1.6, 10.8]),
    ];
    let at_poly = lowered([17.6, f + 1.5, 10.8]);
    let at_press = lowered([20.0, f + 1.5, 10.8]);
    let at_acid = lowered([22.4, f + 1.6, 10.8]);

    film.shot(
        flyby(&exit)
            .look(Look::Pan(crafting::BENCH_LOOK, poly_look))
            .ease(speeds(0.25, 0.8))
            .secs(5.5),
    )
    .title("cogwork.film.t5_plastics", 5.5)
    .menu("cogwork:polymerizer", lowered_block(POLYMERIZER_AT), 4.0)
    .shot(
        dolly(exit[2], at_poly)
            .look(Look::Pan(poly_look, press_look))
            .ease(speeds(0.8, 0.6))
            .secs(5.2),
    )
    .menu("cogwork:molding_press", lowered_block(PANEL_PRESS_AT), 4.0)
    .shot(
        dolly(at_poly, at_press)
            .look(Look::Pan(press_look, acid_look))
            .ease(speeds(0.6, 0.6))
            .secs(5.2),
    )
    .title("cogwork.film.t5_acid", 4.5)
    .menu("cogwork:acid_tank", lowered_block(ACID_TANK_AT), 4.0)
    .shot(
        dolly(at_press, at_acid)
            .look(Look::Pan(acid_look, leach_look))
            .ease(speeds(0.6, 0.6))
            .secs(5.2),
    )
    .title("cogwork.film.t5_leach", 5.0)
    .menu("cogwork:leaching_vat", lowered_block(LEACH_VAT_AT), 4.0)
    .shot(
        dolly(at_acid, END_EYE)
            .look(Look::Pan(leach_look, END_LOOK))
            .ease(speeds(0.6, 0.9))
            .secs(5.0),
    )
}
