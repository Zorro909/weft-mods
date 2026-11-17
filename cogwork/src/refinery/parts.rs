//! The blocks a refinery column is built of besides its controller: casing, heaters,
//! condensers and taps. Built, the casing shows one framed steel panel a face: its `formed`
//! texture connects across the face (`refinery_panel/refinery_panel.ctm`), and a formed port is
//! that panel with its port drawn on top, so the face stays one.
//!
//! Each port holds liquid: a heater takes steam (`liquid::Kind::Steam`) and holds energy for its
//! burner, a condenser takes water (`Kind::Mill`), and a tap hands on what the column made by
//! height, as a coke oven's drain does (`Kind::Drain`); a bucket fills from it.

use weft_sdk::{Block, BlockBuilder, ENERGY, Registry, Use, World, tool_tier};

use super::column::Part;
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Holders, Vessel, use_bucket};

pub static REFINERY_CASING: Block = Block::named("refinery_casing");
pub static REFINERY_HEATER: Block = Block::named("refinery_heater");
pub static REFINERY_CONDENSER: Block = Block::named("refinery_condenser");
pub static REFINERY_TAP: Block = Block::named("refinery_tap");

/// Millibuckets of steam a heater holds, of water a condenser, and of its product a tap.
pub const STEAM_CAPACITY: u32 = 4_000;
pub const WATER_CAPACITY: u32 = 2_000;
pub const TAP_CAPACITY: u32 = 1_000;
/// Energy a heater's burner holds, and takes in a tick.
const BURNER_HOLDS: i64 = 6_000;
const BURNER_TAKES: i64 = 600;

pub fn register(r: &mut Registry) {
    formable(r.block(&REFINERY_CASING), None);
    formable(r.block(&REFINERY_HEATER), Some("refinery_heater_port")).entity(|entity| {
        let burner = crate::machine::energy(BURNER_HOLDS, BURNER_TAKES);
        entity.attach(&HELD).attach_with(&ENERGY, burner)
    });
    formable(
        r.block(&REFINERY_CONDENSER),
        Some("refinery_condenser_port"),
    )
    .entity(|entity| entity.attach(&HELD));
    formable(r.block(&REFINERY_TAP), Some("refinery_tap_port"))
        .on_use(use_tap)
        .entity(|entity| entity.attach(&HELD));
}

/// What every part shares: as hard as iron, mined with a pickaxe.
pub fn part(builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    builder
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
}

/// A part on the column's shell: plain casing until the column is built, then `formed`, the
/// panel; with its `port` on top if it has one.
fn formable<'r>(builder: BlockBuilder<'r>, port: Option<&'static str>) -> BlockBuilder<'r> {
    let builder = part(builder).prop("formed", ["false", "true"]);
    builder.states(move |props, state| {
        let face = if props.is("formed", "true") {
            "refinery_panel"
        } else {
            "refinery_casing"
        };
        let state = state.texture(face);
        match port {
            Some(port) => state.layer(port),
            None => state,
        }
    })
}

/// The part of a refinery column the block at `pos` is, if any.
pub fn part_at(world: &World, pos: Pos) -> Option<Part> {
    let block = world.block_typed(pos);
    let parts = [
        (&REFINERY_CASING, Part::Casing),
        (&super::REFINERY, Part::Controller),
        (&REFINERY_HEATER, Part::Heater),
        (&REFINERY_CONDENSER, Part::Condenser),
        (&REFINERY_TAP, Part::Tap),
    ];
    parts
        .into_iter()
        .find(|(kind, _)| block.is(kind))
        .map(|(_, part)| part)
}

/// Shows the part at `pos` formed or not, when it is one that shows it.
pub fn show_formed(world: &World, pos: Pos, formed: bool) {
    let value = formed.to_string();
    let formable = !matches!(part_at(world, pos), None | Some(Part::Controller));
    if formable && world.prop(pos, "formed").is_some_and(|now| now != value) {
        world.prop_set(pos, "formed", &value);
    }
}

/// An empty bucket used on a tap fills with what it holds.
fn use_tap(click: &mut Use) -> bool {
    let world = click.world();
    let Some(tap) = Vessel::at(&world, click.pos().into()) else {
        return false;
    };
    let mut holders = Holders::outside(world);
    use_bucket(&mut holders, &tap, click.player())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_parts_have_their_textures() {
        let textures = [
            "refinery_casing",
            "refinery_heater_port",
            "refinery_condenser_port",
            "refinery_tap_port",
            "refinery_front",
            "refinery_front_on",
        ];
        for texture in textures {
            let file = format!("textures/block/{texture}.png");
            assert!(crate::has_asset(&file), "no {file}");
        }
        let panel = "textures/block/refinery_panel/refinery_panel.ctm";
        assert!(crate::has_asset(panel), "no {panel}");
    }
}
