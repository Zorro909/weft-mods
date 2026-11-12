//! The liquid seen inside a liquid tube or a tank: a block made for every mod's liquid
//! (`shapes_of` over `#liquids`), showing that liquid's own picture, put into the holder's cell
//! beside it. Its properties say how full it is and which ways it goes on (a tube's arms, a
//! tank's joined sides). Its boxes are built here rather than drawn in Blockbench: they are the
//! liquid's surface, sized by those properties, and there is nothing else to them.

use weft_sdk::{BlockBuilder, Dir, Registry, Tag, WithComponents, World, abi};

use crate::boxes::{Corners, Piece};
use crate::lib::{BlockState, Pos};
use crate::liquid::{self, LiquidKey};

/// Every fill block in a tube, and in a tank.
static IN_TUBE: Tag = Tag::named("cogwork:liquid_in_tube");
static IN_TANK: Tag = Tag::named("cogwork:liquid_in_tank");

/// Steps a fill shows: a tube's in eighths, a tank block's in sixteenths.
pub const TUBE_LEVELS: u32 = 8;
pub const TANK_LEVELS: u32 = 16;

/// Inside a tube's hub, in texels: the hub is 4 to 12, the fill a texel in from its glass. The
/// engine draws faces on whole texels only: closer, the fill would flicker with the glass.
const TUBE_LOW: f32 = 5.0;
const TUBE_HIGH: f32 = 11.0;
/// Inside a tank block, in texels: a texel in from its glass (`tank::shape`), on its base and a
/// texel below its lid on the outside of the tank, edge to edge where it goes on.
const TANK_WALL: f32 = crate::tank::GLASS + 1.0;
const TANK_FLOOR: f32 = crate::tank::BASE;
const TANK_LID: f32 = 16.0 - crate::tank::GLASS - 1.0;

pub fn register(r: &mut Registry) {
    let in_tube = r.shapes_of("liquid_in_tube", "#liquids", "_in_tube");
    with_sides(fill_block(in_tube, &IN_TUBE, TUBE_LEVELS)).states(|props, state| {
        let pieces = tube_pieces(props.int("level"), |dir| props.is(dir.name(), "true"));
        crate::boxes::draw(state, &pieces)
    });
    let in_tank = r.shapes_of("liquid_in_tank", "#liquids", "_in_tank");
    with_sides(fill_block(in_tank, &IN_TANK, TANK_LEVELS)).states(|props, state| {
        let piece = tank_piece(props.int("level"), |dir| props.is(dir.name(), "true"));
        crate::boxes::draw(state, &[piece])
    });
}

/// What every fill block is: seen, never touched or aimed at, and gone with its holder.
fn fill_block<'a>(builder: BlockBuilder<'a>, tag: &Tag, levels: u32) -> BlockBuilder<'a> {
    builder
        .not_solid()
        .tag(abi::comp::block::NOT_TARGETABLE)
        .tag(tag.name())
        .no_item()
        .drops_nothing()
        .prop("level", 1..=levels)
}

/// A `true`/`false` property for each side.
fn with_sides(mut builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
}

/// The liquid's own picture on each face, by `Dir::face`.
const SOURCE_FACES: [&str; 6] = [
    "{source.west}",
    "{source.east}",
    "{source.down}",
    "{source.up}",
    "{source.north}",
    "{source.south}",
];

/// A piece of fill: `corners`, with a face toward each side `face` says.
fn liquid(corners: Corners, face: impl Fn(Dir) -> bool) -> Piece {
    Piece::new(corners, |dir| {
        face(dir).then_some(SOURCE_FACES[dir.face() as usize])
    })
}

