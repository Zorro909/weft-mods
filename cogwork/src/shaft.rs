//! The torque shaft: a thin rod that carries energy between engines and machines. It is a
//! `conductor`, so the engine's energy networks join it with any mod's cables and machines.

use weft_sdk::*;

use crate::lib::{ModelBox, Pos, TypedWorld};

pub static TORQUE_SHAFT: Block = Block::named("torque_shaft");

/// The rod's middle: 4 texels wide in the middle of the block. Its arms are as thick.
const HUB: ModelBox = ModelBox::new([6.0; 3], [10.0; 3]);

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

fn model<'s>(props: &Props, state: State<'s>) -> State<'s> {
    let rod = |state: State<'s>, part: ModelBox| {
        // Its boxes are what it collides with, too.
        state.model_box_faces(part.lo, part.hi, [("torque_shaft", abi::UV_FROM_BOX); 6])
    };
    let mut state = rod(state.texture("torque_shaft"), HUB);
    for dir in Dir::ALL
        .into_iter()
        .filter(|dir| props.is(dir.name(), "true"))
    {
        state = rod(state, HUB.arm(dir));
    }
    state
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
