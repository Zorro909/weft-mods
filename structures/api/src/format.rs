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

use crate::packing;

const MAGIC: &[u8] = b"WSTR";
const VERSION: u8 = 1;

#[derive(Debug, Default, PartialEq)]
pub struct Structure {
    /// Blocks along x, y and z.
    pub size: [u16; 3],
    pub palette: Vec<String>,
    /// Each block's position in `palette`, x fastest, then z, then y.
    pub indices: Vec<u16>,
}

impl Structure {
    /// A structure of `size` from each block's state name, in `indices` order.
    pub fn from_names<'a>(size: [u16; 3], names: impl IntoIterator<Item = &'a str>) -> Structure {
        let mut palette: Vec<String> = Vec::new();
        let indices = names
            .into_iter()
            .map(|name| match palette.iter().position(|p| p == name) {
                Some(i) => i as u16,
                None => {
                    palette.push(name.to_string());
                    (palette.len() - 1) as u16
                }
            })
            .collect();
        Structure {
            size,
            palette,
            indices,
        }
    }

    /// Each block's position from the lowest corner, and its state name.
    pub fn blocks(&self) -> impl Iterator<Item = ([i32; 3], &str)> {
        let [sx, _, sz] = self.size.map(|v| v as usize);
        self.indices.iter().enumerate().map(move |(i, &p)| {
            let pos = [
                (i % sx) as i32,
                (i / (sx * sz)) as i32,
                (i / sx % sz) as i32,
            ];
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
        if version != VERSION {
            return None;
        }
        let body = lz4_flex::decompress_size_prepended(packed).ok()?;
        let mut reader = Reader(&body);
        let size = [reader.u16()?, reader.u16()?, reader.u16()?];
        let count = reader.u16()?;
        let palette = (0..count)
            .map(|_| reader.name())
            .collect::<Option<Vec<String>>>()?;
        let volume = size.iter().map(|&v| v as usize).product();
        let indices = packing::read(reader.0, volume, palette.len())?;
        Some(Structure {
            size,
            palette,
            indices,
        })
    }
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
        Structure::from_names([2, 3, 4], names)
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
        let hut = Structure::from_names([2, 3, 2], ["a"; 12]);
        let positions: Vec<[i32; 3]> = hut.blocks().map(|(pos, _)| pos).collect();
        assert_eq!(positions[1], [1, 0, 0], "x first");
        assert_eq!(positions[2], [0, 0, 1], "then z");
        assert_eq!(positions[4], [0, 1, 0], "then y");
        assert_eq!(positions[11], [1, 2, 1], "the far corner last");
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
}
