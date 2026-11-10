//! The model files of hubs with arms. Sixty-four for tubes and as many for torque shafts are
//! too many to draw one by one, so they are written from here, as Blockbench saves a
//! "Java Block/Item" model. After a change, `WRITE_ARM_MODELS=1 cargo test --release arms`
//! writes them again; without it, the test fails on a file that is not what is written here.

use std::path::{Path, PathBuf};

use weft_sdk::*;

/// Set to anything, it has the test write the files instead of comparing them.
const WRITE: &str = "WRITE_ARM_MODELS";

/// A box's faces in the order Blockbench lists them.
const FACES: [Dir; 6] = [
    Dir::North,
    Dir::East,
    Dir::South,
    Dir::West,
    Dir::Up,
    Dir::Down,
];

/// The two corners of a box, in texels: 16 to a block.
type Corners = ([f32; 3], [f32; 3]);

/// A block made of a hub and arms.
#[derive(Clone, Copy)]
enum Fitting {
    Tube,
    Shaft,
}

impl Fitting {
    /// Its folder of models, as `arms::model` takes it.
    fn block(self) -> &'static str {
        match self {
            Fitting::Tube => "tube",
            Fitting::Shaft => "torque_shaft",
        }
    }

    /// Its hub, in the middle of the block: a tube's 8 texels wide, a shaft's 4. The arms are
    /// as thick.
    fn hub(self) -> Corners {
        match self {
            Fitting::Tube => ([4.0; 3], [12.0; 3]),
            Fitting::Shaft => ([6.0; 3], [10.0; 3]),
        }
    }

    /// The textures the file names, and the picture Blockbench shows for each. A tube has one a
    /// side, which each tube sets to its own (`tube::shape`); the file shows a sorting tube's.
    fn textures(self) -> Vec<(&'static str, String)> {
        match self {
            Fitting::Tube => {
                let side = |dir| format!("block/{}", crate::tube::sorting::arm_texture(dir));
                let sides = Dir::ALL.into_iter().map(|dir| (dir.name(), side(dir)));
                sides
                    .chain([("particle", "block/tube_sorting".into())])
                    .collect()
            }
            Fitting::Shaft => ["particle", "shaft"]
                .map(|name| (name, "block/torque_shaft".into()))
                .to_vec(),
        }
    }

    /// The texture on `face` of the arm toward `arm`, or of the hub: a tube's arm shows its
    /// side's all round, its hub each side's on the face that way.
    fn texture(self, arm: Option<Dir>, face: Dir) -> &'static str {
        match self {
            Fitting::Tube => arm.unwrap_or(face).name(),
            Fitting::Shaft => "shaft",
        }
    }
}

/// A value of a model file, written as Blockbench indents it: a line an entry, a tab a depth.
enum Json {
    /// Text that stays on its line: a string, a list of numbers, a face.
    Inline(String),
    List(Vec<Json>),
    Object(Vec<(&'static str, Json)>),
}

impl Json {
    fn text(text: &str) -> Json {
        Json::Inline(format!("\"{text}\""))
    }

    fn numbers(numbers: &[f32]) -> Json {
        let numbers: Vec<String> = numbers.iter().map(f32::to_string).collect();
        Json::Inline(format!("[{}]", numbers.join(", ")))
    }

    fn written(&self, depth: usize) -> String {
        let tabs = "\t".repeat(depth);
        let lines = |open: char, entries: Vec<String>, close: char| {
            let entries: Vec<String> = entries.iter().map(|e| format!("{tabs}\t{e}")).collect();
            format!("{open}\n{}\n{tabs}{close}", entries.join(",\n"))
        };
        match self {
            Json::Inline(text) => text.clone(),
            Json::List(items) => {
                let items = items.iter().map(|item| item.written(depth + 1));
                lines('[', items.collect(), ']')
            }
            Json::Object(fields) => {
                let field = |(name, value): &(&str, Json)| {
                    format!("\"{name}\": {}", value.written(depth + 1))
                };
                lines('{', fields.iter().map(field).collect(), '}')
            }
        }
    }
}

/// The box as thick as `hub` that reaches from its face toward `side` out to the edge of the
/// block.
fn arm((mut from, mut to): Corners, side: Dir) -> Corners {
    let axis = side.face() as usize / 2;
    // Even faces are the low side of their axis.
    if side.face().is_multiple_of(2) {
        (from[axis], to[axis]) = (0.0, from[axis]);
    } else {
        (from[axis], to[axis]) = (to[axis], 16.0);
    }
    (from, to)
}

/// One box of a model. Each face shows the part of its texture that lies where the box does,
/// as the engine takes it for a face that names none: so a picture drawn over the whole block
/// goes on across hub and arms.
fn element(name: &str, (from, to): Corners, texture: impl Fn(Dir) -> &'static str) -> Json {
    let face = |face: Dir| {
        let uv = Json::numbers(&abi::box_face_uv(face.face() as usize, from, to)).written(0);
        let shown = format!("{{\"uv\": {uv}, \"texture\": \"#{}\"}}", texture(face));
        (face.name(), Json::Inline(shown))
    };
    Json::Object(vec![
        ("name", Json::text(name)),
        ("from", Json::numbers(&from)),
        ("to", Json::numbers(&to)),
        ("faces", Json::Object(FACES.map(face).into())),
    ])
}

/// The model file of `fitting` with `arms`.
fn model_file(fitting: Fitting, arms: &[Dir]) -> String {
    let hub = fitting.hub();
    let mut elements = vec![element("hub", hub, |face| fitting.texture(None, face))];
    for &side in arms {
        let texture = |face| fitting.texture(Some(side), face);
        elements.push(element(side.name(), arm(hub, side), texture));
    }
    let textures = fitting.textures().into_iter();
    let model = Json::Object(vec![
        ("format_version", Json::text("26.3")),
        ("credit", Json::text("Made with Blockbench")),
        (
            "textures",
            Json::Object(textures.map(|(name, at)| (name, Json::text(&at))).collect()),
        ),
        ("elements", Json::List(elements)),
    ]);
    model.written(0) + "\n"
}

/// Where the mod keeps the models of `block`.
fn folder(block: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/models/block")
        .join(block)
}

#[test]
fn arms_reach_from_the_hub_to_the_edge() {
    let hub = Fitting::Tube.hub();
    let west = ([0.0, 4.0, 4.0], [4.0, 12.0, 12.0]);
    assert_eq!(arm(hub, Dir::West), west, "west");
    let up = ([4.0, 12.0, 4.0], [12.0, 16.0, 12.0]);
    assert_eq!(arm(hub, Dir::Up), up, "up");
}

#[test]
fn the_model_files_are_what_is_written_here() {
    let write = std::env::var_os(WRITE).is_some();
    for fitting in [Fitting::Tube, Fitting::Shaft] {
        for set in 0..64 {
            let arms: Vec<Dir> = Dir::ALL
                .into_iter()
                .filter(|dir| set & (1 << dir.face()) != 0)
                .collect();
            let name = super::model(fitting.block(), |dir| arms.contains(&dir));
            let path = folder("").join(format!("{name}.json"));
            let text = model_file(fitting, &arms);
            if write {
                std::fs::write(&path, text).expect("could not write a model file");
            } else {
                let saved = std::fs::read_to_string(&path).unwrap_or_default();
                assert!(saved == text, "{path:?} differs: {WRITE}=1 writes it again");
            }
        }
        let files = std::fs::read_dir(folder(fitting.block())).expect("no folder of models");
        assert_eq!(files.count(), 64, "a model no set of arms draws");
    }
}
