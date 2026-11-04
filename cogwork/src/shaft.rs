//! The torque shaft: a thin rod that carries energy between engines and machines. It is a
//! `conductor`, so the engine's energy networks join it with any mod's cables and machines.

use weft_sdk::*;

use crate::block_state::BlockState;
use crate::pos::Pos;

pub static TORQUE_SHAFT: Block = Block::named("torque_shaft");

/// The rod, in texels: 4 wide in the middle of the block.
const LO: f32 = 6.0;
const HI: f32 = 10.0;

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
    let rod = |s: State<'s>, lo: [f32; 3], hi: [f32; 3]| {
        // Its boxes are what it collides with, too.
        s.model_box_faces(lo, hi, [("torque_shaft", abi::UV_FROM_BOX); 6])
    };
    let mut s = rod(s.texture("torque_shaft"), [LO; 3], [HI; 3]);
    for d in Dir::ALL.into_iter().filter(|d| p.is(d.name(), "true")) {
        let (lo, hi) = crate::tube::arm(d, LO, HI);
        s = rod(s, lo, hi);
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
        let other = w.block_entity(at.into());
        let holds_energy = other != 0 && w.energy(other).is_some();
        let joins = BlockState::at(w, at).has(&CONDUCTOR) || holds_energy;
        next = next.with_flag(d.name(), joins);
    }
    if next != shaft {
        next.place(w, pos);
    }
}
