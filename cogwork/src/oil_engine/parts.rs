//! The blocks an oil engine is built of besides its controller: casing, the combustion chamber,
//! intakes and drives. Casing shows one framed panel a face once the engine is built: its
//! `formed` texture connects across the face (`oil_engine_panel/oil_engine_panel.ctm`), and a
//! formed intake or drive is that panel with its port drawn on top, so the face stays one. An
//! intake is a holder of liquid that takes only oil and water (`liquid::Kind::Intake`); a drive
//! holds a little energy the controller fills, for the block it faces and torque shafts.

use weft_sdk::{
    Block, BlockBuilder, ENERGY, Energy, Props, Registry, State, Use, World, tool_tier,
};

use super::fuel::gauge;
use super::pattern::Part;
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Holders, Vessel, use_bucket};

pub static OIL_ENGINE_CASING: Block = Block::named("oil_engine_casing");
pub static COMBUSTION_CHAMBER: Block = Block::named("combustion_chamber");
pub static OIL_INTAKE: Block = Block::named("oil_intake");
pub static OIL_ENGINE_DRIVE: Block = Block::named("oil_engine_drive");

/// Millibuckets an intake holds until the controller takes them.
pub const INTAKE_CAPACITY: u32 = 1000;
/// Energy a drive holds, and the most it gives a tick: to the block it faces and to energy
/// networks, each.
pub const DRIVE_HOLDS: i64 = 1000;

pub fn register(r: &mut Registry) {
    formable(r.block(&OIL_ENGINE_CASING), "oil_engine_casing", None);
    part(r.block(&COMBUSTION_CHAMBER)).texture("combustion_chamber");
    formable(r.block(&OIL_INTAKE), "oil_intake", Some("oil_intake_port"))
        .on_use(use_intake)
        .entity(|entity| entity.attach(&HELD));
    formable(
        r.block(&OIL_ENGINE_DRIVE),
        "oil_engine_drive",
        Some("oil_engine_drive_port"),
    )
    .entity(|entity| {
        entity.attach_with(
            &ENERGY,
            Energy {
                capacity: DRIVE_HOLDS,
                max_out: DRIVE_HOLDS,
                ..Energy::default()
            },
        )
    });
}

/// What every part shares: as hard as iron, mined with a pickaxe.
fn part(builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    builder
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// A part on the engine's shell: `plain` until the engine is built, then `formed`, the panel
/// with its `port` on top if it has one.
fn formable<'r>(
    builder: BlockBuilder<'r>,
    plain: &'static str,
    port: Option<&'static str>,
) -> BlockBuilder<'r> {
    let builder = part(builder).prop("formed", ["false", "true"]);
    builder.states(move |props, state| {
        if !props.is("formed", "true") {
            return state.texture(plain);
        }
        let panel = state.texture("oil_engine_panel");
        match port {
            Some(port) => panel.layer(port),
            None => panel,
        }
    })
}

/// The controller's look: its gauges at their steps and its lamp lit while it runs. Its front
/// faces north: `facing` turns it.
pub fn controller_look<'s>(props: &Props, state: State<'s>) -> State<'s> {
    let lamp = if props.is("working", "true") {
        "oil_engine_lamp_on"
    } else {
        "oil_engine_lamp"
    };
    let step = |liquid: &str| props.int(liquid).min(super::fuel::GAUGE_STEPS);
    state
        .texture("oil_engine_casing")
        .model_texture("oil", &format!("oil_engine_gauge_oil_{}", step("oil")))
        .model_texture(
            "water",
            &format!("oil_engine_gauge_water_{}", step("water")),
        )
        .model_texture("lamp", lamp)
        .hitbox([0.0; 3], [16.0; 3])
}

/// The part of an oil engine the block at `pos` is, if any.
pub fn part_at(world: &World, pos: Pos) -> Option<Part> {
    let block = world.block_typed(pos);
    let parts = [
        (&OIL_ENGINE_CASING, Part::Casing),
        (&COMBUSTION_CHAMBER, Part::Chamber),
        (&super::OIL_ENGINE_CONTROLLER, Part::Controller),
        (&OIL_INTAKE, Part::Intake),
        (&OIL_ENGINE_DRIVE, Part::Drive),
    ];
    parts
        .into_iter()
        .find(|(kind, _)| block.is(kind))
        .map(|(_, part)| part)
}

/// Shows on the controller at `pos` how much oil and water it holds and whether it runs.
pub fn show_controller(world: &World, pos: Pos, oil: u32, water: u32, working: bool) {
    crate::machine::show_working(world, pos, working);
    let gauges = [("oil", gauge(oil)), ("water", gauge(water))];
    for (prop, step) in gauges {
        let value = step.to_string();
        if world.prop(pos, prop).is_some_and(|now| now != value) {
            world.prop_set(pos, prop, &value);
        }
    }
}

/// Shows the part at `pos` formed or not, when it is casing, an intake or a drive.
pub fn show_formed(world: &World, pos: Pos, formed: bool) {
    let value = formed.to_string();
    let formable = matches!(
        part_at(world, pos),
        Some(Part::Casing | Part::Intake | Part::Drive)
    );
    if formable && world.prop(pos, "formed").is_some_and(|now| now != value) {
        world.prop_set(pos, "formed", &value);
    }
}

/// A bucket of oil or water used on an intake pours into it, an empty one fills from it.
fn use_intake(click: &mut Use) -> bool {
    let world = click.world();
    let Some(intake) = Vessel::at(&world, click.pos().into()) else {
        return false;
    };
    let mut holders = Holders::outside(world);
    use_bucket(&mut holders, &intake, click.player())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_parts_have_their_textures_and_model() {
        let textures = [
            "oil_engine_casing",
            "combustion_chamber",
            "oil_intake",
            "oil_intake_port",
            "oil_engine_drive",
            "oil_engine_drive_port",
            "oil_engine_lamp",
            "oil_engine_lamp_on",
            "oil_engine_controller_front",
        ];
        for texture in textures {
            let file = format!("textures/block/{texture}.png");
            assert!(crate::has_asset(&file), "no {file}");
        }
        for liquid in ["oil", "water"] {
            for step in 0..=super::super::fuel::GAUGE_STEPS {
                let file = format!("textures/block/oil_engine_gauge_{liquid}_{step}.png");
                assert!(crate::has_asset(&file), "no {file}");
            }
        }
        let panel = "textures/block/oil_engine_panel/oil_engine_panel.ctm";
        assert!(crate::has_asset(panel), "no {panel}");
        let model = "models/block/oil_engine_controller.json";
        assert!(crate::has_asset(model), "no {model}");
    }
}
