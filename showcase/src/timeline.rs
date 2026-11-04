//! Reading a film back from the bytes `TIMELINE` carries, as `showcase_api` writes them
//! (`api/src/wire.rs`). Anything cut short, unknown or of another version is refused, and so are
//! numbers that aren't finite, which would stop a film or fling the camera.

use showcase_api::{Ease, Look, Path, Pos, Shot, Step, TIMELINE_VERSION, Timeline};

/// The timeline in `bytes`; `None` if it is cut short, malformed or of another version.
pub fn decode(bytes: &[u8]) -> Option<Timeline> {
    let mut read = Reader(bytes);
    if u16::from_le_bytes(read.take()?) != TIMELINE_VERSION {
        return None;
    }
    let film = read.text()?;
    let origin = read.block()?;
    let steps = read.list(Reader::step)?;
    Some(Timeline {
        film,
        origin,
        steps,
    })
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take::<1>()?[0])
    }

    fn i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.take()?))
    }

    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take()?)).filter(|v| v.is_finite())
    }

    fn f64(&mut self) -> Option<f64> {
        Some(f64::from_le_bytes(self.take()?)).filter(|v| v.is_finite())
    }

    fn pair(&mut self) -> Option<[f64; 2]> {
        Some([self.f64()?, self.f64()?])
    }

    fn point(&mut self) -> Option<[f64; 3]> {
        Some([self.f64()?, self.f64()?, self.f64()?])
    }

    fn block(&mut self) -> Option<Pos> {
        Some(Pos::new(self.i32()?, self.i32()?, self.i32()?))
    }

    fn text(&mut self) -> Option<String> {
        let len = u16::from_le_bytes(self.take()?) as usize;
        let bytes = self.0.get(..len)?;
        self.0 = &self.0[len..];
        String::from_utf8(bytes.to_vec()).ok()
    }

    /// A count, then that many of what `item` reads. A count larger than the bytes left fails
    /// once they run out, without reserving room for it first.
    fn list<T>(&mut self, item: impl Fn(&mut Self) -> Option<T>) -> Option<Vec<T>> {
        let count = u32::from_le_bytes(self.take()?);
        (0..count).map(|_| item(self)).collect()
    }

    fn step(&mut self) -> Option<Step> {
        Some(match self.u8()? {
            0 => Step::Shot(self.shot()?),
            1 => Step::Wait(self.f32()?),
            2 => Step::WaitFor { name: self.text()?, max: self.f32()? },
            3 => Step::Title { key: self.text()?, secs: self.f32()? },
            4 => Step::Cue(self.text()?),
            5 => Step::Menu { menu: self.text()?, at: self.block()?, secs: self.f32()? },
            _ => return None,
        })
    }

    fn shot(&mut self) -> Option<Shot> {
        Some(Shot {
            path: self.path()?,
            look: self.look()?,
            secs: self.f32()?,
            ease: self.ease()?,
            fov: [self.f32()?, self.f32()?],
            blend: self.f32()?,
        })
    }

    fn ease(&mut self) -> Option<Ease> {
        Some(match self.u8()? {
            0 => Ease::Linear,
            1 => Ease::Smooth,
            2 => Ease::In,
            3 => Ease::Out,
            _ => return None,
        })
    }

    fn path(&mut self) -> Option<Path> {
        Some(match self.u8()? {
            0 => Path::Still(self.point()?),
            1 => Path::Line(self.point()?, self.point()?),
            2 => Path::Spline(self.list(Reader::point)?),
            3 => Path::Orbit {
                center: self.point()?,
                radius: self.pair()?,
                height: self.pair()?,
                angle: self.pair()?,
            },
            4 => Path::Follow { subject: self.text()?, offset: self.point()? },
            _ => return None,
        })
    }

    fn look(&mut self) -> Option<Look> {
        Some(match self.u8()? {
            0 => Look::At(self.point()?),
            1 => Look::Pan(self.point()?, self.point()?),
            2 => Look::Subject(self.text()?),
            3 => Look::Ahead,
            4 => Look::Angles { yaw: self.f32()?, pitch: self.f32()? },
            5 => Look::Along(self.list(|read| Some([read.f32()?, read.f32()?]))?),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::*;

    use super::*;

    fn timeline(film: Film) -> Timeline {
        Timeline {
            film: "mymod:tour".into(),
            origin: Pos::new(10, -64, 3),
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
            .menu("mymod:press", Pos::new(2, 0, -3), 4.0)
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
    fn a_timeline_of_another_version_is_refused() {
        let mut bytes = bytes(every_kind());
        for version in [TIMELINE_VERSION - 1, TIMELINE_VERSION + 1] {
            bytes[..2].copy_from_slice(&version.to_le_bytes());
            assert!(decode(&bytes).is_none(), "version {version}");
        }
    }
}
