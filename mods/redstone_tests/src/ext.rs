//! Blocks of "another mod", built on `redstone_api` alone, the way a mod author would (see
//! `docs/redstone-api.md`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use redstone_api::{CONNECTS, EMITS, Emits, INPUT_CHANGED, Inputs, OUTPUT, RECEIVER, SidePowerV, WIRE, WireType};
use weft_sdk::*;

/// A solid emitter: weak 7 to every side while `on`.
pub static BEACON: Block = Block::named("beacon");
/// A receiver that writes down every `INPUT_CHANGED` it gets.
pub static COUNTER: Block = Block::named("counter");
/// A wire of its own kind: links only to itself, powers no block, two brightness levels.
pub static BLUE_WIRE: Block = Block::named("blue_wire");
/// Wire connects to it on every side while `open`, on none otherwise.
pub static SWITCHABLE: Block = Block::named("switchable");
/// An emitter whose output comes from code (`DYNAMO_OUTPUT`) through the `OUTPUT` chain.
pub static DYNAMO: Block = Block::named("dynamo");

/// What every counter got, in order.
pub static LOG: Mutex<Vec<Inputs>> = Mutex::new(Vec::new());
/// The dynamo's weak output to every side; 255 is the world's maximum.
pub static DYNAMO_OUTPUT: AtomicU8 = AtomicU8::new(0);

const ALL_SIDES: u8 = 0b11_1111;

fn weak(strength: u8) -> Emits {
    Emits { sides: [SidePowerV { strength, strong: 0, direct: 1 }; 6] }
}

pub fn register(r: &mut Registry) {
    r.block(&BEACON).prop("on", ["false", "true"]).texture("base:gold_block").hardness(0.5).states(|p, s| {
        let s = s.set(&CONNECTS, ALL_SIDES);
        if p.is("on", "true") { s.set(&EMITS, weak(7)) } else { s }
    });
    r.block(&COUNTER).texture("base:diamond_block").hardness(0.5).tag(RECEIVER.name());
    r.on(&INPUT_CHANGED, |b| LOG.lock().unwrap().extend(b.iter().copied()));
    r.block(&BLUE_WIRE)
        .prop("level", 0..=1)
        .texture("base:glass_blue")
        .hardness(0.0)
        .not_solid()
        .model_box([0.0, 0.0, 0.0], [16.0, 1.0, 16.0])
        .set(&WIRE, WireType { kind: 1, links: 1 << 1, powers_blocks: 0 });
    r.block(&SWITCHABLE).prop("open", ["true", "false"]).texture("base:iron_block").hardness(0.5).states(|p, s| s.set(&CONNECTS, if p.is("open", "true") { ALL_SIDES } else { 0 }));
    r.block(&DYNAMO).texture("base:gold_ore").hardness(0.5).set(&EMITS, weak(0)).set(&CONNECTS, ALL_SIDES);
    r.modify(&OUTPUT, stage::ADD, |m| {
        if DYNAMO.is(m.query().state) {
            m.value = DYNAMO_OUTPUT.load(Ordering::Relaxed) as f64;
        }
    });
}
