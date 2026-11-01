//! The sorting tube: a row of sample items per direction, ghosts set by clicking them as on the
//! assembly bench. An item leaves by a direction whose row holds its kind, else by one whose row
//! is empty. Each direction has a color, on its arm and beside its row, so players match them up
//! without a compass.

use weft_sdk::*;

use super::shape::Arms;
use super::{SLOTS, SORTING_TUBE, TUBE, Tube, kind};
use crate::ghost::{self, SLOT};

static SORTING_MENU: Menu = Menu::named("sorting_tube");

/// Samples per direction.
const ROW: usize = 6;

value! {
    /// Each face's row of sample items, 0 where a row has none.
    #[derive(Default)]
    pub struct Samples { pub rows: [[u32; ROW]; 6] }
}
static SAMPLES: Component<Samples> = Component::named("cogwork:sorting_samples");

/// Each face's color and arm texture.
const SIDES: [(&str, &str); 6] = [
    ("green", "tube_sorting_green"),
    ("yellow", "tube_sorting_yellow"),
    ("black", "tube_sorting_black"),
    ("white", "tube_sorting_white"),
    ("red", "tube_sorting_red"),
    ("blue", "tube_sorting_blue"),
];

fn arm_texture(d: Dir) -> &'static str {
    SIDES[d.face() as usize].1
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SAMPLES,
        Samples::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.machine(&SORTING_TUBE, &SORTING_MENU)
        .block(|b| {
            super::shape::tube(b, "tube_sorting", Arms::PerSide(arm_texture)).tab(&crate::TAB)
        })
        // So the carried items are slots 0..8 as in every tube; out only, so nothing gets
        // shift-clicked in.
        .group("transit", Slots::row(SLOTS as u32).output())
        .attach(&SAMPLES)
        .attach_with(
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
        for (face, (color, _)) in SIDES.into_iter().enumerate() {
            ui.row(S::row().gap(6.0), |ui| {
                ui.image(&format!("cogwork:gui/swatch_{color}"), 10.0, 10.0);
                ui.text_styled(S::col().w(36.0), &format!("cogwork.color.{color}"), &[]);
                ui.row(S::row(), |ui| (0..ROW).for_each(|i| sample(ui, face, i)));
            });
        }
        ui.player_inventory(true);
    });
}

/// The ghost cell of sample `i` in the row of `face`.
fn sample(ui: &mut Ui, face: usize, i: usize) {
    let under = |ui: &mut Ui| ui.image("base:gui/slot", SLOT, SLOT);
    ghost::cell(ui, under, (face * ROW + i) as u32, move |c| {
        c.menu_component(&SAMPLES).unwrap_or_default().rows[face][i]
    });
}

/// Sets sample `cell`, counted along the rows, when `owner` is a sorting tube.
pub fn set_sample(w: &World, owner: u64, cell: u32, item: u32) {
    if let Some(s) = w.component_mut(owner, &SAMPLES)
        && let Some(sample) = s.rows.as_flattened_mut().get_mut(cell as usize)
    {
        *sample = item;
    }
}

fn rows(w: &World, id: u64) -> [[u32; ROW]; 6] {
    w.component(id, &SAMPLES).map_or([[0; ROW]; 6], |s| s.rows)
}

/// Whether the row of `face` names the item.
pub fn passes(w: &World, id: u64, face: u32, stack: ItemStack) -> bool {
    rows(w, id)[face as usize].contains(&stack.item)
}

/// The faces whose rows are empty: they take whatever no row names.
pub fn unfiltered(w: &World, id: u64) -> Vec<u32> {
    let rows = rows(w, id);
    (0..6)
        .filter(|&f| rows[f as usize].iter().all(|&item| item == 0))
        .collect()
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
        for (i, (color, texture)) in SIDES.iter().enumerate() {
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
                SIDES[..i].iter().all(|s| s.0 != *color),
                "{color} used once"
            );
        }
    }
}
