//! What a tube looks like and what it joins: a hub in the middle of the block and an arm toward
//! every neighbor it connects to, one `true`/`false` property per direction. A directing tube
//! always has its arm toward `out`, so it keeps properties only for the other five. A sorting
//! tube draws each direction in its own texture.

use weft_sdk::*;

use super::{DIRECTING_TUBE, TIMBER_TUBE, TUBES};
use crate::lib::{BlockState, ModelBox, Pos, TypedWorld};

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
pub fn tube<'a>(builder: BlockBuilder<'a>, texture: &'static str, arms: Arms) -> BlockBuilder<'a> {
    let mut builder = builder
        .hardness(0.4)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.glass")
        .tag("cogwork:tubes");
    if matches!(arms, Arms::Output(_)) {
        builder = builder.prop("out", Dir::ALL.map(Dir::name));
        for name in OTHER_ARMS {
            builder = builder.prop(name, BOOL);
        }
    } else {
        for dir in Dir::ALL {
            builder = builder.prop(dir.name(), BOOL);
        }
    }
    builder
        .states(move |props, state| model(props, state, texture, arms))
        .on_place(|world, change| refresh(world, change.pos.into()))
        .on_neighbor(|event| refresh(&event.world(), event.pos().into()))
        .on_break(|world, change| super::visual::clear_broken(world, change.pos.into()))
}

/// The hub and the connected arms, each a box of its own, which is what it collides with too.
fn model<'s>(props: &Props, state: State<'s>, texture: &'static str, arms: Arms) -> State<'s> {
    let side = |dir| side_texture(texture, arms, dir);
    let hub = Dir::ALL.map(|dir| (side(dir), abi::UV_FROM_BOX));
    let mut state = state.texture(texture).model_box_faces(HUB.lo, HUB.hi, hub);
    let (out, out_texture) = match arms {
        Arms::Output(out_texture) => (Dir::from_name(props.get("out")), out_texture),
        _ => (None, texture),
    };
    for dir in Dir::ALL {
        let face_texture = match arm_prop(out, dir) {
            None => out_texture,
            Some(name) if props.is(name, "true") => side(dir),
            Some(_) => continue,
        };
        let arm = HUB.arm(dir);
        state = state.model_box_faces(arm.lo, arm.hi, [(face_texture, abi::UV_FROM_BOX); 6]);
    }
    state
}

/// The texture of the hub face and arm toward `dir`, unless that arm is a directing tube's output.
fn side_texture(texture: &'static str, arms: Arms, dir: Dir) -> &'static str {
    match arms {
        Arms::PerSide(texture_toward) => texture_toward(dir),
        _ => texture,
    }
}

/// The property saying whether a tube has its arm toward `dir`: the direction's own on a plain
/// tube. On a directing tube with its output toward `out`, `dir`'s place among the other five
/// directions, and none for the output, whose arm is always there.
fn arm_prop(out: Option<Dir>, dir: Dir) -> Option<&'static str> {
    let Some(out) = out else {
        return Some(dir.name());
    };
    let mut others = Dir::ALL.into_iter().filter(|&other| other != out);
    others.position(|other| other == dir).map(|i| OTHER_ARMS[i])
}

/// Where `tube` sends its items, if it is a directing tube.
fn output(tube: BlockState) -> Option<Dir> {
    tube.prop("out").and_then(Dir::from_name)
}

/// Sets every arm property of the tube at `pos` to what its neighbors are now.
fn refresh(world: &World, pos: Pos) {
    let tube = world.block_typed(pos);
    if !tube.has(&TUBES) {
        return;
    }
    let next = joined(world, pos, tube);
    if next != tube {
        world.set_block_typed(pos, next);
    }
}

/// `tube` with its arm properties set to which neighbors of `pos` it joins.
fn joined(world: &World, pos: Pos, tube: BlockState) -> BlockState {
    let out = output(tube);
    Dir::ALL
        .into_iter()
        .fold(tube, |next, dir| match arm_prop(out, dir) {
            Some(name) => next.with_flag(name, joins(world, pos, tube, dir)),
            None => next,
        })
}

/// Whether `tube` at `pos` reaches toward `dir`: to any other tube but two timber tubes, and to
/// anything holding items.
fn joins(world: &World, pos: Pos, tube: BlockState, dir: Dir) -> bool {
    let at = pos.toward(dir);
    let other = world.block_typed(at);
    if other.has(&TUBES) {
        return !(tube.is(&TIMBER_TUBE) && other.is(&TIMBER_TUBE));
    }
    crate::store::has_inventory(world, at)
}

/// Whether the tube at `pos` has its arm toward `face` out, so a neighbor can hand it items.
pub fn connects_to_tube(world: &World, pos: Pos, dir: Dir) -> bool {
    let tube = world.block_typed(pos);
    match arm_prop(output(tube), dir) {
        Some(name) => tube.prop(name) == Some("true"),
        None => joins(world, pos, tube, dir),
    }
}

/// The tuning wrench turns a directing tube's output to its next connected arm.
pub fn turn_output(click: &mut Use) -> bool {
    let world = click.world();
    if !crate::cogs::holds_wrench(&world, click.player()) {
        return false;
    }
    turn_directing(&world, Pos::from(click.pos()));
    true
}

/// Turns the output of the directing tube at `pos` to its next connected arm.
pub fn turn_directing(world: &World, pos: Pos) {
    let tube = world.block_typed(pos);
    let Some(now) = output(tube).filter(|_| tube.is(&DIRECTING_TUBE)) else {
        return;
    };
    let next = (1..6)
        .map(|turns| Dir::from_face((now.face() + turns) % 6))
        .find(|&dir| connects_to_tube(world, pos, dir));
    if let Some(turned) = next.and_then(|dir| tube.with_prop("out", dir.name())) {
        world.set_block_typed(pos, joined(world, pos, turned));
        world.play_sound(&crate::sounds::WRENCH, pos.center(), 1.0);
    }
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
                .filter_map(|&dir| arm_prop(Some(out), dir))
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
