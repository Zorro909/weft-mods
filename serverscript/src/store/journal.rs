//! The store's file: every change written behind the last, and read back in that order.
//!
//! | Part | Bytes |
//! |---|---|
//! | head | `MAGIC`, then `VERSION` as a `u32` |
//! | change | what it does as a `u8`, its holder, its component's name as text, and a value if it sets one |
//! | holder | which kind as a `u8`, then a player's number as a `u64` or a name as text |
//!
//! Numbers are little-endian; text and a value lie as `pack` says. A change costs its own few
//! bytes however much the store holds. The file is written anew from what the store holds at
//! each start and whenever it has grown to twice that (`Journal`), or it would grow with every
//! change ever made.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use super::Holder;
use super::pack::{read_text, read_value, take, write_text, write_value};
use crate::value::Value;

/// What a store file starts with.
const MAGIC: [u8; 4] = *b"WSSD";
/// The layout of the changes. A file of another version is not read.
const VERSION: u32 = 2;

/// What a file may grow by before it is written anew, however little it holds: a megabyte.
const SMALL_GROWTH: u64 = 1 << 20;

/// Puts a value on a holder, replacing the component's last.
const SET: u8 = 0;
/// Takes a component off a holder.
const REMOVE: u8 = 1;

/// A holder that is a player: their number follows.
const PLAYER: u8 = 0;
/// A holder that is a name: the name follows.
const NAMED: u8 = 1;

/// One change read back from the file: the value `component` has on `holder` from then on,
/// `None` when it was taken off.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub holder: Holder,
    pub component: String,
    pub value: Option<Value>,
}

/// A file's changes in the order made. `whole` is false when it holds more than could be
/// read: another version's file, or an end cut off in the middle of a change.
#[derive(Debug, PartialEq)]
pub struct Read {
    pub records: Vec<Record>,
    pub whole: bool,
}

