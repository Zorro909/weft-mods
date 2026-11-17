//! Plastic and what is made of it: pellets from the polymerizer (`polymerizer`), the catalyst it
//! wears down, and the sheets, gears, pipes and composite panels the molding press makes of them
//! (`molding_press`). Each part carries a tag by its form, so other mods' recipes can ask for
//! one: `pellets/plastic`, `plates/plastic`, `gears/plastic`, `pipes/plastic` and
//! `plates/composite`.

use weft_sdk::{Item, Registry, WithComponents};

pub static PLASTIC_PELLET: Item = Item::named("plastic_pellet");
pub static PLASTIC_SHEET: Item = Item::named("plastic_sheet");
pub static PLASTIC_GEAR: Item = Item::named("plastic_gear");
pub static PLASTIC_PIPE: Item = Item::named("plastic_pipe");
pub static COMPOSITE_PANEL: Item = Item::named("composite_panel");
/// What turns the polymerizer's gas into plastic: it wears a use a batch.
pub static CATALYST: Item = Item::named("catalyst");

/// Batches of plastic one catalyst lasts.
pub const CATALYST_USES: u32 = 64;

/// The parts and the tag each carries for its form.
const PARTS: [(&Item, &str); 5] = [
    (&PLASTIC_PELLET, "pellets/plastic"),
    (&PLASTIC_SHEET, "plates/plastic"),
    (&PLASTIC_GEAR, "gears/plastic"),
    (&PLASTIC_PIPE, "pipes/plastic"),
    (&COMPOSITE_PANEL, "plates/composite"),
];

pub fn register(r: &mut Registry) {
    for (part, tag) in PARTS {
        r.item(part).tag(tag).tab(&crate::TAB);
    }
    r.item(&CATALYST).durability(CATALYST_USES).tab(&crate::TAB);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_part_has_its_name_and_picture() {
        let items = PARTS.iter().map(|(item, _)| *item).chain([&CATALYST]);
        for item in items {
            let name = item.path();
            assert!(crate::translated(&format!("item.cogwork.{name}")), "{name}");
            let icon = format!("textures/item/{name}.png");
            assert!(crate::has_asset(&icon), "no {icon}");
        }
    }
}
