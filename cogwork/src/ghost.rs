//! Ghost cells: places on a machine's screen that show an item without holding it. Clicking one
//! sets it to what the cursor holds, and an empty hand clears it; nothing leaves the player's
//! hands. The assembly bench's pattern and the sorting tube's samples are made of them.
//!
//! Machines keep ghosts by item name: items are numbered anew when mods are added or removed, so
//! a number saved with the world could later name another item.

use weft_sdk::*;

/// Side of a cell in GUI pixels, the same as a slot.
pub const SLOT: f32 = 18.0;
/// How far from a machine a player may set its ghosts, in blocks.
const REACH: f64 = 8.0;

/// An item's full name (`base:stick`) as a component keeps it: UTF-8, padded with zeros. All
/// zeros for no item.
pub type ItemName = [u8; 64];
pub const NO_ITEM: ItemName = [0; 64];

/// The name to keep for `item` (0 for none); `None` if it is too long to keep.
pub fn name_of(item: u32) -> Option<ItemName> {
    if item == 0 {
        return Some(NO_ITEM);
    }
    keep(item_name(item))
}

/// The item `name` names, 0 for none or for one no loaded mod has.
pub fn item_of(name: &ItemName) -> u32 {
    let text = text_of(name);
    if text.is_empty() {
        return 0;
    }
    Item::lookup(text).unwrap_or(0)
}

fn keep(text: &str) -> Option<ItemName> {
    let mut name = NO_ITEM;
    name.get_mut(..text.len())?.copy_from_slice(text.as_bytes());
    Some(name)
}

fn text_of(name: &ItemName) -> &str {
    let len = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    std::str::from_utf8(&name[..len]).unwrap_or("")
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

    #[test]
    fn a_name_is_kept_and_read_back() {
        let name = keep("base:stick").expect("short enough");
        assert_eq!(text_of(&name), "base:stick");
        assert_eq!(text_of(&NO_ITEM), "", "no item");
        let longest = "a".repeat(64);
        assert_eq!(
            text_of(&keep(&longest).expect("fits")),
            longest,
            "all 64 bytes"
        );
        assert_eq!(keep(&"a".repeat(65)), None, "too long to keep");
    }
}
