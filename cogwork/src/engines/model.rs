//! An engine's block: a base plate on the side it was placed against, a trunk pointing the other
//! way that glows with its heat, and a piston plate riding up and down the trunk.

use weft_sdk::signal::Frame;
use weft_sdk::*;

/// Trunk textures by heat level: cool, warm, hot, overheated.
const TRUNKS: [&str; 4] = [
    "engine_trunk_blue",
    "engine_trunk_green",
    "engine_trunk_orange",
    "engine_trunk_red",
];

/// An engine block of `kind` (`spring`, `ember`): it faces away from the block it was placed
/// against, so its trunk points into that block.
pub fn engine<'a>(b: BlockBuilder<'a>, kind: &'static str) -> BlockBuilder<'a> {
    b.facing(Facings::ALL, Toward::Face)
        .prop("heat", 0..=3)
        .prop("stroke", 0..=3)
        .sound_group("base:block.stone")
        .states(move |p, s| shape(p, s, kind))
        .on_use(turn)
}

/// The boxes for one state, written lying on the floor with the trunk up.
fn shape<'s>(p: &Props, s: State<'s>, kind: &str) -> State<'s> {
    let frame = Frame::new(
        p.facing(),
        if p.facing().horizontal() {
            Dir::Up
        } else {
            Dir::North
        },
    );
    let (plate_top, plate_side) = (
        format!("engine_{kind}_plate_top"),
        format!("engine_{kind}_plate_side"),
    );
    let (piston_top, piston_side) = (
        format!("engine_{kind}_piston_top"),
        format!("engine_{kind}_piston_side"),
    );
    let trunk = TRUNKS[p.int("heat").min(3) as usize];
    let lift = 2.0 * p.int("stroke") as f32;
    const WHOLE: [f32; 4] = [0.0, 0.0, 16.0, 16.0];
    const EDGE: [f32; 4] = [0.0, 0.0, 16.0, 4.0];
    s.texture(&plate_top)
        .framed_box(
            frame,
            [0.0; 3],
            [16.0, 4.0, 16.0],
            faces(&plate_side, EDGE, &plate_top, WHOLE),
        )
        .framed_box(
            frame,
            [4.0; 3],
            [12.0, 16.0, 12.0],
            faces(trunk, [4.0, 0.0, 12.0, 12.0], trunk, [4.0, 4.0, 12.0, 12.0]),
        )
        .framed_box(
            frame,
            [0.0, 4.0 + lift, 0.0],
            [16.0, 8.0 + lift, 16.0],
            faces(&piston_side, EDGE, &piston_top, WHOLE),
        )
        .hitbox([0.0; 3], [1.0; 3])
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

/// The tuning wrench points the engine's trunk the next way round.
fn turn(u: &mut Use) -> bool {
    let w = u.world();
    if !crate::cogs::holds_wrench(&w, u.player()) {
        return false;
    }
    let pos = u.pos();
    let now = w
        .prop(pos, "facing")
        .and_then(Dir::from_name)
        .map_or(0, Dir::face);
    w.set_prop(pos, "facing", Dir::from_face((now + 1) % 6).name());
    true
}

/// Where the engine's trunk points: its output.
pub fn output(w: &World, pos: [i32; 3]) -> Option<Dir> {
    w.prop(pos, "facing")
        .and_then(Dir::from_name)
        .map(Dir::opposite)
}
