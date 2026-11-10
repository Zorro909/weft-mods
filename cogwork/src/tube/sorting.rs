//! The sorting tube: a row of sample items per direction, ghosts set by clicking them as on the
//! assembly bench. An item leaves by a direction whose row holds its kind, else by one whose row
//! is empty. Each direction has a color, on its arm and beside its row, so players match them up
//! without a compass.

use weft_sdk::*;

use super::shape::Arms;
use super::{Kind, SLOTS, SORTING_TUBE, tube_entity};
use crate::ghost::{self, ItemKey, SLOT};

static SORTING_MENU: Menu = Menu::named("sorting_tube");

/// Samples per direction.
pub(crate) const ROW: usize = 6;

value! {
    /// Each face's row of sample items, `ghost::NO_ITEM` where a row has none.
    #[derive(Default)]
    pub struct Samples { pub rows: [[ItemKey; ROW]; 6] }
}

/// Named anew when samples became item keys, so a tube saved before starts without samples
/// rather than reading item numbers as keys.
static SAMPLES: Component<Samples> = Component::named("cogwork:sorting_rows");

/// Each face's color and arm texture.
const SIDES: [(&str, &str); 6] = [
    ("green", "tube_sorting_green"),
    ("yellow", "tube_sorting_yellow"),
    ("black", "tube_sorting_black"),
    ("white", "tube_sorting_white"),
    ("red", "tube_sorting_red"),
    ("blue", "tube_sorting_blue"),
];

pub(crate) fn arm_texture(dir: Dir) -> &'static str {
    SIDES[dir.face() as usize].1
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &SAMPLES,
        Samples::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    super::shape::tube(
        r.block(&SORTING_TUBE),
        "tube_sorting",
        Arms::PerSide(arm_texture),
    )
    .tab(&crate::TAB)
    .entity(|entity| tube_entity(entity.attach(&SAMPLES), Kind::Sorting))
    .opens(&SORTING_MENU);
    r.menu(&SORTING_MENU)
        // So the carried items are slots 0..8 as in every tube; out only, so nothing gets
        // shift-clicked in.
        .group("transit", Slots::row(SLOTS as u32).output().holder())
        .player_inventory()
        .screen(screen);
}

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(3.0), |ui| {
        ui.title();
        for (face, (color, _)) in SIDES.into_iter().enumerate() {
            ui.row(S::row().gap(6.0), |ui| {
                ui.image(&format!("cogwork:gui/swatch_{color}"), 10.0, 10.0);
                ui.text_styled(S::column().w(36.0), &format!("cogwork.color.{color}"), &[]);
                ui.row(S::row(), |ui| (0..ROW).for_each(|i| sample(ui, face, i)));
            });
        }
        ui.player_inventory(true);
    });
}

/// The ghost cell of sample `i` in the row of `face`.
fn sample(ui: &mut Ui, face: usize, i: usize) {
    let under = |ui: &mut Ui| ui.image("base:gui/slot", SLOT, SLOT);
    ghost::cell(ui, under, (face * ROW + i) as u32, move |view| {
        ghost::item_of(view.menu_component(&SAMPLES).unwrap_or_default().rows[face][i])
    });
}

/// Sets sample `cell`, counted along the rows, to `item` (0 clears it) when `owner` is a
/// sorting tube.
pub fn set_sample(world: &World, owner: u64, cell: u32, item: u32) {
    let Some(samples) = world.component_mut(owner, &SAMPLES) else {
        return;
    };
    let Some(sample) = samples.rows.as_flattened_mut().get_mut(cell as usize) else {
        return;
    };
    *sample = ghost::key_of(item);
}

/// Whether the row of side `dir` names the item.
pub fn passes(world: &World, id: u64, dir: Dir, stack: ItemStack) -> bool {
    let Some(samples) = world.component(id, &SAMPLES) else {
        return false;
    };
    samples.rows[dir.face() as usize].contains(&ghost::key_of(stack.item))
}

/// The sides whose rows are empty, or name only items no loaded mod has: they take whatever no
/// row names.
pub fn unfiltered(world: &World, id: u64) -> Vec<Dir> {
    let Some(samples) = world.component(id, &SAMPLES) else {
        return Dir::ALL.to_vec();
    };
    Dir::ALL
        .into_iter()
        .filter(|&dir| {
            samples.rows[dir.face() as usize]
                .iter()
                .all(|&key| ghost::item_of(key) == 0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_side_has_its_own_color_pictures_and_name() {
        for (i, (color, texture)) in SIDES.iter().enumerate() {
            assert!(
                crate::has_asset(&format!("textures/block/{texture}.png")),
                "{color} arm texture"
            );
            assert!(
                crate::has_asset(&format!("textures/gui/swatch_{color}.png")),
                "{color} swatch"
            );
            assert!(
                crate::translated(&format!("cogwork.color.{color}")),
                "{color} name in every lang file"
            );
            assert!(
                SIDES[..i].iter().all(|side| side.0 != *color),
                "{color} used once"
            );
        }
    }
}
