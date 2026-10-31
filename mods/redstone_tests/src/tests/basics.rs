//! Dust, lever, lamp and power block (plan Task 8).

use crate::scenario::{Ctx, Scenario, Step};

static FLATLANDS: weft_sdk::Dimension = weft_sdk::Dimension::named("base:flatlands");

const LEVER_OFF: &str = "redstone:lever[facing=up,powered=false]";
const LEVER_ON: &str = "redstone:lever[facing=up,powered=true]";

/// The brightness level dust shows for `strength` at range 16.
fn level16(strength: u8) -> String {
    redstone_api::Range::from_index(0).level(strength, 4).to_string()
}

/// lever on stone → 14 dust → lamp: strengths 15..2 in the tick after the lever, the lamp one
/// tick later.
pub fn lever_dust_lamp() -> Scenario {
    Scenario::new(
        "lever_dust_lamp",
        vec![
            Step::Do(|c| {
                for x in 0..=15 {
                    c.set([x, 0, 0], "base:stone");
                }
                c.set([0, 1, 0], LEVER_OFF);
                for x in 1..=14 {
                    c.set([x, 1, 0], "redstone:dust");
                }
                c.set([15, 1, 0], "redstone:lamp[lit=false]");
            }),
            Step::Wait(2),
            Step::Do(|c| c.set([0, 1, 0], LEVER_ON)),
            Step::Wait(1),
            Step::Do(|c| {
                for x in 1..=14 {
                    let want = 16 - x as u8;
                    c.check_eq(c.strength([x, 1, 0]), want, &format!("strength of dust {x}"));
                    c.check_eq(c.prop([x, 1, 0], "level"), level16(want), &format!("level of dust {x}"));
                }
                c.check_eq(c.prop([15, 1, 0], "lit"), "false".to_string(), "lamp before its tick");
            }),
            Step::Wait(1),
            Step::Do(|c| c.check_eq(c.prop([15, 1, 0], "lit"), "true".to_string(), "lamp")),
        ],
    )
}

fn fade_check(c: &mut Ctx, range: u32) {
    c.check_eq(redstone_api::RANGE.get(), range, "range setting");
    let max = redstone_api::Range::from_index(range).max() as i32;
    c.check_eq(c.strength([1, 1, 0]), max as u8, &format!("range {range}: first dust"));
    c.check_eq(c.strength([max, 1, 0]), 1, &format!("range {range}: dust {max}"));
    c.check_eq(c.prop([max, 1, 0], "level"), "1".to_string(), &format!("range {range}: level of dust {max}"));
    c.check_eq(c.strength([max + 1, 1, 0]), 0, &format!("range {range}: dust {}", max + 1));
    c.check_eq(c.prop([max + 1, 1, 0], "level"), "0".to_string(), &format!("range {range}: level of dust {}", max + 1));
}

/// Ticks for a setting change to settle. In one process the client applies every value the
/// server syncs to the same store (`client/mod.rs:793-797`), so a copy of the previous value
/// still on its way would undo a change made right after it.
const SETTLE: u32 = 10;

/// Poke the power block so redstone solves the line again with the new range.
fn poke(c: &mut Ctx) {
    c.set([0, 1, 0], "redstone:power_block");
}

/// A 65-long line from a power block loses exactly 1 per block in every range.
pub fn fade_ranges() -> Scenario {
    Scenario::new(
        "fade_ranges",
        vec![
            Step::Do(|c| {
                for x in 0..=65 {
                    c.set([x, 0, 0], "base:stone");
                }
                for x in 1..=65 {
                    c.set([x, 1, 0], "redstone:dust");
                }
                c.set([0, 1, 0], "redstone:power_block");
            }),
            Step::Wait(1),
            Step::Do(|c| fade_check(c, 0)),
            Step::Do(|_| redstone_api::RANGE.set(1)),
            Step::Wait(SETTLE),
            Step::Do(poke),
            Step::Wait(1),
            Step::Do(|c| fade_check(c, 1)),
            Step::Do(|_| redstone_api::RANGE.set(2)),
            Step::Wait(SETTLE),
            Step::Do(poke),
            Step::Wait(1),
            Step::Do(|c| fade_check(c, 2)),
            Step::Do(|_| redstone_api::RANGE.set(0)),
            Step::Wait(SETTLE),
            Step::Do(|c| c.check_eq(redstone_api::RANGE.get(), 0, "range back to 16")),
        ],
    )
}

fn shape(c: &Ctx, rel: [i32; 3]) -> String {
    ["west", "east", "north", "south"].map(|d| format!("{d}={}", c.prop(rel, d))).join(",")
}

