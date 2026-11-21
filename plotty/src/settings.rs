//! The settings file the server's admin edits, `config/plotty/settings.toml`, read once when the
//! mod loads. Its default, with comments, is `config/settings.toml` in the mod.

use serde::Deserialize;
use weftkit::ConfigFile;

pub static SETTINGS: ConfigFile<Settings> =
    ConfigFile::named("settings", include_str!("../config/settings.toml"));

/// Who may build outside cities and the spawn.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Wilderness {
    Open,
    Closed,
}

/// Plotty's settings; a key missing from an older file keeps its default.
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(default)]
pub struct Settings {
    /// Golden Woof a player gets the first time they join.
    pub starting_woof: u64,
    /// Normal plots a player may claim, on top of one small plot.
    pub plot_allowance: u32,
    /// Blocks around the spawn that only moderators may change.
    pub spawn_radius: u32,
    pub wilderness: Wilderness,
    /// Golden Woof for a market place for 1, 7 and 30 days.
    pub rent_woof: [u64; 3],
    /// Days after a rent ends before the place becomes free for anyone to rent.
    pub rent_grace_days: u32,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            starting_woof: 100,
            plot_allowance: 1,
            spawn_radius: 32,
            wilderness: Wilderness::Open,
            rent_woof: [10, 50, 150],
            rent_grace_days: 3,
        }
    }
}

/// The settings as loaded.
pub fn get() -> &'static Settings {
    SETTINGS.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_file_says_what_the_defaults_are() {
        let shipped = SETTINGS.parse_default().expect("the shipped file parses");
        assert_eq!(shipped, Settings::default(), "file and code agree");
    }

    #[test]
    fn a_missing_key_keeps_its_default() {
        let read: Settings = toml::from_str("wilderness = \"closed\"").expect("the text parses");
        assert_eq!(read.wilderness, Wilderness::Closed, "the key given is read");
        assert_eq!(read.plot_allowance, 1, "a missing key is the default");
    }
}
