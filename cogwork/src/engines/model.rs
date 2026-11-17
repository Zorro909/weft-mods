//! An engine's block: a base plate on the side it was placed against and a trunk pointing the
//! other way that glows with its heat. Both are the Blockbench model `models/block/engine.json`,
//! lying on the floor with the trunk up. The piston riding the trunk is an entity (`piston`).
//!
//! The engines' items stand that way too, cool, through `models/item/<kind>_engine.json`, which
//! name `engine.json` as their parent and set only its textures. Without them an item would show
//! its block facing south, lying on its side.

use weft_sdk::{BlockBuilder, Dir, Facings, Orientation, Output, Props, State, Toward, Use, World};

use crate::lib::Pos;

/// The model both engines share; each state puts its own textures in the file's places.
const MODEL: &str = "engine";

/// Trunk textures by heat level: cool, warm, hot, overheated.
const TRUNKS: [&str; 4] = [
    "engine_trunk_blue",
    "engine_trunk_green",
    "engine_trunk_orange",
    "engine_trunk_red",
];

/// An engine block of `kind` (`spring`, `ember`): it faces away from the block it was placed
/// against, so its trunk points into that block.
pub fn engine<'a>(builder: BlockBuilder<'a>, kind: &'static str) -> BlockBuilder<'a> {
    builder
        .grid_placement()
        .model_file(MODEL)
        .facing(Facings::ALL, Toward::Face)
        .prop("heat", 0..=3)
        .sound_group("base:block.stone")
        .states(move |props, state| look(props, state, kind))
        .on_use(use_engine)
}

/// One state's turn and textures: the plate of its `kind`, the trunk of its heat.
fn look<'s>(props: &Props, state: State<'s>, kind: &str) -> State<'s> {
    let plate_top = format!("engine_{kind}_plate_top");
    state
        .texture(&plate_top)
        // Its trunk, the model's top, points away from the facing.
        .orientation(Orientation::up(props.facing().opposite()))
        .model_texture("plate_top", &plate_top)
        .model_texture("plate_side", &format!("engine_{kind}_plate_side"))
        .model_texture("trunk", TRUNKS[props.int("heat").min(3) as usize])
        // The whole block, piston and all: nothing walks into the piston's way.
        .hitbox([0.0; 3], [16.0; 3])
}

/// A wrench turns the engine; an empty hand winds a spring engine (`wind`). False passes the
/// click on: the ember engine opens its screen.
fn use_engine(click: &mut Use) -> bool {
    let (world, pos) = (click.world(), Pos::from(click.pos()));
    if crate::cogs::holds_wrench(&world, click.player()) {
        turn(&world, pos);
        return true;
    }
    super::wind::wind(&world, click.player(), pos)
}

/// The tuning wrench on the engine at `pos`: its trunk points the next way round.
pub fn turn(world: &World, pos: Pos) {
    world.prop_cycle(pos, "facing");
    world.play_sound(&crate::sounds::WRENCH, pos.center(), 1.0);
}

/// Where the engine's trunk points: its output.
pub fn output(world: &World, pos: Pos) -> Option<Dir> {
    world
        .prop(pos, "facing")
        .and_then(Dir::from_name)
        .map(Dir::opposite)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The engines' kinds, as their blocks and textures are named.
    const KINDS: [&str; 2] = ["spring", "ember"];

    #[test]
    fn the_engines_have_their_model_and_its_textures() {
        let model = format!("models/block/{MODEL}.json");
        assert!(crate::has_asset(&model), "no {model}");
        let plates = KINDS
            .into_iter()
            .flat_map(|kind| ["top", "side"].map(|face| format!("engine_{kind}_plate_{face}")));
        for texture in plates.chain(TRUNKS.map(String::from)) {
            let file = format!("textures/block/{texture}.png");
            assert!(crate::has_asset(&file), "no {file}");
        }
    }

    #[test]
    fn each_engine_item_is_the_block_model_upright_and_cool() {
        let items = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/models/item");
        for kind in KINDS {
            let file = items.join(format!("{kind}_engine.json"));
            let text = std::fs::read_to_string(&file).expect("each engine item has a model");
            let model: serde_json::Value =
                serde_json::from_str(&text).expect("an item model is JSON");
            let shape = format!("block/{MODEL}");
            assert_eq!(model["parent"], shape, "{kind}: the engine's shape");
            // Its own plate, and the trunk cool.
            let textures = [
                ("plate_top", format!("block/engine_{kind}_plate_top")),
                ("plate_side", format!("block/engine_{kind}_plate_side")),
                ("trunk", format!("block/{}", TRUNKS[0])),
            ];
            for (place, texture) in textures {
                assert_eq!(model["textures"][place], texture, "{kind}: its {place}");
            }
        }
    }
}
