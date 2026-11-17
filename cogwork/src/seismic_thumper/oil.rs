//! A thumper listening for oil: one square of the oil grid a tick (`oil::search`), keeping the
//! `FIELDS` nearest fields within `REACH` of it, and how its screen says where each lies.

use weft_sdk::World;

use super::{FIELDS, Status, Survey};
use crate::lib::Pos;
use crate::oil::search::{self, Found};

/// How far off it hears oil fields, in blocks along the ground.
pub const REACH: i32 = 256;
/// What `Survey::field_depth` holds for a field whose depth is not known.
const UNKNOWN: i32 = i32::MIN;
/// The ways a field may lie, a lang key each under `cogwork.direction.`, clockwise from north.
const DIRECTIONS: [&str; 8] = [
    "north",
    "north_east",
    "east",
    "south_east",
    "south",
    "south_west",
    "west",
    "north_west",
];

/// A field as a thumper's screen lists it: where its seep lies from the thumper, along x and z,
/// and its oil's highest and lowest layer, if known.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Field {
    pub offset: (i32, i32),
    pub depth: Option<(i32, i32)>,
}

impl Field {
    /// Blocks to its seep along the ground, rounded.
    pub fn distance(&self) -> i64 {
        let (x, z) = self.offset;
        (x as f64).hypot(z as f64).round() as i64
    }

    /// The way it lies, as its lang key under `cogwork.direction.`: north is toward less z.
    pub fn direction(&self) -> &'static str {
        let (x, z) = self.offset;
        let turn = (x as f64).atan2(-z as f64).to_degrees().rem_euclid(360.0);
        DIRECTIONS[((turn / 45.0).round() as usize) % DIRECTIONS.len()]
    }
}

/// Listens to the next square of the oil grid around the thumper at `pos`; once it has heard
/// them all, the survey is done.
pub fn listen(world: &World, pos: Pos, survey: &mut Survey) {
    let squares = search::squares_near(pos, REACH);
    let Some(&square) = squares.get(survey.listened as usize) else {
        survey.set_status(Status::Done);
        return;
    };
    survey.listened += 1;
    if let Some(found) = search::field_in(world, square) {
        survey.note_field(field_from(pos, found));
    }
}

/// How `found` lies from a thumper at `pos`.
fn field_from(pos: Pos, found: Found) -> Field {
    Field {
        offset: (found.start.x - pos.x, found.start.z - pos.z),
        depth: found.depth.map(|depth| (depth.top, depth.bottom)),
    }
}

impl Survey {
    /// Notes `field` if it lies within `REACH`, keeping the `FIELDS` nearest, nearest first.
    pub fn note_field(&mut self, field: Field) {
        if field.distance() > REACH as i64 {
            return;
        }
        let mut fields: Vec<Field> = (0..self.fields as usize)
            .filter_map(|i| self.field(i))
            .collect();
        fields.push(field);
        fields.sort_by_key(Field::distance);
        fields.truncate(FIELDS);
        self.fields = fields.len() as u32;
        for (i, kept) in fields.iter().enumerate() {
            let (top, bottom) = kept.depth.unwrap_or((UNKNOWN, UNKNOWN));
            self.field_at[2 * i..2 * i + 2].copy_from_slice(&[kept.offset.0, kept.offset.1]);
            self.field_depth[2 * i..2 * i + 2].copy_from_slice(&[top, bottom]);
        }
    }

    /// The `i`th nearest field it found.
    pub fn field(&self, i: usize) -> Option<Field> {
        if i >= (self.fields as usize).min(FIELDS) {
            return None;
        }
        let (top, bottom) = (self.field_depth[2 * i], self.field_depth[2 * i + 1]);
        Some(Field {
            offset: (self.field_at[2 * i], self.field_at[2 * i + 1]),
            depth: (top != UNKNOWN).then_some((top, bottom)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(x: i32, z: i32) -> Field {
        Field {
            offset: (x, z),
            depth: Some((20, 10)),
        }
    }

    #[test]
    fn a_field_lies_by_the_compass() {
        let ways = [
            (0, -100),
            (70, -70),
            (100, 0),
            (70, 70),
            (0, 100),
            (-70, 70),
            (-100, 0),
            (-70, -70),
        ];
        let named: Vec<&str> = ways.iter().map(|&(x, z)| field(x, z).direction()).collect();
        assert_eq!(named, DIRECTIONS, "north is toward less z, then clockwise");
        assert_eq!(field(10, -100).direction(), "north", "near enough north");
        assert_eq!(field(30, -40).distance(), 50, "along the ground");
    }

    #[test]
    fn it_keeps_the_nearest_fields_within_reach() {
        let mut survey = Survey::default();
        survey.note_field(field(0, 300));
        assert_eq!(survey.fields, 0, "too far to hear");
        for x in [200, 50, 120, 90, 10] {
            survey.note_field(field(x, 0));
        }
        let kept: Vec<i32> = (0..FIELDS)
            .filter_map(|i| survey.field(i))
            .map(|f| f.offset.0)
            .collect();
        assert_eq!(kept, [10, 50, 90, 120], "the four nearest, nearest first");
        assert_eq!(survey.field(FIELDS), None, "no more");
    }

    #[test]
    fn a_field_of_unknown_depth_says_so() {
        let mut survey = Survey::default();
        survey.note_field(Field {
            offset: (5, 5),
            depth: None,
        });
        assert_eq!(survey.field(0).and_then(|f| f.depth), None, "unknown");
    }

    #[test]
    fn every_direction_has_a_name() {
        for way in DIRECTIONS {
            let key = format!("cogwork.direction.{way}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
