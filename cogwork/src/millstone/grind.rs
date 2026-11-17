//! What the millstone holds and what a turn does, over plain values: water and sand go in
//! by the bucket and the block, Ore Rubble by the stack, and every sixth turn grinds one Ore
//! Rubble into raw ore, using up a grind's water and sand.

use super::{Mill, Status};
use crate::ghost::{ItemKey, NO_ITEM};
use crate::rubble::grinding::{self, GRINDS_PER_SAND, WATER_PER_GRIND};
use crate::rubble::{Grade, Ore};

/// Turns of the runner that grind one Ore Rubble.
pub const TURNS: u32 = 6;
/// Millibuckets of water in a bucket, and the most it holds.
pub const BUCKET: u32 = 1_000;
const MOST_WATER: u32 = 2_000;
/// The most grinds of sand it holds.
const MOST_GRIT: u32 = 16;
/// The most Ore Rubble it holds, and the most raw ore it keeps for the player to take out.
const MOST_HELD: u32 = 16;
const MOST_GROUND: u32 = 64;

/// What sneaking with an empty hand takes out of a millstone.
#[derive(PartialEq, Debug)]
pub enum TakenOut {
    /// Raw ore: its item and how many.
    RawOre(u32, u32),
    /// Ore Rubble: its metal, grade and how many.
    OreRubble(Ore, Grade, u32),
    /// Nothing, for the reason shown.
    Nothing(Status),
}

impl Mill {
    pub fn grade(&self) -> Grade {
        Grade::from_saved(self.grade)
    }

    /// Pours in a bucket of water; false when there is no room for it.
    pub fn pour(&mut self) -> bool {
        let fits = self.water + BUCKET <= MOST_WATER;
        if fits {
            self.water += BUCKET;
        }
        fits
    }

    /// Puts in a block of sand; false when there is no room for it.
    pub fn add_sand(&mut self) -> bool {
        let fits = self.grit + GRINDS_PER_SAND <= MOST_GRIT;
        if fits {
            self.grit += GRINDS_PER_SAND;
        }
        fits
    }

    /// Puts in up to `count` Ore Rubble of `ore` at `grade`, and says how many went in: none
    /// while it holds another kind.
    pub fn put_in(&mut self, ore: Ore, grade: Grade, count: u32) -> u32 {
        let same = self.ore.metal() == ore.metal() && self.grade() == grade;
        if self.held > 0 && !same {
            return 0;
        }
        let taken = count.min(MOST_HELD - self.held.min(MOST_HELD));
        if taken > 0 {
            (self.ore, self.grade) = (ore, grade as u32);
            self.held += taken;
        }
        taken
    }

    /// Takes out the Ore Rubble it holds, a turn under way lost: its kind and count.
    pub fn take_held(&mut self) -> (Ore, Grade, u32) {
        let taken = (self.ore, self.grade(), self.held);
        (self.held, self.turns) = (0, 0);
        taken
    }

    /// Takes out the raw ore it ground: its key and count.
    fn take_ground(&mut self) -> (ItemKey, u32) {
        let taken = (self.ground, self.ground_count);
        (self.ground, self.ground_count) = (NO_ITEM, 0);
        taken
    }

    /// Takes out the raw ore it ground, when `item_of` finds its item, or else the Ore Rubble
    /// it holds. Raw ore whose item no loaded mod has stays, for when that mod is back.
    pub fn take_out(&mut self, item_of: impl Fn(ItemKey) -> u32) -> TakenOut {
        let raw_ore = item_of(self.ground);
        if self.ground_count > 0 && raw_ore != 0 {
            let (_, count) = self.take_ground();
            return TakenOut::RawOre(raw_ore, count);
        }
        if self.held > 0 {
            let (ore, grade, count) = self.take_held();
            return TakenOut::OreRubble(ore, grade, count);
        }
        let reason = if self.ground_count > 0 {
            Status::UnknownRawOre
        } else {
            Status::Empty
        };
        TakenOut::Nothing(reason)
    }

