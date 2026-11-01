//! The sorting tube: a row of sample items per direction. An item leaves by a direction whose
//! row holds its kind, else by one whose row is empty. Each direction has a color, on its arm
//! and beside its row, so players match them up without a compass.

use weft_sdk::*;

use super::shape::Arms;
use super::{SLOTS, SORTING_TUBE, TUBE, Tube, kind};

static SORTING_MENU: Menu = Menu::named("sorting_tube");

/// Samples per direction.
const ROW: u32 = 6;

/// Each face's filter group, color and arm texture. The groups keep their compass names
/// because saved tubes hold their samples under them.
const SIDES: [(&str, &str, &str); 6] = [
    ("filter_west", "green", "tube_sorting_green"),
    ("filter_east", "yellow", "tube_sorting_yellow"),
    ("filter_down", "black", "tube_sorting_black"),
    ("filter_up", "white", "tube_sorting_white"),
    ("filter_north", "red", "tube_sorting_red"),
    ("filter_south", "blue", "tube_sorting_blue"),
];

fn group(face: u32) -> &'static str {
    SIDES[face as usize].0
}

fn arm_texture(d: Dir) -> &'static str {
    SIDES[d.face() as usize].2
}

pub fn register(r: &mut Registry) {
    let mut m = r
        .machine(&SORTING_TUBE, &SORTING_MENU)
        .block(|b| {
            super::shape::tube(b, "tube_sorting", Arms::PerSide(arm_texture)).tab(&crate::TAB)
        })
        // First, so the carried items are slots 0..8 as in every tube; out only, so nothing
        // gets shift-clicked in.
        .group("transit", Slots::row(SLOTS as u32).output());
    for face in 0..6 {
        m = m.group(group(face), Slots::row(ROW).limit(1));
    }
    m.attach_with(
        &TUBE,
        Tube {
            kind: kind::SORTING,
            ..Tube::default()
        },
    )
    .screen(screen);
}

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(3.0), |ui| {
        ui.title();
        for (group, color, _) in SIDES {
            ui.row(S::row().gap(6.0), |ui| {
                ui.image(&format!("cogwork:gui/swatch_{color}"), 10.0, 10.0);
                ui.text_styled(S::col().w(36.0), &format!("cogwork.color.{color}"), &[]);
                ui.group(group);
            });
        }
        ui.player_inventory(true);
    });
}

fn row(w: &World, id: u64, face: u32) -> Vec<ItemStack> {
    let Some(g) = w.inventory(id).find_group(group(face)) else {
        return Vec::new();
    };
    (0..g.len())
        .map(|i| g.get(i))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Whether the row of `face` names the item.
pub fn passes(w: &World, id: u64, face: u32, stack: ItemStack) -> bool {
    row(w, id, face).iter().any(|s| s.item == stack.item)
}

/// The faces whose rows are empty: they take whatever no row names.
pub fn unfiltered(w: &World, id: u64) -> Vec<u32> {
    (0..6).filter(|&f| row(w, id, f).is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(path: &str) -> bool {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join(path)
            .exists()
    }

    #[test]
    fn every_side_has_its_own_color_pictures_and_name() {
        let lang = include_str!("../../assets/lang/en_us.lang");
        for (i, (_, color, texture)) in SIDES.iter().enumerate() {
            assert!(
                asset(&format!("textures/block/{texture}.png")),
                "{color} arm texture"
            );
            assert!(
                asset(&format!("textures/gui/swatch_{color}.png")),
                "{color} swatch"
            );
            assert!(
                lang.contains(&format!("cogwork.color.{color} =")),
                "{color} name"
            );
            assert!(
                SIDES[..i].iter().all(|s| s.1 != *color),
                "{color} used once"
            );
        }
    }
}
