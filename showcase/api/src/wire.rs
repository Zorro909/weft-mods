//! A film as bytes, the way `TIMELINE` carries it: the version, the film's name and origin,
//! then its steps. Numbers are little-endian, texts a `u16` length and UTF-8.

use crate::film::{Ease, Film, Look, Path, Shot, Step};

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

/// The timeline in `bytes`; `None` if it is cut short, malformed or of a newer version.
pub fn decode(bytes: &[u8]) -> Option<Timeline> {
    let mut i = In(bytes);
    if u16::from_le_bytes(i.take()?) > VERSION {
        return None;
    }
    let film = i.text()?;
    let origin = [i.i32()?, i.i32()?, i.i32()?];
    let count = i.u32()? as usize;
    let steps = (0..count).map(|_| i.step()).collect::<Option<Vec<_>>>()?;
    Some(Timeline {
        film,
        origin,
        steps: Film { steps },
    })
}

struct Out(Vec<u8>);

impl Out {
    fn f32(&mut self, v: f32) {
        self.0.extend(v.to_le_bytes());
    }

    fn point(&mut self, p: [f64; 3]) {
        p.iter().for_each(|v| self.0.extend(v.to_le_bytes()));
    }

    fn text(&mut self, t: &str) {
        let len = t.len().min(u16::MAX as usize);
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
        }
    }
}

struct In<'a>(&'a [u8]);

impl In<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take()?))
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take()?))
    }

    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take()?))
    }

    fn f64(&mut self) -> Option<f64> {
        Some(f64::from_le_bytes(self.take()?))
    }

    fn point(&mut self) -> Option<[f64; 3]> {
        Some([self.f64()?, self.f64()?, self.f64()?])
    }

    fn pair(&mut self) -> Option<[f64; 2]> {
        Some([self.f64()?, self.f64()?])
    }

    fn text(&mut self) -> Option<String> {
        let len = u16::from_le_bytes(self.take()?) as usize;
        let bytes = self.0.get(..len)?;
        self.0 = &self.0[len..];
        String::from_utf8(bytes.to_vec()).ok()
    }

    fn step(&mut self) -> Option<Step> {
        Some(match self.u8()? {
            0 => Step::Shot(self.shot()?),
            1 => Step::Wait(self.f32()?),
            2 => Step::WaitFor {
                name: self.text()?,
                max: self.f32()?,
            },
            3 => Step::Title {
                key: self.text()?,
                secs: self.f32()?,
            },
            4 => Step::Cue(self.text()?),
            _ => return None,
        })
    }

    fn shot(&mut self) -> Option<Shot> {
        let (path, look, secs) = (self.path()?, self.look()?, self.f32()?);
        let ease = [Ease::Linear, Ease::Smooth, Ease::In, Ease::Out]
            .get(self.u8()? as usize)
            .copied()?;
        let fov = [self.f32()?, self.f32()?];
        Some(Shot {
            path,
            look,
            secs,
            ease,
            fov,
            blend: self.f32()?,
        })
    }

    fn path(&mut self) -> Option<Path> {
        Some(match self.u8()? {
            0 => Path::Still(self.point()?),
            1 => Path::Line(self.point()?, self.point()?),
            2 => {
                let count = self.u32()? as usize;
                Path::Spline((0..count).map(|_| self.point()).collect::<Option<_>>()?)
            }
            3 => Path::Orbit {
                center: self.point()?,
                radius: self.pair()?,
                height: self.pair()?,
                angle: self.pair()?,
            },
            4 => Path::Follow {
                subject: self.text()?,
                offset: self.point()?,
            },
            _ => return None,
        })
    }

    fn look(&mut self) -> Option<Look> {
        Some(match self.u8()? {
            0 => Look::At(self.point()?),
            1 => Look::Pan(self.point()?, self.point()?),
            2 => Look::Subject(self.text()?),
            3 => Look::Ahead,
            4 => Look::Angles {
                yaw: self.f32()?,
                pitch: self.f32()?,
            },
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            .shot(follow("item", [0.0, 1.0, -2.0]).ease(Ease::Out))
            .wait(1.5)
            .wait_for("dug", 30.0)
            .cue("start")
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
    fn a_newer_version_is_refused() {
        let mut bytes = encode("mymod:tour", [0; 3], &every_kind());
        bytes[..2].copy_from_slice(&(VERSION + 1).to_le_bytes());
        assert!(decode(&bytes).is_none(), "newer than it knows");
    }
}
