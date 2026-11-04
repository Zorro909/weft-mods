//! A film as bytes, the way `TIMELINE` carries it: the version, the film's name and origin, then
//! its steps. Numbers are little-endian, counts a `u32`, texts a `u16` length and UTF-8, and every
//! step, path, look and ease starts with a tag byte. The showcase mod reads it back
//! (`showcase/src/timeline.rs`); a change here is a change there, and a new `TIMELINE_VERSION`.

use crate::Pos;
use crate::film::{Ease, Look, Path, Shot, Step};

/// Raised whenever the layout changes; films of any other version are refused.
pub const TIMELINE_VERSION: u16 = 1;

/// A film as the player's client receives it: its full name (`mymod:tour`), the block its
/// positions are counted from, and its steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub film: String,
    pub origin: Pos,
    pub steps: Vec<Step>,
}

impl Timeline {
    /// The bytes `TIMELINE` carries.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Writer(Vec::new());
        out.0.extend(TIMELINE_VERSION.to_le_bytes());
        out.text(&self.film);
        out.block(self.origin);
        out.count(self.steps.len());
        self.steps.iter().for_each(|step| out.step(step));
        out.0
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn tag(&mut self, tag: u8) {
        self.0.push(tag);
    }

    fn count(&mut self, count: usize) {
        self.0.extend((count as u32).to_le_bytes());
    }

    fn f32(&mut self, v: f32) {
        self.0.extend(v.to_le_bytes());
    }

    fn pair(&mut self, pair: [f64; 2]) {
        pair.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    fn point(&mut self, point: [f64; 3]) {
        point.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    fn block(&mut self, block: Pos) {
        <[i32; 3]>::from(block)
            .iter()
            .for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    /// Cut short at 65535 bytes, on a character.
    fn text(&mut self, text: &str) {
        let text = &text[..text.floor_char_boundary(u16::MAX as usize)];
        self.0.extend((text.len() as u16).to_le_bytes());
        self.0.extend(text.as_bytes());
    }

    fn step(&mut self, step: &Step) {
        match step {
            Step::Shot(shot) => {
                self.tag(0);
                self.shot(shot);
            }
            Step::Wait(secs) => {
                self.tag(1);
                self.f32(*secs);
            }
            Step::WaitFor { name, max } => {
                self.tag(2);
                self.text(name);
                self.f32(*max);
            }
            Step::Title { key, secs } => {
                self.tag(3);
                self.text(key);
                self.f32(*secs);
            }
            Step::Cue(name) => {
                self.tag(4);
                self.text(name);
            }
            Step::Menu { menu, at, secs } => {
                self.tag(5);
                self.text(menu);
                self.block(*at);
                self.f32(*secs);
            }
        }
    }

    fn shot(&mut self, shot: &Shot) {
        self.path(&shot.path);
        self.look(&shot.look);
        self.f32(shot.secs);
        self.ease(shot.ease);
        self.f32(shot.fov[0]);
        self.f32(shot.fov[1]);
        self.f32(shot.blend);
    }

    fn ease(&mut self, ease: Ease) {
        self.tag(match ease {
            Ease::Linear => 0,
            Ease::Smooth => 1,
            Ease::In => 2,
            Ease::Out => 3,
        });
    }

    fn path(&mut self, path: &Path) {
        match path {
            Path::Still(eye) => {
                self.tag(0);
                self.point(*eye);
            }
            Path::Line(from, to) => {
                self.tag(1);
                self.point(*from);
                self.point(*to);
            }
            Path::Spline(points) => {
                self.tag(2);
                self.count(points.len());
                points.iter().for_each(|p| self.point(*p));
            }
            Path::Orbit { center, radius, height, angle } => {
                self.tag(3);
                self.point(*center);
                self.pair(*radius);
                self.pair(*height);
                self.pair(*angle);
            }
            Path::Follow { subject, offset } => {
                self.tag(4);
                self.text(subject);
                self.point(*offset);
            }
        }
    }

    fn look(&mut self, look: &Look) {
        match look {
            Look::At(point) => {
                self.tag(0);
                self.point(*point);
            }
            Look::Pan(from, to) => {
                self.tag(1);
                self.point(*from);
                self.point(*to);
            }
            Look::Subject(name) => {
                self.tag(2);
                self.text(name);
            }
            Look::Ahead => self.tag(3),
            Look::Angles { yaw, pitch } => {
                self.tag(4);
                self.f32(*yaw);
                self.f32(*pitch);
            }
            Look::Along(angles) => {
                self.tag(5);
                self.count(angles.len());
                angles.iter().flatten().for_each(|v| self.f32(*v));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film::Film;

    fn bytes(film: Film) -> Vec<u8> {
        let timeline = Timeline {
            film: "mymod:tour".into(),
            origin: Pos::new(10, -64, 3),
            steps: film.steps,
        };
        timeline.encode()
    }

    #[test]
    fn a_timeline_starts_with_its_version_name_and_origin() {
        let bytes = bytes(Film::new().wait(1.0));
        let mut expected = TIMELINE_VERSION.to_le_bytes().to_vec();
        expected.extend(10u16.to_le_bytes());
        expected.extend(b"mymod:tour");
        [10i32, -64, 3]
            .iter()
            .for_each(|v| expected.extend(v.to_le_bytes()));
        expected.extend(1u32.to_le_bytes());
        assert!(bytes.starts_with(&expected), "the header, then one step");
        assert_eq!(bytes[expected.len()], 1, "the wait's tag");
    }

    #[test]
    fn long_texts_are_cut_on_a_character() {
        // 65536 bytes, whose 65535th is the middle of an é.
        let key = "é".repeat(32768);
        let bytes = bytes(Film::new().title(&key, 1.0));
        // The version, the name's length and text, the origin, the count and the step's tag.
        let at = 2 + 2 + "mymod:tour".len() + 12 + 4 + 1;
        let len = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        assert_eq!(len, u16::MAX - 1, "cut before the é it would split");
    }
}
