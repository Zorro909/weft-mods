//! The store's file: every change written behind the last, and read back in that order.
//!
//! | Part | Bytes |
//! |---|---|
//! | head | `MAGIC`, then `VERSION` as a `u32` |
//! | change | what it does as a `u8`, its owner as a `u64`, its data key, its key, and a value unless it removes |
//!
//! Numbers are little-endian; a data key, a key and a value lie as `pack` says. A change costs its own few
//! bytes however much the store holds, and is with the system once made, so stopping the game
//! loses none. The file is written anew from what the store holds at each start (`start`), or
//! it would grow with every change ever made.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::Path;

use super::Place;
use super::pack::{read_key, read_value, take, write_key, write_value};
use crate::value::Value;

/// What a store file starts with.
const MAGIC: [u8; 4] = *b"WSSD";
/// The layout of the changes. A file of another version is not read.
const VERSION: u32 = 1;

/// Puts a value under a key, replacing what was there.
const SET: u8 = 0;
/// Puts a value at the end of the list under a key, starting one.
const APPEND: u8 = 1;
/// Takes a key away.
const REMOVE: u8 = 2;

/// What a change does to its key.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    Set(Value),
    Append(Value),
    Remove,
}

/// One change read back from the file.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub owner: u64,
    pub component: String,
    pub key: String,
    pub change: Change,
}

/// A file's changes in the order made. `whole` is false when it holds more than could be
/// read: another version's file, or an end cut off in the middle of a change.
#[derive(Debug, PartialEq)]
pub struct Read {
    pub records: Vec<Record>,
    pub whole: bool,
}

/// The bytes of the change that sets `place` to `value`; `None` when a name or the value is
/// too long to write.
pub fn set(place: Place, value: &Value) -> Option<Vec<u8>> {
    written(SET, place, Some(value))
}

/// The bytes of the change that appends `item` to the list at `place`.
pub fn append(place: Place, item: &Value) -> Option<Vec<u8>> {
    written(APPEND, place, Some(item))
}

/// The bytes of the change that removes the key at `place`.
pub fn remove(place: Place) -> Option<Vec<u8>> {
    written(REMOVE, place, None)
}

fn written(what: u8, place: Place, value: Option<&Value>) -> Option<Vec<u8>> {
    let mut bytes = vec![what];
    bytes.extend_from_slice(&place.owner.to_le_bytes());
    write_key(place.component, &mut bytes)?;
    write_key(place.key, &mut bytes)?;
    if let Some(value) = value {
        write_value(value, &mut bytes)?;
    }
    Some(bytes)
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
    let owner = u64::from_le_bytes(take(bytes, 8)?.try_into().ok()?);
    let component = read_key(bytes)?;
    let key = read_key(bytes)?;
    let change = match what {
        SET => Change::Set(read_value(bytes)?),
        APPEND => Change::Append(read_value(bytes)?),
        REMOVE => Change::Remove,
        _ => return None,
    };
    Some(Record {
        owner,
        component,
        key,
        change,
    })
}

/// Writes the file at `path` anew, holding `changes`, and opens it for the changes to come.
/// Whole or not at all: the new file takes the old one's place only once it is written.
pub fn start(path: &Path, changes: &[u8]) -> io::Result<File> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder)?;
    }
    let new = path.with_extension("new");
    fs::write(&new, [head().as_slice(), changes].concat())?;
    fs::rename(&new, path)?;
    OpenOptions::new().append(true).open(path)
}

/// Puts `change` at the file's end.
pub fn write(file: &mut File, change: &[u8]) -> io::Result<()> {
    file.write_all(change)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place<'a>(owner: u64, component: &'a str, key: &'a str) -> Place<'a> {
        Place {
            owner,
            component,
            key,
        }
    }

    fn changes() -> Vec<u8> {
        let list = Value::List(vec![Value::Int(1)]);
        [
            set(place(0, "welcome:stats", "visits"), &Value::Int(3)),
            append(place(0, "welcome:log", "lines"), &Value::Text("hi".into())),
            set(place(7, "camps:camp", "at"), &list),
            remove(place(7, "camps:camp", "at")),
        ]
        .into_iter()
        .flat_map(|change| change.expect("the change is short enough"))
        .collect()
    }

    fn records() -> Vec<Record> {
        let record = |owner, component: &str, key: &str, change| Record {
            owner,
            component: component.to_string(),
            key: key.to_string(),
            change,
        };
        let list = Value::List(vec![Value::Int(1)]);
        vec![
            record(0, "welcome:stats", "visits", Change::Set(Value::Int(3))),
            record(
                0,
                "welcome:log",
                "lines",
                Change::Append(Value::Text("hi".into())),
            ),
            record(7, "camps:camp", "at", Change::Set(list)),
            record(7, "camps:camp", "at", Change::Remove),
        ]
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
        assert_eq!(head(), [b'W', b'S', b'S', b'D', 1, 0, 0, 0]);
        let change =
            set(place(2, "a:b", "c"), &Value::Bool(true)).expect("the change is short enough");
        assert_eq!(
            change,
            [
                SET, 2, 0, 0, 0, 0, 0, 0, 0, 3, b'a', b':', b'b', 1, b'c', 2, 1
            ],
            "what, owner, data key, key, value"
        );
        assert_eq!(
            [SET, APPEND, REMOVE],
            [0, 1, 2],
            "changes keep their numbers"
        );
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
        assert_eq!(read.records, records()[..3], "the three before it are");
    }

    #[test]
    fn another_version_is_not_read() {
        let mut bytes = file(&changes());
        bytes[4] = 2;
        let read = parse(&bytes);
        assert!(!read.whole && read.records.is_empty());
        assert!(!parse(b"").whole, "an empty file has no head");
    }

    #[test]
    fn an_unknown_change_ends_the_reading() {
        let mut bytes = file(&changes());
        bytes.extend_from_slice(&[9, 0, 0, 0, 0, 0, 0, 0, 0, 1, b'a', 1, b'b']);
        let read = parse(&bytes);
        assert!(!read.whole);
        assert_eq!(read.records, records(), "what came before is kept");
    }

    #[test]
    fn a_name_too_long_makes_no_change() {
        let long = "k".repeat(256);
        assert_eq!(set(place(0, "a:b", &long), &Value::Int(1)), None);
        assert_eq!(set(place(0, &long, "c"), &Value::Int(1)), None);
        assert_eq!(remove(place(0, "a:b", &long)), None);
    }
}