    /// What it lacks for a turn to count, its Ore Rubble grinding into `raw_ore`.
    pub fn lacks(&self, raw_ore: ItemKey) -> Option<Status> {
        let room = self.ground_count + self.grade().most_raw_ore() <= MOST_GROUND;
        let fits = self.ground == NO_ITEM || (self.ground == raw_ore && room);
        if self.held == 0 {
            Some(Status::Empty)
        } else if raw_ore == NO_ITEM {
            Some(Status::NoRawOre)
        } else if self.water < WATER_PER_GRIND {
            Some(Status::NoWater)
        } else if self.grit == 0 {
            Some(Status::NoSand)
        } else if !fits {
            Some(Status::Full)
        } else {
            None
        }
    }

    /// Turns the runner once with its Ore Rubble grinding into `raw_ore`, and on the last turn
    /// of a grind grinds one, `roll` (0 to 1) deciding the fraction of its yield.
    pub fn turn(&mut self, raw_ore: ItemKey, roll: f64) -> Status {
        if let Some(lack) = self.lacks(raw_ore) {
            return lack;
        }
        self.turns += 1;
        if self.turns < TURNS {
            return Status::Turning;
        }
        self.turns = 0;
        self.held -= 1;
        self.water -= WATER_PER_GRIND;
        self.grit -= 1;
        self.ground = raw_ore;
        self.ground_count += grinding::raw_ore(self.grade(), roll);
        Status::Ground
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_IRON: ItemKey = 11;

    fn iron() -> Ore {
        Ore::of("iron").expect("iron fits")
    }

    /// A millstone ready to grind `held` Poor iron Ore Rubble.
    fn ready(held: u32) -> Mill {
        let mut mill = Mill::default();
        mill.pour();
        mill.add_sand();
        mill.put_in(iron(), Grade::Poor, held);
        mill
    }

    #[test]
    fn six_turns_grind_one_ore_rubble() {
        let mut mill = ready(2);
        for turn in 1..TURNS {
            assert_eq!(mill.turn(RAW_IRON, 0.5), Status::Turning, "turn {turn}");
        }
        assert_eq!(mill.turn(RAW_IRON, 0.5), Status::Ground, "the sixth grinds");
        assert_eq!(mill.held, 1, "one left");
        assert_eq!(
            (mill.ground, mill.ground_count),
            (RAW_IRON, 1),
            "a raw iron"
        );
        assert_eq!(mill.water, BUCKET - WATER_PER_GRIND, "a grind's water");
        assert_eq!(mill.grit, GRINDS_PER_SAND - 1, "a grind's sand");
    }

    #[test]
    fn a_sand_lasts_four_grinds() {
        let mut mill = ready(16);
        mill.water = MOST_WATER;
        let mut ground = 0;
        while mill.turn(RAW_IRON, 0.0) != Status::NoSand {
            ground += u32::from(mill.turns == 0);
        }
        assert_eq!(ground, GRINDS_PER_SAND, "then it wants sand");
    }

    #[test]
    fn a_turn_without_water_or_sand_does_not_count() {
        let mut mill = ready(1);
        mill.water = 0;
        assert_eq!(mill.turn(RAW_IRON, 0.5), Status::NoWater, "dry");
        assert_eq!(mill.turns, 0, "not counted");
        mill.pour();
        mill.grit = 0;
        assert_eq!(mill.turn(RAW_IRON, 0.5), Status::NoSand, "no sand");
        assert_eq!(
            mill.turn(NO_ITEM, 0.5),
            Status::NoRawOre,
            "a metal no mod has"
        );
    }

    #[test]
    fn a_richer_grade_grinds_into_more() {
        let mut mill = Mill::default();
        mill.pour();
        mill.add_sand();
        mill.put_in(iron(), Grade::Pure, 1);
        mill.turns = TURNS - 1;
        assert_eq!(mill.turn(RAW_IRON, 0.2), Status::Ground, "ground");
        assert_eq!(mill.ground_count, 3, "2.5, and the half came up");
    }

    #[test]
    fn it_holds_one_kind_of_ore_rubble_at_a_time() {
        let mut mill = Mill::default();
        assert_eq!(
            mill.put_in(iron(), Grade::Poor, 10),
            10,
            "into an empty one"
        );
        assert_eq!(mill.put_in(iron(), Grade::Poor, 10), 6, "up to sixteen");
        assert_eq!(
            mill.put_in(iron(), Grade::Common, 1),
            0,
            "not another grade"
        );
        let gold = Ore::of("gold").expect("gold fits");
        assert_eq!(mill.put_in(gold, Grade::Poor, 1), 0, "nor another metal");
        let (ore, grade, count) = mill.take_held();
        assert_eq!(
            (ore.metal(), grade, count),
            ("iron", Grade::Poor, 16),
            "taken out"
        );
        assert_eq!(mill.put_in(gold, Grade::Poor, 1), 1, "then gold");
    }

    #[test]
    fn raw_ore_of_another_kind_waits_to_be_taken_out() {
        let mut mill = ready(1);
        (mill.ground, mill.ground_count) = (12, 5);
        assert_eq!(
            mill.turn(RAW_IRON, 0.5),
            Status::Full,
            "raw gold in the way"
        );
        assert_eq!(mill.take_ground(), (12, 5), "taken out");
        assert_eq!(mill.turn(RAW_IRON, 0.5), Status::Turning, "then it turns");
    }

    /// The item of a key while the mod with raw iron is loaded.
    fn loaded(key: ItemKey) -> u32 {
        if key == RAW_IRON { 900 } else { 0 }
    }

    #[test]
    fn sneaking_takes_out_raw_ore_then_ore_rubble() {
        let mut mill = ready(3);
        (mill.ground, mill.ground_count) = (RAW_IRON, 5);
        assert_eq!(
            mill.take_out(loaded),
            TakenOut::RawOre(900, 5),
            "raw iron first"
        );
        assert!(
            matches!(
                mill.take_out(loaded),
                TakenOut::OreRubble(_, Grade::Poor, 3)
            ),
            "then the Ore Rubble"
        );
        let nothing = TakenOut::Nothing(Status::Empty);
        assert_eq!(mill.take_out(loaded), nothing, "then nothing");
    }

    #[test]
    fn raw_ore_of_a_mod_not_loaded_stays_until_it_is_back() {
        let mut mill = ready(2);
        (mill.ground, mill.ground_count) = (12, 5);
        assert!(
            matches!(mill.take_out(loaded), TakenOut::OreRubble(_, _, 2)),
            "the Ore Rubble can still come out"
        );
        let unknown = TakenOut::Nothing(Status::UnknownRawOre);
        assert_eq!(mill.take_out(loaded), unknown, "said so");
        assert_eq!((mill.ground, mill.ground_count), (12, 5), "kept");
        let back = |key| if key == 12 { 901 } else { 0 };
        assert_eq!(
            mill.take_out(back),
            TakenOut::RawOre(901, 5),
            "with its mod back"
        );
    }

    #[test]
    fn a_poor_grind_needs_room_for_one() {
        let mut mill = ready(1);
        (mill.ground, mill.ground_count) = (RAW_IRON, MOST_GROUND - 1);
        assert_eq!(
            mill.lacks(RAW_IRON),
            None,
            "one Poor raw ore fits beside 63"
        );
        mill.ground_count = MOST_GROUND;
        assert_eq!(mill.lacks(RAW_IRON), Some(Status::Full), "none beside 64");
    }

    #[test]
    fn a_pure_grind_needs_room_for_three() {
        let mut mill = Mill::default();
        mill.pour();
        mill.add_sand();
        mill.put_in(iron(), Grade::Pure, 1);
        (mill.ground, mill.ground_count) = (RAW_IRON, MOST_GROUND - 3);
        assert_eq!(mill.lacks(RAW_IRON), None, "three fit beside 61");
        mill.ground_count = MOST_GROUND - 2;
        assert_eq!(mill.lacks(RAW_IRON), Some(Status::Full), "not beside 62");
    }

    #[test]
    fn water_and_sand_stop_at_the_brim() {
        let mut mill = Mill::default();
        assert!(mill.pour() && mill.pour(), "two buckets");
        assert!(!mill.pour(), "not a third");
        assert!((0..4).all(|_| mill.add_sand()), "four sand");
        assert!(!mill.add_sand(), "not a fifth");
    }
}
