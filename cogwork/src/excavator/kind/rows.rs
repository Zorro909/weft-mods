//! The rows of the quarry kinds, from the first quarry to the last: the Timber Quarry, the Iron
//! Quarry, the Steel Excavator and the Gantry Excavator.

use weft_sdk::{Item, Menu, Tag};

use super::super::bit::{DIAMOND_BIT, IRON_BIT, STEEL_BIT, STONE_BIT};
use super::super::rig::{GANTRY_RIG, IRON_RIG, STEEL_RIG, TIMBER_RIG};
use super::super::{
    COMPOSITE_FRAME, EXCAVATOR, IRON_FRAME, IRON_QUARRY, LASHED_FRAME, STEEL_EXCAVATOR,
    STEEL_FRAME, TIMBER_QUARRY,
};
use super::{Cost, Depth, Fit, Inside, Kind, Material, Standby};
use crate::plastics::COMPOSITE_PANEL;
use crate::sounds;

static LOGS: Tag = Tag::named("logs");
static STRING: Item = Item::named("base:string");
static IRON_INGOTS: Tag = Tag::named("ingots/iron");
static STEEL_INGOTS: Tag = Tag::named("ingots/steel");
static TIMBER_MENU: Menu = Menu::named("timber_quarry");
static IRON_MENU: Menu = Menu::named("iron_quarry");
static STEEL_MENU: Menu = Menu::named("steel_excavator");
static EXCAVATOR_MENU: Menu = Menu::named("excavator");

pub static TIMBER: Kind = Kind {
    block: &TIMBER_QUARRY,
    hardness: 2.0,
    sound_group: "base:block.wood",
    menu: &TIMBER_MENU,
    frame: &LASHED_FRAME,
    frame_cost: &[
        Cost {
            material: Material::Tagged(&LOGS),
            count: 1,
        },
        Cost {
            material: Material::Item(&STRING),
            count: 1,
        },
    ],
    frame_batch: 1,
    standby: Standby::Flat(4),
    inside: Inside {
        default: 5,
        largest: Some(7),
    },
    depth: Depth::Below(16),
    bits: &[
        Fit {
            bit: &STONE_BIT,
            draw: 12,
        },
        Fit {
            bit: &IRON_BIT,
            draw: 18,
        },
    ],
    bare: None,
    rubble_per_ore: 1.0,
    rig: &TIMBER_RIG,
    dig_sound: &sounds::TIMBER_DIG,
    frame_sound: &sounds::TIMBER_FRAME,
};

/// The quarry after the Timber Quarry: a frame of iron bars, wider and four times as deep, and
/// a quarter more Ore Rubble. Each bit it takes is a row of `bits`.
pub static IRON: Kind = Kind {
    block: &IRON_QUARRY,
    hardness: 3.0,
    sound_group: "base:block.stone",
    menu: &IRON_MENU,
    frame: &IRON_FRAME,
    frame_cost: &[Cost {
        material: Material::Tagged(&IRON_INGOTS),
        count: 1,
    }],
    frame_batch: 1,
    standby: Standby::BySides,
    inside: Inside {
        default: 9,
        largest: Some(17),
    },
    depth: Depth::Below(64),
    bits: &[
        Fit {
            bit: &IRON_BIT,
            draw: 60,
        },
        Fit {
            bit: &STEEL_BIT,
            draw: 90,
        },
    ],
    bare: None,
    rubble_per_ore: 1.25,
    rig: &IRON_RIG,
    dig_sound: &sounds::DRILL,
    frame_sound: &sounds::FRAME_PLACE,
};

/// The quarry after the Iron Quarry: a frame of steel lattice, a steel ingot for every four of
/// its blocks, up to 63 across and down to the world's lowest layer, and half as much Ore Rubble
/// again.
pub static STEEL: Kind = Kind {
    block: &STEEL_EXCAVATOR,
    hardness: 4.0,
    sound_group: "base:block.stone",
    menu: &STEEL_MENU,
    frame: &STEEL_FRAME,
    frame_cost: &[Cost {
        material: Material::Tagged(&STEEL_INGOTS),
        count: 1,
    }],
    frame_batch: 4,
    standby: Standby::BySides,
    inside: Inside {
        default: 9,
        largest: Some(63),
    },
    depth: Depth::Lowest,
    bits: &[
        Fit {
            bit: &STEEL_BIT,
            draw: 120,
        },
        Fit {
            bit: &DIAMOND_BIT,
            draw: 160,
        },
    ],
    bare: None,
    rubble_per_ore: 1.5,
    rig: &STEEL_RIG,
    dig_sound: &sounds::DRILL,
    frame_sound: &sounds::FRAME_PLACE,
};

