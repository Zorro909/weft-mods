//! The `.wst` structure file: a box of block states, stored the way a world save stores a cube.
//!
//! `WSTR`, a version byte, then LZ4 with the size prepended (like each cube in a region file)
//! around:
//! - the size along x, y and z, a `u16` each
//! - the palette: a `u16` count, then each state's name as a `u16` byte length and UTF-8,
//!   written like the lines of a save's `ids.txt` (`base:furnace[facing=east,lit=false]`).
//!   `core:air` clears what is there
//! - every block's palette index (`packing`), x fastest, then z, then y
//!
//! Numbers are little-endian. Block entity contents (a chest's items, a machine's state) are
//! not stored yet: those blocks are placed empty.

use std::collections::HashMap;

use weftkit::Pos;

use crate::packing;

const MAGIC: &[u8] = b"WSTR";
const VERSION: u8 = 1;
/// The most blocks a structure holds (256 x 256 x 256), so a small file can't claim a box too
/// big to keep in memory.
const MAX_VOLUME: usize = 1 << 24;
/// The palette's count is a `u16`.
const MAX_STATES: usize = u16::MAX as usize;

/// A box of block states. Its parts always agree: one palette index per block, each inside the
/// palette, so it encodes to a file that decodes again.
#[derive(Debug, Default, PartialEq)]
pub struct Structure {
    /// Blocks along x, y and z.
    pub(crate) size: [u16; 3],
    pub(crate) palette: Vec<String>,
    /// Each block's position in `palette`, x fastest, then z, then y.
    pub(crate) indices: Vec<u16>,
}

