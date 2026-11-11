//! Turns a hand-drawn sketch of a settlement into something to look at in the game: roads
//! that curve, squares, fields and hedges drawn onto the ground, and this mod's buildings and
//! trees placed on it. `cargo run --example sketch -- <sketch.txt> <out dir>` writes:
//!
//! - `<sketch>_ground_<n>.json`: the ground, in pieces `/structure load` can take;
//! - `<sketch>.txt`: a layout of the pieces and buildings for `tools/structure-shots.sh --layout`;
//! - `<sketch>.map`: a text map from above, to check what stands where.
//!
//! The sketch format is in `parse.rs`; `examples/sketches/` has some.

#[allow(dead_code)]
#[path = "../buildings/draft.rs"]
mod draft;
#[allow(dead_code)]
#[path = "../buildings/kit.rs"]
mod kit;
mod map;
mod parse;
mod pieces;
mod raster;
// The mod's own file reader and writer; each example uses part of it.
#[allow(dead_code)]
#[path = "../../src/structure_file.rs"]
mod structure_file;

use std::fmt::Write;
use std::path::Path;

use crate::structure_file::Structure;

/// A placed structure's footprint as turned: its north-west corner and size along x and z.
pub struct Footprint {
    pub name: String,
    pub corner: [i32; 2],
    pub size: [i32; 2],
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [file, out] = &args[..] else {
        panic!("usage: sketch <sketch.txt> <out dir>");
    };
    let name = Path::new(file)
        .file_stem()
        .map_or("sketch".to_string(), |s| s.to_string_lossy().into_owned());
    let sketch = parse::read(file);
    let footprints: Vec<Footprint> = sketch.places.iter().map(footprint).collect();
    // A town's buildings stand on paving (`cobble_ground`): the streets pave out to them.
    let paved: Vec<raster::Rect> = (sketch.places.iter().zip(&footprints))
        .filter(|(place, _)| place.styles.contains("cobble_ground"))
        .map(|(_, footprint)| (footprint.corner, footprint.size))
        .collect();
    let ground = raster::Ground::of(&sketch, &paved);
    let out = Path::new(out);
    std::fs::create_dir_all(out)
        .unwrap_or_else(|e| panic!("could not create {}: {e}", out.display()));

    let mut layout = format!("# {name}, from {file}\n");
    for (n, piece) in pieces::cut(&ground).iter().enumerate() {
        let piece_name = format!("{name}_ground_{n}");
        write(
            &out.join(format!("{piece_name}.json")),
            piece.structure.to_json().as_bytes(),
        );
        let [x, z] = piece.corner;
        writeln!(layout, "{piece_name} none {x} {z}").ok();
    }
    for (place, footprint) in sketch.places.iter().zip(&footprints) {
        let [x, z] = footprint.corner;
        writeln!(
            layout,
            "{} {} {x} {z} {}",
            place.name, place.turn, place.styles
        )
        .ok();
    }
    for view in &sketch.views {
        let [e, t] = [view.eye, view.target];
        writeln!(
            layout,
            "view {} {} {} {} {} {} {}",
            view.name, e[0], e[1], e[2], t[0], t[1], t[2]
        )
        .ok();
    }
    write(&out.join(format!("{name}.txt")), layout.as_bytes());
    write(
        &out.join(format!("{name}.map")),
        map::draw(&ground, &footprints).as_bytes(),
    );
    for warning in map::overlaps(&footprints) {
        eprintln!("{name}: {warning}");
    }
    println!("{}", out.join(format!("{name}.txt")).display());
}

/// Where `place` stands, from the size of its file in `assets/structures/`.
fn footprint(place: &parse::Place) -> Footprint {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("assets/structures/{}.json", place.name));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("no structure {}: {e}", place.name));
    let structure = Structure::from_json(&text)
        .unwrap_or_else(|why| panic!("{} is no structure file: {why}", place.name));
    let [sx, _, sz] = structure.size();
    let size = match place.turn.as_str() {
        "right" | "left" => [sz, sx],
        "none" | "back" => [sx, sz],
        other => panic!("{}: no turn {other}", place.name),
    };
    Footprint {
        name: place.name.clone(),
        corner: [place.middle[0] - size[0] / 2, place.middle[1] - size[1] / 2],
        size,
    }
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes)
        .unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
}
