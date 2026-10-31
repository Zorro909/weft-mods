//! Repeater, comparator, buttons and pressure plate (plan Task 10).

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::scenario::{Ctx, Scenario, Step};
use weft_sdk::{ItemStack, max_stack};

const STONE: &str = "base:stone";
const LEVER_OFF: &str = "redstone:lever[facing=up,powered=false]";
const LEVER_ON: &str = "redstone:lever[facing=up,powered=true]";

/// A repeater putting out to the east: its `facing` looks back west, at whoever placed it.
fn repeater_east(delay: u32, powered: bool) -> String {
    format!("redstone:repeater[facing=west,delay={delay},powered={powered},locked=false]")
}

fn floor(c: &mut Ctx, from: [i32; 3], to: [i32; 3]) {
    for x in from[0]..=to[0] {
        for z in from[2]..=to[2] {
            c.set([x, 0, z], STONE);
        }
    }
}

/// When something under test started, in world ticks.
static START: AtomicU64 = AtomicU64::new(0);

fn start(c: &Ctx) {
    START.store(c.ticks(), Ordering::Relaxed);
}

fn since(c: &Ctx) -> u64 {
    c.ticks() - START.load(Ordering::Relaxed)
}

/// The delay being measured.
static DELAY: AtomicU32 = AtomicU32::new(0);

/// lever, 12 dust (3 left at the end), repeater, dust: full strength out, and every delay
/// measured both ways. The lever's change reaches the repeater a tick later.
pub fn repeater_delay_and_restore() -> Scenario {
    const R: [i32; 3] = [14, 1, 0];
    let mut steps = vec![Step::Do(|c| {
        floor(c, [0, 0, 0], [15, 0, 0]);
        c.set([0, 1, 0], LEVER_OFF);
        for x in 1..=13 {
            c.set([x, 1, 0], "redstone:dust");
        }
        c.set(R, &repeater_east(1, false));
        c.set([15, 1, 0], "redstone:dust");
        DELAY.store(0, Ordering::Relaxed);
    })];
    for d in 1..=4u32 {
        steps.push(Step::Do(|c| {
            let d = DELAY.fetch_add(1, Ordering::Relaxed) + 1;
            c.set(R, &repeater_east(d, false));
        }));
        steps.push(Step::Wait(2));
        steps.push(Step::Do(|c| {
            c.set([0, 1, 0], LEVER_ON);
            start(c);
        }));
        steps.push(Step::Until(|c| c.prop(R, "powered") == "true", 10));
        steps.push(Step::Do(|c| {
            let d = DELAY.load(Ordering::Relaxed) as u64;
            c.check_eq(since(c), d + 1, &format!("delay {d} on"));
            c.check_eq(c.strength([13, 1, 0]), 3, "into the repeater");
            c.check_eq(c.prop(R, "delay"), d.to_string(), "delay kept");
        }));
        steps.push(Step::Wait(1));
        steps.push(Step::Do(|c| c.check_eq(c.strength([15, 1, 0]), 15, "out of the repeater")));
        steps.push(Step::Do(|c| {
            c.set([0, 1, 0], LEVER_OFF);
            start(c);
        }));
        steps.push(Step::Until(|c| c.prop(R, "powered") == "false", 10));
        steps.push(Step::Do(|c| {
            let d = DELAY.load(Ordering::Relaxed) as u64;
            c.check_eq(since(c), d + 1, &format!("delay {d} off"));
        }));
        steps.push(Step::Wait(d + 1));
    }
    Scenario::new("repeater_delay_and_restore", steps)
}