/// The last quarry, the Gantry Excavator: a frame of composite panels on steel, a composite panel
/// for every four of its blocks, as large and deep as the world's settings let it, and twice the
/// Ore Rubble. With no bit it digs as excavators always have; a diamond-tipped bit digs twice as
/// hard.
pub static GANTRY: Kind = Kind {
    block: &EXCAVATOR,
    hardness: 4.0,
    sound_group: "base:block.stone",
    menu: &EXCAVATOR_MENU,
    frame: &COMPOSITE_FRAME,
    frame_cost: &[Cost {
        material: Material::Item(&COMPOSITE_PANEL),
        count: 1,
    }],
    frame_batch: 4,
    standby: Standby::BySides,
    inside: Inside {
        default: 9,
        largest: None,
    },
    depth: Depth::World,
    bits: &[Fit {
        bit: &DIAMOND_BIT,
        draw: 240,
    }],
    bare: Some(120),
    rubble_per_ore: 2.0,
    rig: &GANTRY_RIG,
    dig_sound: &sounds::DRILL,
    frame_sound: &sounds::FRAME_PLACE,
};

pub static KINDS: [&Kind; 4] = [&TIMBER, &IRON, &STEEL, &GANTRY];

#[cfg(test)]
mod tests {
    use super::super::super::Frame;
    use super::super::super::depth::Floor;
    use super::*;

    #[test]
    fn the_gantry_digs_as_ever_with_no_bit() {
        let drill = GANTRY.drill(None).expect("it digs bare");
        assert_eq!(drill.draw, 120, "120 a tick, as today");
        assert_eq!(drill.tier, u32::MAX, "it breaks all it reaches");
        assert!(drill.bit.is_none(), "and wears nothing");
    }

    #[test]
    fn the_timber_quarry_needs_a_bit_and_digs_harder_with_iron() {
        assert!(TIMBER.drill(None).is_none(), "no bit, no digging");
        let stone = TIMBER.drill(Some(&STONE_BIT)).expect("a stone bit fits");
        let iron = TIMBER.drill(Some(&IRON_BIT)).expect("an iron bit fits");
        assert_eq!((stone.draw, iron.draw), (12, 18), "work draw by the bit");
        assert!(iron.tier > stone.tier, "iron breaks more");
    }

    #[test]
    fn a_diamond_tipped_bit_works_the_gantry_twice_as_hard() {
        let diamond = GANTRY.drill(Some(&DIAMOND_BIT)).expect("it fits");
        assert_eq!(diamond.draw, 240, "twice the bare draw");
        assert_eq!(diamond.tier, DIAMOND_BIT.tier, "and breaks what it breaks");
        assert!(
            GANTRY
                .drill(Some(&STEEL_BIT))
                .is_some_and(|bare| bare.bit.is_none()),
            "a steel bit is no Gantry bit: it digs bare"
        );
        assert_eq!(GANTRY.rubble_per_ore, 2.0, "twice the Ore Rubble");
    }

    #[test]
    fn a_composite_panel_pays_for_four_gantry_frame_blocks() {
        assert_eq!(GANTRY.frame_batch, 4, "four blocks a panel");
        assert_eq!(GANTRY.frame_cost.len(), 1, "panels only");
    }

    #[test]
    fn a_bit_that_does_not_fit_is_no_bit() {
        assert!(
            !GANTRY
                .bits
                .iter()
                .any(|fit| std::ptr::eq(fit.bit, &STONE_BIT))
        );
        let drill = GANTRY.drill(Some(&STONE_BIT)).expect("bare");
        assert!(drill.bit.is_none(), "the gantry ignores a stone bit");
    }

    #[test]
    fn frames_go_up_at_the_best_drill_s_pace() {
        assert_eq!(GANTRY.build_draw(), 240, "a diamond-tipped bit's");
        assert_eq!(TIMBER.build_draw(), 18, "an iron bit's, bit or none");
        assert_eq!(IRON.build_draw(), 90, "a steel bit's in an iron quarry");
        assert_eq!(STEEL.build_draw(), 160, "a diamond-tipped bit's");
    }

    #[test]
    fn the_iron_quarry_digs_with_an_iron_bit_only() {
        assert!(IRON.drill(None).is_none(), "no bit, no digging");
        assert!(
            IRON.drill(Some(&STONE_BIT)).is_none(),
            "a stone bit is too weak"
        );
        let iron = IRON.drill(Some(&IRON_BIT)).expect("an iron bit fits");
        assert_eq!(iron.draw, 60, "it works an iron bit harder");
        assert_eq!(iron.tier, IRON_BIT.tier, "and breaks what it breaks");
        let steel = IRON.drill(Some(&STEEL_BIT)).expect("a steel bit fits");
        assert_eq!(steel.draw, 90, "harder still with steel");
        assert!(
            TIMBER.drill(Some(&STEEL_BIT)).is_none(),
            "too much for timber"
        );
        assert!(
            IRON.drill(Some(&DIAMOND_BIT)).is_none(),
            "a diamond-tipped bit is the steel excavator's"
        );
    }

