//! The built-in blocks: their states and looks, and on every state the redstone components other
//! mods read (`EMITS`, `CONNECTS`, `RECEIVER`, `WIRE`). Which `Kind` a state is comes from a table
//! built from the states' properties once the game has loaded.

use std::collections::HashMap;
use std::sync::OnceLock;

use redstone_api::{CONNECTS, EMITS, Emits, SidePowerV, WIRE, WireType};
use redstone_core::{Kind, Link, Range, Shape};
use weft_sdk::*;

use crate::driver;

pub static DUST: Block = Block::named("dust");
pub static LEVER: Block = Block::named("lever");
pub static LAMP: Block = Block::named("lamp");
pub static POWER_BLOCK: Block = Block::named("power_block");

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
        for block in [&LEVER, &LAMP, &POWER_BLOCK] {
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
