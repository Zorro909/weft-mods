//! A film as bytes, the way `TIMELINE` carries it: the version, the film's name and origin, then
//! its steps. Numbers are little-endian, counts a `u32`, texts a `u16` length and UTF-8, and every
//! step, path, look and ease starts with a tag byte. `decode` reads it back.

use weft_sdk::{Payload, error};

use crate::decode::decode;
use crate::film::{Ease, Look, Path, Shot, Step};

/// Raised whenever the layout changes; films of any other version are refused.
pub(crate) const VERSION: u16 = 1;

/// A film as the player's client receives it: its full name (`mymod:tour`), the block its
/// positions are counted from, and its steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub film: String,
    pub origin: [i32; 3],
    pub steps: Vec<Step>,
}

impl Payload for Timeline {
    // Plain bytes to the engine, as `Vec<u8>` is: the layout is this file's, checked by `VERSION`.
    fn schema() -> String {
        "bytes".into()
    }

    fn encode(&self) -> Vec<u8> {
        let mut out = Writer(Vec::new());
        out.0.extend(VERSION.to_le_bytes());
        out.text(&self.film);
        out.block(self.origin);
        out.count(self.steps.len());
        self.steps.iter().for_each(|step| out.step(step));
        out.0
    }

    fn decode(bytes: &[u8]) -> Option<Timeline> {
        let timeline = decode(bytes);
        if timeline.is_none() {
            error!(
                "showcase: a film arrived that this version can't read ({} bytes)",
                bytes.len()
            );
        }
        timeline
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

    fn block(&mut self, block: [i32; 3]) {
        block.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    /// Cut short at 65535 bytes, on a character.
    fn text(&mut self, text: &str) {
        let mut len = text.len().min(u16::MAX as usize);
        while !text.is_char_boundary(len) {
            len -= 1;
        }
        self.0.extend((len as u16).to_le_bytes());
        self.0.extend(&text.as_bytes()[..len]);
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
    use crate::film::*;

    fn timeline(film: Film) -> Timeline {
        Timeline {
            film: "mymod:tour".into(),
            origin: [10, -64, 3],
            steps: film.steps,
        }
    }

    fn bytes(film: Film) -> Vec<u8> {
        timeline(film).encode()
    }

    fn every_kind() -> Film {
        Film::new()
            .title("mymod.film.title", 3.0)
            .shot(orbit([1.0, 2.0, 3.0], 8.0).arc(10.0, 200.0).fov(70.0, 50.0))
            .shot(flyby(&[[0.0; 3], [4.0, 5.0, 6.0], [9.0, 1.0, 2.0]]).ease(Ease::In).blend(1.0))
            .shot(dolly([0.0; 3], [1.0; 3]).look(Look::Pan([0.0; 3], [2.0; 3])))
            .shot(hold([1.0; 3]).look(Look::Angles { yaw: 90.0, pitch: -10.0 }))
            .shot(crane([0.0; 3], 6.0, 45.0, [1.0, 9.0]).ease(Ease::Linear))
            .shot(flyby(&[[0.0; 3], [1.0; 3]]).look(Look::Along(vec![[10.0, 0.0], [-170.0, 20.0]])))
            .shot(follow("item", [0.0, 1.0, -2.0]).ease(Ease::Out))
            .wait(1.5)
            .wait_for("dug", 30.0)
            .cue("start")
            .menu("mymod:press", [2, 0, -3], 4.0)
    }

    #[test]
    fn every_step_survives_the_round_trip() {
        let sent = timeline(every_kind());
        assert_eq!(decode(&sent.encode()), Some(sent));
    }

    #[test]
    fn a_timeline_cut_short_anywhere_is_refused() {
        let bytes = bytes(every_kind());
        for len in 0..bytes.len() {
            assert!(
                decode(&bytes[..len]).is_none(),
                "cut at {len} of {}",
                bytes.len()
            );
        }
    }

    #[test]
    fn unknown_tags_are_refused() {
        let mut bytes = bytes(Film::new().wait(1.0));
        // The version, the name's length and text, the origin, the count: then the step's tag.
        let tag = 2 + 2 + "mymod:tour".len() + 12 + 4;
        assert_eq!(bytes[tag], 1, "the wait's tag");
        bytes[tag] = 6;
        assert!(decode(&bytes).is_none(), "a step this version doesn't know");
    }

    #[test]
    fn numbers_that_are_not_finite_are_refused() {
        let wait = bytes(Film::new().wait(f32::NAN));
        assert!(decode(&wait).is_none(), "a wait that never ends");
        let far = bytes(Film::new().shot(hold([f64::INFINITY, 0.0, 0.0])));
        assert!(decode(&far).is_none(), "a camera at infinity");
    }

    #[test]
    fn long_texts_are_cut_on_a_character() {
        // 65536 bytes, whose 65535th is the middle of an é.
        let key = "é".repeat(32768);
        let back = decode(&bytes(Film::new().title(&key, 1.0))).expect("still a timeline");
        let Step::Title { key: cut, .. } = &back.steps[0] else {
            panic!("a title")
        };
        assert!(key.starts_with(cut.as_str()), "a start of the key");
        assert_eq!(
            cut.len(),
            u16::MAX as usize - 1,
            "cut before the é it would split"
        );
    }

    #[test]
    fn a_timeline_of_another_version_is_refused() {
        let mut bytes = bytes(every_kind());
        for version in [VERSION - 1, VERSION + 1] {
            bytes[..2].copy_from_slice(&version.to_le_bytes());
            assert!(decode(&bytes).is_none(), "version {version}");
        }
    }
}
