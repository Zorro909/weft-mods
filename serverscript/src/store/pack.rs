//! How text and a value lie in the store's file.
//!
//! | Part | Bytes |
//! |---|---|
//! | text | its length as a `u64`, then UTF-8 |
//! | value | a tag as a `u8`, then what the tag says |
//!
//! Numbers are little-endian. A length is a `u64`, so nothing a script holds is too long to
//! write. The tags are part of the file format: a new one goes last.

use crate::value::{MAX_DEPTH, Value};

/// A whole number: an `i64`.
const INT: u8 = 0;
/// A number with a fraction: an `f64`.
const FLOAT: u8 = 1;
/// Yes or no: a `u8`, 0 for no.
const BOOL: u8 = 2;
/// Text.
const TEXT: u8 = 3;
/// A list: its count as a `u64`, then that many values.
const LIST: u8 = 4;
/// A map: its count as a `u64`, then that many pairs of a name as text and a value.
const MAP: u8 = 5;

/// Puts `text` at the end of `bytes`.
pub fn write_text(text: &str, bytes: &mut Vec<u8>) {
    write_length(text.len(), bytes);
    bytes.extend_from_slice(text.as_bytes());
}

/// Puts `value` at the end of `bytes`.
pub fn write_value(value: &Value, bytes: &mut Vec<u8>) {
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
            write_text(text, bytes);
        }
        Value::List(values) => {
            bytes.push(LIST);
            write_length(values.len(), bytes);
            values.iter().for_each(|value| write_value(value, bytes));
        }
        Value::Map(entries) => {
            bytes.push(MAP);
            write_length(entries.len(), bytes);
            for (name, value) in entries {
                write_text(name, bytes);
                write_value(value, bytes);
            }
        }
    }
}

fn write_length(length: usize, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&(length as u64).to_le_bytes());
}

/// The text at the front of `bytes`, taken off; `None` for bytes `write_text` didn't write.
pub fn read_text(bytes: &mut &[u8]) -> Option<String> {
    let length = read_length(bytes)?;
    String::from_utf8(take(bytes, length)?.to_vec()).ok()
}

/// The value at the front of `bytes`, taken off; `None` for bytes `write_value` didn't write,
/// lists and maps deeper than a value goes among them.
pub fn read_value(bytes: &mut &[u8]) -> Option<Value> {
    read_nested(bytes, MAX_DEPTH)
}

/// `read_value`, with `deeper` more levels of lists and maps allowed.
fn read_nested(bytes: &mut &[u8], deeper: usize) -> Option<Value> {
    match take(bytes, 1)?[0] {
        INT => Some(Value::Int(i64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        FLOAT => Some(Value::Float(f64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        BOOL => Some(Value::Bool(take(bytes, 1)?[0] != 0)),
        TEXT => read_text(bytes).map(Value::Text),
        LIST => {
            let (count, deeper) = (read_length(bytes)?, deeper.checked_sub(1)?);
            let values = (0..count).map(|_| read_nested(bytes, deeper));
            values.collect::<Option<_>>().map(Value::List)
        }
        MAP => {
            let (count, deeper) = (read_length(bytes)?, deeper.checked_sub(1)?);
            let entry = |_| Some((read_text(bytes)?, read_nested(bytes, deeper)?));
            (0..count).map(entry).collect::<Option<_>>().map(Value::Map)
        }
        _ => None,
    }
}

/// The next `count` bytes, taken off the front.
pub fn take<'a>(bytes: &mut &'a [u8], count: usize) -> Option<&'a [u8]> {
    let (head, rest) = bytes.split_at_checked(count)?;
    *bytes = rest;
    Some(head)
}

fn read_length(bytes: &mut &[u8]) -> Option<usize> {
    let raw = take(bytes, 8)?;
    usize::try_from(u64::from_le_bytes(raw.try_into().ok()?)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        let map = [("returns".to_string(), Value::Int(2))];
        Value::List(vec![
            Value::Int(-12),
            Value::Float(0.25),
            Value::Bool(true),
            Value::Text("Bürger".to_string()),
            Value::List(vec![Value::Int(4), Value::List(Vec::new())]),
            Value::Map(map.into()),
        ])
    }

    fn written(value: &Value) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_value(value, &mut bytes);
        bytes
    }

    #[test]
    fn a_value_comes_back_as_written() {
        let bytes = written(&sample());
        let mut rest = bytes.as_slice();
        assert_eq!(read_value(&mut rest), Some(sample()));
        assert!(rest.is_empty(), "nothing is left over");
    }

    #[test]
    fn text_comes_back_as_written() {
        let mut bytes = Vec::new();
        write_text("käse", &mut bytes);
        bytes.push(7);
        let mut rest = bytes.as_slice();
        assert_eq!(read_text(&mut rest).as_deref(), Some("käse"));
        assert_eq!(rest, [7], "what follows stays");
    }

    #[test]
    fn the_layout_is_the_one_files_hold() {
        assert_eq!(written(&Value::Int(1)), [INT, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            written(&Value::Text("a".into())),
            [TEXT, 1, 0, 0, 0, 0, 0, 0, 0, b'a'],
            "a length is eight bytes"
        );
        assert_eq!(
            written(&Value::List(vec![Value::Bool(true)])),
            [LIST, 1, 0, 0, 0, 0, 0, 0, 0, BOOL, 1]
        );
        let map = [("a".to_string(), Value::Bool(false))];
        assert_eq!(
            written(&Value::Map(map.into())),
            [
                MAP, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, b'a', BOOL, 0
            ],
            "a count, then each name and its value"
        );
        assert_eq!(
            [INT, FLOAT, BOOL, TEXT, LIST, MAP],
            [0, 1, 2, 3, 4, 5],
            "tags keep their numbers"
        );
    }

    #[test]
    fn cut_off_bytes_are_refused() {
        let bytes = written(&sample());
        for length in 0..bytes.len() {
            let mut cut = &bytes[..length];
            assert_eq!(read_value(&mut cut), None, "{length} bytes are not all");
        }
    }

    #[test]
    fn a_count_larger_than_the_bytes_is_refused() {
        let mut huge = [LIST, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0, INT].as_slice();
        assert_eq!(read_value(&mut huge), None, "the list ends early");
    }

    #[test]
    fn lists_deeper_than_a_value_goes_are_refused() {
        let one_inside = [LIST, 1, 0, 0, 0, 0, 0, 0, 0];
        let nested = |levels: usize| [one_inside.repeat(levels), vec![BOOL, 1]].concat();
        assert!(read_value(&mut nested(MAX_DEPTH).as_slice()).is_some());
        assert_eq!(read_value(&mut nested(MAX_DEPTH + 1).as_slice()), None);
        let endless = nested(1_000_000);
        assert_eq!(
            read_value(&mut endless.as_slice()),
            None,
            "without using up the stack"
        );
    }

    #[test]
    fn an_unknown_tag_is_refused() {
        assert_eq!(read_value(&mut [99].as_slice()), None);
    }

    #[test]
    fn text_that_is_not_text_is_refused() {
        assert_eq!(
            read_text(&mut [1, 0, 0, 0, 0, 0, 0, 0, 0xFF].as_slice()),
            None
        );
    }
}
