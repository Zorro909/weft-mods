//! The file format of one shard: a header, then records one after the other, each setting or
//! removing one key, and zeros to the end of the file. The newest record of a key counts.
//!
//! ```text
//! header  "WEFTKITS"  format: u32  0: u32
//! record  "WREC"  length: u32  checksum: u32 | kind: u8  key length: u32  key  value
//! ```
//!
//! Numbers are little-endian. `length` counts the bytes after the checksum, which is the CRC-32
//! of the record's place in the file (a u64) and those bytes: a record copied into a value, as
//! a player's text could hold, doesn't check out anywhere else. A record that doesn't check out
//! is skipped up to the next one that does, so damage in the middle of a file costs only the
//! records it hit.

use std::ops::Range;

/// The bytes before the first record.
pub(super) const HEADER_LEN: usize = 16;
/// What a store's file starts with.
const MAGIC: &[u8; 8] = b"WEFTKITS";
/// The version of this layout: a file of another is read only as far as its records check out.
const FORMAT: u32 = 1;
/// What each record starts with, where the search for one goes on after damage.
const MARKER: &[u8; 4] = b"WREC";
/// The marker, length and checksum.
const PREFIX_LEN: usize = 12;
/// The kind and the key's length, after the prefix.
const HEAD_LEN: usize = 5;
/// Where a record's key starts, from the record's start.
pub(super) const KEY_START: usize = PREFIX_LEN + HEAD_LEN;
/// The kind of a record giving its key a value.
const SET: u8 = 1;
/// The kind of a record taking its key out.
const REMOVE: u8 = 2;

/// The header of a new file.
pub(super) fn header() -> [u8; HEADER_LEN] {
    let mut header = [0; HEADER_LEN];
    header[..8].copy_from_slice(MAGIC);
    header[8..12].copy_from_slice(&FORMAT.to_le_bytes());
    header
}

/// The format `bytes`, a whole file, says it has, if that is one this code doesn't know: a newer
/// version of the store wrote it, and rewriting it here would lose what that version keeps.
pub(super) fn unknown_format(bytes: &[u8]) -> Option<u32> {
    if !bytes.starts_with(MAGIC) || bytes.len() < HEADER_LEN {
        return None;
    }
    Some(u32_at(bytes, MAGIC.len())).filter(|format| *format != FORMAT)
}

/// The bytes a record of `key` and `value` takes; a removal has no value.
pub(super) fn encoded_len(key: &[u8], value: Option<&[u8]>) -> usize {
    PREFIX_LEN + HEAD_LEN + key.len() + value.map_or(0, <[u8]>::len)
}

/// Whether a record of a key and a value this long can be written: its length is a u32.
pub(super) fn fits(key_len: usize, value_len: usize) -> bool {
    HEAD_LEN.saturating_add(key_len).saturating_add(value_len) <= u32::MAX as usize
}

/// Writes the record of `key` and `value` at the start of `out`, which is long enough and lies
/// at `start` of the file.
pub(super) fn write(out: &mut [u8], start: usize, key: &[u8], value: Option<&[u8]>) {
    let len = encoded_len(key, value);
    let body = &mut out[PREFIX_LEN..len];
    body[0] = if value.is_some() { SET } else { REMOVE };
    body[1..HEAD_LEN].copy_from_slice(&(key.len() as u32).to_le_bytes());
    body[HEAD_LEN..HEAD_LEN + key.len()].copy_from_slice(key);
    if let Some(value) = value {
        body[HEAD_LEN + key.len()..].copy_from_slice(value);
    }
    let checksum = checksum(start, body);
    out[..4].copy_from_slice(MARKER);
    out[4..8].copy_from_slice(&((len - PREFIX_LEN) as u32).to_le_bytes());
    out[8..PREFIX_LEN].copy_from_slice(&checksum.to_le_bytes());
}

/// One record found in a file, by where its parts lie.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Record {
    pub(super) start: usize,
    pub(super) len: usize,
    pub(super) key: Range<usize>,
    /// `None` for a removal.
    pub(super) value: Option<Range<usize>>,
}

