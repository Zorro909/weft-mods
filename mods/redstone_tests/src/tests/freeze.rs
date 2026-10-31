//! Torches, pending changes across unload and restart, and circuits frozen at the edge of the
//! loaded area (plan Task 9).

use std::sync::atomic::{AtomicU8, Ordering};

use crate::scenario::{Ctx, Restart, Scenario, Step};

/// A torch clock: a wall torch on stone S, a block above the torch it strong-powers, dust on S
/// next to that block. The block powers the dust, the dust powers S, S turns the torch off.
const S: [i32; 3] = [0, 0, 0];
const TORCH: [i32; 3] = [1, 0, 0];
const ABOVE: [i32; 3] = [1, 1, 0];
const DUST: [i32; 3] = [0, 1, 0];

fn build_clock(c: &mut Ctx) {
    c.set(S, "base:stone");
    c.set(ABOVE, "base:stone");
    c.set(TORCH, "redstone:torch[facing=east,lit=true]");
    c.set(DUST, "redstone:dust");
}

fn clock_loaded(c: &mut Ctx) -> bool {
    [S, TORCH, ABOVE, DUST].iter().all(|p| c.loaded(*p))
}

/// The torch's last state seen by `alternates`: 0 off, 1 lit, 2 none yet.
static LAST: AtomicU8 = AtomicU8::new(2);

fn torch_lit(c: &Ctx) -> u8 {
    match c.prop(TORCH, "lit").as_str() {
        "true" => 1,
        "false" => 0,
        _ => 3,
    }
}

/// Steps checking that the torch changes in each of `ticks` ticks.
fn alternating(steps: &mut Vec<Step>, ticks: u32) {
    steps.push(Step::Do(|_| LAST.store(2, Ordering::Relaxed)));
    for _ in 0..=ticks {
        steps.push(Step::Do(|c| {
            let now = torch_lit(c);
            if now == 3 {
                return c.fail(format!("no torch at the torch's place: {}", c.get(TORCH)));
            }
            let last = LAST.swap(now, Ordering::Relaxed);
            if last == now {
                c.fail(format!("torch stayed lit={} for a tick", now == 1));
            }
        }));
        steps.push(Step::Wait(1));
    }
}

/// A self-feeding torch toggles every tick.
pub fn torch_clock_runs() -> Scenario {
    let mut steps = vec![Step::Do(build_clock), Step::Wait(3)];
    alternating(&mut steps, 20);
    Scenario::new("torch_clock_runs", steps)
}

/// Ticks for a setting change to settle (see `basics::SETTLE`).
const SETTLE: u32 = 10;

/// lever, 39 dust, lamp: in the 64-level range one network over 3 cubes on X.
const LEVER: [i32; 3] = [0, 1, 0];
const LAMP: [i32; 3] = [40, 1, 0];

/// The player moves away until the lamp's end of the line unloads while the lever's stays. The
/// flipped lever then changes nothing; back again, the lamp lights within a retry.
pub fn freeze_across_border() -> Scenario {
    Scenario::new(
        "freeze_across_border",
        vec![
            Step::Do(|_| redstone_api::RANGE.set(2)),
            Step::Wait(SETTLE),
            Step::Do(|c| {
                for x in 0..=40 {
                    c.set([x, 0, 0], "base:stone");
                }
                c.set(LEVER, "redstone:lever[facing=up,powered=false]");
                for x in 1..=39 {
                    c.set([x, 1, 0], "redstone:dust");
                }
                c.set(LAMP, "redstone:lamp[lit=false]");
            }),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(c.prop(LAMP, "lit"), "false".to_string(), "lamp before")),
            // West, one block a tick, until the lamp's cube is gone.
            Step::Until(
                |c| {
                    c.go([-1.0, 0.0, 0.0]);
                    !c.loaded(LAMP)
                },
                900,
            ),
            Step::Do(|c| {
                c.check(c.loaded(LEVER), "the lever's end is still loaded");
                c.set(LEVER, "redstone:lever[facing=up,powered=true]");
            }),
            Step::Wait(3),
            Step::Do(|c| {
                c.check(!c.loaded(LAMP), "the lamp's end is still unloaded");
                c.check_eq(c.prop([1, 1, 0], "level"), "0".to_string(), "frozen dust next to the lever");
                c.check_eq(c.strength([1, 1, 0]), 0, "frozen dust strength");
            }),
            Step::Do(|c| c.go_back()),
            Step::Until(|c| c.loaded(LAMP), 900),
            Step::Do(|c| c.check_eq(c.prop(LAMP, "lit"), "false".to_string(), "lamp right after reload")),
            // A frozen circuit is retried every 20 ticks, the lamp then reacts a tick later.
            Step::Until(|c| c.prop(LAMP, "lit") == "true", 25),
            Step::Do(|c| {
                c.check_eq(c.prop([1, 1, 0], "level"), "3".to_string(), "dust next to the lever");
                let max = redstone_api::Range::from_index(2).max();
                c.check_eq(c.strength([39, 1, 0]), max - 38, "dust next to the lamp");
            }),
            Step::Do(|_| redstone_api::RANGE.set(0)),
            Step::Wait(SETTLE),
            Step::Do(|c| c.check_eq(redstone_api::RANGE.get(), 0, "range back to 16")),
        ],
    )
}

/// A torch clock unloaded mid-flip runs on, a flip every tick, once its area is back.
pub fn pending_survives_unload() -> Scenario {
    let mut steps = vec![
        Step::Do(build_clock),
        Step::Wait(5),
        Step::Do(|c| c.go([3000.0, 0.0, 0.0])),
        Step::Until(|c| !c.loaded(TORCH), 400),
        Step::Wait(40),
        Step::Do(|c| c.go_back()),
        Step::Until(clock_loaded, 900),
        Step::Wait(2),
    ];
    alternating(&mut steps, 10);
    Scenario::new("pending_survives_unload", steps)
}

/// A torch broken with a change pending stays broken.
pub fn break_pending_torch() -> Scenario {
    let mut steps = vec![Step::Do(build_clock), Step::Wait(4), Step::Do(|c| c.set(TORCH, "core:air"))];
    for _ in 0..5 {
        steps.push(Step::Wait(1));
        steps.push(Step::Do(|c| c.check_eq(c.get(TORCH), "core:air".to_string(), "where the torch was")));
    }
    Scenario::new("break_pending_torch", steps)
}

/// First launch: a torch clock left running when the game quits.
pub fn restart_before() -> Scenario {
    let mut steps = vec![Step::Do(build_clock), Step::Wait(3)];
    alternating(&mut steps, 4);
    Scenario::new("restart_before", steps).restart(Restart::Before)
}

/// Second launch on the same world: the clock's pending change was saved, so it runs on.
pub fn restart_after() -> Scenario {
    let mut steps = vec![Step::Until(clock_loaded, 900), Step::Wait(5)];
    alternating(&mut steps, 10);
    steps.push(Step::Do(|c| {
        for p in [DUST, TORCH, ABOVE, S] {
            c.set(p, "core:air");
        }
    }));
    Scenario::new("restart_after", steps).restart(Restart::After)
}
