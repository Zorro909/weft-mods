//! Hubs with arms toward some of their six sides, as tubes, liquid tubes and torque shafts are.
//! Each kind is drawn from one Blockbench model, `assets/models/block/<block>/hub_and_arm.json`:
//! its elements named `hub` stay in the middle of the block, and those named `arm` point north
//! and are turned toward every side the block has an arm. A face whose texture is named after
//! the arm's side (`#north`) shows the texture of the side it is turned to.
//!
//! The model is built into the mod and put together when its block states are made (`boxes`),
//! so there is no file for each set of arms. Where the hub and an arm meet neither has a face,
//! and an open arm has none at its end either: through a see-through texture they would show as
//! panes across the tube. A hub wider than its arms opens toward each open arm, and that arm's
//! own parts close it round (the timber liquid tube's flanges).

use std::collections::HashMap;
use std::sync::LazyLock;

use serde_json::Value;
use weft_sdk::{Dir, Facings};

use crate::boxes::{Corners, Piece};

/// The models of the hubs with arms, from their files built into the mod.
pub static TUBE: LazyLock<HubAndArms> = LazyLock::new(|| {
    let file = include_str!("../../assets/models/block/tube/hub_and_arm.json");
    HubAndArms::read("tube", file)
});
pub static LIQUID_TUBE: LazyLock<HubAndArms> = LazyLock::new(|| {
    let file = include_str!("../../assets/models/block/liquid_tube/hub_and_arm.json");
    HubAndArms::read("liquid_tube", file)
});
pub static TIMBER_LIQUID_TUBE: LazyLock<HubAndArms> = LazyLock::new(|| {
    let file = include_str!("../../assets/models/block/timber_liquid_tube/hub_and_arm.json");
    HubAndArms::read("timber_liquid_tube", file)
});
pub static TORQUE_SHAFT: LazyLock<HubAndArms> = LazyLock::new(|| {
    let file = include_str!("../../assets/models/block/torque_shaft/hub_and_arm.json");
    HubAndArms::read("torque_shaft", file)
});

/// The side the arms of a hub-and-arm model point to.
const MODEL_ARM: Dir = Dir::North;

/// An arm of a hub, by whether its end has a face.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arm {
    /// Its end drawn: there may be nothing beside it, as beside a directing tube's output.
    Closed,
    /// Its end left out, and the hub's face toward it: it lies against what it joins, and the
    /// liquid in it goes through.
    Open,
}

/// A hub with arms, as its model file has it.
pub struct HubAndArms {
    hub: Vec<Part>,
    arm: Vec<Part>,
    /// The model's textures by the names its faces use, under `block/`.
    textures: HashMap<String, String>,
}

/// A box of the model, the name of the texture its face toward each side shows (without `#`).
#[derive(Clone)]
struct Part {
    corners: Corners,
    faces: [Option<String>; 6],
}

impl HubAndArms {
    /// The model in the Blockbench file `json`, which `file` names for what goes wrong. A file
    /// that is not such a model stops the mod from loading: the tests read every one first.
    pub fn read(file: &str, json: &str) -> HubAndArms {
        let model: Value = serde_json::from_str(json).unwrap_or_else(|why| panic!("{file}: {why}"));
        let textures = model["textures"]
            .as_object()
            .map_or_else(HashMap::new, |all| {
                let texture = |(name, at): (&String, &Value)| {
                    let at = at.as_str().unwrap_or_default();
                    (name.clone(), at.trim_start_matches("block/").to_string())
                };
                all.iter().map(texture).collect()
            });
        let mut hub_and_arms = HubAndArms {
            hub: Vec::new(),
            arm: Vec::new(),
            textures,
        };
        let elements = model["elements"].as_array().map_or(&[][..], Vec::as_slice);
        for element in elements {
            let part = part(file, element);
            match element["name"].as_str() {
                Some("hub") => hub_and_arms.hub.push(part),
                Some("arm") => hub_and_arms.arm.push(part),
                other => panic!("{file}: an element named {other:?}, neither hub nor arm"),
            }
        }
        hub_and_arms
    }

    /// The texture the model gives its faces named `name`.
    pub fn texture(&self, name: &str) -> &str {
        self.textures.get(name).map_or("", String::as_str)
    }

