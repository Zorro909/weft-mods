//! Section 3 (Tier 2): the steel and steam foundry on Pad S3. A coke oven bakes coal into coke,
//! sulfur and creosote (piped east to S4's flotation cell); a blast furnace fed by a driven
//! bellows smelts steel blooms; a hot bloom waits on the forge anvil beside stacked steel blocks;
//! and two firebox-and-boiler pairs raise steam for a steam engine (whose torque shaft runs east
//! to S4's Steel Excavator) and an automated steam trip hammer.

use showcase_api::{Film, Look, Set, dolly};
use weft_sdk::Dir;

use super::camera::speeds;
use super::iron_yard;
use super::parts::{
    CHEST, area, engine, fuel, fuel_item, liquid_tubes, prefill, shafts, stock_group, tubes,
};
use super::site::{VALLEY_TOP, above, lowered, lowered_block};
use crate::bellows::DRIVEN_BELLOWS;
use crate::blast_furnace::{BLAST_FURNACE, FIRE_BRICKS};
use crate::coke_oven::{COKE_OVEN, COKE_OVEN_BRICKS, COKE_OVEN_DRAIN};
use crate::firebox::{BOILER, FIREBOX};
use crate::forge_anvil::{FORGE_ANVIL, FORGING};
use crate::lib::{Area, BlockState, Pos};
use crate::steam_engine::STEAM_ENGINE;
use crate::steel::STEEL_BLOCK;
use crate::tank::LIQUID_TANK;
use crate::trip_hammer::TRIP_HAMMER;

/// Floor height above Pad S3's gravel, in blocks from the works.
const FLOOR: i32 = VALLEY_TOP + 1;

/// Coke oven (3 by 3 by 3) and its creosote drain line running east to Pad S4.
const COKE_DOOR: Pos = Pos::new(19, FLOOR + 1, 31);
const COKE_DRAIN: Pos = Pos::new(20, FLOOR, 30);
const COKE_CHEST: Pos = Pos::new(18, FLOOR, 32);

/// Blast furnace (3 by 3 by 4), driven bellows and its 3 ember engines.
const BLAST_DOOR: Pos = Pos::new(26, FLOOR, 31);
const BLAST_BELLOWS: Pos = Pos::new(24, FLOOR, 30);
const BLAST_ENGINES: [(Pos, Dir); 3] = [
    (Pos::new(22, FLOOR, 30), Dir::East),
    (Pos::new(23, FLOOR + 1, 30), Dir::Down),
    (Pos::new(23, FLOOR, 29), Dir::South),
];
const BLAST_CHEST: Pos = Pos::new(28, FLOOR, 32);

/// Forge anvil with a hot bloom, flanked by steel blocks.
const ANVIL_AT: Pos = Pos::new(26, FLOOR, 35);

/// Steam plant: water tank, two firebox+boiler pairs, steam engine and trip hammer.
const STEAM_WATER: Area = Area {
    lo: Pos::new(32, FLOOR, 29),
    hi: Pos::new(33, FLOOR + 1, 30),
};
const FIREBOXES: [Pos; 2] = [Pos::new(33, FLOOR, 33), Pos::new(35, FLOOR, 33)];
const BOILERS: [Pos; 2] = [Pos::new(33, FLOOR + 1, 33), Pos::new(35, FLOOR + 1, 33)];
const STEAM_ENGINE_AT: Pos = Pos::new(37, FLOOR, 34);
const HAMMER_AT: Pos = Pos::new(40, FLOOR, 34);
const HAMMER_CHEST: Pos = Pos::new(41, FLOOR, 35);

/// Row where creosote leaves S3 east toward S4, and where the steam engine's shaft leaves east.
pub const CREOSOTE_Z: i32 = 27;
pub const SHAFT_Z: i32 = 32;

/// Where S3 hands the camera off to S4, heading east.
pub const END_EYE: [f64; 3] = lowered([42.5, FLOOR as f64 + 2.8, 40.0]);
pub const END_LOOK: [f64; 3] = lowered([39.0, FLOOR as f64 + 1.0, 34.0]);

pub fn build(set: &Set) {
    coke_oven(set);
    blast_furnace(set);
    anvil_area(set);
    steam_plant(set);
}

/// 3 by 3 by 3 coke oven with glowing door, creosote drain and long creosote line east to S4.
fn coke_oven(set: &Set) {
    set.fill(
        area(Pos::new(18, FLOOR, 29), Pos::new(20, FLOOR + 2, 31)),
        BlockState::of(&COKE_OVEN_BRICKS),
    );
    set.place(Pos::new(19, FLOOR + 1, 30), BlockState::AIR);
    set.place(COKE_DOOR, COKE_OVEN.with(&[("facing", "south")]).into());
    set.place(COKE_DRAIN, BlockState::of(&COKE_OVEN_DRAIN));
    tubes(set, Pos::new(19, FLOOR + 1, 32), Pos::new(19, FLOOR, 32));
    set.place(COKE_CHEST, BlockState::of(&CHEST));
    liquid_tubes(set, &creosote_line());
}

