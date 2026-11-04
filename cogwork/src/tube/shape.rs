//! What a tube looks like and what it joins: a hub in the middle of the block and an arm toward
//! every neighbor it connects to, one `true`/`false` property per direction. A directing tube
//! always has its arm toward `out`, so it keeps properties only for the other five. A sorting
//! tube draws each direction in its own texture.

use weft_sdk::*;

use super::{DIRECTING_TUBE, TIMBER_TUBE, TUBES};
use crate::lib::{BlockState, ModelBox, Pos};

/// The tube's hub: 8 texels wide in the middle of the block. Its arms are as thick.
const HUB: ModelBox = ModelBox::new([4.0; 3], [12.0; 3]);
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
        .on_placed(|w, change| refresh(w, change.pos.into()))
        .on_neighbor(|e| refresh(&e.world(), e.pos().into()))
        .on_broken(|w, change| super::visual::clear_broken(w, change.pos.into()))
}

/// The hub and the connected arms, each a box of its own, which is what it collides with too.
fn model<'s>(p: &Props, s: State<'s>, texture: &'static str, arms: Arms) -> State<'s> {
    let side = |d| side_texture(texture, arms, d);
    let hub = Dir::ALL.map(|d| (side(d), abi::UV_FROM_BOX));
    let mut s = s.texture(texture).model_box_faces(HUB.lo, HUB.hi, hub);
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
        let arm = HUB.arm(d);
        s = s.model_box_faces(arm.lo, arm.hi, [(face_texture, abi::UV_FROM_BOX); 6]);
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

/// Where `tube` sends its items, if it is a directing tube.
fn output(tube: BlockState) -> Option<Dir> {
    tube.prop("out").and_then(Dir::from_name)
}

/// Sets every arm property of the tube at `pos` to what its neighbors are now.
fn refresh(w: &World, pos: Pos) {
    let tube = BlockState::at(w, pos);
    if !tube.has(&TUBES) {
        return;
    }
    let next = joined(w, pos, tube);
    if next != tube {
        next.place(w, pos);
    }
}

/// `tube` with its arm properties set to which neighbors of `pos` it joins.
fn joined(w: &World, pos: Pos, tube: BlockState) -> BlockState {
    let out = output(tube);
    Dir::ALL
        .into_iter()
        .fold(tube, |next, d| match arm_prop(out, d) {
            Some(name) => next.with_flag(name, joins(w, pos, tube, d)),
            None => next,
        })
}

/// Whether `tube` at `pos` reaches toward `d`: to any other tube but two timber tubes, and to
/// anything holding items.
fn joins(w: &World, pos: Pos, tube: BlockState, d: Dir) -> bool {
    let at = pos.neighbor(d.face());
    let other = BlockState::at(w, at);
    if other.has(&TUBES) {
        return !(tube.is(&TIMBER_TUBE) && other.is(&TIMBER_TUBE));
    }
    crate::store::has_inventory(w, at)
}

/// Whether the tube at `pos` has its arm toward `face` out, so a neighbor can hand it items.
pub fn connects_to_tube(w: &World, pos: Pos, face: u32) -> bool {
    let tube = BlockState::at(w, pos);
    let d = Dir::from_face(face);
    match arm_prop(output(tube), d) {
        Some(name) => tube.prop(name) == Some("true"),
        None => joins(w, pos, tube, d),
    }
}

/// The tuning wrench turns a directing tube's output to its next connected arm.
pub fn turn_output(u: &mut Use) -> bool {
    let w = u.world();
    if !crate::cogs::holds_wrench(&w, u.player()) {
        return false;
    }
    let pos = Pos::from(u.pos());
    let tube = BlockState::at(&w, pos);
    let Some(now) = output(tube).filter(|_| tube.is(&DIRECTING_TUBE)) else {
        return true;
    };
    let next = (1..6)
        .map(|k| Dir::from_face((now.face() + k) % 6))
        .find(|d| connects_to_tube(&w, pos, d.face()));
    if let Some(turned) = next.and_then(|d| tube.with_prop("out", d.name())) {
        joined(&w, pos, turned).place(&w, pos);
        w.play_sound(&crate::sounds::WRENCH, pos.center(), 1.0);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

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