/// The bytes of the change that gives `component` on `holder` the `value`, or with `None`
/// takes it off.
pub fn change(holder: &Holder, component: &str, value: Option<&Value>) -> Vec<u8> {
    let mut bytes = vec![if value.is_some() { SET } else { REMOVE }];
    match holder {
        Holder::Player(number) => {
            bytes.push(PLAYER);
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        Holder::Named(name) => {
            bytes.push(NAMED);
            write_text(name, &mut bytes);
        }
    }
    write_text(component, &mut bytes);
    if let Some(value) = value {
        write_value(value, &mut bytes);
    }
    bytes
}

fn head() -> Vec<u8> {
    [MAGIC.as_slice(), &VERSION.to_le_bytes()].concat()
}

/// The changes in the file at `path`; none when there is no file yet.
pub fn read(path: &Path) -> io::Result<Read> {
    match fs::read(path) {
        Ok(bytes) => Ok(parse(&bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(parse(&head())),
        Err(error) => Err(error),
    }
}

fn parse(bytes: &[u8]) -> Read {
    let mut records = Vec::new();
    let Some(mut rest) = bytes.strip_prefix(head().as_slice()) else {
        return Read {
            records,
            whole: false,
        };
    };
    while !rest.is_empty() {
        let Some(record) = record(&mut rest) else {
            return Read {
                records,
                whole: false,
            };
        };
        records.push(record);
    }
    Read {
        records,
        whole: true,
    }
}

fn record(bytes: &mut &[u8]) -> Option<Record> {
    let what = take(bytes, 1)?[0];
    let holder = match take(bytes, 1)?[0] {
        PLAYER => Holder::Player(u64::from_le_bytes(take(bytes, 8)?.try_into().ok()?)),
        NAMED => Holder::Named(read_text(bytes)?),
        _ => return None,
    };
    let component = read_text(bytes)?;
    let value = match what {
        SET => Some(read_value(bytes)?),
        REMOVE => None,
        _ => return None,
    };
    Some(Record {
        holder,
        component,
        value,
    })
}

/// The file, open for the changes to come.
pub struct Journal {
    file: File,
    path: PathBuf,
    /// How many bytes it had when last written anew.
    anew: u64,
    /// How many were put at its end since.
    grown: u64,
}

impl Journal {
    /// Writes the file at `path` anew, holding `changes`, and opens it for the changes to come.
    /// Whole or not at all: the new file takes the old one's place only once it is on the disk.
    pub fn start(path: &Path, changes: &[u8]) -> io::Result<Journal> {
        if let Some(folder) = path.parent() {
            fs::create_dir_all(folder)?;
        }
        let bytes = [head().as_slice(), changes].concat();
        let new = path.with_extension("new");
        let mut file = File::create(&new)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&new, path)?;
        Ok(Journal {
            file: OpenOptions::new().append(true).open(path)?,
            path: path.to_path_buf(),
            anew: bytes.len() as u64,
            grown: 0,
        })
    }

    /// Puts `changes` at the file's end.
    pub fn append(&mut self, changes: &[u8]) -> io::Result<()> {
        self.file.write_all(changes)?;
        self.grown += changes.len() as u64;
        Ok(())
    }

    /// Whether the file has doubled since it was written anew, and by more than a little.
    pub fn outgrown(&self) -> bool {
        self.grown > self.anew.max(SMALL_GROWTH)
    }

    /// Writes the file anew, holding `changes` in place of all it held.
    pub fn rewrite(&mut self, changes: &[u8]) -> io::Result<()> {
        *self = Journal::start(&self.path, changes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records() -> Vec<Record> {
        let record = |holder, component: &str, value| Record {
            holder,
            component: component.to_string(),
            value,
        };
        let camp = Value::List(vec![Value::Int(1)]);
        vec![
            record(
                Holder::Named("server".into()),
                "welcome:stats",
                Some(Value::Int(3)),
            ),
            record(Holder::Player(7), "camps:camp", Some(camp)),
            record(Holder::Player(7), "camps:camp", None),
        ]
    }

    fn changes() -> Vec<u8> {
        let bytes =
            |record: Record| change(&record.holder, &record.component, record.value.as_ref());
        records().into_iter().flat_map(bytes).collect()
    }

    fn file(changes: &[u8]) -> Vec<u8> {
        [head().as_slice(), changes].concat()
    }

    #[test]
    fn changes_come_back_in_the_order_made() {
        let read = parse(&file(&changes()));
        assert_eq!(
            read,
            Read {
                records: records(),
                whole: true
            }
        );
    }

    #[test]
    fn the_layout_is_the_one_files_hold() {
        assert_eq!(head(), [b'W', b'S', b'S', b'D', 2, 0, 0, 0]);
        let set = change(&Holder::Player(2), "a:b", Some(&Value::Bool(true)));
        assert_eq!(
            set,
            [
                SET, PLAYER, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, b'a', b':', b'b', 2, 1
            ],
            "what, holder, component, value"
        );
        let removed = change(&Holder::Named("x".into()), "a:b", None);
        assert_eq!(
            removed,
            [
                REMOVE, NAMED, 1, 0, 0, 0, 0, 0, 0, 0, b'x', 3, 0, 0, 0, 0, 0, 0, 0, b'a', b':',
                b'b'
            ],
            "a name as text, and no value"
        );
        assert_eq!([SET, REMOVE], [0, 1], "changes keep their numbers");
        assert_eq!([PLAYER, NAMED], [0, 1], "and so do holders");
    }

    #[test]
    fn a_head_alone_holds_no_changes() {
        let read = parse(&head());
        assert!(read.whole && read.records.is_empty());
    }

    #[test]
    fn a_cut_off_end_keeps_the_changes_before_it() {
        let bytes = file(&changes());
        let read = parse(&bytes[..bytes.len() - 1]);
        assert!(!read.whole, "the last change is not all there");
        assert_eq!(read.records, records()[..2], "the two before it are");
    }

    #[test]
    fn another_version_is_not_read() {
        let mut bytes = file(&changes());
        bytes[4] = 1;
        let read = parse(&bytes);
        assert!(!read.whole && read.records.is_empty());
        assert!(!parse(b"").whole, "an empty file has no head");
    }

    #[test]
    fn an_unknown_change_or_holder_ends_the_reading() {
        for unknown in [[9, PLAYER], [SET, 9]] {
            let mut bytes = file(&changes());
            bytes.extend_from_slice(&unknown);
            bytes.extend_from_slice(&[0; 8]);
            let read = parse(&bytes);
            assert!(!read.whole);
            assert_eq!(read.records, records(), "what came before is kept");
        }
    }
}
