//! Another mod's emitters, receivers and wires, and the same circuit in two dimensions (plan
//! Task 11). The blocks are in `ext.rs`.

use std::sync::atomic::Ordering;

use redstone_api::{CHANGED, Changed, FULL, SidePowerV};
use weft_sdk::Dimension;

use crate::ext::{DYNAMO_OUTPUT, LOG};
use crate::scenario::{Ctx, Scenario, Step};

const STONE: &str = "base:stone";

fn floor(c: &mut Ctx, from: [i32; 3], to: [i32; 3]) {
    for x in from[0]..=to[0] {
        for z in from[2]..=to[2] {
            c.set([x, 0, z], STONE);
        }
    }
}

/// Dust next to a beacon gets its 7, and nothing once it is off.
pub fn ext_emitter_powers_dust() -> Scenario {
    Scenario::new(
        "ext_emitter_powers_dust",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [2, 0, 0]);
                c.set([0, 1, 0], "redstone_tests:beacon[on=true]");
                c.set([1, 1, 0], "redstone:dust");
                c.set([2, 1, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.check_eq(c.strength([1, 1, 0]), 7, "dust next to the beacon");
                c.check_eq(c.strength([2, 1, 0]), 6, "the next dust");
                c.check_eq(c.prop([1, 1, 0], "west"), "side".to_string(), "dust links to it");
                c.set([0, 1, 0], "redstone_tests:beacon[on=false]");
            }),
            Step::Wait(2),
            Step::Do(|c| c.check_eq(c.strength([1, 1, 0]), 0, "beacon off")),
        ],
    )
}

const COUNTER: [i32; 3] = [0, 1, 0];

/// A lever on top of the counter and a dust line into its east side: switching the lever on is
/// exactly one event, with what reaches each side.
pub fn ext_receiver_gets_sides() -> Scenario {
    Scenario::new(
        "ext_receiver_gets_sides",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [3, 0, 0]);
                c.set(COUNTER, "redstone_tests:counter");
                c.set([0, 2, 0], "redstone:lever[facing=up,powered=false]");
                c.set([1, 1, 0], "redstone:dust");
                c.set([2, 1, 0], "redstone:dust");
                c.set([3, 1, 0], "redstone:power_block");
            }),
            Step::Wait(3),
            Step::Do(|c| {
                let at = c.at(COUNTER);
                let got: Vec<_> = LOG.lock().unwrap().iter().filter(|e| e.pos == at).map(|e| e.sides).collect();
                c.check_eq(got.len(), 1, "one event for the dust");
                LOG.lock().unwrap().clear();
                c.set([0, 2, 0], "redstone:lever[facing=up,powered=true]");
            }),
            Step::Wait(4),
            Step::Do(|c| {
                let at = c.at(COUNTER);
                let got: Vec<_> = LOG.lock().unwrap().iter().filter(|e| e.pos == at).map(|e| e.sides).collect();
                let none = SidePowerV { strength: 0, strong: 0, direct: 0 };
                let mut want = [none; 6];
                // Face order -X +X -Y +Y -Z +Z.
                want[1] = SidePowerV { strength: 14, strong: 0, direct: 1 };
                want[3] = SidePowerV { strength: 15, strong: 1, direct: 1 };
                c.check_eq(got, vec![want], "events after the lever");
            }),
        ],
    )
}

/// Blue wire next to dust: neither links to the other; blue links to blue; it powers no block.
pub fn ext_wire_isolated() -> Scenario {
    Scenario::new(
        "ext_wire_isolated",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [3, 0, 1]);
                c.set([0, 1, 0], "redstone:lever[facing=up,powered=true]");
                for x in 1..=3 {
                    c.set([x, 1, 0], "redstone:dust");
                    c.set([x, 1, 1], "redstone_tests:blue_wire");
                }
                c.set([0, 1, 1], "redstone_tests:beacon[on=true]");
                // A torch on the block under the last blue wire.
                c.set([3, 0, 2], "redstone:torch[facing=south,lit=true]");
            }),
            Step::Wait(4),
            Step::Do(|c| {
                c.check_eq(c.strength([3, 1, 0]), 13, "dust");
                c.check_eq(c.strength([1, 1, 1]), 7, "blue next to the beacon");
                c.check_eq(c.strength([3, 1, 1]), 5, "blue, not fed by the dust beside it");
                c.check_eq(c.prop([3, 1, 1], "level"), "1".to_string(), "blue lit");
                c.check_eq(c.prop([2, 1, 0], "south"), "none".to_string(), "dust doesn't run to blue");
                c.check_eq(c.prop([3, 0, 2], "lit"), "true".to_string(), "the block under blue wire isn't powered");
                c.set([0, 1, 1], "redstone_tests:beacon[on=false]");
            }),
            Step::Wait(2),
            Step::Do(|c| c.check_eq(c.prop([3, 1, 1], "level"), "0".to_string(), "blue dark")),
        ],
    )
}

