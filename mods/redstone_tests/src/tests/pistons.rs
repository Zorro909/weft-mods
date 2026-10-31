//! Pistons and sticky pistons (plan Task 13). Every piston here sits at `P` pushing east, with
//! its lever on the south side unless said otherwise.

use crate::scenario::{Ctx, Scenario, Step};

const STONE: &str = "base:stone";
const P: [i32; 3] = [0, 1, 0];
const LEVER: [i32; 3] = [0, 1, 1];
const LEVER_OFF: &str = "redstone:lever[facing=up,powered=false]";
const LEVER_ON: &str = "redstone:lever[facing=up,powered=true]";
const PISTON: &str = "redstone:piston[facing=east,phase=retracted]";
const STICKY: &str = "redstone:sticky_piston[facing=east,phase=retracted]";
/// Enough for a move to start and land, whatever tick the change is seen in.
const MOVE: u32 = 6;

fn floor(c: &mut Ctx, from: [i32; 3], to: [i32; 3]) {
    for x in from[0]..=to[0] {
        for z in from[2]..=to[2] {
            c.set([x, 0, z], STONE);
        }
    }
}

/// The piston at `P`, its lever (off) and a floor to `x`.
fn piston(c: &mut Ctx, state: &str, x: i32) {
    floor(c, [0, 0, 0], [x, 0, 1]);
    c.set(P, state);
    c.set(LEVER, LEVER_OFF);
}

fn block(c: &Ctx, rel: [i32; 3]) -> String {
    c.get(rel).split('[').next().unwrap_or("").to_string()
}

fn phase(c: &Ctx, rel: [i32; 3]) -> String {
    c.prop(rel, "phase")
}

fn s(v: &str) -> String {
    v.to_string()
}

/// Three stones pushed one step, and left there when the head goes back. While moving, the
/// stones are where they were and the cell the far one moves into is held.
pub fn piston_pushes_line() -> Scenario {
    Scenario::new(
        "piston_pushes_line",
        vec![
            Step::Do(|c| {
                piston(c, PISTON, 5);
                for x in 1..=3 {
                    c.set([x, 1, 0], STONE);
                }
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Until(|c| block(c, [4, 1, 0]) != "core:air", MOVE),
            Step::Do(|c| {
                c.check_eq(block(c, [4, 1, 0]), s("redstone:moving"), "destination held");
                c.check_eq(block(c, [1, 1, 0]), s(STONE), "stones stay while moving");
                c.check_eq(phase(c, P), s("extending"), "phase");
            }),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("extended"), "extended");
                c.check_eq(block(c, [1, 1, 0]), s("redstone:piston_head"), "head");
                c.check_eq((2..=4).map(|x| block(c, [x, 1, 0])).collect::<Vec<_>>(), vec![s(STONE); 3], "stones moved");
                c.set(LEVER, LEVER_OFF);
            }),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "retracted");
                c.check_eq(block(c, [1, 1, 0]), s("core:air"), "head gone");
                c.check_eq(block(c, [4, 1, 0]), s(STONE), "stones stay");
            }),
        ],
    )
}

/// 12 blocks move, 13 don't: two pistons on one lever between them.
pub fn piston_limit() -> Scenario {
    Scenario::new(
        "piston_limit",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [14, 0, 2]);
                c.set(P, PISTON);
                c.set([0, 1, 2], PISTON);
                c.set(LEVER, LEVER_OFF);
                for x in 1..=12 {
                    c.set([x, 1, 0], STONE);
                }
                for x in 1..=13 {
                    c.set([x, 1, 2], STONE);
                }
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("extended"), "12 blocks");
                c.check_eq(block(c, [13, 1, 0]), s(STONE), "the 12th moved");
                c.check_eq(phase(c, [0, 1, 2]), s("retracted"), "13 blocks");
                c.check_eq(block(c, [1, 1, 2]), s(STONE), "nothing moved");
                c.check_eq(block(c, [14, 1, 2]), s("core:air"), "nothing moved at the end");
            }),
        ],
    )
}

/// A torch at the end of the line breaks; the stones take its place.
pub fn piston_breaks_torch_in_path() -> Scenario {
    Scenario::new(
        "piston_breaks_torch_in_path",
        vec![
            Step::Do(|c| {
                piston(c, PISTON, 4);
                c.set([1, 1, 0], STONE);
                c.set([2, 1, 0], STONE);
                c.set([3, 1, 0], "redstone:torch[facing=up,lit=true]");
            }),
            Step::Wait(3),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("extended"), "extended");
                c.check_eq(block(c, [3, 1, 0]), s(STONE), "stone where the torch was");
                c.check_eq(block(c, [4, 1, 0]), s("core:air"), "nothing past it");
            }),
        ],
    )
}

/// A power block in front of a sticky piston doesn't power it.
pub fn piston_not_powered_from_face() -> Scenario {
    Scenario::new(
        "piston_not_powered_from_face",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [2, 0, 0]);
                c.set(P, STICKY);
                c.set([1, 1, 0], "redstone:power_block");
            }),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "retracted");
                c.check_eq(block(c, [1, 1, 0]), s("redstone:power_block"), "power block stays");
            }),
        ],
    )
}

/// Blocked by a 13th block, extends as soon as it is taken away.
pub fn blocked_piston_extends_when_cleared() -> Scenario {
    Scenario::new(
        "blocked_piston_extends_when_cleared",
        vec![
            Step::Do(|c| {
                piston(c, PISTON, 14);
                for x in 1..=13 {
                    c.set([x, 1, 0], STONE);
                }
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "blocked");
                c.set([13, 1, 0], "core:air");
            }),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("extended"), "cleared");
                c.check_eq(block(c, [13, 1, 0]), s(STONE), "line moved");
            }),
        ],
    )
}

