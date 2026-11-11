//! What a tube looks like and what it joins: a hub in the middle of the block and an arm toward
//! every neighbor it connects to, one `true`/`false` property per direction. A directing tube
//! always has its arm toward `out`, so it keeps properties only for the other five. A sorting
//! tube draws each direction in its own texture. The hub and arms are drawn from the Blockbench
//! model `assets/models/block/tube/hub_and_arm.json` (`arms`), which names a texture for each
//! side.

use weft_sdk::{BlockBuilder, Dir, Props, State, Use, WithComponents, World, tool_tier};

use super::{DIRECTING_TUBE, TIMBER_TUBE, TUBES};
use crate::lib::{BlockState, Pos, TypedWorld};

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

/// The model with the connected arms, and each side's texture on the hub's face and the arm
/// that way. Its boxes are what it collides with, too.
fn model<'s>(props: &Props, state: State<'s>, texture: &'static str, arms: Arms) -> State<'s> {
    let out = match arms {
        Arms::Output(_) => Dir::from_name(props.get("out")),
        _ => None,
    };
    // The output has no property: its arm is always there.
    let has_arm = |dir| arm_prop(out, dir).is_none_or(|name| props.is(name, "true"));
    let side = |name: &str| {
        let dir = Dir::from_name(name).expect("a tube's model names its textures after sides");
        side_texture(texture, arms, out, dir).to_string()
    };
    let pieces = crate::arms::TUBE.pieces(has_arm, side);
    crate::boxes::draw(state.texture(texture), &pieces)
}

/// The texture of the hub face and arm toward `dir` of a tube that sends its items `out`.
fn side_texture(texture: &'static str, arms: Arms, out: Option<Dir>, dir: Dir) -> &'static str {
    match arms {
        Arms::PerSide(texture_toward) => texture_toward(dir),
        Arms::Output(out_texture) if out == Some(dir) => out_texture,
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
    fn a_directing_tube_colors_only_the_side_of_its_output() {
        let arms = Arms::Output("out");
        let toward = |dir| side_texture("tube", arms, Some(Dir::East), dir);
        assert_eq!(toward(Dir::East), "out", "the output");
        assert_eq!(toward(Dir::West), "tube", "any other side");
        let plain = side_texture("tube", Arms::Same, None, Dir::East);
        assert_eq!(plain, "tube", "a plain tube all round");
    }

    #[test]
    fn every_tube_model_has_a_texture_to_set_for_each_side() {
        let models = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/block/tube");
        let files = std::fs::read_dir(models).expect("cogwork has no tube models");
        for file in files.flatten() {
            let model = std::fs::read_to_string(file.path()).unwrap_or_default();
            for dir in Dir::ALL {
                let texture = format!("\"{}\": \"block/", dir.name());
                assert!(
                    model.contains(&texture),
                    "{:?} has none {texture}",
                    file.path()
                );
            }
        }
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
