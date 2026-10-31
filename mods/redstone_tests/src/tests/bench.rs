//! Benchmarks, not part of `all`: a large circuit, a hundred of them, and one big network, each
//! driven by a 1-tick torch clock so everything changes every tick. They check that the circuits
//! run; `redstone`'s timings in the log (or the F3 "mods per tick" line) say what they cost.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use redstone_core::Pos;

use crate::scenario::{Ctx, Scenario, Step};

static FLATLANDS: weft_sdk::Dimension = weft_sdk::Dimension::named("base:flatlands");
/// In the air over flatlands: nothing but what the benchmark builds.
const AT: [f64; 3] = [0.5, 125.0, 0.5];

const STONE: &str = "base:stone";
const DUST: &str = "redstone:dust";
/// A repeater putting out to the south (`facing` looks back at its input).
const REPEATER_SOUTH: &str = "redstone:repeater[facing=north,delay=1,powered=false,locked=false]";

/// The 1-tick torch clock at `b`: a wall torch on stone, the block above it strong-powered, dust
/// on the stone next to that block turning the torch off. The powered block at `b + [1, 1, 0]`
/// drives whatever starts at `b + [2, 1, 0]`.
fn clock(c: &mut Ctx, b: Pos) {
    let at = |d: Pos| [b[0] + d[0], b[1] + d[1], b[2] + d[2]];
    c.set(at([0, 0, 0]), STONE);
    c.set(at([1, 1, 0]), STONE);
    c.set(at([1, 0, 0]), "redstone:torch[facing=east,lit=true]");
    c.set(at([0, 1, 0]), DUST);
}

/// The x of the three branches of `circuit`.
const BRANCHES: [i32; 3] = [4, 8, 12];

/// 15 × 13: the clock, 13 dust east, three branches south off it (dust, repeater, 9 dust, lamp).
/// 46 dust, 3 repeaters, 3 lamps and a torch, all changing every tick.
fn circuit(c: &mut Ctx, b: Pos) {
    let at = |d: Pos| [b[0] + d[0], b[1] + d[1], b[2] + d[2]];
    clock(c, b);
    for x in 2..=14 {
        c.set(at([x, 0, 0]), STONE);
        c.set(at([x, 1, 0]), DUST);
    }
    for x in BRANCHES {
        for z in 1..=12 {
            c.set(at([x, 0, z]), STONE);
        }
        c.set(at([x, 1, 1]), DUST);
        c.set(at([x, 1, 2]), REPEATER_SOUTH);
        for z in 3..=11 {
            c.set(at([x, 1, z]), DUST);
        }
        c.set(at([x, 1, 12]), "redstone:lamp[lit=false]");
    }
}

/// Copies of `circuit` in a 10 × 10 grid around the player, 2 blocks apart.
fn grid() -> impl Iterator<Item = Pos> {
    (0..10).flat_map(|i| (0..10).map(move |j| [-85 + 17 * i, 0, -75 + 15 * j]))
}

fn grid_loaded(c: &mut Ctx) -> bool {
    [[-85, 0, -75], [82, 0, -75], [-85, 0, 72], [82, 0, 72]].iter().all(|p| c.loaded(*p))
}

/// The lamps `watch` looks at, how many changes it saw, and their last states.
static LAMPS: Mutex<Vec<Pos>> = Mutex::new(Vec::new());
static CHANGES: AtomicU32 = AtomicU32::new(0);
static LAST: Mutex<Vec<bool>> = Mutex::new(Vec::new());

/// Watches the lamps of the circuits at `origins` from now on.
fn watch_lamps(origins: impl Iterator<Item = Pos>) {
    *LAMPS.lock().unwrap() = origins.flat_map(|b| BRANCHES.map(|x| [b[0] + x, b[1] + 1, b[2] + 12])).collect();
}

/// How many ticks `watch` looks.
const WATCH: u32 = 20;

/// Steps that watch the lamps for `WATCH` ticks and fail unless they changed at least a
/// quarter of the time each.
fn watch(steps: &mut Vec<Step>) {
    steps.push(Step::Do(|_| {
        CHANGES.store(0, Ordering::Relaxed);
        LAST.lock().unwrap().clear();
    }));
    for _ in 0..WATCH {
        steps.push(Step::Do(|c| {
            let now: Vec<bool> = LAMPS.lock().unwrap().iter().map(|p| c.prop(*p, "lit") == "true").collect();
            let mut last = LAST.lock().unwrap();
            if last.len() == now.len() {
                CHANGES.fetch_add(now.iter().zip(last.iter()).filter(|(a, b)| a != b).count() as u32, Ordering::Relaxed);
            }
            *last = now;
        }));
        steps.push(Step::Wait(1));
    }
    steps.push(Step::Do(|c| {
        let n = LAMPS.lock().unwrap().len() as u32;
        let changes = CHANGES.load(Ordering::Relaxed);
        if changes < n * (WATCH / 4) {
            c.fail(format!("{changes} lamp changes in {WATCH} ticks over {n} lamps"));
        }
    }));
}

/// One large circuit, run for 800 ticks.
pub fn bench_circuit() -> Scenario {
    let mut steps = vec![
        Step::Do(|c| {
            circuit(c, [0, 0, 0]);
            watch_lamps([[0, 0, 0]].into_iter());
        }),
        Step::Wait(800),
    ];
    watch(&mut steps);
    Scenario::new("bench_circuit", steps).in_dimension(&FLATLANDS, AT).bench()
}

/// A hundred large circuits at once, run for 800 ticks.
pub fn bench_stress() -> Scenario {
    let mut steps = vec![
        Step::Until(grid_loaded, 1200),
        // Half a grid per tick: all at once would be more neighbor updates than Weft does in one.
        Step::Do(|c| grid().take(50).for_each(|b| circuit(c, b))),
        Step::Wait(1),
        Step::Do(|c| {
            grid().skip(50).for_each(|b| circuit(c, b));
            watch_lamps(grid());
        }),
        Step::Wait(800),
    ];
    watch(&mut steps);
    Scenario::new("bench_stress", steps).in_dimension(&FLATLANDS, AT).bench()
}

/// The side of `bench_sheet`'s dust sheet.
const SHEET: i32 = 32;
static SHEET_SEEN: AtomicU32 = AtomicU32::new(0);

/// The clock driving one network of 32 × 32 dust, run for 800 ticks.
pub fn bench_sheet() -> Scenario {
    let mut steps = vec![
        Step::Do(|c| {
            clock(c, [0, 0, 0]);
            for x in 2..2 + SHEET {
                for z in 0..SHEET {
                    c.set([x, 0, z], STONE);
                    c.set([x, 1, z], DUST);
                }
            }
        }),
        Step::Wait(800),
        Step::Do(|_| SHEET_SEEN.store(0, Ordering::Relaxed)),
    ];
    for _ in 0..10 {
        steps.push(Step::Do(|c| {
            if c.prop([2, 1, 0], "level") != "0" {
                SHEET_SEEN.fetch_add(1, Ordering::Relaxed);
            }
        }));
        steps.push(Step::Wait(1));
    }
    steps.push(Step::Do(|c| {
        let on = SHEET_SEEN.load(Ordering::Relaxed);
        if !(3..=7).contains(&on) {
            c.fail(format!("the sheet's first dust was powered in {on} of 10 ticks, want about half"));
        }
    }));
    Scenario::new("bench_sheet", steps).in_dimension(&FLATLANDS, AT).bench()
}
