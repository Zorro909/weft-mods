//! Item tubes: blocks that carry items between machines, every item drawn as it moves inside.
//! A tube keeps the stacks it carries in its own slots, so breaking one spills exactly them,
//! and its `Tube` component holds where each of them is on its way.

mod extract;
mod overflow;
mod route;
mod shape;
pub(crate) mod sorting;
mod travel;
mod visual;

pub use extract::inject;
use shape::Arms;
pub use shape::{connects_to_tube, turn_directing};
pub use sorting::set_sample;
use weft_sdk::*;

use crate::lib::{BlockState, Pos};

/// Items one tube carries at once.
pub const SLOTS: usize = 8;
/// The face of an item put in at the hub instead of through a face.
pub const NONE: u32 = 6;

/// Blocks per tick an item slows down to in any tube but a gilded one.
pub const BASE_SPEED: f32 = 0.06;
/// Blocks per tick a gilded tube speeds items up to.
pub const GILDED_SPEED: f32 = 0.3;

value! {
    /// One item on its way through a tube; its stack is in the tube's slot of the same index.
    /// `pos` runs from 0 at the face it came in by through 0.5 at the hub to 1 at the face it
    /// leaves by.
    #[derive(Default)]
    pub struct Motion { pub item: u32, pub visual: u64, pub pos: f32, pub speed: f32, pub from: u32, pub to: u32 }
}

impl Motion {
    /// The face it came in by; `None` when it was put in at the hub.
    pub fn came_by(&self) -> Option<Dir> {
        side(self.from)
    }

    /// The face it leaves by; `None` while it has yet to choose one.
    pub fn leaves_by(&self) -> Option<Dir> {
        side(self.to)
    }
}

/// The side a saved face number stands for; `None` for `NONE`.
fn side(face: u32) -> Option<Dir> {
    Dir::ALL.get(face as usize).copied()
}

value! {
    /// A tube's state: its kind, a countdown its kind may use (the timber tube's pulls), the
    /// items in it, and how long it has been full without passing anything on. New fields go
    /// last: a saved tube loads with them 0.
    #[derive(Default)]
    pub struct Tube { pub kind: u32, pub timer: u32, pub moving: [Motion; SLOTS], pub stuck: u32 }
}

pub static TUBE: Component<Tube> = Component::named("cogwork:tube");
/// Every tube block, whatever its kind.
pub static TUBES: Tag = Tag::named("cogwork:tubes");

pub mod kind {
    pub const STONE: u32 = 0;
    pub const TIMBER: u32 = 1;
    pub const GILDED: u32 = 2;
    pub const SORTING: u32 = 3;
    pub const DIRECTING: u32 = 4;
    pub const VOID: u32 = 5;
}

pub static STONE_TUBE: Block = Block::named("stone_tube");
pub static TIMBER_TUBE: Block = Block::named("timber_tube");
pub static GILDED_TUBE: Block = Block::named("gilded_tube");
pub static SORTING_TUBE: Block = Block::named("sorting_tube");
pub static DIRECTING_TUBE: Block = Block::named("directing_tube");
pub static VOID_TUBE: Block = Block::named("void_tube");

/// The tubes without a screen: block, kind and texture.
static PLAIN: [(&Block, u32, &str); 4] = [
    (&STONE_TUBE, kind::STONE, "tube_stone"),
    (&TIMBER_TUBE, kind::TIMBER, "tube_timber"),
    (&GILDED_TUBE, kind::GILDED, "tube_gilded"),
    (&VOID_TUBE, kind::VOID, "tube_void"),
];

pub fn register(r: &mut Registry) {
    r.entity_component(&TUBE, Tube::default(), component_flags::SAVE);
    r.system(&TUBE, travel::tick_tubes);
    for (block, kind, texture) in PLAIN {
        shape::tube(r.block(block), texture, Arms::Same)
            .tab(&crate::TAB)
            .entity(move |e| tube_entity(e, kind));
    }
    shape::tube(
        r.block(&DIRECTING_TUBE),
        "tube_directing",
        Arms::Output("tube_directing_out"),
    )
    .on_use(shape::turn_output)
    .tab(&crate::TAB)
    .entity(|e| tube_entity(e, kind::DIRECTING));
    sorting::register(r);
    visual::register(r);
}

/// A tube's block entity: its `Tube` state and slots for the items it carries.
fn tube_entity(e: EntityBuilder<'_>, kind: u32) -> EntityBuilder<'_> {
    let e = e
        .attach_with(
            &TUBE,
            Tube {
                kind,
                ..Tube::default()
            },
        )
        .inventory(SLOTS as u32);
    if kind != self::kind::TIMBER {
        return e;
    }
    // Engines drive the timber tube's pulls.
    e.attach_with(
        &ENERGY,
        Energy {
            stored: 0,
            capacity: 2_000,
            max_in: 200,
            max_out: 0,
        },
    )
}

/// Whether the block at `pos` is a tube. Asked of its block, which is cheaper than reading
/// its `Tube`.
pub fn is_tube(w: &World, pos: Pos) -> bool {
    BlockState::at(w, pos).has(&TUBES)
}

/// A slot of the tube `at` (its entity, or its block) that neither holds a stack nor has an
/// item on its way.
fn free_slot(w: &World, at: impl Into<Holder>, tube: &Tube) -> Option<usize> {
    let inv = w.inventory(at);
    (0..SLOTS).find(|&k| inv.get(k as u32).is_empty() && tube.moving[k].item == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_face_reads_as_its_side() {
        let m = Motion {
            from: Dir::North.face(),
            to: NONE,
            ..Motion::default()
        };
        assert_eq!(m.came_by(), Some(Dir::North), "the face it came by");
        assert_eq!(m.leaves_by(), None, "no side chosen yet");
    }
}