/// The whole, checked record at `start` of `bytes`, if there is one.
pub(super) fn read(bytes: &[u8], start: usize) -> Option<Record> {
    let prefix = bytes.get(start..start.checked_add(PREFIX_LEN)?)?;
    if &prefix[..4] != MARKER {
        return None;
    }
    let body_len = u32_at(prefix, 4) as usize;
    let checksum = u32_at(prefix, 8);
    let body_start = start + PREFIX_LEN;
    let body = bytes.get(body_start..body_start.checked_add(body_len)?)?;
    if body_len < HEAD_LEN || self::checksum(start, body) != checksum {
        return None;
    }
    let key_len = u32_at(body, 1) as usize;
    let key_start = body_start + HEAD_LEN;
    let value_start = key_start.checked_add(key_len)?;
    let end = body_start + body_len;
    let value = match body[0] {
        SET if value_start <= end => Some(value_start..end),
        REMOVE if value_start == end => None,
        _ => return None,
    };
    Some(Record {
        start,
        len: PREFIX_LEN + body_len,
        key: key_start..value_start,
        value,
    })
}

/// What reading a whole file found.
#[derive(Debug, Default, PartialEq)]
pub(super) struct Scan {
    pub(super) records: Vec<Record>,
    /// Where the records end and new ones go.
    pub(super) end: usize,
    /// Bytes that were neither header, record nor the zeros after the records.
    pub(super) damaged: usize,
}

/// Every record of `bytes`, a whole file, skipping what doesn't check out.
pub(super) fn scan(bytes: &[u8]) -> Scan {
    if bytes.iter().all(|byte| *byte == 0) {
        return Scan::default();
    }
    let mut scan = Scan::default();
    if !bytes.starts_with(&header()) {
        scan.damaged = HEADER_LEN.min(bytes.len());
    }
    let mut start = HEADER_LEN;
    while start < bytes.len() {
        if let Some(record) = read(bytes, start) {
            start += record.len;
            scan.records.push(record);
        } else if bytes[start..].iter().all(|byte| *byte == 0) {
            break;
        } else {
            let next = next_record(bytes, start + 1);
            scan.damaged += damage_end(bytes, next) - start;
            start = next;
        }
    }
    scan.end = start.min(bytes.len());
    scan
}

/// The start of the first whole record at or after `from`, or the end of `bytes`.
fn next_record(bytes: &[u8], from: usize) -> usize {
    (from..bytes.len())
        .find(|start| bytes[*start..].starts_with(MARKER) && read(bytes, *start).is_some())
        .unwrap_or(bytes.len())
}

/// Where damage reaching `next` ends: there, or at the last byte that isn't zero if no record
/// follows it, since the zeros after it are only room.
fn damage_end(bytes: &[u8], next: usize) -> usize {
    if next < bytes.len() {
        return next;
    }
    bytes
        .iter()
        .rposition(|byte| *byte != 0)
        .map_or(0, |last| last + 1)
}