impl Structure {
    /// A structure of `size` from each block's state name, in `blocks` order. `None` unless
    /// there is one name per block, at most `MAX_VOLUME` of them, and at most 65535 different
    /// names, none longer than 65535 bytes.
    pub fn from_names<'a>(
        size: [u16; 3],
        names: impl IntoIterator<Item = &'a str>,
    ) -> Option<Structure> {
        let volume = volume(size)?;
        let mut palette: Vec<String> = Vec::new();
        let mut seen: HashMap<&str, u16> = HashMap::new();
        let mut indices = Vec::with_capacity(volume);
        for name in names {
            let index = match seen.get(name) {
                Some(&index) => index,
                None if palette.len() < MAX_STATES && name.len() <= u16::MAX as usize => {
                    seen.insert(name, palette.len() as u16);
                    palette.push(name.to_string());
                    palette.len() as u16 - 1
                }
                None => return None,
            };
            if indices.len() == volume {
                return None;
            }
            indices.push(index);
        }
        (indices.len() == volume).then_some(Structure {
            size,
            palette,
            indices,
        })
    }

    /// Blocks along x, y and z.
    pub fn size(&self) -> [i32; 3] {
        self.size.map(|side| side as i32)
    }

    /// Each block's position from the lowest corner, and its state name.
    pub fn blocks(&self) -> impl Iterator<Item = (Pos, &str)> {
        let [sx, _, sz] = self.size.map(|v| v as usize);
        self.indices.iter().enumerate().map(move |(i, &p)| {
            let pos = Pos::new(
                (i % sx) as i32,
                (i / (sx * sz)) as i32,
                (i / sx % sz) as i32,
            );
            (pos, self.palette[p as usize].as_str())
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        for v in self.size {
            body.extend_from_slice(&v.to_le_bytes());
        }
        body.extend_from_slice(&(self.palette.len() as u16).to_le_bytes());
        for name in &self.palette {
            body.extend_from_slice(&(name.len() as u16).to_le_bytes());
            body.extend_from_slice(name.as_bytes());
        }
        packing::write(&mut body, &self.indices, self.palette.len());
        let mut file = MAGIC.to_vec();
        file.push(VERSION);
        file.extend_from_slice(&lz4_flex::compress_prepend_size(&body));
        file
    }

    /// `None` for anything but a whole file of this version.
    pub fn decode(file: &[u8]) -> Option<Structure> {
        let (&version, packed) = file.strip_prefix(MAGIC)?.split_first()?;
        if version != VERSION || !plausible_body(packed) {
            return None;
        }
        let body = lz4_flex::decompress_size_prepended(packed).ok()?;
        let mut reader = Reader(&body);
        let size = [reader.u16()?, reader.u16()?, reader.u16()?];
        let count = reader.u16()?;
        let palette = (0..count)
            .map(|_| reader.name())
            .collect::<Option<Vec<String>>>()?;
        let indices = packing::read(reader.0, volume(size)?, palette.len())?;
        Some(Structure {
            size,
            palette,
            indices,
        })
    }
}

/// The blocks in a box of `size`, `None` past `MAX_VOLUME`.
fn volume(size: [u16; 3]) -> Option<usize> {
    let volume = size.iter().map(|&side| side as usize).product();
    (volume <= MAX_VOLUME).then_some(volume)
}

/// Whether the size LZ4 prepended could be true: it makes data at most 255 times smaller, so a
/// file claiming more is refused before that much memory is taken.
fn plausible_body(packed: &[u8]) -> bool {
    let Some(&size) = packed.first_chunk::<4>() else {
        return false;
    };
    u32::from_le_bytes(size) as usize <= packed.len().saturating_mul(255)
}

/// The unread rest of a file's body.
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let (head, rest) = self.0.split_at_checked(n)?;
        self.0 = rest;
        Some(head)
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.take(2)?.try_into().ok()?))
    }

    fn name(&mut self) -> Option<String> {
        let len = self.u16()? as usize;
        String::from_utf8(self.take(len)?.to_vec()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hut() -> Structure {
        let names = (0..2 * 3 * 4).map(|i| {
            if i % 5 == 0 {
                "base:oak_planks"
            } else if i % 3 == 0 {
                "core:air"
            } else {
                "base:furnace[facing=east,lit=false]"
            }
        });
        Structure::from_names([2, 3, 4], names).expect("one name per block")
    }

    #[test]
    fn a_structure_reads_back() {
        let hut = hut();
        assert_eq!(
            Structure::decode(&hut.encode()),
            Some(hut),
            "the same after a round trip"
        );
    }

    #[test]
    fn the_palette_holds_each_name_once() {
        assert_eq!(
            hut().palette,
            [
                "base:oak_planks",
                "base:furnace[facing=east,lit=false]",
                "core:air"
            ],
            "first seen first"
        );
    }

    #[test]
    fn blocks_run_x_then_z_then_y() {
        let hut = Structure::from_names([2, 3, 2], ["a"; 12]).expect("one name per block");
        let positions: Vec<Pos> = hut.blocks().map(|(pos, _)| pos).collect();
        assert_eq!(positions[1], Pos::new(1, 0, 0), "x first");
        assert_eq!(positions[2], Pos::new(0, 0, 1), "then z");
        assert_eq!(positions[4], Pos::new(0, 1, 0), "then y");
        assert_eq!(positions[11], Pos::new(1, 2, 1), "the far corner last");
    }

    #[test]
    fn other_files_fail() {
        let file = hut().encode();
        assert_eq!(Structure::decode(b"PNG"), None, "wrong magic");
        assert_eq!(
            Structure::decode(&file[..file.len() - 3]),
            None,
            "truncated"
        );
        let mut newer = file.clone();
        newer[MAGIC.len()] = VERSION + 1;
        assert_eq!(Structure::decode(&newer), None, "another version");
    }

    #[test]
    fn a_huge_box_of_one_state_is_refused() {
        // A palette of one state needs no index words, so this file is a few bytes long.
        let huge = Structure {
            size: [u16::MAX; 3],
            palette: vec!["core:air".to_string()],
            indices: Vec::new(),
        };
        assert_eq!(
            Structure::decode(&huge.encode()),
            None,
            "too many blocks to hold"
        );
    }

    #[test]
    fn a_body_claiming_too_much_is_refused() {
        let mut file = MAGIC.to_vec();
        file.push(VERSION);
        file.extend_from_slice(&u32::MAX.to_le_bytes());
        file.extend_from_slice(&[0x10, b'x']);
        assert_eq!(Structure::decode(&file), None, "4 GiB from 2 bytes");
        let honest = lz4_flex::compress_prepend_size(&vec![0; 1 << 20]);
        assert!(plausible_body(&honest), "the best LZ4 can do is allowed");
    }

    #[test]
    fn names_must_fill_the_box() {
        assert!(
            Structure::from_names([2, 1, 2], ["a"; 3]).is_none(),
            "one short"
        );
        assert!(
            Structure::from_names([2, 1, 2], ["a"; 5]).is_none(),
            "one over"
        );
        let too_long = "a".repeat(u16::MAX as usize + 1);
        assert!(
            Structure::from_names([1, 1, 1], [too_long.as_str()]).is_none(),
            "a name too long to write"
        );
    }

    #[test]
    fn at_most_65535_states_fit_the_palette() {
        let names: Vec<String> = (0..u16::MAX as u32 + 1).map(|i| i.to_string()).collect();
        let names = || names.iter().map(String::as_str);
        assert!(
            Structure::from_names([65535, 1, 1], names().take(65535)).is_some(),
            "a full palette"
        );
        assert!(
            Structure::from_names([256, 1, 256], names()).is_none(),
            "one state more"
        );
    }
}
