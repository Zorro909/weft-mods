//! The buildings: every file of `assets/structures/`, which the engine makes template
//! `structgen:<name>` of by itself. For the settlements this registers each turned the other
//! three ways, `structgen:<name>/right`, `/back` and `/left`, turned here (`turn.rs`), and,
//! for a building with ground in its box, a copy of each way whose ground is the `paving`
//! slot, cobblestone unless a palette says, `structgen:<name>/paved`, `/right/paved` and so
//! on, so a town paves round its houses.

use std::collections::HashMap;
use std::sync::OnceLock;

use weft_sdk::*;
use weftkit::Pos;

use crate::structure_file::Structure;
use crate::turn::Turn;

/// Where a building's cell takes the ground's own top block (`@ground`), and what its paved
/// copies have there instead.
const GROUND: &str = "@ground";
const PAVING: &str = "paving";
const PAVING_DEFAULT: &str = "base:cobblestone";

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));

/// What the settlements need to know of a building.
struct Building {
    /// Unturned.
    size: [i32; 3],
    /// Its chests, unturned, for loot.
    chests: Vec<Pos>,
    /// By turn: the plain template, and the paved one if it has ground to pave.
    placed: [(&'static Template, Option<&'static Template>); 4],
}

/// By template name, `structgen:hof`.
static BUILDINGS: OnceLock<HashMap<String, Building>> = OnceLock::new();

pub fn register(r: &mut Registry) {
    let mut buildings = HashMap::new();
    for (name, file) in EMBEDDED {
        match Structure::from_json(file) {
            Ok(structure) => {
                let building = register_turned(r, name, &structure);
                buildings.insert(
                    Template::generated(name.to_string()).name().to_string(),
                    building,
                );
            }
            Err(why) => error!("assets/structures/{name}.json: {why}; left out of the settlements"),
        }
    }
    let _ = BUILDINGS.set(buildings);
}

/// Registers the turned and paved copies of `structure`, the file `name`.
fn register_turned(r: &mut Registry, name: &str, structure: &Structure) -> Building {
    let paves = structure.blocks().any(|(_, cell)| cell == GROUND);
    let placed = Turn::ALL.map(|turn| {
        let plain = match turn {
            Turn::None => Template::generated(name.to_string()),
            _ => copy(
                r,
                structure,
                turn,
                false,
                format!("{name}/{}", turn_name(turn)),
            ),
        };
        let paved = paves.then(|| {
            let way = match turn {
                Turn::None => String::new(),
                _ => format!("/{}", turn_name(turn)),
            };
            copy(r, structure, turn, true, format!("{name}{way}/paved"))
        });
        (plain, paved)
    });
    let chests = structure.blocks().filter(|(_, cell)| is_chest(cell));
    Building {
        size: structure.size(),
        chests: chests.map(|([x, y, z], _)| Pos::new(x, y, z)).collect(),
        placed,
    }
}

/// Template `name`: `structure` turned by `turn`, its ground paved if `paved`.
fn copy(
    r: &mut Registry,
    structure: &Structure,
    turn: Turn,
    paved: bool,
    name: String,
) -> &'static Template {
    let template = Template::generated(name);
    let size = structure.size();
    let mut builder = r.template(template);
    for (slot, block) in structure.slots() {
        builder = builder.slot(slot, block);
    }
    if paved {
        builder = builder.slot(PAVING, PAVING_DEFAULT);
    }
    for ([x, y, z], cell) in structure.blocks() {
        let at = turn.pos(Pos::new(x, y, z), size);
        let cell = match cell {
            GROUND if paved => format!("@{PAVING}"),
            _ => turn.name(cell),
        };
        builder = builder.fill(&cell, [at.x, at.y, at.z], [at.x, at.y, at.z]);
    }
    template
}

fn turn_name(turn: Turn) -> &'static str {
    match turn {
        Turn::None => "none",
        Turn::Right => "right",
        Turn::Back => "back",
        Turn::Left => "left",
    }
}

fn is_chest(cell: &str) -> bool {
    cell == "base:chest" || cell.starts_with("base:chest[")
}

fn building(template: &Template) -> Option<&'static Building> {
    BUILDINGS.get()?.get(template.name())
}

/// The size of `template`, a building, unturned.
pub fn size(template: &Template) -> Option<[i32; 3]> {
    building(template).map(|b| b.size)
}

/// Where `template`'s chests are once turned by `turn`, from its lowest corner.
pub fn chests(template: &Template, turn: Turn) -> Vec<Pos> {
    let Some(b) = building(template) else {
        return Vec::new();
    };
    b.chests
        .iter()
        .map(|&chest| turn.pos(chest, b.size))
        .collect()
}

/// The template that places building `template` turned by `turn`, with its ground paved if
/// `paved` and it has ground.
pub fn placed(template: &Template, turn: Turn, paved: bool) -> Option<&'static Template> {
    let (plain, paved_copy) = building(template)?.placed[turn as usize];
    Some(paved_copy.filter(|_| paved).unwrap_or(plain))
}
