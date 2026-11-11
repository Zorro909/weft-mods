//! The liquid seen inside a liquid tube or a tank: a block made for every mod's liquid
//! (`shapes_of` over `#liquids`), showing that liquid's own picture, put into the holder's cell
//! beside it. Its properties say how full it is and which ways it goes on (a tube's arms, a
//! tank's joined sides). Its boxes are built here rather than drawn in Blockbench: they are the
//! liquid's surface, sized by those properties, and there is nothing else to them.

use weft_sdk::*;

use crate::lib::{BlockState, Pos};
use crate::liquid::{self, LiquidKey};

/// Every fill block in a tube, and in a tank.
static IN_TUBE: Tag = Tag::named("cogwork:liquid_in_tube");
static IN_TANK: Tag = Tag::named("cogwork:liquid_in_tank");

/// Steps a fill shows: a tube's in eighths, a tank block's in sixteenths.
pub const TUBE_LEVELS: u32 = 8;
pub const TANK_LEVELS: u32 = 16;

/// Inside a tube's hub, in texels: the hub is 4 to 12, the fill half a texel in from its glass.
const TUBE_LOW: f32 = 4.5;
const TUBE_HIGH: f32 = 11.5;
/// Where an arm's fill stops short of the block's edge, so it never lies in the plane of the
/// arm's end.
const TUBE_EDGE: f32 = 0.25;
/// Inside a tank block, in texels: half a texel in from its glass (`tank::files`), above its
/// base and below its lid on the outside of the tank, edge to edge where it goes on.
const TANK_WALL: f32 = 1.5;
const TANK_FLOOR: f32 = 4.0;
const TANK_LID: f32 = 14.5;

pub fn register(r: &mut Registry) {
    let in_tube = r.shapes_of("liquid_in_tube", "#liquids", "_in_tube");
    with_sides(fill_block(in_tube, &IN_TUBE, TUBE_LEVELS)).states(|props, state| {
        tube_boxes(props.int("level"), |dir| props.is(dir.name(), "true"))
            .into_iter()
            .fold(state, |state, (min, max)| state.model_box(min, max))
    });
    let in_tank = r.shapes_of("liquid_in_tank", "#liquids", "_in_tank");
    with_sides(fill_block(in_tank, &IN_TANK, TANK_LEVELS)).states(|props, state| {
        let (min, max) = tank_box(props.int("level"), |dir| props.is(dir.name(), "true"));
        state.model_box(min, max)
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

/// The boxes of a tube's fill `level` eighths up its hub, with `arm` toward each side it has an
/// arm: a level surface through the hub and the sideways arms, the arm down full whenever there
/// is any, the arm up only when the tube is full.
fn tube_boxes(level: u32, arm: impl Fn(Dir) -> bool) -> Vec<([f32; 3], [f32; 3])> {
    let top = TUBE_LOW + (TUBE_HIGH - TUBE_LOW) * level as f32 / TUBE_LEVELS as f32;
    let hub = ([TUBE_LOW; 3], [TUBE_HIGH, top, TUBE_HIGH]);
    let mut boxes = vec![hub];
    for dir in Dir::ALL.into_iter().filter(|&dir| arm(dir)) {
        let (mut min, mut max) = hub;
        let axis = dir.face() as usize / 2;
        // Even faces are the low side of their axis.
        if dir.face().is_multiple_of(2) {
            (min[axis], max[axis]) = (TUBE_EDGE, TUBE_LOW);
        } else {
            (min[axis], max[axis]) = (TUBE_HIGH, 16.0 - TUBE_EDGE);
        }
        match dir {
            Dir::Up if level < TUBE_LEVELS => continue,
            Dir::Down | Dir::Up => {}
            _ => max[1] = top,
        }
        boxes.push((min, max));
    }
    boxes
}

/// The box of a tank block's fill `level` sixteenths up its inside, reaching the edge on each
/// side `joined` to more of the tank.
fn tank_box(level: u32, joined: impl Fn(Dir) -> bool) -> ([f32; 3], [f32; 3]) {
    let low = |dir| if joined(dir) { 0.0 } else { TANK_WALL };
    let high = |dir| if joined(dir) { 16.0 } else { 16.0 - TANK_WALL };
    let floor = if joined(Dir::Down) { 0.0 } else { TANK_FLOOR };
    let lid = if joined(Dir::Up) { 16.0 } else { TANK_LID };
    let top = floor + (lid - floor) * level as f32 / TANK_LEVELS as f32;
    (
        [low(Dir::West), floor, low(Dir::North)],
        [high(Dir::East), top, high(Dir::South)],
    )
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

    #[test]
    fn a_tube_shows_one_level_through_its_hub_and_sideways_arms() {
        let boxes = tube_boxes(4, |dir| matches!(dir, Dir::West | Dir::Up | Dir::Down));
        assert_eq!(boxes.len(), 3, "the hub, the arm west and down, not up");
        let [hub, west, down] = [boxes[0], boxes[1], boxes[2]];
        assert_eq!(hub.1[1], 8.0, "halfway up the hub");
        assert_eq!(west.1[1], 8.0, "the arm as high");
        assert_eq!(
            (down.0[1], down.1[1]),
            (TUBE_EDGE, TUBE_LOW),
            "the arm down full"
        );
        assert_eq!(
            tube_boxes(TUBE_LEVELS, |dir| dir == Dir::Up).len(),
            2,
            "up once full"
        );
    }

    #[test]
    fn a_tank_fill_reaches_through_to_the_rest_of_the_tank() {
        let (min, max) = tank_box(TANK_LEVELS, |dir| matches!(dir, Dir::East | Dir::Up));
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
        let (_, half) = tank_box(TANK_LEVELS / 2, |dir| dir == Dir::Down);
        assert_eq!(
            half[1],
            (0.0 + TANK_LID) / 2.0,
            "half of a block on more tank"
        );
    }

    #[test]
    fn a_level_shows_anything_worth_keeping() {
        assert_eq!(level_of(5, 1000, 8), 0, "a dribble shows nothing");
        assert_eq!(level_of(20, 1000, 8), 1, "a little shows the first step");
        assert_eq!(level_of(990, 1000, 8), 7, "short of full");
        assert_eq!(level_of(1000, 1000, 8), 8, "full");
    }
}
