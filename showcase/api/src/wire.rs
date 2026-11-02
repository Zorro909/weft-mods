//! A film as bytes, the way `TIMELINE` carries it: the version, the film's name and origin,
//! then its steps. Numbers are little-endian, texts a `u16` length and UTF-8. `decode` reads it
//! back.

use crate::film::{Film, Look, Path, Shot, Step};

/// Raised whenever the layout changes; the showcase mod refuses newer films than it knows.
pub const VERSION: u16 = 1;

/// A film as the player's client receives it.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub film: String,
    pub origin: [i32; 3],
    pub steps: Film,
}

pub(crate) fn encode(film: &str, origin: [i32; 3], steps: &Film) -> Vec<u8> {
    let mut out = Out(Vec::new());
    out.0.extend(VERSION.to_le_bytes());
    out.text(film);
    origin.iter().for_each(|v| out.0.extend(v.to_le_bytes()));
    out.0.extend((steps.steps.len() as u32).to_le_bytes());
    steps.steps.iter().for_each(|s| out.step(s));
    out.0
}

struct Out(Vec<u8>);

impl Out {
    fn f32(&mut self, v: f32) {
        self.0.extend(v.to_le_bytes());
    }

    fn point(&mut self, p: [f64; 3]) {
        p.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    /// Cut short at 65535 bytes, on a character.
    fn text(&mut self, t: &str) {
        let mut len = t.len().min(u16::MAX as usize);
        while !t.is_char_boundary(len) {
            len -= 1;
        }
        self.0.extend((len as u16).to_le_bytes());
        self.0.extend(&t.as_bytes()[..len]);
    }

    fn step(&mut self, step: &Step) {
        match step {
            Step::Shot(shot) => {
                self.0.push(0);
                self.shot(shot);
            }
            Step::Wait(secs) => {
                self.0.push(1);
                self.f32(*secs);
            }
            Step::WaitFor { name, max } => {
                self.0.push(2);
                self.text(name);
                self.f32(*max);
            }
            Step::Title { key, secs } => {
                self.0.push(3);
                self.text(key);
                self.f32(*secs);
            }
            Step::Cue(name) => {
                self.0.push(4);
                self.text(name);
            }
            Step::Menu { menu, at, secs } => {
                self.0.push(5);
                self.text(menu);
                at.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
                self.f32(*secs);
            }
        }
    }

    fn shot(&mut self, shot: &Shot) {
        self.path(&shot.path);
        self.look(&shot.look);
        self.f32(shot.secs);
        self.0.push(shot.ease as u8);
        self.f32(shot.fov[0]);
        self.f32(shot.fov[1]);
        self.f32(shot.blend);
    }

    fn path(&mut self, path: &Path) {
        match path {
            Path::Still(p) => {
                self.0.push(0);
                self.point(*p);
            }
            Path::Line(a, b) => {
                self.0.push(1);
                self.point(*a);
                self.point(*b);
            }
            Path::Spline(points) => {
                self.0.push(2);
                self.0.extend((points.len() as u32).to_le_bytes());
                points.iter().for_each(|p| self.point(*p));
            }
            Path::Orbit {
                center,
                radius,
                height,
                angle,
            } => {
                self.0.push(3);
                self.point(*center);
                [radius, height, angle]
                    .iter()
                    .for_each(|p| p.iter().for_each(|v| self.0.extend(v.to_le_bytes())));
            }
            Path::Follow { subject, offset } => {
                self.0.push(4);
                self.text(subject);
                self.point(*offset);
            }
        }
    }

    fn look(&mut self, look: &Look) {
        match look {
            Look::At(p) => {
                self.0.push(0);
                self.point(*p);
            }
            Look::Pan(a, b) => {
                self.0.push(1);
                self.point(*a);
                self.point(*b);
            }
            Look::Subject(name) => {
                self.0.push(2);
                self.text(name);
            }
            Look::Ahead => self.0.push(3),
            Look::Angles { yaw, pitch } => {
                self.0.push(4);
                self.f32(*yaw);
                self.f32(*pitch);
            }
            Look::Along(angles) => {
                self.0.push(5);
                self.0.extend((angles.len() as u32).to_le_bytes());
                angles.iter().flatten().for_each(|v| self.f32(*v));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::decode;
    use crate::film::*;

    fn every_kind() -> Film {
        Film::new()
            .title("mymod.film.title", 3.0)
            .shot(orbit([1.0, 2.0, 3.0], 8.0).arc(10.0, 200.0).fov(70.0, 50.0))
            .shot(
                flyby(&[[0.0; 3], [4.0, 5.0, 6.0], [9.0, 1.0, 2.0]])
                    .ease(Ease::In)
                    .blend(1.0),
            )
            .shot(dolly([0.0; 3], [1.0; 3]).look(Look::Pan([0.0; 3], [2.0; 3])))
            .shot(hold([1.0; 3]).look(Look::Angles {
                yaw: 90.0,
                pitch: -10.0,
            }))
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
        let film = every_kind();
        let bytes = encode("mymod:tour", [10, -64, 3], &film);
        let back = decode(&bytes).expect("a timeline it wrote itself");
        assert_eq!(
            back,
            Timeline {
                film: "mymod:tour".into(),
                origin: [10, -64, 3],
                steps: film
            }
        );
    }

    #[test]
    fn a_cut_short_timeline_is_refused() {
        let bytes = encode("mymod:tour", [0; 3], &every_kind());
        assert!(
            decode(&bytes[..bytes.len() - 1]).is_none(),
            "one byte short"
        );
    }

    #[test]
    fn numbers_that_are_not_finite_are_refused() {
        let wait = encode("mymod:tour", [0; 3], &Film::new().wait(f32::NAN));
        assert!(decode(&wait).is_none(), "a wait that never ends");
        let far = encode(
            "mymod:tour",
            [0; 3],
            &Film::new().shot(hold([f64::INFINITY, 0.0, 0.0])),
        );
        assert!(decode(&far).is_none(), "a camera at infinity");
    }

    #[test]
    fn long_texts_are_cut_on_a_character() {
        // 65536 bytes, whose 65535th is the middle of an é.
        let key = "é".repeat(32768);
        let bytes = encode("mymod:tour", [0; 3], &Film::new().title(&key, 1.0));
        let back = decode(&bytes).expect("still a timeline");
        let Step::Title { key: cut, .. } = &back.steps.steps[0] else {
            panic!("a title")
        };
        assert!(
            key.starts_with(cut.as_str()) && cut.len() == u16::MAX as usize - 1,
            "{}",
            cut.len()
        );
    }

    #[test]
    fn a_newer_version_is_refused() {
        let mut bytes = encode("mymod:tour", [0; 3], &every_kind());
        bytes[..2].copy_from_slice(&(VERSION + 1).to_le_bytes());
        assert!(decode(&bytes).is_none(), "newer than it knows");
    }
}
