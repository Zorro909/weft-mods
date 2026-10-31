//! The built-in blocks: their states and looks, and on every state the redstone components other
//! mods read (`EMITS`, `CONNECTS`, `RECEIVER`, `WIRE`). Which `Kind` a state is comes from a table
//! built from the states' properties once the game has loaded.

use std::collections::HashMap;
use std::sync::OnceLock;

use redstone_api::{CONNECTS, EMITS, Emits, SidePowerV, WIRE, WireType};
use redstone_core::{Kind, Link, Range, Shape};
use weft_sdk::*;

use crate::{driver, persist, plates};

pub static DUST: Block = Block::named("dust");
pub static LEVER: Block = Block::named("lever");
pub static LAMP: Block = Block::named("lamp");
pub static POWER_BLOCK: Block = Block::named("power_block");
pub static TORCH: Block = Block::named("torch");
pub static REPEATER: Block = Block::named("repeater");
pub static COMPARATOR: Block = Block::named("comparator");
pub static STONE_BUTTON: Block = Block::named("stone_button");
pub static WOOD_BUTTON: Block = Block::named("wood_button");
pub static PLATE: Block = Block::named("pressure_plate");

pub static TAB: CreativeTab = CreativeTab::named("redstone");
static DUST_ITEM: Item = Item::named("dust");

/// Brightness levels of dust (spec §8).
pub const DUST_LEVELS: u8 = 4;
/// Dust's side properties, in `redstone_core::Dir::HORIZONTAL` order.
const SIDES: [&str; 4] = ["west", "east", "north", "south"];
const SIDE_VALUES: [&str; 3] = ["none", "side", "up"];

/// How high dust lies, in pixels.
const DUST_H: f32 = 0.25;