/// Dust reshapes the tick after a block announces it stopped connecting, not before.
pub fn ext_connectable_change() -> Scenario {
    fn shape(c: &Ctx) -> String {
        ["west", "east", "north", "south"].map(|d| format!("{d}={}", c.prop([1, 1, 0], d))).join(",")
    }
    Scenario::new(
        "ext_connectable_change",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [1, 0, 0]);
                c.set([0, 1, 0], "redstone_tests:switchable[open=true]");
                c.set([1, 1, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.check_eq(shape(c), "west=side,east=side,north=none,south=none".to_string(), "linked");
                c.set([0, 1, 0], "redstone_tests:switchable[open=false]");
                c.check_eq(shape(c), "west=side,east=side,north=none,south=none".to_string(), "in the tick it was announced");
            }),
            Step::Wait(1),
            Step::Do(|c| c.check_eq(shape(c), "west=side,east=side,north=side,south=side".to_string(), "alone")),
        ],
    )
}

/// An output from code: set the value, announce, dust follows.
pub fn ext_dynamic_output() -> Scenario {
    fn announce(c: &mut Ctx) {
        c.w.emit(&CHANGED, Changed { pos: c.at([0, 1, 0]) });
    }
    Scenario::new(
        "ext_dynamic_output",
        vec![
            Step::Do(|c| {
                DYNAMO_OUTPUT.store(0, Ordering::Relaxed);
                floor(c, [0, 0, 0], [1, 0, 0]);
                c.set([0, 1, 0], "redstone_tests:dynamo");
                c.set([1, 1, 0], "redstone:dust");
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.check_eq(c.strength([1, 1, 0]), 0, "at 0");
                DYNAMO_OUTPUT.store(9, Ordering::Relaxed);
                announce(c);
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.check_eq(c.strength([1, 1, 0]), 9, "at 9");
                DYNAMO_OUTPUT.store(FULL, Ordering::Relaxed);
                announce(c);
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.check_eq(c.strength([1, 1, 0]), 15, "at full, range 16");
                DYNAMO_OUTPUT.store(0, Ordering::Relaxed);
                announce(c);
            }),
            Step::Wait(2),
            Step::Do(|c| c.check_eq(c.strength([1, 1, 0]), 0, "back at 0")),
        ],
    )
}

static FLATLANDS: Dimension = Dimension::named("base:flatlands");
static OVERWORLD: Dimension = Dimension::named("base:overworld");
/// Where the player hovers in both dimensions: the circuits are at the same coordinates.
const AT: [f64; 3] = [8.5, 125.0, 8.5];

const LEVER: [i32; 3] = [0, 1, 0];
const LAMP: [i32; 3] = [4, 1, 0];

fn build(c: &mut Ctx) {
    floor(c, [0, 0, 0], [4, 0, 0]);
    c.set(LEVER, "redstone:lever[facing=up,powered=false]");
    for x in 1..=3 {
        c.set([x, 1, 0], "redstone:dust");
    }
    c.set(LAMP, "redstone:lamp[lit=false]");
}

/// The same lever, dust and lamp at the same coordinates in the flatlands and the overworld:
/// switching one on lights only its own lamp.
pub fn two_dimensions() -> Scenario {
    Scenario::new(
        "two_dimensions",
        vec![
            Step::Do(build),
            Step::Wait(3),
            Step::Do(|c| c.visit(&OVERWORLD, AT)),
            // Until the move happened: the same coordinates are loaded in the dimension it left.
            Step::Wait(10),
            Step::Until(|c| c.loaded(LAMP) && c.loaded(LEVER), 400),
            Step::Do(build),
            Step::Wait(3),
            Step::Do(|c| c.set(LEVER, "redstone:lever[facing=up,powered=true]")),
            Step::Wait(3),
            Step::Do(|c| {
                c.check_eq(c.prop(LAMP, "lit"), "true".to_string(), "overworld lamp");
                // The overworld copy goes again; the scenario clears the flatlands one.
                for x in (0..=4).rev() {
                    c.set([x, 1, 0], "core:air");
                    c.set([x, 0, 0], "core:air");
                }
                c.visit(&FLATLANDS, AT);
            }),
            Step::Wait(10),
            Step::Until(|c| c.loaded(LAMP) && c.loaded(LEVER), 400),
            Step::Wait(25),
            Step::Do(|c| {
                c.check_eq(c.prop(LEVER, "powered"), "false".to_string(), "flatlands lever");
                c.check_eq(c.strength([1, 1, 0]), 0, "flatlands dust");
                c.check_eq(c.prop(LAMP, "lit"), "false".to_string(), "flatlands lamp");
                c.set(LEVER, "redstone:lever[facing=up,powered=true]");
            }),
            Step::Wait(3),
            Step::Do(|c| c.check_eq(c.prop(LAMP, "lit"), "true".to_string(), "flatlands lamp, its own lever on")),
        ],
    )
    .in_dimension(&FLATLANDS, AT)
}
