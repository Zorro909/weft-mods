//! The wand: hit a block with it for the selection's first corner, use it on one for the second.
//! The same corners `/structure pos1` and `pos2` set.

use weft_sdk::*;

use crate::selection;

pub static WAND: Item = Item::named("wand");

pub fn register(r: &mut Registry) {
    r.item(&WAND)
        .texture("wand")
        .max_stack(1)
        .tab(&base_api::tabs::TOOLS)
        .on_use(|wand| {
            let Some((pos, _)) = wand.target_block() else {
                return false;
            };
            selection::set_corner(&wand.world(), wand.entity(), 1, pos.into());
            true
        });
    // Hitting a block with the wand marks it instead of breaking it.
    r.before_event(&actions::BREAK_BLOCK, |hit| {
        if hit.entity() == 0 || hit.item() != WAND.id() {
            return Verdict::Abstain;
        }
        selection::set_corner(&hit.world(), hit.entity(), 0, hit.pos().into());
        Verdict::Deny
    });
}