pub fn register(r: &mut Registry) {
    r.creative_tab(&TAB, &DUST_ITEM);

    r.block(&DUST)
        .prop("level", 0..=(DUST_LEVELS as u32 - 1))
        .prop(SIDES[0], SIDE_VALUES)
        .prop(SIDES[1], SIDE_VALUES)
        .prop(SIDES[2], SIDE_VALUES)
        .prop(SIDES[3], SIDE_VALUES)
        .hardness(0.0)
        .not_solid()
        .needs(Dir::Down.face(), "")
        .item_texture("dust")
        .tab(&TAB)
        .set(&WIRE, WireType { kind: 0, links: u32::MAX, powers_blocks: 1 })
        .on_neighbor(driver::neighbor)
        .states(|p, s| {
            let mut s = s.texture(&format!("dust_{}", p.int("level")));
            s = s.model_box([5.0, 0.0, 5.0], [11.0, DUST_H, 11.0]);
            for (i, side) in SIDES.iter().enumerate() {
                let v = p.get(side);
                if v == "none" {
                    continue;
                }
                let (lo, hi) = arm(i);
                s = s.model_box(lo, hi);
                if v == "up" {
                    let (lo, hi) = climb(i);
                    s = s.model_box(lo, hi);
                }
            }
            s
        });

    r.block(&LEVER)
        .facing(Facings::ALL, Toward::Face)
        .prop("powered", ["false", "true"])
        .hardness(0.5)
        .not_solid()
        .texture("lever")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .on_use(|u| {
            let (w, p) = (u.world(), u.pos());
            let on = w.prop(p, "powered") == Some("true");
            w.set_prop(p, "powered", if on { "false" } else { "true" });
            driver::queue(p);
            true
        })
        .states(|p, s| {
            let k = lever_kind(p);
            let Kind::Lever { on, attached } = k else { unreachable!() };
            // Drawn for a lever on the floor, then turned onto its block's face.
            let handle = if on { ([7.0, 3.0, 9.0], [9.0, 10.0, 11.0]) } else { ([7.0, 3.0, 5.0], [9.0, 10.0, 7.0]) };
            let (blo, bhi) = onto(attached, [5.0, 0.0, 4.0], [11.0, 3.0, 12.0]);
            let (hlo, hhi) = onto(attached, handle.0, handle.1);
            device_components(s.needs(attached.index() as u32, "").model_box(blo, bhi).model_box(hlo, hhi), k)
        });

    r.block(&LAMP)
        .prop("lit", ["false", "true"])
        .hardness(0.3)
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .states(|p, s| {
            let lit = p.is("lit", "true");
            let s = if lit { s.texture("lamp_on").light(15) } else { s.texture("lamp_off") };
            device_components(s, Kind::Lamp { lit })
        });

    r.block(&POWER_BLOCK)
        .hardness(1.0)
        .texture("power_block")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .states(|_, s| device_components(s, Kind::PowerBlock));

    // Standing (`facing=up`) or on a wall, facing away from it. Placed lit.
    r.block(&TORCH)
        .facing(Facings::only(&[Dir::Up, Dir::North, Dir::East, Dir::South, Dir::West]), Toward::Face)
        .prop("lit", ["true", "false"])
        .hardness(0.0)
        .not_solid()
        .item_texture("torch")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .states(|p, s| {
            let k = torch_kind(p);
            let Kind::Torch { lit, attached } = k else { unreachable!() };
            // Wall torches stand straight against the wall, like base's.
            let (lo, hi) = match attached {
                redstone_core::Dir::North => ([7.0, 3.0, 0.0], [9.0, 13.0, 2.0]),
                redstone_core::Dir::East => ([14.0, 3.0, 7.0], [16.0, 13.0, 9.0]),
                redstone_core::Dir::South => ([7.0, 3.0, 14.0], [9.0, 13.0, 16.0]),
                redstone_core::Dir::West => ([0.0, 3.0, 7.0], [2.0, 13.0, 9.0]),
                _ => ([7.0, 0.0, 7.0], [9.0, 10.0, 9.0]),
            };
            let s = if lit { s.texture("torch_on").light(7) } else { s.texture("torch_off") };
            device_components(s.needs(attached.index() as u32, "").model_box(lo, hi), k)
        });

    // Repeaters and comparators face whoever placed them and put out the other way.
    r.block(&REPEATER)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("delay", 1..=4)
        .prop("powered", ["false", "true"])
        .prop("locked", ["false", "true"])
        .hardness(0.0)
        .not_solid()
        .needs(Dir::Down.face(), "")
        .item_texture("repeater")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .on_use(|u| {
            let (w, p) = (u.world(), u.pos());
            let d = w.prop(p, "delay").and_then(|d| d.parse::<u32>().ok()).unwrap_or(1);
            w.set_prop(p, "delay", &(d % 4 + 1).to_string());
            driver::queue(p);
            true
        })
        .states(|p, s| {
            let k = repeater_kind(p);
            let Kind::Repeater { facing, delay, powered, locked } = k else { unreachable!() };
            let mut s = s.texture(if powered { "repeater_on" } else { "repeater_off" }).model_box([0.0, 0.0, 0.0], [16.0, 2.0, 16.0]);
            // Drawn putting out to the east: the output post, and the delay post further back the
            // shorter the delay.
            let back = 9.0 - 2.0 * (delay as f32 - 1.0);
            let mut boxes = vec![([11.0, 2.0, 7.0], [13.0, 7.0, 9.0]), ([back - 2.0, 2.0, 7.0], [back, 7.0, 9.0])];
            if locked {
                boxes.push(([back - 2.0, 2.0, 2.0], [back, 4.0, 14.0]));
            }
            for (lo, hi) in boxes {
                let (lo, hi) = turn(facing, lo, hi);
                s = s.model_box(lo, hi);
            }
            device_components(s, k)
        });

    r.block(&COMPARATOR)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .prop("mode", ["compare", "subtract"])
        .prop("output", 0..=63)
        .hardness(0.0)
        .not_solid()
        .needs(Dir::Down.face(), "")
        .item_texture("comparator")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .on_use(|u| {
            let (w, p) = (u.world(), u.pos());
            let subtract = w.prop(p, "mode") == Some("subtract");
            w.set_prop(p, "mode", if subtract { "compare" } else { "subtract" });
            driver::queue(p);
            true
        })
        .states(|p, s| {
            let k = comparator_kind(p);
            let Kind::Comparator { facing, subtract, output } = k else { unreachable!() };
            let mut s = s.texture(if output > 0 { "comparator_on" } else { "comparator_off" }).model_box([0.0, 0.0, 0.0], [16.0, 2.0, 16.0]);
            // Two back posts, and the front one taller in subtract mode.
            let front = if subtract { 7.0 } else { 5.0 };
            for (lo, hi) in [([3.0, 2.0, 3.0], [5.0, 7.0, 5.0]), ([3.0, 2.0, 11.0], [5.0, 7.0, 13.0]), ([11.0, 2.0, 7.0], [13.0, front, 9.0])] {
                let (lo, hi) = turn(facing, lo, hi);
                s = s.model_box(lo, hi);
            }
            device_components(s, k)
        });

    for (block, wood) in [(&STONE_BUTTON, false), (&WOOD_BUTTON, true)] {
        r.block(block)
            .facing(Facings::ALL, Toward::Face)
            .prop("on", ["false", "true"])
            .hardness(0.5)
            .not_solid()
            .texture(if wood { "button_wood" } else { "button_stone" })
            .tab(&TAB)
            .on_neighbor(driver::neighbor)
            .on_use(|u| {
                driver::press(&u.world(), u.pos());
                true
            })
            .states(move |p, s| {
                let k = Kind::Button { on: p.is("on", "true"), attached: core_dir(p.facing()).opposite(), wood };
                let Kind::Button { on, attached, .. } = k else { unreachable!() };
                let (lo, hi) = onto(attached, [5.0, 0.0, 6.0], [11.0, if on { 1.0 } else { 2.0 }, 10.0]);
                device_components(s.needs(attached.index() as u32, "").model_box(lo, hi), k)
            });
    }

    r.block(&PLATE)
        .prop("on", ["false", "true"])
        .hardness(0.5)
        .not_solid()
        .needs(Dir::Down.face(), "")
        .texture("pressure_plate")
        .tab(&TAB)
        .on_neighbor(driver::neighbor)
        .states(|p, s| {
            let on = p.is("on", "true");
            device_components(s.model_box([1.0, 0.0, 1.0], [15.0, if on { 0.5 } else { 1.0 }, 15.0]), Kind::Plate { on })
        });

    // Devices that can have a change pending keep it in their block entity; plates keep when
    // they last saw someone in theirs.
    persist::register(r, &[&TORCH, &LAMP, &REPEATER, &COMPARATOR, &STONE_BUTTON, &WOOD_BUTTON]);
    plates::register(r);
}