/// Repeater A (east) fed by a power block; repeater B feeding A's south side locks it, so A holds
/// its output when its input goes; unlocked, A follows its input again.
pub fn repeater_lock() -> Scenario {
    const A: [i32; 3] = [1, 1, 0];
    const FEED: [i32; 3] = [0, 1, 0];
    const B: [i32; 3] = [1, 1, 1];
    const B_FEED: [i32; 3] = [1, 1, 2];
    Scenario::new(
        "repeater_lock",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [2, 0, 2]);
                c.set(A, &repeater_east(1, false));
                // Putting out to the north, into A's side.
                c.set(B, "redstone:repeater[facing=south,delay=1,powered=false,locked=false]");
                c.set(FEED, "redstone:power_block");
            }),
            Step::Wait(4),
            Step::Do(|c| c.check_eq(c.prop(A, "powered"), "true".to_string(), "A on")),
            Step::Do(|c| c.set(B_FEED, "redstone:power_block")),
            Step::Wait(4),
            Step::Do(|c| {
                c.check_eq(c.prop(B, "powered"), "true".to_string(), "B on");
                c.check_eq(c.prop(A, "locked"), "true".to_string(), "A locked");
                c.set(FEED, "core:air");
            }),
            Step::Wait(6),
            Step::Do(|c| c.check_eq(c.prop(A, "powered"), "true".to_string(), "locked A holds its output")),
            Step::Do(|c| c.set(B_FEED, "core:air")),
            Step::Wait(6),
            Step::Do(|c| {
                c.check_eq(c.prop(A, "locked"), "false".to_string(), "A unlocked");
                c.check_eq(c.prop(A, "powered"), "false".to_string(), "unlocked A follows its input");
            }),
        ],
    )
}

/// Ticks a block was seen in some state.
static COUNT: AtomicU32 = AtomicU32::new(0);
static COUNT2: AtomicU32 = AtomicU32::new(0);

/// A one-tick pulse into a delay-4 repeater comes out 4 ticks long.
pub fn repeater_pulse_stretch() -> Scenario {
    const R: [i32; 3] = [1, 1, 0];
    let mut steps = vec![
        Step::Do(|c| {
            floor(c, [0, 0, 0], [2, 0, 0]);
            c.set(R, &repeater_east(4, false));
            COUNT.store(0, Ordering::Relaxed);
        }),
        Step::Wait(2),
        Step::Do(|c| c.set([0, 1, 0], "redstone:power_block")),
        Step::Wait(1),
        Step::Do(|c| c.set([0, 1, 0], "core:air")),
    ];
    for _ in 0..14 {
        steps.push(Step::Wait(1));
        steps.push(Step::Do(|c| {
            if c.prop(R, "powered") == "true" {
                COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }
    steps.push(Step::Do(|c| c.check_eq(COUNT.load(Ordering::Relaxed), 4, "ticks on")));
    Scenario::new("repeater_pulse_stretch", steps)
}

/// A comparator putting out to the east: `facing` looks back west.
fn comparator_east(mode: &str) -> String {
    format!("redstone:comparator[facing=west,mode={mode},output=0]")
}

/// Back 12 (lever and 4 dust), side 11 (power block and 5 dust from the south): compare gives
/// 12, subtract 1.
pub fn comparator_modes() -> Scenario {
    const C: [i32; 3] = [5, 1, 0];
    Scenario::new(
        "comparator_modes",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [6, 0, 0]);
                floor(c, [5, 0, 1], [5, 0, 6]);
                c.set([0, 1, 0], LEVER_ON);
                for x in 1..=4 {
                    c.set([x, 1, 0], "redstone:dust");
                }
                for z in 1..=5 {
                    c.set([5, 1, z], "redstone:dust");
                }
                c.set([5, 1, 6], "redstone:power_block");
                c.set(C, &comparator_east("compare"));
                c.set([6, 1, 0], "redstone:dust");
            }),
            Step::Wait(4),
            Step::Do(|c| {
                c.check_eq(c.strength([4, 1, 0]), 12, "back");
                c.check_eq(c.strength([5, 1, 1]), 11, "side");
                c.check_eq(c.prop(C, "output"), "12".to_string(), "compare");
                c.check_eq(c.strength([6, 1, 0]), 12, "dust after compare");
                c.set(C, "redstone:comparator[facing=west,mode=subtract,output=12]");
            }),
            Step::Wait(3),
            Step::Do(|c| {
                c.check_eq(c.prop(C, "output"), "1".to_string(), "subtract");
                c.check_eq(c.prop(C, "mode"), "subtract".to_string(), "mode kept");
                c.check_eq(c.strength([6, 1, 0]), 1, "dust after subtract");
                // Side 15, stronger than back: compare gives 0.
                c.set([5, 1, 2], "redstone:power_block");
                c.set(C, "redstone:comparator[facing=west,mode=compare,output=1]");
            }),
            Step::Wait(3),
            Step::Do(|c| {
                c.check_eq(c.strength([5, 1, 1]), 15, "side");
                c.check_eq(c.prop(C, "output"), "0".to_string(), "compare, back weaker");
            }),
        ],
    )
}