    /// The pieces of the hub with the arm `arm` gives toward each side, if any, each face
    /// showing the texture `texture` gives for its name in the file.
    pub fn pieces(
        &self,
        arm: impl Fn(Dir) -> Option<Arm>,
        texture: impl Fn(&str) -> String,
    ) -> Vec<Piece> {
        let shown = |part: &Part| Piece {
            corners: part.corners,
            faces: part
                .faces
                .clone()
                .map(|name| name.map(|name| texture(&name))),
        };
        let mut pieces: Vec<Piece> = self.hub.iter().map(shown).collect();
        let face = |dir: Dir| dir.face() as usize;
        for side in Dir::ALL {
            let Some(kind) = arm(side) else {
                continue;
            };
            if kind == Arm::Open {
                for hub in &mut pieces[..self.hub.len()] {
                    hub.faces[face(side)] = None;
                }
            }
            for part in &self.arm {
                let mut piece = shown(&turned(part, side));
                if kind == Arm::Open && reaches_the_edge(part) {
                    piece.faces[face(side)] = None;
                }
                pieces.push(piece);
            }
        }
        crate::boxes::hide_shared_faces(&mut pieces);
        pieces
    }
}

/// The part an element of the model file is.
fn part(file: &str, element: &Value) -> Part {
    let corner = |key: &str| -> [f32; 3] {
        let numbers = element[key].as_array().filter(|numbers| numbers.len() == 3);
        let numbers = numbers.unwrap_or_else(|| panic!("{file}: an element's {key} is no point"));
        std::array::from_fn(|axis| numbers[axis].as_f64().unwrap_or_default() as f32)
    };
    let faces = Dir::ALL.map(|dir| {
        let texture = element["faces"][dir.name()]["texture"].as_str()?;
        Some(texture.trim_start_matches('#').to_string())
    });
    Part {
        corners: (corner("from"), corner("to")),
        faces,
    }
}

/// Whether `part` of an arm pointing north reaches the block's north edge, where the arm ends.
fn reaches_the_edge(part: &Part) -> bool {
    part.corners.0[2] == 0.0
}

/// `part`, an arm pointing north, turned to point toward `side`.
fn turned(part: &Part, side: Dir) -> Part {
    let at = |corner: [f32; 3]| turn(corner.map(|at| at - 8.0), side).map(|at| at + 8.0);
    let (a, b) = (at(part.corners.0), at(part.corners.1));
    let corners = (
        std::array::from_fn(|axis| a[axis].min(b[axis])),
        std::array::from_fn(|axis| a[axis].max(b[axis])),
    );
    let mut faces: [Option<String>; 6] = Default::default();
    for dir in Dir::ALL {
        let offset = dir.offset();
        let toward = [offset.x, offset.y, offset.z].map(|at| at as f32);
        let name = part.faces[dir.face() as usize].clone();
        // The arm's own side's texture goes where the arm does.
        let name = name.map(|name| {
            if name == MODEL_ARM.name() {
                side.name().to_string()
            } else {
                name
            }
        });
        faces[Dir::nearest(turn(toward, side), Facings::ALL).face() as usize] = name;
    }
    Part { corners, faces }
}

