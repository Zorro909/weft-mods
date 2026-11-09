//! An engine's block: a base plate on the side it was placed against and a trunk pointing the
//! other way that glows with its heat. The piston riding the trunk is an entity (`piston`).

use weft_sdk::*;

use crate::lib::{ModelBox, Pos};

/// The base plate, 4 texels thick, and the trunk standing on it, 8 wide: lying on the floor,
/// before they are turned to the engine's facing.
const PLATE: ModelBox = ModelBox::new([0.0; 3], [16.0, 4.0, 16.0]);
const TRUNK: ModelBox = ModelBox::new([4.0; 3], [12.0, 16.0, 12.0]);

/// Trunk textures by heat level: cool, warm, hot, overheated.
const TRUNKS: [&str; 4] = [
    "engine_trunk_blue",
    "engine_trunk_green",
    "engine_trunk_orange",
    "engine_trunk_red",
];

/// An engine block of `kind` (`spring`, `ember`): it faces away from the block it was placed
/// against, so its trunk points into that block.
pub fn engine<'a>(builder: BlockBuilder<'a>, kind: &'static str) -> BlockBuilder<'a> {
    builder
        .facing(Facings::ALL, Toward::Face)
        .prop("heat", 0..=3)
        .sound_group("base:block.stone")
        .states(move |props, state| shape(props, state, kind))
        .on_use(use_wrench)
}

/// The boxes for one state, written lying on the floor with the trunk up.
fn shape<'s>(props: &Props, state: State<'s>, kind: &str) -> State<'s> {
    // Its trunk, the model's top, points away from the facing.
    let orientation = Orientation::up(props.facing().opposite());
    let (plate_top, plate_side) = (
        format!("engine_{kind}_plate_top"),
        format!("engine_{kind}_plate_side"),
    );
    let trunk = TRUNKS[props.int("heat").min(3) as usize];
    const WHOLE: [f32; 4] = [0.0, 0.0, 16.0, 16.0];
    const EDGE: [f32; 4] = [0.0, 0.0, 16.0, 4.0];
    state
        .texture(&plate_top)
        .orientation(orientation)
        .model_box_faces(
            PLATE.lo,
            PLATE.hi,
            faces(&plate_side, EDGE, &plate_top, WHOLE),
        )
        .model_box_faces(
            TRUNK.lo,
            TRUNK.hi,
            faces(trunk, [4.0, 0.0, 12.0, 12.0], trunk, [4.0, 4.0, 12.0, 12.0]),
        )
        // The whole block, piston and all: nothing walks into the piston's way.
        .hitbox(ModelBox::BLOCK.lo, ModelBox::BLOCK.hi)
}

/// A box's faces: `side` all round, `end` on its bottom and top.
fn faces<'t>(
    side: &'t str,
    side_uv: [f32; 4],
    end: &'t str,
    end_uv: [f32; 4],
) -> [(&'t str, [f32; 4]); 6] {
    [
        (side, side_uv),
        (side, side_uv),
        (end, end_uv),
        (end, end_uv),
        (side, side_uv),
        (side, side_uv),
    ]
}

fn use_wrench(click: &mut Use) -> bool {
    let world = click.world();
    if !crate::cogs::holds_wrench(&world, click.player()) {
        return false;
    }
    turn(&world, Pos::from(click.pos()));
    true
}

/// The tuning wrench on the engine at `pos`: its trunk points the next way round.
pub fn turn(world: &World, pos: Pos) {
    world.prop_cycle(pos, "facing");
    world.play_sound(&crate::sounds::WRENCH, pos.center(), 1.0);
}

/// Where the engine's trunk points: its output.
pub fn output(world: &World, pos: Pos) -> Option<Dir> {
    world
        .prop(pos, "facing")
        .and_then(Dir::from_name)
        .map(Dir::opposite)
}
