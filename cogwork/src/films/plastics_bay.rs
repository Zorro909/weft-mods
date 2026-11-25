//! Section 8 (Tier 5a): plastics, sulfuric acid and ore leaching on Pad P, terraced into the
//! slope south of the hall's east end. All five machines stand in one open row along z = 14 facing
//! east so each feeds the next inline through directing or pressure tubes with zero front-row
//! clutter between the camera and the machines, powered from behind by three steam engines
//! (3 × 160 = 480 E/t against the 370 E/t drawn by all five machines).

use showcase_api::{Film, Look, Set, dolly, flyby};

use super::camera::speeds;
use super::crafting;
use super::parts::{
    CHEST, fuel_item, liquid_tubes, prefill, pressure_tubes, shafts, stock_group, stock_rubble,
};
use super::site::{PAD_P_TOP, above, lowered, lowered_block};
use crate::acid_tank::{ACID_TANK, BREW};
use crate::firebox::{BOILER, FIREBOX};
use crate::leaching_vat::{LEACH, LEACHING_VAT};
use crate::lib::{Area, BlockState, Pos};
use crate::molding_press::MOLDING_PRESS;
use crate::polymerizer::POLYMERIZER;
use crate::rubble::Grade;
use crate::steam_engine::STEAM_ENGINE;
use crate::tank::LIQUID_TANK;
use crate::tube::DIRECTING_TUBE;

/// Floor height above Pad P's gravel, in blocks from the works.
const FLOOR: i32 = PAD_P_TOP + 1;

/// Five machines standing inline along z = 14 facing east, with directing tubes, acid buffer tank
/// and output chests inline between them so the entire north front (z = 13) stays open gravel.
const POLYMERIZER_AT: Pos = Pos::new(15, FLOOR, 14);
const PELLET_TUBE: Pos = Pos::new(16, FLOOR, 14);
const SHEET_PRESS_AT: Pos = Pos::new(17, FLOOR, 14);
const SHEET_TUBE: Pos = Pos::new(18, FLOOR, 14);
const PANEL_PRESS_AT: Pos = Pos::new(19, FLOOR, 14);
const PANEL_CHEST: Pos = Pos::new(20, FLOOR, 14);
const ACID_TANK_AT: Pos = Pos::new(21, FLOOR, 14);
const ACID_BUFFER: Pos = Pos::new(23, FLOOR, 14);
const LEACH_VAT_AT: Pos = Pos::new(25, FLOOR, 14);
const PURE_CHEST: Pos = Pos::new(26, FLOOR, 14);

/// Local gas buffer tank on the west end and water tank directly behind the acid tank.
const GAS_TANK: Area = Area {
    lo: Pos::new(13, FLOOR, 14),
    hi: Pos::new(13, FLOOR + 1, 14),
};
const ACID_WATER: Area = Area {
    lo: Pos::new(21, FLOOR, 16),
    hi: Pos::new(21, FLOOR + 1, 16),
};

/// Dedicated rear steam plant driving Pad P's torque shaft: 3 firebox+boiler pairs (60 mB/tick
/// steam, with 96,000 mB gravity water above boiler level) and 3 steam engines (3 × 160 = 480 E/t
/// against the 370 E/t drawn by all five machines).
const STEAM_WATER: Area = Area {
    lo: Pos::new(15, FLOOR, 18),
    hi: Pos::new(19, FLOOR + 1, 18),
};
const FIREBOXES: [Pos; 3] = [
    Pos::new(16, FLOOR, 17),
    Pos::new(17, FLOOR, 17),
    Pos::new(18, FLOOR, 17),
];
const BOILERS: [Pos; 3] = [
    Pos::new(16, FLOOR + 1, 17),
    Pos::new(17, FLOOR + 1, 17),
    Pos::new(18, FLOOR + 1, 17),
];
const STEAM_ENGINES: [Pos; 3] = [
    Pos::new(15, FLOOR, 16),
    Pos::new(19, FLOOR, 16),
    Pos::new(24, FLOOR, 16),
];

/// Where S8 hands the camera off to S9, high over Pad P's gravel (4+ blocks above the adjacent
/// grass terraces) while holding on the leaching vat.
pub const END_EYE: [f64; 3] = lowered([25.5, FLOOR as f64 + 8.5, 11.8]);
pub const END_LOOK: [f64; 3] = lowered(above(LEACH_VAT_AT, 0.2));