/// A box drawn for a device putting out to the east, turned to put out to `out`.
fn turn(out: redstone_core::Dir, lo: [f32; 3], hi: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    use redstone_core::Dir as D;
    let map = |[x, y, z]: [f32; 3]| match out {
        D::West => [16.0 - x, y, 16.0 - z],
        D::South => [16.0 - z, y, x],
        D::North => [z, y, 16.0 - x],
        _ => [x, y, z],
    };
    let (p, q) = (map(lo), map(hi));
    (std::array::from_fn(|i| p[i].min(q[i])), std::array::from_fn(|i| p[i].max(q[i])))
}

/// The flat arm of dust toward side `i` (`Dir::HORIZONTAL` order), in pixels.
fn arm(i: usize) -> ([f32; 3], [f32; 3]) {
    match i {
        0 => ([0.0, 0.0, 5.0], [5.0, DUST_H, 11.0]),
        1 => ([11.0, 0.0, 5.0], [16.0, DUST_H, 11.0]),
        2 => ([5.0, 0.0, 0.0], [11.0, DUST_H, 5.0]),
        _ => ([5.0, 0.0, 11.0], [11.0, DUST_H, 16.0]),
    }
}

/// The piece of dust running up the side of the neighbor block on side `i`.
fn climb(i: usize) -> ([f32; 3], [f32; 3]) {
    match i {
        0 => ([0.0, 0.0, 5.0], [DUST_H, 16.0, 11.0]),
        1 => ([16.0 - DUST_H, 0.0, 5.0], [16.0, 16.0, 11.0]),
        2 => ([5.0, 0.0, 0.0], [11.0, 16.0, DUST_H]),
        _ => ([5.0, 0.0, 16.0 - DUST_H], [11.0, 16.0, 16.0]),
    }
}

/// A box drawn for something on the floor (height = y), moved onto the face `attached` of its
/// block: the height axis points away from that face.
fn onto(attached: redstone_core::Dir, lo: [f32; 3], hi: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    use redstone_core::Dir as D;
    let map = |[a, h, b]: [f32; 3]| match attached {
        D::Down => [a, h, b],
        D::Up => [a, 16.0 - h, b],
        D::West => [h, a, b],
        D::East => [16.0 - h, a, b],
        D::North => [a, b, h],
        D::South => [a, b, 16.0 - h],
    };
    let (p, q) = (map(lo), map(hi));
    (std::array::from_fn(|i| p[i].min(q[i])), std::array::from_fn(|i| p[i].max(q[i])))
}

fn lever_kind(p: &Props) -> Kind {
    Kind::Lever { on: p.is("powered", "true"), attached: core_dir(p.facing()).opposite() }
}

fn repeater_kind(p: &Props) -> Kind {
    let delay = p.int("delay") as u8;
    Kind::Repeater { facing: core_dir(p.facing()).opposite(), delay, powered: p.is("powered", "true"), locked: p.is("locked", "true") }
}

fn comparator_kind(p: &Props) -> Kind {
    Kind::Comparator { facing: core_dir(p.facing()).opposite(), subtract: p.is("mode", "subtract"), output: p.int("output") as u8 }
}

fn torch_kind(p: &Props) -> Kind {
    Kind::Torch { lit: p.is("lit", "true"), attached: core_dir(p.facing()).opposite() }
}

