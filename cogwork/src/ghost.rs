//! Ghost cells: places on a machine's screen that show an item without holding it. Clicking one
//! sets it to what the cursor holds, and an empty hand clears it; nothing leaves the player's
//! hands. The assembly bench's pattern and the sorting tube's samples are made of them.
//!
//! Machines keep ghosts by a hash of the item's name: items are numbered anew when mods are
//! added or removed, so a number saved with the world could later name another item, and a
//! hash keeps a name of any length in 8 bytes.

use std::collections::HashMap;
use std::sync::OnceLock;

use twox_hash::XxHash3_64;
use weft_sdk::*;

/// Side of a cell in GUI pixels, the same as a slot.
pub const SLOT: f32 = 18.0;
/// How far from a machine a player may set its ghosts, in blocks.
const REACH: f64 = 8.0;

/// An item as a machine keeps it: the hash of its full name (`base:stick`), `NO_ITEM` for none.
pub type ItemKey = u64;
pub const NO_ITEM: ItemKey = 0;

/// The key to keep for `item` (0 for none).
pub fn key_of(item: u32) -> ItemKey {
    if item == 0 {
        return NO_ITEM;
    }
    hash(item_name(item))
}

/// The item `key` stands for, 0 for none or for one no loaded mod has.
pub fn item_of(key: ItemKey) -> u32 {
    items_by_key().get(&key).copied().unwrap_or(0)
}

/// XXH3, whose values are fixed by its specification, so keys saved with a world stay right.
fn hash(name: &str) -> ItemKey {
    // 0 is no item.
    XxHash3_64::oneshot(name.as_bytes()).max(1)
}

/// Every loaded item by its key, made the first time it is needed: the items don't change while
/// the game runs.
fn items_by_key() -> &'static HashMap<ItemKey, u32> {
    static TABLE: OnceLock<HashMap<ItemKey, u32>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = HashMap::new();
        let names = (1..).map(|item| (item, item_name(item)));
        for (item, name) in names.take_while(|(_, name)| !name.is_empty()) {
            if let Some(other) = table.insert(hash(name), item) {
                let other = item_name(other);
                error!("cogwork: {other} and {name} share a ghost key; ghosts show {name}");
            }
        }
        table
    })
}

value! {
    /// The ghost of `cell` of the machine `owner` is to be `item` (0 clears it).
    pub struct SetGhost {
        pub owner: u64,
        pub cell: u32,
        pub item: u32,
    }
}
static SET_GHOST: Channel<SetGhost> = Channel::named("set_ghost");

pub fn register(r: &mut Registry) {
    r.packet(&SET_GHOST, set);
}

/// One cell, three parts on top of each other: `under` (a slot, or a picture of one), a button
/// over it that sets the ghost, and the ghost `shown` names (0 for none), drawn smaller. A click
/// reaches the button through the ghost.
pub fn cell(
    ui: &mut Ui,
    under: impl FnOnce(&mut Ui),
    cell: u32,
    shown: impl Fn(&ClientView) -> u32 + Send + Sync + 'static,
) {
    ui.row(S::row().gap(-SLOT), |ui| {
        under(ui);
        let size = S::default().class("cogwork:ghost_cell").size(SLOT, SLOT);
        ui.button("", size, move |c| click(c, cell));
        // Text pixels are half a GUI pixel: 1.5 draws the ghost 12 wide, 3 in from the edges.
        ui.column(S::col().size(SLOT, SLOT).pad(3.0), |ui| {
            ui.text_bind_styled(S::col().scale(1.5), move |c, out| {
                let ghost = match shown(c) {
                    0 => Arg::Text(""),
                    item => Arg::Item(item),
                };
                out.set("cogwork.ghost", &[ghost]);
            });
        });
    });
}

/// Client side of a click on `cell`: sends what is on the cursor.
fn click(c: &ClientView, cell: u32) {
    let Some(owner) = c.menu_owner() else {
        return;
    };
    let client = c.client();
    let item = client.cursor_stack().item;
    client.send(&SET_GHOST, &SetGhost { owner, cell, item });
}

/// Server side: sets the ghost when the player is at the machine, whichever kind it is.
fn set(p: &Packet, g: SetGhost) {
    let w = p.world();
    let (Some(at), Some(me)) = (w.entity(g.owner), w.entity(p.player())) else {
        return;
    };
    let near = (0..3).map(|i| (at.pos[i] - me.pos[i]).powi(2)).sum::<f64>() <= REACH * REACH;
    let known = g.item == 0 || !item_name(g.item).is_empty();
    if near && known {
        crate::bench::set_ghost(&w, g.owner, g.cell, g.item);
        crate::tube::set_sample(&w, g.owner, g.cell, g.item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// XXH3 of `base:stick`.
    const STICK_KEY: ItemKey = 0x64bb_9627_730c_0234;

    #[test]
    fn the_keys_never_change() {
        // Ghosts saved with worlds hold these: a change would empty them all.
        assert_eq!(hash("base:stick"), STICK_KEY, "base:stick");
        assert_ne!(
            hash("cogwork:stone_tube"),
            hash("cogwork:stone_cog"),
            "like names differ"
        );
    }
}