/// The checksum of the record at `start` with `body`.
fn checksum(start: usize, body: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&(start as u64).to_le_bytes());
    hasher.update(body);
    hasher.finalize()
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    let mut word = [0; 4];
    word.copy_from_slice(&bytes[at..at + 4]);
    u32::from_le_bytes(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file of the header and records of `changes`, then `zeros` zero bytes.
    fn file(changes: &[(&[u8], Option<&[u8]>)], zeros: usize) -> Vec<u8> {
        let mut bytes = header().to_vec();
        for (key, value) in changes {
            let start = bytes.len();
            bytes.resize(start + encoded_len(key, *value), 0);
            write(&mut bytes[start..], start, key, *value);
        }
        bytes.resize(bytes.len() + zeros, 0);
        bytes
    }

    fn key_and_value<'a>(bytes: &'a [u8], record: &Record) -> (&'a [u8], Option<&'a [u8]>) {
        let value = record.value.clone().map(|range| &bytes[range]);
        (&bytes[record.key.clone()], value)
    }

    #[test]
    fn records_read_back_as_written() {
        let bytes = file(
            &[(b"k", Some(b"value")), (b"gone", None), (b"", Some(b""))],
            8,
        );
        let scan = scan(&bytes);
        let read: Vec<_> = scan
            .records
            .iter()
            .map(|record| key_and_value(&bytes, record))
            .collect();
        let expected: [(&[u8], Option<&[u8]>); 3] =
            [(b"k", Some(b"value")), (b"gone", None), (b"", Some(b""))];
        assert_eq!(read, expected, "keys, values and removals come back");
        assert_eq!(scan.end, bytes.len() - 8, "new records go before the zeros");
        assert_eq!(scan.damaged, 0, "nothing is damaged");
    }

    #[test]
    fn a_file_of_zeros_is_empty() {
        assert_eq!(scan(&[0; 64]), Scan::default(), "no records and no damage");
        assert_eq!(scan(&[]), Scan::default(), "nor in no bytes");
    }

    #[test]
    fn a_damaged_record_costs_only_itself() {
        let mut bytes = file(
            &[(b"a", Some(b"1")), (b"b", Some(b"2")), (b"c", Some(b"3"))],
            4,
        );
        let second = HEADER_LEN + encoded_len(b"a", Some(b"1"));
        bytes[second + PREFIX_LEN + HEAD_LEN] ^= 0xff;
        let scan = scan(&bytes);
        let keys: Vec<_> = scan
            .records
            .iter()
            .map(|record| &bytes[record.key.clone()])
            .collect();
        assert_eq!(keys, [b"a", b"c"], "the records around it are read");
        assert_eq!(
            scan.damaged,
            encoded_len(b"b", Some(b"2")),
            "the damage is one record"
        );
    }

    #[test]
    fn a_cut_off_record_at_the_end_is_damage() {
        let whole = file(&[(b"a", Some(b"1")), (b"b", Some(b"22"))], 0);
        let mut bytes = whole[..whole.len() - 1].to_vec();
        bytes.extend([0; 8]);
        let scan = scan(&bytes);
        assert_eq!(scan.records.len(), 1, "the whole record is read");
        let cut_off = encoded_len(b"b", Some(b"22")) - 1;
        assert_eq!(
            scan.damaged, cut_off,
            "the cut-off one is damage, the zeros after it aren't"
        );
        assert_eq!(scan.end, bytes.len(), "nothing whole follows it");
    }

    #[test]
    fn a_wrong_header_is_damage_but_its_records_are_read() {
        let mut bytes = file(&[(b"a", Some(b"1"))], 0);
        bytes[0] = b'X';
        let scan = scan(&bytes);
        assert_eq!(scan.records.len(), 1, "the record is read");
        assert_eq!(scan.damaged, HEADER_LEN, "the header is the damage");
    }

    #[test]
    fn zeros_inside_the_records_are_damage() {
        let mut bytes = file(&[(b"a", Some(b"1")), (b"b", Some(b"2"))], 0);
        bytes[HEADER_LEN..HEADER_LEN + 4].fill(0);
        let scan = scan(&bytes);
        assert_eq!(scan.records.len(), 1, "the record after the zeros is read");
        assert_eq!(
            scan.damaged,
            encoded_len(b"a", Some(b"1")),
            "the zeroed record is damage"
        );
    }

    #[test]
    fn a_record_that_lies_about_its_parts_is_refused() {
        let mut bytes = file(&[(b"key", None)], 0);
        assert!(read(&bytes, HEADER_LEN).is_some(), "the removal reads");
        let body = HEADER_LEN + PREFIX_LEN;
        let lie = |at: usize, byte: u8| {
            let mut copy = bytes.clone();
            copy[at] = byte;
            let checksum = checksum(HEADER_LEN, &copy[body..]);
            copy[HEADER_LEN + 8..body].copy_from_slice(&checksum.to_le_bytes());
            read(&copy, HEADER_LEN)
        };
        assert_eq!(lie(body, 9), None, "an unknown kind");
        assert_eq!(lie(body + 1, 2), None, "a removal with a value");
        assert_eq!(lie(body + 1, 9), None, "a key longer than the record");
        bytes.truncate(HEADER_LEN + 4);
        assert_eq!(
            read(&bytes, HEADER_LEN),
            None,
            "a record cut off in its prefix"
        );
    }

    #[test]
    fn a_record_hidden_in_a_value_is_not_taken_after_damage() {
        let mut hidden = vec![0; encoded_len(b"admin", Some(b"yes"))];
        write(&mut hidden, HEADER_LEN, b"admin", Some(b"yes"));
        let mut bytes = file(&[(b"name", Some(&hidden))], 8);
        bytes[HEADER_LEN..HEADER_LEN + PREFIX_LEN].fill(0);
        let scan = scan(&bytes);
        assert_eq!(
            scan.records,
            [],
            "the copy checks out only where it was made for"
        );
        assert!(scan.damaged > 0, "the record without its prefix is damage");
    }

    #[test]
    fn a_record_longer_than_a_u32_does_not_fit() {
        assert!(fits(16, 1024), "a small record fits");
        assert!(!fits(0, u32::MAX as usize), "four gibibytes don't");
        assert!(!fits(usize::MAX, 1), "nor does a length that overflows");
    }

    #[test]
    fn only_a_format_this_code_does_not_know_is_unknown() {
        assert_eq!(unknown_format(&header()), None, "this format is known");
        let mut newer = header();
        newer[8] = 2;
        assert_eq!(unknown_format(&newer), Some(2), "a newer one isn't");
        assert_eq!(
            unknown_format(b"garbage that is long enough"),
            None,
            "garbage is damage"
        );
        assert_eq!(unknown_format(MAGIC), None, "and so is a header cut off");
    }
}
