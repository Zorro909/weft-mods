//! The settlement plan's last step: fills what is still open under its buildings' bottom layers
//! down to the ground, reading the finished village as the plan has put it. The engine's
//! foundation fills only from the terrain's height up, so where a cave runs just under the
//! ground or opens there, a bottom layer stands over air. It is no feature of its own at
//! `abi::stage::POST_PROCESS`, because a plan can't find another feature's starts.

use weftkit::Pos;

use super::frontage::Site;
use super::ground::Cover;
use super::plan::Plan;

/// How deep a gap under a floor is filled at most, in blocks: a deeper one is a cave going on
/// down, left open rather than plugged with a column.
const DEEPEST: i32 = 8;

/// Fills the gaps under every bottom-layer block of `sites` with the block of the ground under
/// each.
pub fn fill_under(p: &mut Plan, sites: &[Site]) {
    for site in sites {
        let [sx, _, sz] = site.size;
        for dz in 0..sz {
            for dx in 0..sx {
                fill_column(p, site.corner.add_x(dx).add_z(dz));
            }
        }
    }
}

/// Fills the gap under the floor block at `floor`, if there is one to fill.
fn fill_column(p: &mut Plan, floor: Pos) {
    let cover_at = |depth: i32| Cover::of(p.read(floor.add_y(-depth)));
    let Some(depth) = gap(cover_at) else {
        return;
    };
    let ground = p.read(floor.add_y(-depth - 1));
    for y in 1..=depth {
        p.set_state(floor.add_y(-y), ground);
    }
}

/// How many open cells lie under a floor, if they are a gap to fill: the floor (depth 0) is
/// solid, and solid ground comes under the open cells within `DEEPEST`. `cover_at` says what
/// the cell `depth` blocks under the floor is.
fn gap(cover_at: impl Fn(i32) -> Cover) -> Option<i32> {
    if cover_at(0) != Cover::Solid {
        return None;
    }
    let depth = (1..=DEEPEST + 1).find(|&depth| cover_at(depth) != Cover::Open)?;
    let grounded = depth > 1 && cover_at(depth) == Cover::Solid;
    grounded.then_some(depth - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A column read down from the floor: `s` solid, `o` open, `w` water; what is past the end
    /// is solid.
    fn column(cells: &str) -> impl Fn(i32) -> Cover + '_ {
        move |depth| match cells.as_bytes().get(depth as usize) {
            Some(b'o') => Cover::Open,
            Some(b'w') => Cover::Liquid,
            _ => Cover::Solid,
        }
    }

    #[test]
    fn a_floor_on_the_ground_has_no_gap() {
        assert_eq!(gap(column("ss")), None);
    }

    #[test]
    fn a_gap_down_to_the_ground_is_filled() {
        assert_eq!(gap(column("sos")), Some(1), "one block");
        assert_eq!(gap(column("sooos")), Some(3), "three blocks");
    }

    #[test]
    fn open_floors_are_left_alone() {
        assert_eq!(gap(column("oos")), None, "air in the bottom layer");
    }

    #[test]
    fn a_gap_too_deep_is_a_cave_and_left() {
        let deep = format!("s{}s", "o".repeat(9));
        assert_eq!(gap(column(&deep)), None, "nine blocks down");
        let deepest = format!("s{}s", "o".repeat(8));
        assert_eq!(gap(column(&deepest)), Some(8), "eight is filled");
    }

    #[test]
    fn a_gap_over_water_is_left() {
        assert_eq!(gap(column("sow")), None);
    }
}
