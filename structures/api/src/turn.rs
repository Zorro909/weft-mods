//! Turning a structure a quarter at a time about the vertical axis: where its blocks go, and
//! which way the blocks then face.

use weft_sdk::{state_prop, state_with};

/// The horizontal directions in clockwise order seen from above (north is -z, east +x).
const CLOCKWISE: [&str; 4] = ["north", "east", "south", "west"];

/// How far a structure is turned, clockwise seen from above.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Turn {
    #[default]
    None,
    Right,
    Back,
    Left,
}

impl Turn {
    pub const ALL: [Turn; 4] = [Turn::None, Turn::Right, Turn::Back, Turn::Left];

    /// Undoes this turn.
    pub fn inverse(self) -> Turn {
        Turn::ALL[(4 - self as usize) % 4]
    }

    /// The size along x, y and z once turned: a quarter turn swaps x and z.
    pub fn size(self, [sx, sy, sz]: [i32; 3]) -> [i32; 3] {
        match self {
            Turn::Right | Turn::Left => [sz, sy, sx],
            Turn::None | Turn::Back => [sx, sy, sz],
        }
    }

    /// Where block `pos` of a structure of `size` lands once the structure is turned.
    pub fn pos(self, [x, y, z]: [i32; 3], [sx, _, sz]: [i32; 3]) -> [i32; 3] {
        match self {
            Turn::None => [x, y, z],
            Turn::Right => [sz - 1 - z, y, x],
            Turn::Back => [sx - 1 - x, y, sz - 1 - z],
            Turn::Left => [z, y, sx - 1 - x],
        }
    }

    /// A horizontal direction turned (`north` turned right is `east`); other values stay.
    pub fn direction(self, name: &str) -> &str {
        match CLOCKWISE.iter().position(|&d| d == name) {
            Some(i) => CLOCKWISE[(i + self as usize) % 4],
            None => name,
        }
    }

    /// An `axis` value turned: a quarter turn swaps `x` and `z`.
    pub fn axis(self, axis: &str) -> &str {
        match (self, axis) {
            (Turn::Right | Turn::Left, "x") => "z",
            (Turn::Right | Turn::Left, "z") => "x",
            _ => axis,
        }
    }
}

/// `state` turned: its `facing`, its `axis` and the sides it connects to (`north=true`) follow
/// the turn. Properties the block doesn't have are left alone, and so is a value it can't take.
pub(crate) fn turn_state(state: u32, turn: Turn) -> u32 {
    if turn == Turn::None {
        return state;
    }
    let mut turned = state;
    if let Some(facing) = state_prop(state, "facing") {
        turned = with(turned, "facing", turn.direction(facing));
    }
    if let Some(axis) = state_prop(state, "axis") {
        turned = with(turned, "axis", turn.axis(axis));
    }
    for side in CLOCKWISE {
        if let Some(value) = state_prop(state, side) {
            turned = with(turned, turn.direction(side), value);
        }
    }
    turned
}

fn with(state: u32, name: &str, value: &str) -> u32 {
    state_with(state, name, value).unwrap_or(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quarter_turn_swaps_the_sides() {
        assert_eq!(Turn::Right.size([2, 3, 5]), [5, 3, 2], "x and z swap");
        assert_eq!(
            Turn::Back.size([2, 3, 5]),
            [2, 3, 5],
            "a half turn keeps them"
        );
    }

    #[test]
    fn turning_right_moves_east_to_south() {
        let size = [4, 1, 2];
        assert_eq!(
            Turn::Right.pos([3, 0, 0], size),
            [1, 0, 3],
            "the north-east corner becomes the south-east one"
        );
        assert_eq!(
            Turn::Left.pos([3, 0, 0], size),
            [0, 0, 0],
            "and turning left, the north-west one"
        );
    }

    #[test]
    fn every_turn_comes_back() {
        let size = [4, 2, 3];
        for turn in Turn::ALL {
            let turned = turn.size(size);
            for (x, z) in (0..4).flat_map(|x| (0..3).map(move |z| (x, z))) {
                let back = turn.inverse().pos(turn.pos([x, 1, z], size), turned);
                assert_eq!(back, [x, 1, z], "{turn:?} undone");
            }
        }
    }

    #[test]
    fn directions_turn_clockwise() {
        assert_eq!(Turn::Right.direction("north"), "east", "right");
        assert_eq!(Turn::Back.direction("east"), "west", "back");
        assert_eq!(Turn::Left.direction("north"), "west", "left");
        assert_eq!(Turn::Right.direction("up"), "up", "up stays up");
    }

    #[test]
    fn axes_swap_on_a_quarter_turn() {
        assert_eq!(Turn::Left.axis("x"), "z", "quarter");
        assert_eq!(Turn::Back.axis("x"), "x", "half");
        assert_eq!(Turn::Right.axis("y"), "y", "upright stays");
    }
}
