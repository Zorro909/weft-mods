//! One item on its way through a tube: in by a face, to the hub in the middle, and out by the
//! face it chooses there.

use weft_sdk::*;

/// The face of an item put in at the hub instead of through a face, and of one that has yet to
/// choose the face it leaves by.
pub(super) const NONE: u32 = 6;
/// Where along its way an item is at the hub, and at the face it leaves by.
pub const HUB: f32 = 0.5;
const FAR_FACE: f32 = 1.0;

/// Blocks per tick an item slows down to in any tube but a gilded one.
pub const BASE_SPEED: f32 = 0.06;
/// Blocks per tick a gilded tube speeds items up to.
pub const GILDED_SPEED: f32 = 0.3;

value! {
    /// One item on its way through a tube; its stack is in the tube's slot of the same index.
    /// `pos` runs from 0 at the face it came in by through 0.5 at the hub to 1 at the face it
    /// leaves by. A slot with nothing on its way has `item` 0.
    #[derive(Default)]
    pub struct Motion { pub item: u32, pub visual: u64, pub pos: f32, pub speed: f32, pub from: u32, pub to: u32 }
}

impl Motion {
    /// `item` coming in by the face `by`.
    pub fn entering(item: u32, by: Dir) -> Motion {
        Motion {
            pos: 0.0,
            from: by.face(),
            ..Motion::at_hub(item)
        }
    }

    /// `item` put in at the hub, by a machine or another mod.
    pub fn at_hub(item: u32) -> Motion {
        Motion {
            item,
            visual: 0,
            pos: HUB,
            speed: BASE_SPEED,
            from: NONE,
            to: NONE,
        }
    }

    /// The same item gone on into the next tube by that tube's face `by`: as fast, and with the
    /// same picture.
    pub fn handed_on(self, by: Dir) -> Motion {
        Motion {
            pos: (self.pos - FAR_FACE).max(0.0),
            from: by.face(),
            to: NONE,
            ..self
        }
    }

    /// Whether nothing is on its way here.
    pub fn is_none(&self) -> bool {
        self.item == 0
    }

    /// The face it came in by; `None` when it was put in at the hub.
    pub fn came_by(&self) -> Option<Dir> {
        side(self.from)
    }

    /// The face it leaves by; `None` while it has yet to choose one.
    pub fn leaves_by(&self) -> Option<Dir> {
        side(self.to)
    }

    /// Sends it on from the hub out by `side`.
    pub fn leave_by(&mut self, side: Dir) {
        self.to = side.face();
    }

    /// Sends it back from the face it could not leave by, to choose again at the hub.
    pub fn turn_back(&mut self) {
        (self.from, self.to, self.pos) = (self.to, NONE, 0.0);
    }

    /// Whether it has come as far as the hub.
    pub fn reached_hub(&self) -> bool {
        self.pos >= HUB
    }

    /// Whether it has come to the face it leaves by.
    pub fn at_far_face(&self) -> bool {
        self.pos >= FAR_FACE
    }

    /// Moves it a tick along: as far as the hub, and on to the far face once it has chosen one.
    pub fn step(&mut self) {
        let chosen = self.leaves_by().is_some();
        if self.pos < HUB || chosen {
            let stop = if chosen { FAR_FACE } else { HUB };
            self.pos = (self.pos + self.speed).min(stop);
        }
    }
}

/// The side a saved face number stands for; `None` for `NONE`.
fn side(face: u32) -> Option<Dir> {
    Dir::ALL.get(face as usize).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_face_reads_as_its_side() {
        let motion = Motion::entering(1, Dir::North);
        assert_eq!(motion.came_by(), Some(Dir::North), "the face it came by");
        assert_eq!(motion.leaves_by(), None, "no side chosen yet");
        assert_eq!(Motion::at_hub(1).came_by(), None, "put in at the hub");
    }

    #[test]
    fn an_item_waits_at_the_hub_until_it_has_a_way_out() {
        let mut motion = Motion::entering(1, Dir::West);
        for _ in 0..20 {
            motion.step();
        }
        assert_eq!(motion.pos, HUB, "no further than the hub");
        assert!(
            motion.reached_hub() && !motion.at_far_face(),
            "waiting there"
        );
        motion.leave_by(Dir::Up);
        for _ in 0..20 {
            motion.step();
        }
        assert!(motion.at_far_face(), "on to the face it chose");
    }

    #[test]
    fn an_item_keeps_its_speed_and_picture_into_the_next_tube() {
        let mut motion = Motion {
            visual: 7,
            speed: 0.3,
            pos: 1.0,
            ..Motion::entering(1, Dir::West)
        };
        motion.leave_by(Dir::East);
        let next = motion.handed_on(Dir::West);
        assert_eq!((next.visual, next.speed), (7, 0.3), "as it was");
        assert_eq!(next.came_by(), Some(Dir::West), "in by the facing side");
        assert_eq!((next.leaves_by(), next.pos), (None, 0.0), "at the start");
    }

    #[test]
    fn an_item_turned_back_goes_to_the_hub_again() {
        let mut motion = Motion::entering(1, Dir::West);
        motion.leave_by(Dir::East);
        motion.turn_back();
        assert_eq!(motion.came_by(), Some(Dir::East), "from where it stood");
        assert_eq!(
            (motion.leaves_by(), motion.pos),
            (None, 0.0),
            "to choose again"
        );
    }
}