pub fn build(set: &Set) {
    // Dedicated torque shaft behind the row, driven by three steam engines (480 E/t in vs 370 E/t
    // drawn across all five machines).
    shafts(set, Pos::new(15, FLOOR, 15), Pos::new(25, FLOOR, 15));
    set.fill(STEAM_WATER, BlockState::of(&LIQUID_TANK));
    for (&firebox, &boiler) in FIREBOXES.iter().zip(&BOILERS) {
        set.place(firebox, FIREBOX.with(&[("facing", "north")]).into());
        set.place(boiler, BOILER.with(&[("facing", "north")]).into());
    }
    for at in STEAM_ENGINES {
        // Facing east puts the north flywheel axle directly against the shaft at z = 15.
        set.place(at, STEAM_ENGINE.with(&[("facing", "east")]).into());
    }
    pressure_tubes(set, &steam_line());

    // Polymerizer and gas buffer tank.
    set.fill(GAS_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &[Pos::new(14, FLOOR, 14)]);
    set.place(
        POLYMERIZER_AT,
        POLYMERIZER.with(&[("facing", "east")]).into(),
    );
    set.place(PELLET_TUBE, DIRECTING_TUBE.with(&[("out", "east")]).into());

    // Sheet molding press -> directing tube -> panel molding press -> inline composite panel chest.
    set.place(
        SHEET_PRESS_AT,
        MOLDING_PRESS.with(&[("facing", "east")]).into(),
    );
    set.place(SHEET_TUBE, DIRECTING_TUBE.with(&[("out", "east")]).into());
    set.place(
        PANEL_PRESS_AT,
        MOLDING_PRESS.with(&[("facing", "east")]).into(),
    );
    set.place(PANEL_CHEST, BlockState::of(&CHEST));

    // Acid tank, rear water tank bridged over the shaft, inline pressure tubes + 16,000 mB acid
    // buffer tank, leaching vat, and inline pure-ore chest.
    set.fill(ACID_WATER, BlockState::of(&LIQUID_TANK));
    liquid_tubes(
        set,
        &[Pos::new(21, FLOOR + 1, 15), Pos::new(21, FLOOR + 1, 14)],
    );
    set.place(ACID_TANK_AT, ACID_TANK.with(&[("facing", "east")]).into());
    set.place(ACID_BUFFER, BlockState::of(&LIQUID_TANK));
    pressure_tubes(set, &acid_line());
    set.place(
        LEACH_VAT_AT,
        LEACHING_VAT.with(&[("facing", "east")]).into(),
    );
    set.place(PURE_CHEST, BlockState::of(&CHEST));
}

fn steam_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (15..=24).map(|x| Pos::new(x, FLOOR + 2, 17)).collect();
    for x in [15, 19, 24] {
        blocks.push(Pos::new(x, FLOOR + 1, 17));
        blocks.push(Pos::new(x, FLOOR + 1, 16));
    }
    blocks
}

fn acid_line() -> Vec<Pos> {
    vec![Pos::new(22, FLOOR, 14), Pos::new(24, FLOOR, 14)]
}

/// Fuels Pad P's boilers, stocks catalysts, molds, pellets, sheets, steel, sulfur and Rich Ore
/// Rubble, and prefills gas, water, steam and sulfuric acid. Fired at the start of S8 (~9 s before
/// the polymerizer menu and ~15 s before the molding press menu) so every machine is mid-cycle and
/// the 64-use molds still have ~30 uses left on camera.
pub fn start(set: &Set) {
    for firebox in FIREBOXES {
        fuel_item(set, firebox, "cogwork:coke");
    }
    prefill(
        set,
        STEAM_WATER.blocks().chain(BOILERS),
        "base:water",
        [1, 1],
    );
    prefill(
        set,
        steam_line().into_iter().chain(STEAM_ENGINES),
        "cogwork:steam",
        [1, 1],
    );
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
    prefill(
        set,
        [ACID_BUFFER].into_iter(),
        "cogwork:sulfuric_acid",
        [1, 8],
    );

    let world = set.world();
    if let Some(brew) = world.component_mut(world.block_entity(set.at(ACID_TANK_AT)), &BREW) {
        brew.water = 2_000;
        brew.acid = 0;
    }
    if let Some(leach) = world.component_mut(world.block_entity(set.at(LEACH_VAT_AT)), &LEACH) {
        leach.acid = 1_200;
    }
    stock_rubble(set, LEACH_VAT_AT, "input", "iron", Grade::Rich, 3);
}

