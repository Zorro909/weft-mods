//! Writing block models as Blockbench saves a "Java Block/Item" model, for the families of
//! models too many to draw one by one: tubes and torque shafts (`arms::files`), tanks
//! (`tank::files`). They are kept as `generated` says.

use std::path::{Path, PathBuf};

use weft_sdk::*;

pub use crate::generated::WRITE;

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
pub type Corners = ([f32; 3], [f32; 3]);

/// A value of a model file, written as Blockbench indents it: a line an entry, a tab a depth.
pub enum Json {
    /// Text that stays on its line: a string, a list of numbers, a face.
    Inline(String),
    List(Vec<Json>),
    Object(Vec<(&'static str, Json)>),
}

impl Json {
    pub fn text(text: &str) -> Json {
        Json::Inline(format!("\"{text}\""))
    }

    pub fn numbers(numbers: &[f32]) -> Json {
        let numbers: Vec<String> = numbers.iter().map(f32::to_string).collect();
        Json::Inline(format!("[{}]", numbers.join(", ")))
    }

    pub fn written(&self, depth: usize) -> String {
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

/// One box of a model, with a face toward each side `texture` names a texture for. Each face
/// shows the part of its texture that lies where the box does, as the engine takes it for a face
/// that names none: so a picture drawn over the whole block goes on across boxes.
pub fn element<'a>(
    name: &str,
    (from, to): Corners,
    texture: impl Fn(Dir) -> Option<&'a str>,
) -> Json {
    let face = |face: Dir| {
        let uv = Json::numbers(&abi::box_face_uv(face.face() as usize, from, to)).written(0);
        let shown = format!("{{\"uv\": {uv}, \"texture\": \"#{}\"}}", texture(face)?);
        Some((face.name(), Json::Inline(shown)))
    };
    Json::Object(vec![
        ("name", Json::text(name)),
        ("from", Json::numbers(&from)),
        ("to", Json::numbers(&to)),
        (
            "faces",
            Json::Object(FACES.into_iter().filter_map(face).collect()),
        ),
    ])
}

/// A model file of `elements`, naming those of `textures` its faces use and the particle, with
/// the picture Blockbench shows for each.
pub fn model(textures: Vec<(&'static str, String)>, elements: Vec<Json>) -> String {
    let empty = elements.is_empty();
    let elements = Json::List(elements);
    let textures = as_saved(textures, &elements.written(1)).into_iter();
    let mut fields = vec![
        ("format_version", Json::text("26.3")),
        ("credit", Json::text("Made with Blockbench")),
        (
            "textures",
            Json::Object(textures.map(|(name, at)| (name, Json::text(&at))).collect()),
        ),
    ];
    // Blockbench leaves the list out of a model with nothing in it.
    if !empty {
        fields.push(("elements", elements));
    }
    Json::Object(fields).written(0) + "\n"
}

/// `textures` as Blockbench saves them: only those `faces` use, and the particle just before
/// the texture of the same picture, or last.
fn as_saved(textures: Vec<(&'static str, String)>, faces: &str) -> Vec<(&'static str, String)> {
    let used = |name: &str| faces.contains(&format!("\"#{name}\""));
    let (particle, mut saved): (Vec<_>, Vec<_>) = textures
        .into_iter()
        .filter(|(name, _)| *name == "particle" || used(name))
        .partition(|(name, _)| *name == "particle");
    for particle in particle {
        let same = saved.iter().position(|(_, picture)| *picture == particle.1);
        saved.insert(same.unwrap_or(saved.len()), particle);
    }
    saved
}

/// Where the mod keeps the models of `block`.
pub fn folder(block: &str) -> PathBuf {
    crate::generated::asset("models/block").join(block)
}

/// Writes the model `text` to `path`, or checks it is there (`generated`).
pub fn save_or_check(path: &Path, text: &str) {
    crate::generated::save_or_check(path, text.as_bytes());
}
