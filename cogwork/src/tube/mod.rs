//! Item tubes: blocks that carry items between machines, every item drawn as it moves inside.
//! A tube keeps the stacks it carries in its own slots, so breaking one spills exactly them,
//! and its `Tube` component holds where each of them is on its way.

mod extract;
mod motion;
mod overflow;
mod route;
mod shape;
pub(crate) mod sorting;
mod travel;
mod visual;

pub use extract::inject;
use motion::Motion;
use shape::Arms;
pub use shape::{connects_to_tube, turn_directing};
pub use sorting::set_sample;
use strum::FromRepr;
use weft_sdk::*;

use crate::lib::{Pos, TypedWorld};

/// Items one tube carries at once.
pub const SLOTS: usize = 8;

value! {
    /// A tube's state: its kind (`Kind`), a countdown its kind may use (the timber tube's pulls),
    /// the items in it, and how long it has been full without passing anything on. New fields go
    /// last: a saved tube loads with them 0.
    #[derive(Default)]
    pub struct Tube { pub kind: u32, pub timer: u32, pub moving: [Motion; SLOTS], pub stuck: u32 }
}

pub static TUBE: Component<Tube> = Component::named("cogwork:tube");
/// Every tube block, whatever its kind.
pub static TUBES: Tag = Tag::named("cogwork:tubes");

/// What a tube does with the items it carries. Kept in `Tube::kind` as its number, so the order
/// is part of the save format.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr)]
#[repr(u32)]
pub enum Kind {
    /// Passes items on whichever way they can go.
    #[default]
    Stone,
    /// Pulls items out of the machines it touches, with engine power.
    Timber,
    /// Speeds items up.
    Gilded,
    /// Sends each item the way its samples say.
    Sorting,
    /// Sends every item out by one arm.
    Directing,
    /// Destroys what reaches its hub.
    Void,
}

impl Tube {
    /// An empty tube of `kind`.
    fn of(kind: Kind) -> Tube {
        Tube {
            kind: kind as u32,
            ..Tube::default()
        }
    }

    pub fn kind(&self) -> Kind {
        Kind::from_repr(self.kind).unwrap_or_default()
    }
}

pub static STONE_TUBE: Block = Block::named("stone_tube");
pub static TIMBER_TUBE: Block = Block::named("timber_tube");
pub static GILDED_TUBE: Block = Block::named("gilded_tube");
pub static SORTING_TUBE: Block = Block::named("sorting_tube");
pub static DIRECTING_TUBE: Block = Block::named("directing_tube");
pub static VOID_TUBE: Block = Block::named("void_tube");

/// The tubes without a screen: block, kind and texture.
static PLAIN: [(&Block, Kind, &str); 4] = [
    (&STONE_TUBE, Kind::Stone, "tube_stone"),
    (&TIMBER_TUBE, Kind::Timber, "tube_timber"),
    (&GILDED_TUBE, Kind::Gilded, "tube_gilded"),
    (&VOID_TUBE, Kind::Void, "tube_void"),
];

pub fn register(r: &mut Registry) {
    r.entity_component(&TUBE, Tube::default(), component_flags::SAVE);
    r.system(&TUBE, travel::tick_tubes);
    for (block, kind, texture) in PLAIN {
        shape::tube(r.block(block), texture, Arms::Same)
            .tab(&crate::TAB)
            .entity(move |entity| tube_entity(entity, kind));
    }
    shape::tube(
        r.block(&DIRECTING_TUBE),
        "tube_directing",
        Arms::Output("tube_directing_out"),
    )
    .on_use(shape::turn_output)
    .tab(&crate::TAB)
    .entity(|entity| tube_entity(entity, Kind::Directing));
    sorting::register(r);
    visual::register(r);
}

/// A tube's block entity: its `Tube` state and slots for the items it carries.
fn tube_entity(builder: EntityBuilder<'_>, kind: Kind) -> EntityBuilder<'_> {
    let builder = builder
        .attach_with(&TUBE, Tube::of(kind))
        .inventory(SLOTS as u32);
    if kind != Kind::Timber {
        return builder;
    }
    // Engines drive the timber tube's pulls.
    builder.attach_with(&ENERGY, crate::machine::energy(2_000, 200))
}

/// Whether the block at `pos` is a tube. Asked of its block, which is cheaper than reading
/// its `Tube`.
pub fn is_tube(world: &World, pos: Pos) -> bool {
    world.block_typed(pos).has(&TUBES)
}

/// A slot of the tube `at` (its entity, or its block) that neither holds a stack nor has an
/// item on its way.
fn free_slot(world: &World, at: impl Into<Holder>, tube: &Tube) -> Option<usize> {
    let inventory = world.inventory(at);
    (0..SLOTS).find(|&slot| inventory.get(slot as u32).is_empty() && tube.moving[slot].is_none())
}

/// Empties `slot` of the tube `id`: its stack, the item's picture, and where it was on its way.
fn clear_slot(world: &World, id: u64, tube: &mut Tube, slot: usize) {
    world.inventory(id).set(slot as u32, ItemStack::EMPTY);
    visual::remove(world, tube.moving[slot].visual);
    tube.moving[slot] = Motion::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_saved_kind_is_a_stone_tube() {
        let tube = Tube {
            kind: 99,
            ..Tube::default()
        };
        assert_eq!(tube.kind(), Kind::Stone, "a kind from a newer cogwork");
        assert_eq!(Tube::of(Kind::Void).kind(), Kind::Void, "its own kind");
    }
}