/// Exits the hall cleanly through its wide-open east gable end (clear of every post, beam, and
/// chest), climbs well above Pad P's retaining wall, then sweeps west-to-east past all five
/// unobstructed machines with every title clear of its screen.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let east_look = lowered([31.5, 9.5, 12.5]);
    let poly_look = lowered(above(POLYMERIZER_AT, 0.2));
    let press_look = lowered(above(PANEL_PRESS_AT, 0.2));
    let acid_look = lowered(above(ACID_TANK_AT, 0.2));

    // Climb inside the hall under the y = 6 east beam and exit through the south-middle of the
    // east opening (4+ blocks from both the yellow gantry frame at x = 28, z = 1 and the SE post
    // at x = 22, z = 7), then climb straight up well above the roof eave and grass bank onto Pad P.
    let east_door = lowered([24.8, 4.6, 3.6]);
    let at_poly_start = lowered([15.6, f + 2.2, 11.2]);
    let over_wall = [
        east_door,
        lowered([25.4, 12.0, 5.4]),
        lowered([21.8, 12.0, 11.4]),
        at_poly_start,
    ];
    let to_poly = lowered([16.4, f + 2.0, 11.2]);
    let at_poly = lowered([18.0, f + 2.0, 11.2]);
    let to_acid = lowered([20.4, f + 2.4, 11.5]);
    let at_acid = lowered([22.0, f + 2.5, 11.5]);
    let to_leach = lowered([24.2, f + 2.6, 11.6]);
    let at_leach = lowered([25.0, f + 2.8, 11.6]);

    film.cue("start_plastics")
        .shot(
            dolly(crafting::end_eye(), east_door)
                .look(Look::Pan(crafting::BENCH_LOOK, east_look))
                .ease(speeds(0.25, 1.3))
                .secs(4.8),
        )
        .shot(
            flyby(&over_wall)
                .look(Look::Pan(east_look, poly_look))
                .ease(speeds(1.3, 0.8))
                .secs(7.0),
        )
        .title("cogwork.film.t5_plastics", 3.0)
        .shot(
            dolly(at_poly_start, to_poly)
                .look_at(poly_look)
                .ease(speeds(0.8, 0.6))
                .secs(3.2),
        )
        .menu("cogwork:polymerizer", lowered_block(POLYMERIZER_AT), 3.8)
        .shot(
            dolly(to_poly, at_poly)
                .look(Look::Pan(poly_look, press_look))
                .ease(speeds(0.6, 0.6))
                .secs(4.2),
        )
        .menu("cogwork:molding_press", lowered_block(PANEL_PRESS_AT), 3.8)
        .shot(
            dolly(at_poly, to_acid)
                .look(Look::Pan(press_look, acid_look))
                .ease(speeds(0.6, 0.6))
                .secs(4.2),
        )
        .title("cogwork.film.t5_acid", 3.0)
        .shot(
            dolly(to_acid, at_acid)
                .look_at(acid_look)
                .ease(speeds(0.6, 0.6))
                .secs(3.2),
        )
        .menu("cogwork:acid_tank", lowered_block(ACID_TANK_AT), 3.8)
        .shot(
            dolly(at_acid, to_leach)
                .look(Look::Pan(acid_look, END_LOOK))
                .ease(speeds(0.6, 0.6))
                .secs(4.2),
        )
        .title("cogwork.film.t5_leach", 3.0)
        .shot(
            dolly(to_leach, at_leach)
                .look_at(END_LOOK)
                .ease(speeds(0.6, 0.6))
                .secs(3.2),
        )
        .menu("cogwork:leaching_vat", lowered_block(LEACH_VAT_AT), 3.8)
        .shot(
            dolly(at_leach, END_EYE)
                .look_at(END_LOOK)
                .ease(speeds(0.6, 0.8))
                .secs(4.4),
        )
}
