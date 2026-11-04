//! A name in an event or packet, where text has to be a fixed size: `Name::from("mymod:tour")`,
//! and back with `<&str>::from(&name)`.

use weft_sdk::*;

/// Bytes a name holds; longer text is cut short.
const LEN: usize = 64;

value! {
    /// UTF-8, padded with zeros.
    pub struct Name {
        bytes: [u8; LEN],
    }
}

impl From<&str> for Name {
    /// `text`, cut short at 64 bytes on a character.
    fn from(text: &str) -> Name {
        let text = &text[..text.floor_char_boundary(LEN)];
        let mut bytes = [0; LEN];
        bytes[..text.len()].copy_from_slice(text.as_bytes());
        Name { bytes }
    }
}

impl<'a> From<&'a Name> for &'a str {
    fn from(name: &'a Name) -> &'a str {
        let len = name.bytes.iter().position(|b| *b == 0).unwrap_or(LEN);
        std::str::from_utf8(&name.bytes[..len]).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(name: &Name) -> &str {
        name.into()
    }

    #[test]
    fn names_survive_the_round_trip() {
        assert_eq!(text(&Name::from("cogwork:tubes")), "cogwork:tubes");
    }

    #[test]
    fn long_names_are_cut_on_a_character() {
        let long = "é".repeat(40);
        assert_eq!(text(&Name::from(long.as_str())), "é".repeat(32));
    }

    #[test]
    fn a_name_that_is_not_utf8_reads_as_nothing() {
        let mut name = Name::from("ok");
        name.bytes[0] = 0xFF;
        assert_eq!(text(&name), "", "bad bytes from a peer");
    }
}