    #[test]
    fn the_steel_excavator_digs_with_steel_or_diamond() {
        assert!(STEEL.drill(None).is_none(), "no bit, no digging");
        assert!(STEEL.drill(Some(&IRON_BIT)).is_none(), "iron is too weak");
        let steel = STEEL.drill(Some(&STEEL_BIT)).expect("a steel bit fits");
        let diamond = STEEL.drill(Some(&DIAMOND_BIT)).expect("diamond fits");
        assert_eq!(
            (steel.draw, diamond.draw),
            (120, 160),
            "work draw by the bit"
        );
        assert_eq!(diamond.tier, DIAMOND_BIT.tier, "it breaks what it breaks");
        assert_eq!(STEEL.rubble_per_ore, 1.5, "half as much again");
        assert_eq!((STEEL.inside.default, STEEL.inside.largest), (9, Some(63)));
    }

    #[test]
    fn the_steel_excavator_digs_to_the_lowest_layer_whatever_the_world_allows() {
        let frame = Frame::from(([0, 60, 0], [10, 64, 10]));
        let lowest_y = -64;
        let unlimited = STEEL.floor(&frame, Floor::unlimited(), lowest_y);
        assert_eq!(unlimited, Floor::at(-64), "it stops at the lowest layer");
        let higher = STEEL.floor(&frame, Floor::at(-20), lowest_y);
        assert_eq!(higher, Floor::at(-20), "where the world stops first");
        assert_eq!(
            GANTRY.floor(&frame, Floor::unlimited(), lowest_y),
            Floor::unlimited(),
            "the gantry digs on"
        );
    }

    #[test]
    fn a_steel_ingot_pays_for_four_frame_blocks() {
        assert_eq!(STEEL.frame_batch, 4, "steel: four");
        assert_eq!(IRON.frame_batch, 1, "iron: each");
        assert_eq!(TIMBER.frame_batch, 1, "timber: each");
    }

    #[test]
    fn the_iron_quarry_is_wider_deeper_and_richer() {
        let frame = Frame::from(([0, 10, 0], [18, 14, 18]));
        let floor = IRON.floor(&frame, Floor::at(-64), -64);
        assert!(!floor.is_above(-54), "64 below its layer of 10");
        assert!(floor.is_above(-55), "and no further");
        assert_eq!(IRON.standby(&frame), 12 + 17 + 17, "by its sides");
        assert_eq!((IRON.inside.default, IRON.inside.largest), (9, Some(17)));
        assert_eq!(IRON.rubble_per_ore, 1.25, "a quarter more Ore Rubble");
    }

    #[test]
    fn a_timber_quarry_stops_sixteen_below() {
        let frame = Frame::from(([0, 10, 0], [6, 14, 6]));
        let floor = TIMBER.floor(&frame, Floor::at(-64), -64);
        assert!(!floor.is_above(-6), "16 below its layer of 10");
        assert!(floor.is_above(-7), "and no further");
        let shallow = TIMBER.floor(&frame, Floor::at(0), -64);
        assert!(shallow.is_above(-1), "the world's floor comes first");
        assert_eq!(
            GANTRY.floor(&frame, Floor::at(-64), -64),
            Floor::at(-64),
            "the world's"
        );
    }

    #[test]
    fn standby_is_flat_for_the_timber_quarry() {
        let frame = Frame::from(([0, 0, 0], [8, 4, 8]));
        assert_eq!(TIMBER.standby(&frame), 4, "a light frame");
        assert_eq!(GANTRY.standby(&frame), 12 + 7 + 7, "by its sides");
        assert_eq!(STEEL.standby(&frame), 12 + 7 + 7, "by its sides");
    }

    #[test]
    fn the_slot_rules_name_what_goes_in() {
        assert_eq!(TIMBER.material_rule(), "#logs base:string");
        assert_eq!(IRON.material_rule(), "#ingots/iron");
        assert_eq!(STEEL.material_rule(), "#ingots/steel");
        assert!(GANTRY.material_rule().ends_with(":composite_panel"));
        // Outside the game an item's mod is not known: only the names after it.
        let rule = TIMBER.bit_rule();
        let bits: Vec<&str> = rule.split(' ').collect();
        let named = |name| bits.iter().any(|bit| bit.ends_with(name));
        assert!(named(":stone_bit") && named(":iron_bit"), "{bits:?}");
    }
}