fn chest_inventory(c: &Ctx, rel: [i32; 3]) -> Option<weft_sdk::Inventory> {
    let id = c.w.block_entity(c.at(rel));
    (id != 0).then(|| c.w.inventory(id))
}

/// Fills the chest at `rel` to `slots` full stacks of stone.
fn fill(c: &mut Ctx, rel: [i32; 3], slots: u32) {
    let Some(inv) = chest_inventory(c, rel) else { return c.fail("no chest inventory") };
    let stone = weft_sdk::Item::lookup("base:stone").expect("stone item");
    for i in 0..inv.size() {
        inv.set(i, if i < slots { ItemStack::new(stone, max_stack(stone)) } else { ItemStack::EMPTY });
    }
}

/// Chest, comparator, dust: empty 0, one stack 1, full the range's maximum, at 16 and 64.
pub fn comparator_reads_chest() -> Scenario {
    const CHEST: [i32; 3] = [0, 1, 0];
    const C: [i32; 3] = [1, 1, 0];
    fn out(c: &Ctx) -> String {
        c.prop(C, "output")
    }
    Scenario::new(
        "comparator_reads_chest",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [2, 0, 0]);
                c.set(CHEST, "base:chest[facing=north]");
                c.set(C, &comparator_east("compare"));
                c.set([2, 1, 0], "redstone:dust");
            }),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(out(c), "0".to_string(), "empty")),
            Step::Do(|c| fill(c, CHEST, 1)),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(out(c), "1".to_string(), "one stack")),
            Step::Do(|c| fill(c, CHEST, 27)),
            Step::Wait(3),
            Step::Do(|c| {
                c.check_eq(out(c), "15".to_string(), "full, range 16");
                c.check_eq(c.strength([2, 1, 0]), 15, "dust, range 16");
            }),
            Step::Do(|_| redstone_api::RANGE.set(2)),
            Step::Wait(13),
            Step::Do(|c| {
                c.check_eq(out(c), "63".to_string(), "full, range 64");
                c.check_eq(c.strength([2, 1, 0]), 63, "dust, range 64");
            }),
            Step::Do(|c| fill(c, CHEST, 0)),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(out(c), "0".to_string(), "emptied")),
            Step::Do(|_| redstone_api::RANGE.set(0)),
            Step::Wait(10),
            Step::Do(|c| c.check_eq(redstone_api::RANGE.get(), 0, "range back to 16")),
        ],
    )
}

/// Chest, stone, comparator: it reads through one solid block.
pub fn comparator_reads_through_block() -> Scenario {
    const CHEST: [i32; 3] = [0, 1, 0];
    const C: [i32; 3] = [2, 1, 0];
    Scenario::new(
        "comparator_reads_through_block",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [3, 0, 0]);
                c.set(CHEST, "base:chest[facing=north]");
                c.set([1, 1, 0], STONE);
                c.set(C, &comparator_east("compare"));
                c.set([3, 1, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| fill(c, CHEST, 27)),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(c.prop(C, "output"), "15".to_string(), "through stone")),
        ],
    )
}