pub fn core_dir(d: Dir) -> redstone_core::Dir {
    redstone_core::Dir::from_index(d.face() as usize)
}

/// `EMITS`, `CONNECTS` and the `RECEIVER` tag of a built-in device's state, so other mods see it
/// like any emitter and receiver.
fn device_components(s: State<'_>, k: Kind) -> State<'_> {
    let d = redstone_core::device(k, Range::UNCLAMPED);
    let sides = d.output.map(|o| SidePowerV { strength: o.strength, strong: o.strong as u8, direct: 1 });
    let s = s.set(&EMITS, Emits { sides }).set(&CONNECTS, d.connects);
    if d.receives { s.tag("redstone:receiver") } else { s }
}

/// What the built-in device states are, both ways.
struct Tables {
    kinds: HashMap<u32, Kind>,
    states: HashMap<Kind, u32>,
}

fn tables() -> &'static Tables {
    static T: OnceLock<Tables> = OnceLock::new();
    T.get_or_init(|| {
        let mut kinds = HashMap::new();
        for block in [&LEVER, &LAMP, &POWER_BLOCK, &TORCH, &REPEATER, &COMPARATOR, &STONE_BUTTON, &WOOD_BUTTON, &PLATE] {
            let mut s = block.id();
            while s != u32::MAX && block.is(s) {
                if let Some(k) = kind_from_state(block, s) {
                    kinds.insert(s, k);
                }
                s += 1;
            }
        }
        let states = kinds.iter().map(|(s, k)| (*k, *s)).collect();
        check_dust_layout();
        Tables { kinds, states }
    })
}

fn kind_from_state(block: &Block, s: u32) -> Option<Kind> {
    let prop = |n: &str| state_prop(s, n).unwrap_or("");
    let k = if std::ptr::eq(block, &LEVER) {
        let facing = Dir::from_name(prop("facing"))?;
        Kind::Lever { on: prop("powered") == "true", attached: core_dir(facing).opposite() }
    } else if std::ptr::eq(block, &LAMP) {
        Kind::Lamp { lit: prop("lit") == "true" }
    } else if std::ptr::eq(block, &TORCH) {
        let facing = Dir::from_name(prop("facing"))?;
        Kind::Torch { lit: prop("lit") == "true", attached: core_dir(facing).opposite() }
    } else if std::ptr::eq(block, &REPEATER) {
        let facing = core_dir(Dir::from_name(prop("facing"))?).opposite();
        Kind::Repeater { facing, delay: prop("delay").parse().ok()?, powered: prop("powered") == "true", locked: prop("locked") == "true" }
    } else if std::ptr::eq(block, &COMPARATOR) {
        let facing = core_dir(Dir::from_name(prop("facing"))?).opposite();
        Kind::Comparator { facing, subtract: prop("mode") == "subtract", output: prop("output").parse().ok()? }
    } else if std::ptr::eq(block, &STONE_BUTTON) || std::ptr::eq(block, &WOOD_BUTTON) {
        let facing = Dir::from_name(prop("facing"))?;
        Kind::Button { on: prop("on") == "true", attached: core_dir(facing).opposite(), wood: std::ptr::eq(block, &WOOD_BUTTON) }
    } else if std::ptr::eq(block, &PLATE) {
        Kind::Plate { on: prop("on") == "true" }
    } else {
        Kind::PowerBlock
    };
    Some(k)
}

/// The built-in device a state is, if any.
pub fn kind_of(state: u32) -> Option<Kind> {
    tables().kinds.get(&state).copied()
}

/// The state of a built-in device.
pub fn state_of(k: Kind) -> Option<u32> {
    tables().states.get(&k).copied()
}

fn side_value(shape: &Shape, i: usize) -> u32 {
    if shape.sides[i] == Link::Up {
        2
    } else if shape.points[i] {
        1
    } else {
        0
    }
}

/// Dust's state for a brightness level and shape. States go through every combination of the
/// properties in order, the last changing fastest (`level`, then the four sides).
pub fn dust_state(level: u8, shape: &Shape) -> u32 {
    let i = (0..4).fold(level as u32, |acc, i| acc * 3 + side_value(shape, i));
    DUST.id() + i
}

/// `dust_state` relies on the engine's state order; say so loudly if that ever changes.
fn check_dust_layout() {
    let shape = Shape { sides: [Link::Up, Link::None, Link::Flat, Link::None], points: [true, false, true, true] };
    let want = DUST.with(&[("level", "2"), ("west", "up"), ("east", "none"), ("north", "side"), ("south", "side")]);
    if dust_state(2, &shape) != want {
        error!("dust state layout changed: computed {} but the engine says {want}", dust_state(2, &shape));
    }
}
