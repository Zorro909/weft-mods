//! Puts every structure compiled into the mod into base's overworld, all the same way.

use weft_sdk::*;

use crate::format::Structure;

/// One chance per cell of this many blocks square.
const SPACING: u32 = 256;
/// How likely a cell is to get the structure.
const CHANCE: f32 = 0.3;

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

pub fn register(r: &mut Registry) {
    for (name, file) in EMBEDDED {
        let Some(structure) = Structure::decode(file) else {
            error!(
                "assets/structures/{name}.wst is not a structure file of this version, left out"
            );
            continue;
        };
        let template = Template::generated(name.to_string());
        add_template(r, template, &structure);
        r.structure(&base_api::OVERWORLD, template)
            .spacing(SPACING)
            .chance(CHANCE);
    }
}

fn add_template(r: &mut Registry, template: &Template, structure: &Structure) {
    let mut builder = r.template(template);
    for (pos, block) in structure.blocks() {
        builder = builder.fill(block, pos, pos);
    }
}