/// Stone button on for exactly 10 ticks, wood for 15; pressing again while on doesn't extend.
pub fn buttons_lengths() -> Scenario {
    const STONE_B: [i32; 3] = [0, 1, 0];
    const WOOD_B: [i32; 3] = [3, 1, 0];
    let mut steps = vec![
        Step::Do(|c| {
            floor(c, [0, 0, 0], [3, 0, 0]);
            c.set(STONE_B, "redstone:stone_button[facing=up,on=false]");
            c.set(WOOD_B, "redstone:wood_button[facing=up,on=false]");
            c.set([1, 1, 0], "redstone:dust");
            COUNT.store(0, Ordering::Relaxed);
            COUNT2.store(0, Ordering::Relaxed);
        }),
        Step::Wait(2),
        Step::Do(|c| {
            redstone_api::press(c.w, c.at(STONE_B));
            redstone_api::press(c.w, c.at(WOOD_B));
        }),
    ];
    for i in 0..40 {
        steps.push(Step::Wait(1));
        steps.push(Step::Do(|c| {
            if c.prop(STONE_B, "on") == "true" {
                COUNT.fetch_add(1, Ordering::Relaxed);
            }
            if c.prop(WOOD_B, "on") == "true" {
                COUNT2.fetch_add(1, Ordering::Relaxed);
            }
        }));
        if i == 1 {
            steps.push(Step::Do(|c| c.check_eq(c.strength([1, 1, 0]), 15, "dust next to the pressed button")));
        }
        // Both still on.
        if i == 5 {
            steps.push(Step::Do(|c| {
                redstone_api::press(c.w, c.at(STONE_B));
                redstone_api::press(c.w, c.at(WOOD_B));
            }));
        }
    }
    steps.push(Step::Do(|c| {
        c.check_eq(COUNT.load(Ordering::Relaxed), redstone_core::BUTTON_STONE, "stone button ticks on");
        c.check_eq(COUNT2.load(Ordering::Relaxed), redstone_core::BUTTON_WOOD, "wood button ticks on");
        c.check_eq(c.strength([1, 1, 0]), 0, "dust after");
    }));
    Scenario::new("buttons_lengths", steps)
}

const PLATE: [i32; 3] = [0, 1, 0];

/// Entities on the plate's cell other than block entities.
fn on_plate(c: &Ctx) -> Vec<u64> {
    let p = c.at(PLATE);
    let center = [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5];
    c.w.near(center, 2.0, None).into_iter().filter(|id| c.w.block_entity(p) != *id && *id != c.player).collect()
}

/// An item dropped on the plate turns it on; gone, the plate turns off 20 ticks later.
pub fn plate_on_off() -> Scenario {
    Scenario::new(
        "plate_on_off",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [1, 0, 0]);
                c.set(PLATE, "redstone:pressure_plate[on=false]");
                c.set([1, 1, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| {
                let p = c.at(PLATE);
                let stone = weft_sdk::Item::lookup("base:stone").expect("stone item");
                c.w.spawn_item([p[0] as f64 + 0.5, p[1] as f64 + 0.3, p[2] as f64 + 0.5], ItemStack::new(stone, 1));
            }),
            Step::Until(|c| c.prop(PLATE, "on") == "true", 40),
            Step::Wait(2),
            Step::Do(|c| c.check_eq(c.strength([1, 1, 0]), 15, "dust next to the plate")),
            Step::Do(|c| {
                let items = on_plate(c);
                c.check(!items.is_empty(), "the item is on the plate");
                for id in items {
                    c.w.remove(id);
                }
                start(c);
            }),
            Step::Until(|c| c.prop(PLATE, "on") == "false", 40),
            Step::Do(|c| {
                let t = since(c);
                let want = redstone_core::PLATE_OFF as u64;
                c.check((want - 1..=want + 1).contains(&t), &format!("off {t} ticks after the item went, want {want} (±1 for the system's place in the tick)"));
            }),
        ],
    )
}
