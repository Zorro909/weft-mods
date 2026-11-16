//! Who a player is, kept across reconnecting, respawning and restarting the server: the SDK's
//! `World::player_key` as a value that can be saved and sent, as `Pos` is for a position. The
//! SDK hands the key out as a `String`, which is neither a `Value` (entity and block components,
//! events, channels) nor fixed in size.

use weft_sdk::{World, value};

/// Bytes a key holds: the engine's keys are 64 hex digits.
const LEN: usize = 64;

value! {
    /// A player's persistent public identity, the one their saves and permissions go by. Keep
    /// this, not their name, to know a player again: two players may share a name, and one
    /// may change theirs. A server where a player has no identity keys them by name instead;
    /// such a key is that name. UTF-8, padded with zeros.
    #[derive(Eq, Hash)]
    pub struct PlayerKey {
        bytes: [u8; LEN],
    }
}

impl PlayerKey {
    /// The key of the player `entity`; `None` for an entity that is no player, and for one
    /// keyed by a name too long to keep.
    pub fn of(world: &World, entity: u64) -> Option<PlayerKey> {
        PlayerKey::new(&world.player_key(entity)?)
    }

    /// The key whose text is `text`, as `World::player_key` gives it; `None` for text no key
    /// can be: empty, over 64 bytes, or holding a zero byte.
    pub fn new(text: &str) -> Option<PlayerKey> {
        if text.is_empty() || text.len() > LEN || text.contains('\0') {
            return None;
        }
        let mut bytes = [0; LEN];
        bytes[..text.len()].copy_from_slice(text.as_bytes());
        Some(PlayerKey { bytes })
    }

    /// Its text, as `World::player_key` gives it; empty for bytes from a peer or a file that
    /// aren't text.
    pub fn as_str(&self) -> &str {
        let len = self.bytes.iter().position(|b| *b == 0).unwrap_or(LEN);
        std::str::from_utf8(&self.bytes[..len]).unwrap_or("")
    }
}

impl std::fmt::Display for PlayerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Saved in a `Table` as its text.
#[cfg(feature = "store")]
impl serde::Serialize for PlayerKey {
    fn serialize<S: serde::Serializer>(&self, to: S) -> Result<S::Ok, S::Error> {
        to.serialize_str(self.as_str())
    }
}

/// Read back from its text, refusing text no key can be.
#[cfg(feature = "store")]
impl<'de> serde::Deserialize<'de> for PlayerKey {
    fn deserialize<D: serde::Deserializer<'de>>(from: D) -> Result<PlayerKey, D::Error> {
        let text = String::deserialize(from)?;
        PlayerKey::new(&text)
            .ok_or_else(|| serde::de::Error::custom(format!("{text:?} is no player key")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";

    #[test]
    fn a_key_keeps_its_text() {
        let key = PlayerKey::new(KEY).expect("an engine's key fits");
        assert_eq!(key.as_str(), KEY, "the same text back");
        assert_eq!(key.to_string(), KEY, "shown as its text");
        let name = PlayerKey::new("Ann").expect("a short name fits");
        assert_eq!(name.as_str(), "Ann", "a name used as a key");
    }

    #[test]
    fn text_no_key_can_be_is_refused() {
        assert_eq!(PlayerKey::new(""), None, "empty");
        assert_eq!(PlayerKey::new(&"a".repeat(65)), None, "too long");
        assert_eq!(PlayerKey::new("an\0n"), None, "a zero byte");
        assert!(PlayerKey::new(&"é".repeat(32)).is_some(), "64 bytes fit");
    }

    #[test]
    fn keys_are_equal_only_by_their_whole_text() {
        let ann = PlayerKey::new("Ann");
        assert_eq!(ann, PlayerKey::new("Ann"), "the same player");
        assert_ne!(ann, PlayerKey::new("Anne"), "another player");
    }

    #[test]
    fn bytes_that_are_not_text_read_as_nothing() {
        let mut key = PlayerKey::new("Ann").expect("a short name fits");
        key.bytes[0] = 0xFF;
        assert_eq!(key.as_str(), "", "bad bytes from a peer");
    }

    #[test]
    fn it_is_saved_and_sent_as_its_bytes() {
        assert_eq!(
            std::mem::size_of::<PlayerKey>(),
            LEN,
            "no more than its text"
        );
        let key = PlayerKey::new(KEY).expect("an engine's key fits");
        assert_eq!(
            &key.bytes[..],
            KEY.as_bytes(),
            "the text from the first byte"
        );
    }

    #[cfg(feature = "store")]
    #[test]
    fn a_table_keeps_it_as_text() {
        let key = PlayerKey::new(KEY).expect("an engine's key fits");
        let saved = postcard::to_allocvec(&key).expect("written");
        assert_eq!(saved.len(), 1 + KEY.len(), "its length, then its text");
        let read: PlayerKey = postcard::from_bytes(&saved).expect("read back");
        assert_eq!(read, key, "the same key");
        let empty = postcard::to_allocvec("").expect("written");
        let refused = postcard::from_bytes::<PlayerKey>(&empty);
        assert!(refused.is_err(), "no key is empty");
    }
}
