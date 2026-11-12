//! The torque shaft: a thin rod that carries energy between engines and machines. It is a
//! `conductor`, so the engine's energy networks join it with any mod's cables and machines.
//! Its middle and its arms are drawn from the Blockbench model
//! `assets/models/block/torque_shaft/hub_and_arm.json` (`arms`).

use weft_sdk::{
    Block, CONDUCTOR, Dir, ENERGY, Props, Registry, State, WithComponents, World, tool_tier,
};

use crate::arms::Arm;
use crate::lib::{Pos, TypedWorld};

pub static TORQUE_SHAFT: Block = Block::named("torque_shaft");

pub fn register(r: &mut Registry) {
    let mut builder = r.block(&TORQUE_SHAFT);
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(model)
        .on_place(|world, change| refresh(world, change.pos.into()))
        .on_neighbor(|event| refresh(&event.world(), event.pos().into()))
        .tag("conductor")
        .hardness(0.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB);
}

/// The model with an arm toward every side it reaches. Its boxes are what it collides with, too.
fn model<'s>(props: &Props, state: State<'s>) -> State<'s> {
    // Its end stays: what it joins may be thinner, as another mod's cable.
    let arm = |dir: Dir| props.is(dir.name(), "true").then_some(Arm::Closed);
    let model = &crate::arms::TORQUE_SHAFT;
    let pieces = model.pieces(arm, |name| model.texture(name).to_string());
    crate::boxes::draw(state.texture(model.texture("particle")), &pieces)
}

/// Reaches toward every conductor and every block entity that holds energy. `CONDUCTOR` is the
/// engine's own tag.
fn refresh(world: &World, pos: Pos) {
    let shaft = world.block_typed(pos);
    if !shaft.is(&TORQUE_SHAFT) {
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