/// `at`, a point measured from the middle of the block or a direction, turned as an arm pointing
/// north is turned to point toward `side`.
fn turn([x, y, z]: [f32; 3], side: Dir) -> [f32; 3] {
    match side {
        Dir::North => [x, y, z],
        Dir::South => [-x, y, -z],
        Dir::East => [-z, y, x],
        Dir::West => [z, y, -x],
        Dir::Up => [x, -z, y],
        Dir::Down => [x, z, -y],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hub of 4 to 12 with its arm north, each face named after its side.
    const MODEL: &str = r##"{
        "textures": {"north": "block/red", "west": "block/green", "particle": "block/tube"},
        "elements": [
            {"name": "hub", "from": [4, 4, 4], "to": [12, 12, 12],
             "faces": {"north": {"texture": "#north"}, "west": {"texture": "#west"}}},
            {"name": "arm", "from": [4, 4, 0], "to": [12, 12, 4],
             "faces": {"north": {"texture": "#north"}, "up": {"texture": "#north"}}}
        ]
    }"##;

    fn named(name: &str) -> String {
        name.to_string()
    }

    #[test]
    fn an_arm_turns_to_its_side() {
        let model = HubAndArms::read("test", MODEL);
        let pieces = model.pieces(|dir| (dir == Dir::West).then_some(Arm::Closed), named);
        assert_eq!(pieces.len(), 2, "the hub and one arm");
        let west = &pieces[1];
        assert_eq!(
            west.corners,
            ([0.0, 4.0, 4.0], [4.0, 12.0, 12.0]),
            "from the hub to the edge"
        );
        let face = |dir: Dir| west.faces[dir.face() as usize].as_deref();
        assert_eq!(
            face(Dir::West),
            Some("west"),
            "its end, in the west side's texture"
        );
        assert_eq!(face(Dir::Up), Some("west"), "its top too");
        assert_eq!(face(Dir::North), None, "no face where the model has none");
    }

    #[test]
    fn an_arm_up_reaches_the_top() {
        let model = HubAndArms::read("test", MODEL);
        let up = &model.pieces(|dir| (dir == Dir::Up).then_some(Arm::Closed), named)[1];
        assert_eq!(up.corners, ([4.0, 12.0, 4.0], [12.0, 16.0, 12.0]), "up");
        assert_eq!(
            up.faces[Dir::Up.face() as usize].as_deref(),
            Some("up"),
            "its end"
        );
    }

    #[test]
    fn a_see_through_tube_shows_no_pane_inside() {
        let model = HubAndArms::read("test", MODEL);
        let pieces = model.pieces(|dir| (dir == Dir::West).then_some(Arm::Open), named);
        let [hub, west] = &pieces[..] else {
            panic!("the hub and one arm");
        };
        let face = |piece: &Piece, dir: Dir| piece.faces[dir.face() as usize].is_some();
        assert!(!face(hub, Dir::West), "none where the arm meets the hub");
        assert!(!face(west, Dir::East), "nor on the arm there");
        assert!(!face(west, Dir::West), "an open end has none");
        assert!(face(west, Dir::Up), "its sides stay");
    }

    #[test]
    fn a_hub_wider_than_its_arm_opens_into_it_and_a_flange_closes_it_round() {
        let model = HubAndArms::read(
            "test",
            r##"{
            "textures": {"wood": "block/wood"},
            "elements": [
                {"name": "hub", "from": [3, 3, 3], "to": [13, 13, 13],
                 "faces": {"north": {"texture": "#wood"}, "up": {"texture": "#wood"}}},
                {"name": "arm", "from": [4, 4, 0], "to": [12, 12, 3],
                 "faces": {"north": {"texture": "#wood"}}},
                {"name": "arm", "from": [3, 12, 2], "to": [13, 13, 3],
                 "faces": {"north": {"texture": "#wood"}}}
            ]
        }"##,
        );
        let pieces = model.pieces(|dir| (dir == Dir::North).then_some(Arm::Open), named);
        let [hub, arm, flange] = &pieces[..] else {
            panic!("the hub, the arm and its flange");
        };
        let face = |piece: &Piece, dir: Dir| piece.faces[dir.face() as usize].is_some();
        assert!(!face(hub, Dir::North), "the hub opens toward the arm");
        assert!(face(hub, Dir::Up), "and nowhere else");
        assert!(!face(arm, Dir::North), "the arm's end is open");
        assert!(
            face(flange, Dir::North),
            "the flange short of the edge keeps its face"
        );
    }

    #[test]
    fn the_mods_models_have_a_hub_and_an_arm() {
        for model in [&TUBE, &LIQUID_TUBE, &TIMBER_LIQUID_TUBE, &TORQUE_SHAFT] {
            assert!(!model.hub.is_empty() && !model.arm.is_empty(), "both");
            assert!(
                !model.texture("particle").is_empty(),
                "a picture to break into"
            );
        }
    }

    #[test]
    fn a_hub_alone_has_no_arms() {
        let model = HubAndArms::read("test", MODEL);
        assert_eq!(model.pieces(|_| None, named).len(), 1, "the hub");
        assert_eq!(model.texture("particle"), "tube", "under block/");
    }
}