/// The creosote liquid tube run from the coke oven's drain east to S3's eastern edge.
fn creosote_line() -> Vec<Pos> {
    let mut blocks = vec![
        Pos::new(21, FLOOR, 30),
        Pos::new(21, FLOOR, 29),
        Pos::new(21, FLOOR, 28),
    ];
    blocks.extend((21..=42).map(|x| Pos::new(x, FLOOR, CREOSOTE_Z)));
    blocks
}

/// 3 by 3 by 4 blast furnace, its driven bellows and ember engines, and output chest.
fn blast_furnace(set: &Set) {
    set.fill(
        area(Pos::new(25, FLOOR, 29), Pos::new(27, FLOOR + 3, 31)),
        BlockState::of(&FIRE_BRICKS),
    );
    set.fill(
        area(Pos::new(26, FLOOR + 1, 30), Pos::new(26, FLOOR + 3, 30)),
        BlockState::AIR,
    );
    set.place(
        BLAST_DOOR,
        BLAST_FURNACE.with(&[("facing", "south")]).into(),
    );
    set.place(
        BLAST_BELLOWS,
        DRIVEN_BELLOWS.with(&[("facing", "east")]).into(),
    );
    shafts(set, Pos::new(23, FLOOR, 30), Pos::new(23, FLOOR, 30));
    for (at, toward) in BLAST_ENGINES {
        engine(set, at, toward);
    }
    tubes(set, Pos::new(26, FLOOR, 32), Pos::new(27, FLOOR, 32));
    set.place(BLAST_CHEST, BlockState::of(&CHEST));
}

/// Forge anvil loaded with a hot bloom, flanked by steel blocks.
fn anvil_area(set: &Set) {
    set.place(ANVIL_AT, FORGE_ANVIL.with(&[("facing", "south")]).into());
    for at in [
        Pos::new(24, FLOOR, 35),
        Pos::new(24, FLOOR + 1, 35),
        Pos::new(28, FLOOR, 35),
    ] {
        set.place(at, BlockState::of(&STEEL_BLOCK));
    }
}

/// Two firebox+boiler pairs feeding a steam engine and trip hammer on a steel plinth.
fn steam_plant(set: &Set) {
    set.fill(STEAM_WATER, BlockState::of(&LIQUID_TANK));
    for (&firebox, &boiler) in FIREBOXES.iter().zip(&BOILERS) {
        set.place(firebox, FIREBOX.with(&[("facing", "south")]).into());
        set.place(boiler, BOILER.with(&[("facing", "south")]).into());
    }
    liquid_tubes(set, &boiler_water_line());
    liquid_tubes(set, &steam_line());
    set.place(
        STEAM_ENGINE_AT,
        STEAM_ENGINE.with(&[("facing", "south")]).into(),
    );
    // Steel plinth flush in the gravel pad beneath the trip hammer.
    set.place(HAMMER_AT.add_y(-1), BlockState::of(&STEEL_BLOCK));
    set.place(HAMMER_AT, TRIP_HAMMER.with(&[("facing", "south")]).into());
    tubes(set, Pos::new(40, FLOOR, 35), Pos::new(40, FLOOR, 35));
    set.place(HAMMER_CHEST, BlockState::of(&CHEST));
    // Torque shaft out the steam engine's east flywheel axle and behind the trip hammer to S4.
    shafts(set, Pos::new(38, FLOOR, 34), Pos::new(38, FLOOR, SHAFT_Z));
    shafts(
        set,
        Pos::new(39, FLOOR, SHAFT_Z),
        Pos::new(42, FLOOR, SHAFT_Z),
    );
}

fn boiler_water_line() -> Vec<Pos> {
    vec![
        Pos::new(33, FLOOR + 1, 31),
        Pos::new(33, FLOOR + 1, 32),
        Pos::new(34, FLOOR + 1, 33),
    ]
}

fn steam_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (33..=40).map(|x| Pos::new(x, FLOOR + 2, 33)).collect();
    blocks.extend([
        Pos::new(37, FLOOR + 1, 33),
        Pos::new(37, FLOOR + 1, 34),
        Pos::new(40, FLOOR + 1, 33),
        Pos::new(40, FLOOR + 1, 34),
    ]);
    blocks
}

