//! A film as bytes, the way `TIMELINE` carries it: the version, the film's name and origin, then
//! its steps. Each type is written and read back by one `Wire` impl, so the two sides can't
//! drift apart. Numbers are little-endian, counts a `u32`, texts a `u16` length and UTF-8, and
//! every step, path, look and ease starts with a tag byte (`film.rs`).
//!
//! Anything cut short, unknown or of another version is refused, and so are numbers that aren't
//! finite, which would stop a film or fling the camera.

mod film;

use crate::Pos;
use crate::film::Step;

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
        let mut out = Vec::new();
        TIMELINE_VERSION.put(&mut out);
        self.film.put(&mut out);
        self.origin.put(&mut out);
        self.steps.put(&mut out);
        out
    }

    /// The timeline in `bytes`; `None` if it is cut short, malformed or of another version.
    pub fn decode(mut bytes: &[u8]) -> Option<Timeline> {
        let bytes = &mut bytes;
        if u16::get(bytes)? != TIMELINE_VERSION {
            return None;
        }
        Some(Timeline {
            film: Wire::get(bytes)?,
            origin: Wire::get(bytes)?,
            steps: Wire::get(bytes)?,
        })
    }
}

/// A value in a timeline: `put` writes it, `get` reads it back from the front of `bytes`.
trait Wire: Sized {
    fn put(&self, out: &mut Vec<u8>);
    fn get(bytes: &mut &[u8]) -> Option<Self>;
}

fn take<const N: usize>(bytes: &mut &[u8]) -> Option<[u8; N]> {
    let (head, rest) = bytes.split_first_chunk::<N>()?;
    *bytes = rest;
    Some(*head)
}

/// A number, read back only where `valid` holds.
macro_rules! number {
    ($ty:ty, $valid:expr) => {
        impl Wire for $ty {
            fn put(&self, out: &mut Vec<u8>) {
                out.extend(self.to_le_bytes());
            }

            fn get(bytes: &mut &[u8]) -> Option<$ty> {
                Some(<$ty>::from_le_bytes(take(bytes)?)).filter($valid)
            }
        }
    };
}

number!(u8, |_| true);
number!(u16, |_| true);
number!(u32, |_| true);
number!(i32, |_| true);
number!(f32, |v: &f32| v.is_finite());
number!(f64, |v: &f64| v.is_finite());

/// A `u16` length and UTF-8, cut short at 65535 bytes on a character.
impl Wire for String {
    fn put(&self, out: &mut Vec<u8>) {
        let text = &self[..self.floor_char_boundary(u16::MAX as usize)];
        (text.len() as u16).put(out);
        out.extend(text.as_bytes());
    }

    fn get(bytes: &mut &[u8]) -> Option<String> {
        let len = u16::get(bytes)? as usize;
        let text = bytes.get(..len)?;
        *bytes = &bytes[len..];
        String::from_utf8(text.to_vec()).ok()
    }
}

/// A `u32` count, then each item.
impl<T: Wire> Wire for Vec<T> {
    fn put(&self, out: &mut Vec<u8>) {
        (self.len() as u32).put(out);
        self.iter().for_each(|item| item.put(out));
    }

    /// A count larger than the bytes left fails once they run out, without reserving room for
    /// it first.
    fn get(bytes: &mut &[u8]) -> Option<Vec<T>> {
        let count = u32::get(bytes)?;
        (0..count).map(|_| T::get(bytes)).collect()
    }
}

/// Each item, without a count.
impl<T: Wire, const N: usize> Wire for [T; N] {
    fn put(&self, out: &mut Vec<u8>) {
        self.iter().for_each(|item| item.put(out));
    }

    fn get(bytes: &mut &[u8]) -> Option<[T; N]> {
        let items: Vec<T> = (0..N).map(|_| T::get(bytes)).collect::<Option<_>>()?;
        items.try_into().ok()
    }
}

impl Wire for Pos {
    fn put(&self, out: &mut Vec<u8>) {
        <[i32; 3]>::from(*self).put(out);
    }

    fn get(bytes: &mut &[u8]) -> Option<Pos> {
        <[i32; 3]>::get(bytes).map(Pos::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film::*;

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

    /// One of every step, path, look and ease.
    fn every_kind() -> Film {
        let along = vec![[10.0, 0.0], [-170.0, 20.0]];
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
            .shot(flyby(&[[0.0; 3], [1.0; 3]]).look(Look::Along(along)))
            .shot(follow("item", [0.0, 1.0, -2.0]).ease(Ease::Out))
            .wait(1.5)
            .wait_for("dug", 30.0)
            .cue("start")
            .menu("mymod:press", Pos::new(2, 0, -3), 4.0)
    }

    /// Where the first step's tag is: after the version, the name's length and text, the origin
    /// and the count.
    const FIRST_TAG: usize = 2 + 2 + "mymod:tour".len() + 12 + 4;

    #[test]
    fn every_step_survives_the_round_trip() {
        let sent = timeline(every_kind());
        assert_eq!(Timeline::decode(&sent.encode()), Some(sent));
    }

    #[test]
    fn a_timeline_cut_short_anywhere_is_refused() {
        let bytes = bytes(every_kind());
        for len in 0..bytes.len() {
            let cut = Timeline::decode(&bytes[..len]);
            assert!(cut.is_none(), "cut at {len} of {}", bytes.len());
        }
    }

    #[test]
    fn unknown_tags_are_refused() {
        let mut bytes = bytes(Film::new().wait(1.0));
        assert_eq!(bytes[FIRST_TAG], 1, "the wait's tag");
        bytes[FIRST_TAG] = 6;
        let unknown = Timeline::decode(&bytes);
        assert!(unknown.is_none(), "a step this version doesn't know");
    }

    #[test]
    fn numbers_that_are_not_finite_are_refused() {
        let wait = bytes(Film::new().wait(f32::NAN));
        assert!(Timeline::decode(&wait).is_none(), "a wait that never ends");
        let far = bytes(Film::new().shot(hold([f64::INFINITY, 0.0, 0.0])));
        assert!(Timeline::decode(&far).is_none(), "a camera at infinity");
    }

    #[test]
    fn long_texts_are_cut_on_a_character() {
        // 65536 bytes, whose 65535th is the middle of an é.
        let key = "é".repeat(32768);
        let back = Timeline::decode(&bytes(Film::new().title(&key, 1.0)));
        let Some(Step::Title { key: cut, .. }) = back.expect("still a timeline").steps.pop() else {
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
        for version in [TIMELINE_VERSION - 1, TIMELINE_VERSION + 1] {
            bytes[..2].copy_from_slice(&version.to_le_bytes());
            assert!(Timeline::decode(&bytes).is_none(), "version {version}");
        }
    }
}
