//! Reading a film back from the bytes `wire` writes. Anything cut short, unknown or newer than
//! this version is refused, and so are numbers that aren't finite, which would stop a film or
//! fling the camera.

use crate::film::{Ease, Film, Look, Path, Shot, Step};
use crate::wire::{Timeline, VERSION};

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
        Some(f32::from_le_bytes(self.take()?)).filter(|v| v.is_finite())
    }

    fn f64(&mut self) -> Option<f64> {
        Some(f64::from_le_bytes(self.take()?)).filter(|v| v.is_finite())
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
            5 => Step::Menu {
                menu: self.text()?,
                at: [self.i32()?, self.i32()?, self.i32()?],
                secs: self.f32()?,
            },
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
            5 => {
                let count = self.u32()? as usize;
                Look::Along(
                    (0..count)
                        .map(|_| Some([self.f32()?, self.f32()?]))
                        .collect::<Option<_>>()?,
                )
            }
            _ => return None,
        })
    }
}