/// Stocks the coke oven, blast furnace, forge anvil, boilers and trip hammer from t = 0 so every
/// multiblock is glowing, smoking and forging by the time the camera reaches S3.
pub fn start(set: &Set) {
    stock_group(set, COKE_DOOR, "input", &[("base:coal", 64)]);
    prefill(set, creosote_line().into_iter(), "cogwork:creosote", [1, 1]);

    stock_group(set, BLAST_DOOR, "input", &[("base:iron_ingot", 64)]);
    stock_group(set, BLAST_DOOR, "fuel", &[("cogwork:coke", 64)]);
    for (at, _) in BLAST_ENGINES {
        fuel(set, at);
    }

    let world = set.world();
    if let Some(forging) = world.component_mut(world.block_entity(set.at(ANVIL_AT)), &FORGING) {
        forging.bloom = 1;
        forging.strikes = 2;
    }

    for firebox in FIREBOXES {
        fuel_item(set, firebox, "cogwork:coke");
    }
    let water = STEAM_WATER
        .blocks()
        .chain(boiler_water_line())
        .chain(BOILERS);
    prefill(set, water, "base:water", [1, 1]);
    let steam = steam_line().into_iter().chain([STEAM_ENGINE_AT, HAMMER_AT]);
    prefill(set, steam, "cogwork:steam", [1, 1]);
    stock_group(set, HAMMER_AT, "input", &[("cogwork:steel_bloom", 64); 3]);
}

/// Steady dolly east along the foundry: coke oven, blast furnace, forge anvil, firebox & boiler,
/// steam engine and trip hammer.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let coke_look = lowered(above(COKE_DOOR, 0.2));
    let blast_look = lowered([26.0, f + 1.8, 32.0]);
    let anvil_look = lowered(above(ANVIL_AT, 0.2));
    let boiler_look = lowered([34.0, f + 1.2, 33.0]);
    let engine_look = lowered(above(STEAM_ENGINE_AT, 0.3));

    let at_coke = lowered([20.5, f + 3.2, 39.5]);
    let at_blast = lowered([26.0, f + 3.5, 39.5]);
    let at_anvil = lowered([29.5, f + 2.8, 39.5]);
    let at_firebox = lowered([33.5, f + 2.8, 39.5]);
    let at_boiler = lowered([36.0, f + 2.8, 39.5]);
    let at_engine = lowered([39.0, f + 2.8, 39.8]);

    film.cue("noon_1")
        .title("cogwork.film.t2_coke", 5.5)
        .shot(
            dolly(iron_yard::END_EYE, at_coke)
                .look(Look::Pan(iron_yard::END_LOOK, coke_look))
                .ease(speeds(1.0, 0.8))
                .secs(4.5),
        )
        .menu("cogwork:coke_oven", lowered_block(COKE_DOOR), 3.5)
        .shot(
            dolly(at_coke, at_blast)
                .look(Look::Pan(coke_look, blast_look))
                .ease(speeds(0.8, 0.8))
                .secs(5.0),
        )
        .title("cogwork.film.t2_blast", 5.0)
        .menu("cogwork:blast_furnace", lowered_block(BLAST_DOOR), 3.5)
        .shot(
            dolly(at_blast, at_anvil)
                .look(Look::Pan(blast_look, anvil_look))
                .ease(speeds(0.8, 0.8))
                .secs(4.5),
        )
        .title("cogwork.film.t2_anvil", 4.5)
        .shot(
            dolly(at_anvil, at_firebox)
                .look(Look::Pan(anvil_look, boiler_look))
                .ease(speeds(0.8, 0.8))
                .secs(4.5),
        )
        .title("cogwork.film.t2_steam", 5.0)
        .menu("cogwork:firebox", lowered_block(FIREBOXES[0]), 3.5)
        .shot(
            dolly(at_firebox, at_boiler)
                .look_at(boiler_look)
                .ease(speeds(0.8, 0.7))
                .secs(4.5),
        )
        .menu("cogwork:boiler", lowered_block(BOILERS[0]), 3.5)
        .shot(
            dolly(at_boiler, at_engine)
                .look(Look::Pan(boiler_look, engine_look))
                .ease(speeds(0.7, 0.8))
                .secs(4.5),
        )
        .title("cogwork.film.t2_hammer", 5.0)
        .menu("cogwork:steam_engine", lowered_block(STEAM_ENGINE_AT), 3.5)
        .shot(
            dolly(at_engine, END_EYE)
                .look(Look::Pan(engine_look, END_LOOK))
                .ease(speeds(0.8, 0.9))
                .secs(4.5),
        )
        .menu("cogwork:trip_hammer", lowered_block(HAMMER_AT), 3.5)
        .shot(
            dolly(END_EYE, lowered([45.5, f + 2.9, 40.2]))
                .look_at(END_LOOK)
                .ease(speeds(0.9, 1.0))
                .secs(4.0),
        )
}