/// The pieces of a tube's fill `level` eighths up its hub, with `arm` toward each side it has
/// an arm: a level surface through the hub and the sideways arms, the arm down full whenever
/// there is any, the arm up only when the tube is full. The liquid goes on into what each arm
/// joins, so an arm's end has no face but up, where the liquid stops.
fn tube_pieces(level: u32, arm: impl Fn(Dir) -> bool) -> Vec<Piece> {
    let top = height(TUBE_LOW, TUBE_HIGH, level, TUBE_LEVELS);
    let hub = ([TUBE_LOW; 3], [TUBE_HIGH, top, TUBE_HIGH]);
    let mut pieces = vec![liquid(hub, |_| true)];
    for side in Dir::ALL.into_iter().filter(|&dir| arm(dir)) {
        let (mut min, mut max) = hub;
        let axis = side.face() as usize / 2;
        // Even faces are the low side of their axis.
        if side.face().is_multiple_of(2) {
            (min[axis], max[axis]) = (0.0, TUBE_LOW);
        } else {
            (min[axis], max[axis]) = (TUBE_HIGH, 16.0);
        }
        match side {
            Dir::Up if level < TUBE_LEVELS => continue,
            Dir::Down | Dir::Up => {}
            _ => max[1] = top,
        }
        pieces.push(liquid((min, max), |dir| dir != side || dir == Dir::Up));
    }
    crate::boxes::hide_shared_faces(&mut pieces);
    pieces
}

/// The fill of a tank block `level` sixteenths up its inside, reaching the edge on each side
/// `joined` to more of the tank, with no face there: the liquid goes on. Its top shows unless
/// the block is full and joined above, where the liquid goes on up.
fn tank_piece(level: u32, joined: impl Fn(Dir) -> bool) -> Piece {
    let low = |dir| if joined(dir) { 0.0 } else { TANK_WALL };
    let high = |dir| if joined(dir) { 16.0 } else { 16.0 - TANK_WALL };
    let floor = if joined(Dir::Down) { 0.0 } else { TANK_FLOOR };
    let lid = if joined(Dir::Up) { 16.0 } else { TANK_LID };
    let top = height(floor, lid, level, TANK_LEVELS);
    let corners = (
        [low(Dir::West), floor, low(Dir::North)],
        [high(Dir::East), top, high(Dir::South)],
    );
    let full = level >= TANK_LEVELS;
    liquid(corners, |dir| match dir {
        Dir::Up => !(full && joined(Dir::Up)),
        _ => !joined(dir),
    })
}

/// The height of a surface `level` steps of `levels` from `low` up to `high`, on a whole texel.
fn height(low: f32, high: f32, level: u32, levels: u32) -> f32 {
    low + ((high - low) * level as f32 / levels as f32).round()
}

/// The fill of `liquid` for a tube at `level` (1 and up) with arms toward `arm`, if a loaded
/// mod has that liquid.
pub fn in_tube(liquid: LiquidKey, level: u32, arm: impl Fn(Dir) -> bool) -> Option<BlockState> {
    let name = liquid::name(liquid)?;
    let fill = BlockState::named(&format!("{name}_in_tube"))?;
    with_level(fill, level, arm)
}

/// The fill of `liquid` for a tank block at `level` (1 and up), joined toward `joined`.
pub fn in_tank(liquid: LiquidKey, level: u32, joined: impl Fn(Dir) -> bool) -> Option<BlockState> {
    let name = liquid::name(liquid)?;
    let fill = BlockState::named(&format!("{name}_in_tank"))?;
    with_level(fill, level, joined)
}

fn with_level(fill: BlockState, level: u32, side: impl Fn(Dir) -> bool) -> Option<BlockState> {
    let fill = fill.with_prop("level", &level.to_string())?;
    Some(
        Dir::ALL
            .into_iter()
            .fold(fill, |fill, dir| fill.with_flag(dir.name(), side(dir))),
    )
}

/// Makes the fill in the tube at `pos` `fill`, none for `None`.
pub fn show_in_tube(world: &World, pos: Pos, fill: Option<BlockState>) {
    show(world, pos, &IN_TUBE, fill);
}

/// Makes the fill in the tank block at `pos` `fill`, none for `None`.
pub fn show_in_tank(world: &World, pos: Pos, fill: Option<BlockState>) {
    show(world, pos, &IN_TANK, fill);
}

fn show(world: &World, pos: Pos, tag: &Tag, fill: Option<BlockState>) {
    let old = world
        .parts(pos)
        .into_iter()
        .find(|&part| tag.on_block(part));
    let new = fill.map(BlockState::id);
    match (old, new) {
        (Some(old), Some(new)) if old != new => drop(world.part_replace(pos, old, new)),
        (Some(old), None) => drop(world.part_remove(pos, old)),
        (None, Some(new)) => drop(world.part_add(pos, new)),
        _ => {}
    }
}

