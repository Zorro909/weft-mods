//! How a key and a value lie in the store's file.
//!
//! | Part | Bytes |
//! |---|---|
//! | key | its length as a `u8`, then UTF-8 |
//! | value | a tag as a `u8`, then what the tag says |
//!
//! Numbers are little-endian. The tags are part of the file format: a new one goes last.

use crate::value::Value;

/// A whole number: an `i64`.
const INT: u8 = 0;
/// A number with a fraction: an `f64`.
const FLOAT: u8 = 1;
/// Yes or no: a `u8`, 0 for no.
const BOOL: u8 = 2;
/// Text: its length as a `u32`, then UTF-8.
const TEXT: u8 = 3;
/// A list: its count as a `u32`, then that many values.
const LIST: u8 = 4;

/// Puts `key` at the end of `bytes`; `None` when it is longer than the 255 bytes its length
/// can say.
pub fn write_key(key: &str, bytes: &mut Vec<u8>) -> Option<()> {
    bytes.push(u8::try_from(key.len()).ok()?);
    bytes.extend_from_slice(key.as_bytes());
    Some(())
}

/// Puts `value` at the end of `bytes`; `None` when a text or a list is longer than a `u32` says.
pub fn write_value(value: &Value, bytes: &mut Vec<u8>) -> Option<()> {
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
            bytes.extend_from_slice(&u32::try_from(text.len()).ok()?.to_le_bytes());
            bytes.extend_from_slice(text.as_bytes());
        }
        Value::List(values) => {
            bytes.push(LIST);
            bytes.extend_from_slice(&u32::try_from(values.len()).ok()?.to_le_bytes());
            for value in values {
                write_value(value, bytes)?;
            }
        }
    }
    Some(())
}

/// The key at the front of `bytes`, taken off; `None` for bytes `write_key` didn't write.
pub fn read_key(bytes: &mut &[u8]) -> Option<String> {
    let length = usize::from(take(bytes, 1)?[0]);
    text(take(bytes, length)?)
}

/// The value at the front of `bytes`, taken off; `None` for bytes `write_value` didn't write.
pub fn read_value(bytes: &mut &[u8]) -> Option<Value> {
    match take(bytes, 1)?[0] {
        INT => Some(Value::Int(i64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        FLOAT => Some(Value::Float(f64::from_le_bytes(
            take(bytes, 8)?.try_into().ok()?,
        ))),
        BOOL => Some(Value::Bool(take(bytes, 1)?[0] != 0)),
        TEXT => {
            let length = long(bytes)?;
            text(take(bytes, length)?).map(Value::Text)
        }
        LIST => {
            let count = long(bytes)?;
            let values: Option<Vec<Value>> = (0..count).map(|_| read_value(bytes)).collect();
            values.map(Value::List)
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

/// A `u32` length or count, taken off the front.
fn long(bytes: &mut &[u8]) -> Option<usize> {
    let raw = take(bytes, 4)?;
    usize::try_from(u32::from_le_bytes(raw.try_into().ok()?)).ok()
}

fn text(bytes: &[u8]) -> Option<String> {
    String::from_utf8(bytes.to_vec()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        Value::List(vec![
            Value::Int(-12),
            Value::Float(0.25),
            Value::Bool(true),
            Value::Text("Bürger".to_string()),
            Value::List(vec![Value::Int(4), Value::List(Vec::new())]),
        ])
    }

    fn written(value: &Value) -> Vec<u8> {
        let mut bytes = Vec::new();
        write_value(value, &mut bytes).expect("the value fits its lengths");
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
    fn a_key_comes_back_as_written() {
        let mut bytes = Vec::new();
        write_key("käse", &mut bytes).expect("a short key fits");
        bytes.push(7);
        let mut rest = bytes.as_slice();
        assert_eq!(read_key(&mut rest).as_deref(), Some("käse"));
        assert_eq!(rest, [7], "what follows stays");
    }

    #[test]
    fn the_layout_is_the_one_files_hold() {
        assert_eq!(written(&Value::Int(1)), [INT, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            written(&Value::Text("a".into())),
            [TEXT, 1, 0, 0, 0, b'a'],
            "a text's length is four bytes"
        );
        assert_eq!(
            written(&Value::List(vec![Value::Bool(true)])),
            [LIST, 1, 0, 0, 0, BOOL, 1]
        );
        assert_eq!(
            [INT, FLOAT, BOOL, TEXT, LIST],
            [0, 1, 2, 3, 4],
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
        let mut huge = [LIST, 0xFF, 0xFF, 0xFF, 0xFF, INT].as_slice();
        assert_eq!(read_value(&mut huge), None, "the list ends early");
    }

    #[test]
    fn an_unknown_tag_is_refused() {
        assert_eq!(read_value(&mut [99].as_slice()), None);
    }

    #[test]
    fn a_key_that_is_not_text_is_refused() {
        assert_eq!(read_key(&mut [1, 0xFF].as_slice()), None);
    }

    #[test]
    fn a_key_too_long_for_its_length_is_not_written() {
        let mut bytes = Vec::new();
        assert_eq!(write_key(&"k".repeat(256), &mut bytes), None);
        assert_eq!(write_key(&"k".repeat(255), &mut bytes), Some(()));
    }
}
