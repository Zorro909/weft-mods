//! The sorting tube: a row of sample items per direction. An item leaves by a direction whose
//! row holds its kind, else by one whose row is empty.

use weft_sdk::*;

use super::{SLOTS, SORTING_TUBE, TUBE, Tube, kind};

static SORTING_MENU: Menu = Menu::named("sorting_tube");

/// Samples per direction.
const ROW: u32 = 6;

/// The filter group of each face.
fn group(face: u32) -> &'static str {
    [
        "filter_west",
        "filter_east",
        "filter_down",
        "filter_up",
        "filter_north",
        "filter_south",
    ][face as usize]
}

pub fn register(r: &mut Registry) {
    let mut m = r
        .machine(&SORTING_TUBE, &SORTING_MENU)
        .block(|b| super::shape::tube(b, "tube_sorting", None).tab(&crate::TAB))
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
        for face in 0..6 {
            ui.row(S::row().gap(6.0), |ui| {
                ui.text_styled(
                    S::col().w(44.0),
                    &format!("cogwork.side.{}", Dir::from_face(face).name()),
                    &[],
                );
                ui.group(group(face));
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