/// How many steps of `levels` a holder of `capacity` with `amount` in shows: at least one for
/// anything worth keeping, all of them only when full.
pub fn level_of(amount: u32, capacity: u32, levels: u32) -> u32 {
    if amount < liquid::DRIBBLE {
        return 0;
    }
    if amount >= capacity {
        return levels;
    }
    let level = (amount as u64 * levels as u64).div_ceil(capacity.max(1) as u64) as u32;
    level.clamp(1, levels - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(piece: &Piece, dir: Dir) -> bool {
        piece.faces[dir.face() as usize].is_some()
    }

    #[test]
    fn a_tube_shows_one_level_through_its_hub_and_sideways_arms() {
        let pieces = tube_pieces(4, |dir| matches!(dir, Dir::West | Dir::Up | Dir::Down));
        assert_eq!(pieces.len(), 3, "the hub, the arm west and down, not up");
        let [hub, west, down] = [&pieces[0], &pieces[1], &pieces[2]];
        assert_eq!(hub.corners.1[1], 8.0, "halfway up the hub");
        assert_eq!(west.corners.1[1], 8.0, "the arm as high");
        let inside = |at: f32| 4.0 < at && at < 12.0;
        let (low, high) = hub.corners;
        assert!(inside(low[0]) && inside(high[0]), "off the glass, 4 to 12");
        assert_eq!(
            (down.corners.0[1], down.corners.1[1]),
            (0.0, TUBE_LOW),
            "the arm down full"
        );
        assert_eq!(
            tube_pieces(TUBE_LEVELS, |dir| dir == Dir::Up).len(),
            2,
            "up once full"
        );
    }

    #[test]
    fn liquid_shows_no_face_where_it_goes_on() {
        let pieces = tube_pieces(TUBE_LEVELS, |dir| matches!(dir, Dir::West | Dir::Up));
        let [hub, west, up] = [&pieces[0], &pieces[1], &pieces[2]];
        assert!(
            !has(hub, Dir::West) && !has(west, Dir::East),
            "into the arm"
        );
        assert!(!has(west, Dir::West), "into what the arm joins");
        assert!(has(up, Dir::Up), "its surface, at the top of the arm up");
        let inside = tank_piece(TANK_LEVELS, |dir| dir != Dir::North);
        assert!(has(&inside, Dir::North), "behind the glass");
        assert!(!has(&inside, Dir::East), "on into the next block");
        assert!(!has(&inside, Dir::Up), "on up, full");
        let below_the_top = tank_piece(TANK_LEVELS - 1, |_| true);
        assert!(has(&below_the_top, Dir::Up), "its surface");
    }

    #[test]
    fn a_tank_fill_reaches_through_to_the_rest_of_the_tank() {
        let (min, max) = tank_piece(TANK_LEVELS, |dir| matches!(dir, Dir::East | Dir::Up)).corners;
        assert_eq!(
            (min[0], max[0]),
            (TANK_WALL, 16.0),
            "behind the glass west, on east"
        );
        assert_eq!(
            (min[1], max[1]),
            (TANK_FLOOR, 16.0),
            "on its base, into the block above"
        );
        let (_, half) = tank_piece(TANK_LEVELS / 2, |dir| dir == Dir::Down).corners;
        assert_eq!(
            half[1],
            (TANK_LID / 2.0).round(),
            "half of a block on more tank"
        );
    }

    #[test]
    fn every_surface_lies_on_a_whole_texel() {
        for level in 1..=TANK_LEVELS {
            let (_, top) = tank_piece(level, |_| false).corners;
            assert_eq!(top[1].fract(), 0.0, "level {level}: the engine rounds it");
        }
        for level in 1..=TUBE_LEVELS {
            let top = tube_pieces(level, |_| false)[0].corners.1[1];
            assert_eq!(top.fract(), 0.0, "level {level} in a tube");
        }
    }

    #[test]
    fn a_level_shows_anything_worth_keeping() {
        assert_eq!(level_of(5, 1000, 8), 0, "a dribble shows nothing");
        assert_eq!(level_of(20, 1000, 8), 1, "a little shows the first step");
        assert_eq!(level_of(990, 1000, 8), 7, "short of full");
        assert_eq!(level_of(1000, 1000, 8), 8, "full");
    }
}