/// Spec §8: alone a cross, a line, a corner, a step up and a cut step.
pub fn dust_shapes() -> Scenario {
    Scenario::new(
        "dust_shapes",
        vec![
            Step::Do(|c| {
                // Alone.
                c.set([0, 0, 0], "base:stone");
                c.set([0, 1, 0], "redstone:dust");
                // A line.
                for x in 4..=6 {
                    c.set([x, 0, 0], "base:stone");
                    c.set([x, 1, 0], "redstone:dust");
                }
                // A corner.
                for p in [[10, 0, 0], [11, 0, 0], [10, 0, 1]] {
                    c.set(p, "base:stone");
                    c.set([p[0], 1, p[2]], "redstone:dust");
                }
                // A step up onto the block east of the lower dust, and the same step cut by a
                // block above the lower dust.
                for x in [14, 18] {
                    c.set([x, 0, 0], "base:stone");
                    c.set([x + 1, 0, 0], "base:stone");
                    c.set([x + 1, 1, 0], "base:stone");
                    c.set([x, 1, 0], "redstone:dust");
                    c.set([x + 1, 2, 0], "redstone:dust");
                }
                c.set([18, 2, 0], "base:stone");
            }),
            Step::Wait(1),
            Step::Do(|c| {
                let cross = "west=side,east=side,north=side,south=side".to_string();
                c.check_eq(shape(c, [0, 1, 0]), cross.clone(), "alone");
                c.check_eq(shape(c, [5, 1, 0]), "west=side,east=side,north=none,south=none".to_string(), "line middle");
                c.check_eq(shape(c, [4, 1, 0]), "west=side,east=side,north=none,south=none".to_string(), "line end");
                c.check_eq(shape(c, [10, 1, 0]), "west=none,east=side,north=none,south=side".to_string(), "corner");
                c.check_eq(shape(c, [14, 1, 0]), "west=side,east=up,north=none,south=none".to_string(), "step lower");
                c.check_eq(shape(c, [15, 2, 0]), "west=side,east=side,north=none,south=none".to_string(), "step upper");
                c.check_eq(shape(c, [18, 1, 0]), cross.clone(), "cut step lower");
                c.check_eq(shape(c, [19, 2, 0]), cross, "cut step upper");
            }),
        ],
    )
}

/// Flatlands ends at y = 143 (cubes -2..8). Nothing above the top is ever loaded, and redstone
/// must not take that for an unloaded area (G1): dust in the top layer still powers.
pub fn dust_at_height_limit() -> Scenario {
    fn top(c: &Ctx) -> i32 {
        143 - c.origin[1]
    }
    Scenario::new(
        "dust_at_height_limit",
        vec![
            Step::Do(|c| {
                let t = top(c);
                c.set([0, t - 1, 0], "base:stone");
                c.set([1, t - 1, 0], "base:stone");
                c.set([0, t, 0], LEVER_OFF);
                c.set([1, t, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| {
                let t = top(c);
                c.set([0, t, 0], LEVER_ON);
            }),
            Step::Wait(1),
            Step::Do(|c| {
                let t = top(c);
                c.check_eq(c.strength([1, t, 0]), 15, "dust at the top");
                c.check_eq(c.prop([1, t, 0], "level"), "3".to_string(), "level at the top");
            }),
        ],
    )
    .in_dimension(&FLATLANDS, [0.5, 137.0, 0.5])
}

/// Lines of 15 dust, a lever at the start of each, side by side in banks of 34 so they stay
/// within the loaded area.
const LINES: i32 = 134;

/// Where line `i` starts (its lever).
fn line(i: i32) -> [i32; 3] {
    [18 * (i / 34), 1, 2 * (i % 34)]
}

fn clock_toggle(c: &mut Ctx, on: bool) {
    for i in 0..LINES {
        c.set(line(i), if on { LEVER_ON } else { LEVER_OFF });
    }
}

fn clock_check(c: &mut Ctx, on: bool) {
    for i in [0, LINES / 2, LINES - 1] {
        let [x, y, z] = line(i);
        c.check_eq(c.strength([x + 15, y, z]), on as u8, &format!("end of line {i}, lever {on}"));
    }
}

/// 2,010 dust switching every 2 ticks for 40 ticks, driven by levers (torches come later), stays
/// in step and under Weft's neighbor-update budget (`scripts/test.sh` checks the log for a storm).
pub fn long_line_clock() -> Scenario {
    let mut steps = vec![Step::Do(|c| {
        for i in 0..LINES {
            let [x0, _, z] = line(i);
            for x in x0..=x0 + 15 {
                c.set([x, 0, z], "base:stone");
            }
            c.set(line(i), LEVER_OFF);
            for x in x0 + 1..=x0 + 15 {
                c.set([x, 1, z], "redstone:dust");
            }
        }
    })];
    steps.push(Step::Wait(2));
    for _ in 0..10 {
        steps.push(Step::Do(|c| clock_toggle(c, true)));
        steps.push(Step::Wait(2));
        steps.push(Step::Do(|c| {
            clock_check(c, true);
            clock_toggle(c, false);
        }));
        steps.push(Step::Wait(2));
        steps.push(Step::Do(|c| clock_check(c, false)));
    }
    Scenario::new("long_line_clock", steps)
}
