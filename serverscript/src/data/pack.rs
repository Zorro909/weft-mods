//! How an entity's script data lies in its component's bytes: one entry after another, each a
//! key and a value.
//!
//! | Part | Bytes |
//! |---|---|
//! | key | its length as a `u8`, then UTF-8 |
//! | value | a tag as a `u8`, then what the tag says |
//!
//! Numbers are little-endian. The tags are part of the save format: a new one goes last.

use std::collections::BTreeMap;

use crate::value::Value;

/// An entity's script data, by key.
pub type Entries = BTreeMap<String, Value>;

/// A whole number: an `i64`.
const INT: u8 = 0;
/// A number with a fraction: an `f64`.
const FLOAT: u8 = 1;
/// Yes or no: a `u8`, 0 for no.
const BOOL: u8 = 2;
/// Text: its length as a `u16`, then UTF-8.
const TEXT: u8 = 3;
/// A list: its count as a `u16`, then that many values.
const LIST: u8 = 4;

/// `entries` as bytes; `None` when a key, a text or a list is longer than its length can say.
pub fn pack(entries: &Entries) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    for (key, value) in entries {
        bytes.push(u8::try_from(key.len()).ok()?);
        bytes.extend_from_slice(key.as_bytes());
        pack_value(value, &mut bytes)?;
    }
    Some(bytes)
}

fn pack_value(value: &Value, bytes: &mut Vec<u8>) -> Option<()> {
    match value {
        Value::Int(number) => {
            bytes.push(INT);
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        Value::Float(number) => {
            bytes.push(FLOAT);
            bytes.extend_from_slice(&number.to_le_bytes());
        }
        Value::Bool(flag) => bytes.extend_from_slice(&[BOOL, u8::from(*flag)]),
        Value::Text(text) => {
            bytes.push(TEXT);
            bytes.extend_from_slice(&u16::try_from(text.len()).ok()?.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        Value::List(values) => {
            bytes.push(LIST);
            bytes.extend_from_slice(&u16::try_from(values.len()).ok()?.to_le_bytes());
            for value in values {
                pack_value(value, bytes)?;
            }
        }
    }
    Some(())
}

/// The entries `pack` wrote; `None` for bytes it didn't write.
pub fn unpack(mut bytes: &[u8]) -> Option<Entries> {
    let mut entries = Entries::new();
    while !bytes.is_empty() {
        let key_length = usize::from(take(&mut bytes, 1)?[0]);
        let key = text(take(&mut bytes, key_length)?)?;
        entries.insert(key, unpack_value(&mut bytes)?);
    }
    Some(entries)
}

fn unpack_value(bytes: &mut &[u8]) -> Option<Value> {
    match take(bytes, 1)?[0] {
        INT => Some(Value::Int(i64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        FLOAT => Some(Value::Float(f64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        BOOL => Some(Value::Bool(take(bytes, 1)?[0] != 0)),
        TEXT => {
            let length = short(bytes)?;
            text(take(bytes, length)?).map(Value::Text)
        }
        LIST => {
            let count = short(bytes)?;
            let values: Option<Vec<Value>> = (0..count).map(|_| unpack_value(bytes)).collect();
            values.map(Value::List)
        }
        _ => None,
    }
}

/// The next `count` bytes, taken off the front.
fn take<'a>(bytes: &mut &'a [u8], count: usize) -> Option<&'a [u8]> {
    let (head, rest) = bytes.split_at_checked(count)?;
    *bytes = rest;
    Some(head)
}

/// A `u16` length or count, taken off the front.
fn short(bytes: &mut &[u8]) -> Option<usize> {
    let raw = take(bytes, 2)?;
    Some(usize::from(u16::from_le_bytes([raw[0], raw[1]])))
}

fn text(bytes: &[u8]) -> Option<String> {
    String::from_utf8(bytes.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Entries {
        Entries::from([
            ("coins".to_string(), Value::Int(-12)),
            ("heat".to_string(), Value::Float(0.25)),
            ("vip".to_string(), Value::Bool(true)),
            ("rank".to_string(), Value::Text("Bürger".to_string())),
            (
                "home".to_string(),
                Value::List(vec![Value::Int(4), Value::List(Vec::new())]),
            ),
        ])
    }

    #[test]
    fn entries_come_back_as_packed() {
        let bytes = pack(&sample()).expect("the sample fits its lengths");
        assert_eq!(unpack(&bytes), Some(sample()));
    }

    #[test]
    fn no_bytes_are_no_entries() {
        assert_eq!(unpack(&[]), Some(Entries::new()));
    }

    #[test]
    fn the_layout_is_the_one_saves_hold() {
        let entries = Entries::from([("a".to_string(), Value::Int(1))]);
        let bytes = pack(&entries).expect("one small entry packs");
        assert_eq!(
            bytes,
            [1, b'a', INT, 1, 0, 0, 0, 0, 0, 0, 0],
            "key, tag, little-endian"
        );
        assert_eq!(
            [INT, FLOAT, BOOL, TEXT, LIST],
            [0, 1, 2, 3, 4],
            "tags keep their numbers"
        );
    }

    #[test]
    fn cut_off_bytes_are_refused() {
        let bytes = pack(&sample()).expect("the sample fits its lengths");
        for length in 1..bytes.len() {
            let cut = unpack(&bytes[..length]);
            assert_ne!(cut, Some(sample()), "{length} bytes are not the whole");
        }
        assert_eq!(
            unpack(&[1, b'a', INT, 1]),
            None,
            "a number needs its eight bytes"
        );
    }

    #[test]
    fn an_unknown_tag_is_refused() {
        assert_eq!(unpack(&[1, b'a', 99]), None);
    }

    #[test]
    fn a_key_that_is_not_text_is_refused() {
        assert_eq!(unpack(&[1, 0xFF, BOOL, 1]), None);
    }

    #[test]
    fn a_key_too_long_for_its_length_does_not_pack() {
        let entries = Entries::from([("k".repeat(256), Value::Int(1))]);
        assert_eq!(pack(&entries), None);
    }
}