/// A sticky piston out long enough pulls its block back.
pub fn sticky_pull() -> Scenario {
    Scenario::new(
        "sticky_pull",
        vec![
            Step::Do(|c| {
                piston(c, STICKY, 3);
                c.set([1, 1, 0], STONE);
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(block(c, [2, 1, 0]), s(STONE), "pushed");
                c.check_eq(c.prop([1, 1, 0], "sticky"), s("true"), "sticky head");
                c.set(LEVER, LEVER_OFF);
            }),
            Step::Wait(MOVE),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "retracted");
                c.check_eq(block(c, [1, 1, 0]), s(STONE), "pulled back");
                c.check_eq(block(c, [2, 1, 0]), s("core:air"), "nothing left in front");
            }),
        ],
    )
}

/// One tick of power: the block snaps to where it was going and stays there.
pub fn sticky_drop_on_one_tick() -> Scenario {
    Scenario::new(
        "sticky_drop_on_one_tick",
        vec![
            Step::Do(|c| {
                piston(c, STICKY, 3);
                c.set([1, 1, 0], STONE);
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(1),
            Step::Do(|c| c.set(LEVER, LEVER_OFF)),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "retracted");
                c.check_eq(block(c, [2, 1, 0]), s(STONE), "dropped where it was pushed");
                c.check_eq(block(c, [1, 1, 0]), s("core:air"), "not pulled");
            }),
        ],
    )
}

/// Two heads into the same cell: neither moves until one loses power.
pub fn pistons_conflict_both_blocked() -> Scenario {
    const B: [i32; 3] = [2, 1, 0];
    Scenario::new(
        "pistons_conflict_both_blocked",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, 0], [2, 0, 1]);
                c.set(P, PISTON);
                c.set(B, "redstone:piston[facing=west,phase=retracted]");
                c.set(LEVER, LEVER_OFF);
                c.set([2, 1, 1], LEVER_OFF);
            }),
            Step::Wait(2),
            Step::Do(|c| {
                c.set(LEVER, LEVER_ON);
                c.set([2, 1, 1], LEVER_ON);
            }),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq((phase(c, P), phase(c, B)), (s("retracted"), s("retracted")), "both blocked");
                c.set([2, 1, 1], LEVER_OFF);
            }),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq((phase(c, P), phase(c, B)), (s("extended"), s("retracted")), "the one still powered goes");
                c.check_eq(c.prop([1, 1, 0], "facing"), s("east"), "its head");
            }),
        ],
    )
}

/// The player standing in front of the piston is pushed one block.
pub fn piston_pushes_player() -> Scenario {
    Scenario::new(
        "piston_pushes_player",
        vec![
            Step::Do(|c| {
                piston(c, PISTON, 4);
                c.stand([1, 1, 0]);
            }),
            Step::Wait(20),
            Step::Do(|c| {
                let x = c.player_at()[0];
                c.check((x - 1.5).abs() < 0.3, &format!("standing in front, x = {x}"));
                c.set(LEVER, LEVER_ON);
            }),
            Step::Wait(MOVE + 4),
            Step::Do(|c| {
                let x = c.player_at()[0];
                c.check((x - 2.5).abs() < 0.3, &format!("pushed one block, x = {x}"));
                c.check_eq(phase(c, P), s("extended"), "extended");
            }),
        ],
    )
}

/// A repeater counting down its delay is pushed and still fires in the same tick as one that
/// stayed. Both put out north and are fed from the south by a power block.
pub fn piston_pushes_own_repeater_with_pending() -> Scenario {
    const PUSHED: [i32; 3] = [1, 1, 0];
    const LANDED: [i32; 3] = [2, 1, 0];
    const STAYED: [i32; 3] = [1, 1, 3];
    const REPEATER: &str = "redstone:repeater[facing=south,delay=4,powered=false,locked=false]";
    Scenario::new(
        "piston_pushes_own_repeater_with_pending",
        vec![
            Step::Do(|c| {
                floor(c, [0, 0, -1], [3, 0, 4]);
                c.set(P, PISTON);
                c.set([0, 1, -1], LEVER_OFF);
                c.set(PUSHED, REPEATER);
                c.set(STAYED, REPEATER);
            }),
            Step::Wait(3),
            Step::Do(|c| {
                c.set([0, 1, -1], LEVER_ON);
                c.set([1, 1, 1], "redstone:power_block");
                c.set([1, 1, 4], "redstone:power_block");
            }),
            Step::Until(
                |c| {
                    let landed = c.prop(LANDED, "powered") == "true";
                    let stayed = c.prop(STAYED, "powered") == "true";
                    if landed != stayed {
                        c.fail(format!("pushed one powered {landed}, the other {stayed}"));
                    }
                    landed || stayed
                },
                15,
            ),
            Step::Do(|c| {
                c.check_eq(block(c, LANDED), s("redstone:repeater"), "pushed");
                c.check_eq(c.prop(LANDED, "delay"), s("4"), "delay kept");
                c.check_eq(phase(c, P), s("extended"), "extended");
            }),
        ],
    )
}

/// A chest (another mod's block entity) doesn't move, so the piston stays in.
pub fn chest_unpushable() -> Scenario {
    Scenario::new(
        "chest_unpushable",
        vec![
            Step::Do(|c| {
                piston(c, PISTON, 3);
                c.set([1, 1, 0], "base:chest[facing=north]");
            }),
            Step::Wait(2),
            Step::Do(|c| c.set(LEVER, LEVER_ON)),
            Step::Wait(MOVE + 2),
            Step::Do(|c| {
                c.check_eq(phase(c, P), s("retracted"), "blocked");
                c.check_eq(block(c, [1, 1, 0]), s("base:chest"), "chest stays");
            }),
        ],
    )
}
