//! A mod's settings file, read once when the mod starts: `mods/<mod>/config/<name>.toml` under
//! the folder the game runs in. A missing file is first written from the default the mod ships,
//! comments and all, so the server's admin has one to edit. A file that doesn't parse stops the
//! game from starting, naming the file and the line at fault: running on defaults instead would
//! quietly undo what the admin set.
//!
//! The SDK's `Config` stays the tool for a single value players or operators change in the game:
//! it has a screen, `/rule` and syncing. This is for what that can't hold, such as lists, tables
//! and values per tier, kept in a file the admin edits.
//!
//! ```ignore
//! static SETTINGS: ConfigFile<Settings> =
//!     ConfigFile::named("settings", include_str!("../config/settings.toml"));
//! SETTINGS.load(); // in `init`
//! let rent = SETTINGS.get().stall_rent;
//! ```

use std::io::ErrorKind;
use std::path::Path;
use std::sync::OnceLock;

use serde::de::DeserializeOwned;

/// A settings file of the mod that compiles it: `load` it in `init`, then `get` it anywhere.
pub struct ConfigFile<T> {
    name: &'static str,
    default: &'static str,
    value: OnceLock<T>,
}

impl<T: DeserializeOwned> ConfigFile<T> {
    /// The file `<name>.toml`, written from `default`, the TOML the mod ships, where it is missing.
    pub const fn named(name: &'static str, default: &'static str) -> Self {
        ConfigFile {
            name,
            default,
            value: OnceLock::new(),
        }
    }

    /// Reads the file, writing it from the default first if it is missing. A file that can't be
    /// written or read, or doesn't parse, panics, which fails the mod's load and stops the game
    /// with the reason.
    pub fn load(&self) {
        let folder = Path::new("mods").join(weft_sdk::mod_id()).join("config");
        if let Err(fault) = self.load_from(&folder) {
            panic!("{fault}");
        }
    }

    /// The settings `load` read.
    pub fn get(&self) -> &T {
        self.value
            .get()
            .unwrap_or_else(|| panic!("config {} was read before `load` in init", self.name))
    }

    /// The default the mod ships, parsed: for a mod's test that it is valid.
    pub fn parse_default(&self) -> Result<T, String> {
        parse(
            self.default,
            &format!("the default of config {}", self.name),
        )
    }

    fn load_from(&self, folder: &Path) -> Result<(), String> {
        let path = folder.join(format!("{}.toml", self.name));
        let text = read_or_write_default(&path, self.default)?;
        let value = parse(&text, &path.display().to_string())?;
        let _ = self.value.set(value);
        Ok(())
    }
}

/// The text of `path`, which is first written from `default` if it is missing.
fn read_or_write_default(path: &Path, default: &str) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            write_default(path, default)?;
            Ok(default.to_string())
        }
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

fn write_default(path: &Path, default: &str) -> Result<(), String> {
    let written = match path.parent() {
        Some(folder) => {
            std::fs::create_dir_all(folder).and_then(|()| std::fs::write(path, default))
        }
        None => std::fs::write(path, default),
    };
    written.map_err(|error| format!("could not write the default to {}: {error}", path.display()))
}

/// `text` read as TOML; `source` names it in the error, which also gives the line.
fn parse<T: DeserializeOwned>(text: &str, source: &str) -> Result<T, String> {
    toml::from_str(text).map_err(|error| format!("{source} is not valid: {error}"))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Settings {
        rent: u32,
        tiers: Vec<String>,
    }

    const DEFAULT: &str = "# What a stall costs a day.\nrent = 5\ntiers = [\"hamlet\", \"town\"]\n";

    /// An empty folder of its own under the system's temporary folder.
    fn empty_folder(name: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("weftkit-config-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        folder
    }

    #[test]
    fn a_missing_file_is_written_from_the_default() {
        let folder = empty_folder("missing");
        let config: ConfigFile<Settings> = ConfigFile::named("settings", DEFAULT);
        config.load_from(&folder).expect("the default loads");
        assert_eq!(config.get().rent, 5, "the default's value is read");
        let written =
            std::fs::read_to_string(folder.join("settings.toml")).expect("it was written");
        assert_eq!(written, DEFAULT, "the file keeps the default's comments");
    }

    #[test]
    fn the_admins_file_wins_over_the_default() {
        let folder = empty_folder("edited");
        std::fs::create_dir_all(&folder).expect("the folder can be made");
        std::fs::write(folder.join("settings.toml"), "rent = 9\ntiers = []\n").expect("written");
        let config: ConfigFile<Settings> = ConfigFile::named("settings", DEFAULT);
        config.load_from(&folder).expect("the file loads");
        let expected = Settings {
            rent: 9,
            tiers: Vec::new(),
        };
        assert_eq!(config.get(), &expected, "the file's values are read");
    }

    #[test]
    fn a_broken_file_is_refused_with_its_path_and_line() {
        let folder = empty_folder("broken");
        std::fs::create_dir_all(&folder).expect("the folder can be made");
        std::fs::write(
            folder.join("settings.toml"),
            "tiers = []\nrent = \"five\"\n",
        )
        .expect("written");
        let config: ConfigFile<Settings> = ConfigFile::named("settings", DEFAULT);
        let fault = config.load_from(&folder).expect_err("a string is no rent");
        assert!(
            fault.contains("settings.toml"),
            "the fault names the file: {fault}"
        );
        assert!(
            fault.contains("line 2"),
            "the fault names the line: {fault}"
        );
    }

    #[test]
    fn the_default_parses() {
        let config: ConfigFile<Settings> = ConfigFile::named("settings", DEFAULT);
        let tiers = config.parse_default().expect("the default is valid").tiers;
        assert_eq!(tiers, ["hamlet", "town"], "the default's list is read");
    }
}
