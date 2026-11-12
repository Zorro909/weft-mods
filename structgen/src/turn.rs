//! Turning a structure a quarter at a time about the vertical axis: where its blocks go, and
//! which way the blocks then face.
//!
//! Block states are turned here and not by the engine: up to v26.11.4 its `turned_state` turns
//! `facing`, `attach` and `front` but leaves a log's `axis` and a ladder's sides as they were,
//! so a turned timber frame would show its beams across and its ladders off the wall. This
//! turns names (`base:oak_log[axis=x]`) rather than states, for the copies `buildings.rs`
//! registers.

use weftkit::Pos;

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

    /// The size along x, y and z once turned: a quarter turn swaps x and z.
    pub fn size(self, [sx, sy, sz]: [i32; 3]) -> [i32; 3] {
        match self {
            Turn::Right | Turn::Left => [sz, sy, sx],
            Turn::None | Turn::Back => [sx, sy, sz],
        }
    }

    /// Where block `pos` of a structure of `size` lands once the structure is turned.
    pub fn pos(self, Pos { x, y, z }: Pos, [sx, _, sz]: [i32; 3]) -> Pos {
        match self {
            Turn::None => Pos::new(x, y, z),
            Turn::Right => Pos::new(sz - 1 - z, y, x),
            Turn::Back => Pos::new(sx - 1 - x, y, sz - 1 - z),
            Turn::Left => Pos::new(z, y, sx - 1 - x),
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

    /// Property `name=value` once turned: directions and axes turn, and so do sides. `to` is
    /// where a big block's other cell (`core:covered`) points: a bed's head to its foot.
    fn prop<'a>(self, name: &'a str, value: &'a str) -> (&'a str, &'a str) {
        match name {
            "facing" | "front" | "attach" | "to" => (name, self.direction(value)),
            "axis" => (name, self.axis(value)),
            "north" | "east" | "south" | "west" => (self.direction(name), value),
            _ => (name, value),
        }
    }
}

impl Turn {
    /// A block state or slot name turned: `base:oak_stairs[facing=north]` turned right is
    /// `base:oak_stairs[facing=east]`. Properties that don't point anywhere stay.
    pub fn name(self, name: &str) -> String {
        let Some((block, props)) = name.strip_suffix(']').and_then(|n| n.split_once('[')) else {
            return name.to_string();
        };
        let turned: Vec<String> = props
            .split(',')
            .filter_map(|prop| prop.split_once('='))
            .map(|(prop, value)| {
                let (prop, value) = self.prop(prop, value);
                format!("{prop}={value}")
            })
            .collect();
        format!("{block}[{}]", turned.join(","))
    }
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
            Turn::Right.pos(Pos::new(3, 0, 0), size),
            Pos::new(1, 0, 3),
            "the north-east corner becomes the south-east one"
        );
        assert_eq!(
            Turn::Left.pos(Pos::new(3, 0, 0), size),
            Pos::new(0, 0, 0),
            "and turning left, the north-west one"
        );
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

    #[test]
    fn devices_turn_where_they_point_and_hang() {
        let repeater = Turn::Right.prop("front", "north");
        assert_eq!(repeater, ("front", "east"), "a repeater's front");
        let lever = Turn::Back.prop("attach", "west");
        assert_eq!(lever, ("attach", "east"), "a lever on a wall");
        let button = Turn::Left.prop("attach", "down");
        assert_eq!(button, ("attach", "down"), "one on the floor stays");
    }

    #[test]
    fn names_turn_with_their_properties() {
        assert_eq!(
            Turn::Right.name("base:oak_log[axis=x]"),
            "base:oak_log[axis=z]",
            "a beam lies the other way"
        );
        let ladder = "base:ladder[north=true,east=false,south=false,west=false]";
        assert_eq!(
            Turn::Back.name(ladder),
            "base:ladder[south=true,west=false,north=false,east=false]",
            "a ladder moves to the opposite wall"
        );
        assert_eq!(
            Turn::Left.name("@roof/stair[facing=north,half=bottom]"),
            "@roof/stair[facing=west,half=bottom]",
            "a slot turns as its block would"
        );
        assert_eq!(
            Turn::Right.name("base:glass"),
            "base:glass",
            "no properties"
        );
    }

    #[test]
    fn a_bed_head_points_at_its_foot_once_turned() {
        // A bed facing north has its head south of its foot, pointing north at it; turned right,
        // the bed faces east and its head west of the foot points east.
        assert_eq!(Turn::Right.prop("facing", "north"), ("facing", "east"));
        assert_eq!(Turn::Right.prop("to", "north"), ("to", "east"), "the head");
    }

    #[test]
    fn sides_move_round() {
        let ladder = Turn::Right.prop("north", "true");
        assert_eq!(ladder, ("east", "true"), "the side moves");
        assert_eq!(
            Turn::Right.prop("lit", "true"),
            ("lit", "true"),
            "others stay"
        );
    }
}
