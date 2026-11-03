//! What a tube looks like and what it joins: a hub in the middle of the block and an arm toward
//! every neighbor it connects to, one `true`/`false` property per direction. A directing tube
//! always has its arm toward `out`, so it keeps properties only for the other five. A sorting
//! tube draws each direction in its own texture.

use weft_sdk::*;

use super::{DIRECTING_TUBE, TIMBER_TUBE, TUBES};
use crate::neighbor;

/// The tube's walls, in texels: 8 wide in the middle of the block.
const LO: f32 = 4.0;
const HI: f32 = 12.0;
/// A directing tube's arms besides its output, by their place among the other five directions.
const OTHER_ARMS: [&str; 5] = ["arm1", "arm2", "arm3", "arm4", "arm5"];
const BOOL: [&str; 2] = ["false", "true"];

/// How a tube's arms look besides its own texture.
#[derive(Clone, Copy)]
pub enum Arms {
    /// All in the tube's texture.
    Same,
    /// A directing tube: its `out` arm in this texture.
    Output(&'static str),
    /// A sorting tube: the arm and hub face toward each direction in that direction's texture.
    PerSide(fn(Dir) -> &'static str),
}

/// A tube block: its properties, model, and joining up with its neighbors.
pub fn tube<'a>(b: BlockBuilder<'a>, texture: &'static str, arms: Arms) -> BlockBuilder<'a> {
    let mut b = b
        .hardness(0.4)
        .mineable("pickaxe")
        .sound_group("base:block.glass")
        .tag("cogwork:tubes");
    if matches!(arms, Arms::Output(_)) {
        b = b.prop("out", Dir::ALL.map(Dir::name));
        for name in OTHER_ARMS {
            b = b.prop(name, BOOL);
        }
    } else {
        for d in Dir::ALL {
            b = b.prop(d.name(), BOOL);
        }
    }
    b.states(move |p, s| model(p, s, texture, arms))
        .on_placed(|w, change| refresh(w, change.pos))
        .on_neighbor(|e| refresh(&e.world(), e.pos()))
        .on_broken(|w, change| super::visual::clear_broken(w, change.pos))
}

/// The hub and the connected arms, each a box of its own, which is what it collides with too.
fn model<'s>(p: &Props, s: State<'s>, texture: &'static str, arms: Arms) -> State<'s> {
    let side = |d| side_texture(texture, arms, d);
    let hub = Dir::ALL.map(|d| (side(d), abi::UV_FROM_BOX));
    let mut s = s.texture(texture).model_box_faces([LO; 3], [HI; 3], hub);
    let (out, out_texture) = match arms {
        Arms::Output(out_texture) => (Dir::from_name(p.get("out")), out_texture),
        _ => (None, texture),
    };
    for d in Dir::ALL {
        let face_texture = match arm_prop(out, d) {
            None => out_texture,
            Some(name) if p.is(name, "true") => side(d),
            Some(_) => continue,
        };
        let (lo, hi) = arm(d, LO, HI);
        s = s.model_box_faces(lo, hi, [(face_texture, abi::UV_FROM_BOX); 6]);
    }
    s
}

/// The texture of the hub face and arm toward `d`, unless that arm is a directing tube's output.
fn side_texture(texture: &'static str, arms: Arms, d: Dir) -> &'static str {
    match arms {
        Arms::PerSide(texture_toward) => texture_toward(d),
        _ => texture,
    }
}

/// The box from the hub's face toward `d` out to the edge of the block, `lo`..`hi` thick.
pub fn arm(d: Dir, lo: f32, hi: f32) -> ([f32; 3], [f32; 3]) {
    let axis = d.face() as usize / 2;
    let (mut min, mut max) = ([lo; 3], [hi; 3]);
    if d.face().is_multiple_of(2) {
        (min[axis], max[axis]) = (0.0, lo);
    } else {
        (min[axis], max[axis]) = (hi, 16.0);
    }
    (min, max)
}

/// The property saying whether a tube has its arm toward `d`: the direction's own on a plain
/// tube. On a directing tube with its output toward `out`, `d`'s place among the other five
/// directions, and none for the output, whose arm is always there.
fn arm_prop(out: Option<Dir>, d: Dir) -> Option<&'static str> {
    let Some(out) = out else {
        return Some(d.name());
    };
    let mut others = Dir::ALL.into_iter().filter(|&o| o != out);
    others.position(|o| o == d).map(|i| OTHER_ARMS[i])
}

/// Where the tube `state` sends its items, if it is a directing tube.
fn output(state: u32) -> Option<Dir> {
    state_prop(state, "out").and_then(Dir::from_name)
}

/// Sets every arm property of the tube at `pos` to what its neighbors are now.
fn refresh(w: &World, pos: [i32; 3]) {
    let state = w.get(pos);
    if !TUBES.on_block(state) {
        return;
    }
    let next = joined(w, pos, state);
    if next != state {
        w.set(pos, next);
    }
}

/// `state` with its arm properties set to which neighbors of `pos` it joins.
fn joined(w: &World, pos: [i32; 3], state: u32) -> u32 {
    let out = output(state);
    Dir::ALL.into_iter().fold(state, |next, d| {
        let Some(name) = arm_prop(out, d) else {
            return next;
        };
        let on = if joins(w, pos, state, d) {
            "true"
        } else {
            "false"
        };
        state_with(next, name, on).unwrap_or(next)
    })
}

/// Whether the tube `state` at `pos` reaches toward `d`: to any other tube but two timber tubes,
/// and to anything holding items.
fn joins(w: &World, pos: [i32; 3], state: u32, d: Dir) -> bool {
    let at = neighbor(pos, d.face());
    let other = w.get(at);
    if TUBES.on_block(other) {
        return !(TIMBER_TUBE.is(state) && TIMBER_TUBE.is(other));
    }
    crate::store::has_inventory(w, at)
}

/// Whether the tube at `pos` has its arm toward `face` out, so a neighbor can hand it items.
pub fn connects_to_tube(w: &World, pos: [i32; 3], face: u32) -> bool {
    let state = w.get(pos);
    let d = Dir::from_face(face);
    match arm_prop(output(state), d) {
        Some(name) => state_prop(state, name) == Some("true"),
        None => joins(w, pos, state, d),
    }
}

/// The tuning wrench turns a directing tube's output to its next connected arm.
pub fn turn_output(u: &mut Use) -> bool {
    let w = u.world();
    if !crate::cogs::holds_wrench(&w, u.player()) {
        return false;
    }
    let pos = u.pos();
    let state = w.get(pos);
    let Some(now) = output(state).filter(|_| DIRECTING_TUBE.is(state)) else {
        return true;
    };
    let next = (1..6)
        .map(|k| Dir::from_face((now.face() + k) % 6))
        .find(|d| connects_to_tube(&w, pos, d.face()));
    if let Some(turned) = next.and_then(|d| state_with(state, "out", d.name())) {
        w.set(pos, joined(&w, pos, turned));
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

    #[test]
    fn a_directing_tube_names_its_arms_after_the_output() {
        assert_eq!(
            arm_prop(None, Dir::Up),
            Some("up"),
            "plain tubes by direction"
        );
        assert_eq!(
            arm_prop(Some(Dir::Up), Dir::Up),
            None,
            "the output has none"
        );
        for out in Dir::ALL {
            let mut names: Vec<_> = Dir::ALL
                .iter()
                .filter_map(|&d| arm_prop(Some(out), d))
                .collect();
            names.sort();
            assert_eq!(
                names,
                OTHER_ARMS,
                "the others get one each, toward {}",
                out.name()
            );
        }
    }
}
