//! Shafts: rods that carry energy between engines and machines, the iron torque shaft and the
//! wooden axle of the early game, which does the same for less. Each is a `conductor`, so the
//! engine's energy networks join it with any mod's cables and machines. Their middles and arms are
//! drawn from the Blockbench models `assets/models/block/<shaft>/hub_and_arm.json` (`arms`).

use std::sync::LazyLock;

use weft_sdk::{
    Block, CONDUCTOR, Dir, ENERGY, Props, Registry, State, WithComponents, World, tool_tier,
};

use crate::arms::{Arm, HubAndArms};
use crate::lib::{Pos, TypedWorld};

pub static TORQUE_SHAFT: Block = Block::named("torque_shaft");
pub static WOODEN_AXLE: Block = Block::named("wooden_axle");

/// A kind of shaft: its block, its model, the tool that mines it quickest and its sounds.
struct Shaft {
    block: &'static Block,
    model: &'static LazyLock<HubAndArms>,
    tool: &'static str,
    sound_group: &'static str,
}

static SHAFTS: [Shaft; 2] = [
    Shaft {
        block: &TORQUE_SHAFT,
        model: &crate::arms::TORQUE_SHAFT,
        tool: "pickaxe",
        sound_group: "base:block.stone",
    },
    Shaft {
        block: &WOODEN_AXLE,
        model: &crate::arms::WOODEN_AXLE,
        tool: "axe",
        sound_group: "base:block.wood",
    },
];

pub fn register(r: &mut Registry) {
    for shaft in &SHAFTS {
        register_shaft(r, shaft);
    }
}

fn register_shaft(r: &mut Registry, shaft: &'static Shaft) {
    let mut builder = r.block(shaft.block);
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(move |props, state| model(props, state, shaft.model))
        .on_place(move |world, change| refresh(world, change.pos.into(), shaft.block))
        .on_neighbor(move |event| refresh(&event.world(), event.pos().into(), shaft.block))
        .tag("conductor")
        .hardness(0.5)
        .mined_by(shaft.tool, tool_tier::HAND)
        .sound_group(shaft.sound_group)
        .tab(&crate::TAB);
}

/// The model with an arm toward every side it reaches. Its boxes are what it collides with, too.
fn model<'s>(props: &Props, state: State<'s>, model: &HubAndArms) -> State<'s> {
    // Its end stays: what it joins may be thinner, as another mod's cable.
    let arm = |dir: Dir| props.is(dir.name(), "true").then_some(Arm::Closed);
    let pieces = model.pieces(arm, |name| model.texture(name).to_string());
    crate::boxes::draw(state.texture(model.texture("particle")), &pieces)
}

/// Reaches the `block` shaft at `pos` toward every conductor and every block entity that holds
/// energy. `CONDUCTOR` is the engine's own tag.
fn refresh(world: &World, pos: Pos, block: &Block) {
    let shaft = world.block_typed(pos);
    if !shaft.is(block) {
        return;
    }
    let mut next = shaft;
    for dir in Dir::ALL {
        let at = pos.toward(dir);
        let joins = world.block_typed(at).has(&CONDUCTOR) || world.component(at, &ENERGY).is_some();
        next = next.with_flag(dir.name(), joins);
    }
    if next != shaft {
        world.set_block_typed(pos, next);
    }
}
