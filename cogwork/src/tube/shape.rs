//! What a tube looks like and what it joins: a hub in the middle of the block and an arm toward
//! every neighbor it connects to, one `true`/`false` property per direction.

use weft_sdk::signal::Frame;
use weft_sdk::*;

use super::{DIRECTING_TUBE, TIMBER_TUBE, TUBES, step};

/// The tube's walls, in texels: 8 wide in the middle of the block.
const LO: f32 = 4.0;
const HI: f32 = 12.0;

/// A tube block: its properties, model, and joining up with its neighbors. `out_texture` marks
/// a directing tube, whose `out` arm looks different.
pub fn tube<'a>(
    b: BlockBuilder<'a>,
    texture: &'static str,
    out_texture: Option<&'static str>,
) -> BlockBuilder<'a> {
    let mut b = b
        .hardness(0.4)
        .mineable("pickaxe")
        .sound_group("base:block.glass")
        .tag("cogwork:tubes");
    for d in Dir::ALL {
        b = b.prop(d.name(), ["false", "true"]);
    }
    if out_texture.is_some() {
        b = b.prop("out", Dir::ALL.map(Dir::name));
    }
    b.states(move |p, s| model(p, s, texture, out_texture))
        .on_placed(|w, change| refresh(w, change.pos))
        .on_neighbor(|e| refresh(&e.world(), e.pos()))
        .on_broken(|w, change| super::visual::clear_broken(w, change.pos))
}

/// The hub and the connected arms, each a box of its own for collision too.
fn model<'s>(
    p: &Props,
    s: State<'s>,
    texture: &'static str,
    out_texture: Option<&'static str>,
) -> State<'s> {
    let mut s = s.texture(texture);
    s = with_box(s, texture, [LO; 3], [HI; 3]);
    for d in Dir::ALL.into_iter().filter(|d| p.is(d.name(), "true")) {
        let face_texture = match out_texture {
            Some(out) if p.get("out") == d.name() => out,
            _ => texture,
        };
        let (lo, hi) = arm(d, LO, HI);
        s = with_box(s, face_texture, lo, hi);
    }
    s
}

fn with_box<'s>(s: State<'s>, texture: &str, lo: [f32; 3], hi: [f32; 3]) -> State<'s> {
    // A floor facing north is the block's own axes; `-1` takes each face's picture from the box.
    let frame = Frame::new(Dir::Down, Dir::North);
    s.framed_box(frame, lo, hi, [(texture, [-1.0; 4]); 6])
        .collision(lo, hi)
}

/// The box from the hub's face toward `d` out to the edge of the block, `lo`..`hi` thick.
pub fn arm(d: Dir, lo: f32, hi: f32) -> ([f32; 3], [f32; 3]) {
    let axis = d.face() as usize / 2;
    let (mut min, mut max) = ([lo; 3], [hi; 3]);
    if d.face() % 2 == 0 {
        (min[axis], max[axis]) = (0.0, lo);
    } else {
        (min[axis], max[axis]) = (hi, 16.0);
    }
    (min, max)
}

/// Sets every arm property of the tube at `pos` to what its neighbors are now.
fn refresh(w: &World, pos: [i32; 3]) {
    let state = w.get(pos);
    if !TUBES.on_block(state) {
        return;
    }
    let mut next = state;
    for d in Dir::ALL {
        let on = if joins(w, pos, state, d) {
            "true"
        } else {
            "false"
        };
        next = state_with(next, d.name(), on).unwrap_or(next);
    }
    if next != state {
        w.set(pos, next);
    }
}

/// Whether the tube `state` at `pos` reaches toward `d`: to any other tube but two timber tubes,
/// and to anything holding items.
fn joins(w: &World, pos: [i32; 3], state: u32, d: Dir) -> bool {
    let at = step(pos, d.face());
    let other = w.get(at);
    if TUBES.on_block(other) {
        return !(TIMBER_TUBE.is(state) && TIMBER_TUBE.is(other));
    }
    crate::store::has_inventory(w, at).is_some()
}

/// Whether the tube at `pos` has its arm toward `face` out, so a neighbor can hand it items.
pub fn connects_to_tube(w: &World, pos: [i32; 3], face: u32) -> bool {
    w.prop(pos, Dir::from_face(face).name()) == Some("true")
}

/// The tuning wrench turns a directing tube's output to its next connected arm.
pub fn turn_output(u: &mut Use) -> bool {
    let w = u.world();
    if !crate::cogs::holds_wrench(&w, u.player()) {
        return false;
    }
    let pos = u.pos();
    let now = w
        .prop(pos, "out")
        .and_then(Dir::from_name)
        .map_or(0, Dir::face);
    let next = (1..=6)
        .map(|k| Dir::from_face((now + k) % 6))
        .find(|d| w.prop(pos, d.name()) == Some("true"));
    if let Some(d) = next.filter(|_| DIRECTING_TUBE.is(w.get(pos))) {
        w.set_prop(pos, "out", d.name());
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arms_reach_from_the_hub_to_the_edge() {
        assert_eq!(
            arm(Dir::West, LO, HI),
            ([0.0, 4.0, 4.0], [4.0, 12.0, 12.0]),
            "west"
        );
        assert_eq!(
            arm(Dir::Up, LO, HI),
            ([4.0, 12.0, 4.0], [12.0, 16.0, 12.0]),
            "up"
        );
        assert_eq!(
            arm(Dir::South, LO, HI),
            ([4.0, 4.0, 12.0], [12.0, 12.0, 16.0]),
            "south"
        );
    }
}
