//! An engine's block: a base plate on the side it was placed against and a trunk pointing the
//! other way that glows with its heat. Both are the Blockbench model `models/block/engine.json`,
//! lying on the floor with the trunk up. The piston riding the trunk is an entity (`piston`).

use weft_sdk::*;

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
        .model_file(MODEL)
        .facing(Facings::ALL, Toward::Face)
        .prop("heat", 0..=3)
        .sound_group("base:block.stone")
        .states(move |props, state| look(props, state, kind))
        .on_use(use_wrench)
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

fn use_wrench(click: &mut Use) -> bool {
    let world = click.world();
    if !crate::cogs::holds_wrench(&world, click.player()) {
        return false;
    }
    turn(&world, Pos::from(click.pos()));
    true
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

    #[test]
    fn the_engines_have_their_model_and_its_textures() {
        let model = format!("models/block/{MODEL}.json");
        assert!(crate::has_asset(&model), "no {model}");
        let plates = ["spring", "ember"]
            .into_iter()
            .flat_map(|kind| ["top", "side"].map(|face| format!("engine_{kind}_plate_{face}")));
        for texture in plates.chain(TRUNKS.map(String::from)) {
            let file = format!("textures/block/{texture}.png");
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
