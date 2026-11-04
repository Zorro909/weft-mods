//! The torque shaft: a thin rod that carries energy between engines and machines. It is a
//! `conductor`, so the engine's energy networks join it with any mod's cables and machines.

use weft_sdk::*;

use crate::lib::{BlockState, ModelBox, Pos};

pub static TORQUE_SHAFT: Block = Block::named("torque_shaft");

/// The rod's middle: 4 texels wide in the middle of the block. Its arms are as thick.
const HUB: ModelBox = ModelBox::new([6.0; 3], [10.0; 3]);

pub fn register(r: &mut Registry) {
    let mut b = r.block(&TORQUE_SHAFT);
    for d in Dir::ALL {
        b = b.prop(d.name(), ["false", "true"]);
    }
    b.states(model)
        .on_placed(|w, change| refresh(w, change.pos.into()))
        .on_neighbor(|e| refresh(&e.world(), e.pos().into()))
        .tag("conductor")
        .hardness(0.5)
        .mineable("pickaxe")
        .sound_group("base:block.stone")
        .tab(&crate::TAB);
}

fn model<'s>(p: &Props, s: State<'s>) -> State<'s> {
    let rod = |s: State<'s>, part: ModelBox| {
        // Its boxes are what it collides with, too.
        s.model_box_faces(part.lo, part.hi, [("torque_shaft", abi::UV_FROM_BOX); 6])
    };
    let mut s = rod(s.texture("torque_shaft"), HUB);
    for d in Dir::ALL.into_iter().filter(|d| p.is(d.name(), "true")) {
        s = rod(s, HUB.arm(d));
    }
    s
}

/// Reaches toward every conductor and every block entity that holds energy. `CONDUCTOR` is the
/// engine's own tag.
fn refresh(w: &World, pos: Pos) {
    let shaft = BlockState::at(w, pos);
    if !shaft.is(&TORQUE_SHAFT) {
        return;
    }
    let mut next = shaft;
    for d in Dir::ALL {
        let at = pos.toward(d);
        let joins = BlockState::at(w, at).has(&CONDUCTOR) || w.component(at, &ENERGY).is_some();
        next = next.with_flag(d.name(), joins);
    }
    if next != shaft {
        next.place(w, pos);
    }
}
