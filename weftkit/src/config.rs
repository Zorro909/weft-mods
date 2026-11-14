//! A mod's settings file, read once when the mod starts: `config/<mod>/<name>.toml` under the
//! game folder (`folder.rs`), not under `mods/`, where the engine takes every folder for a mod.
//! A missing file is first written whole from the default the mod ships, comments and all, so
//! the server's admin has one to edit. A file that doesn't parse stops the game from starting,
//! naming the file and the line: running on defaults would quietly undo what the admin set.
//!
//! The SDK's `Config` stays the tool for a single value changed in the game; this is for lists,
//! tables and values per tier, kept in a file the admin edits.
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
        if let Err(fault) = self.load_from(&crate::folder::mod_folder("config")) {
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
        std::fs::create_dir_all(folder)
            .map_err(|error| format!("could not make {}: {error}", folder.display()))?;
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
            let unfinished = path.with_extension("toml.new");
            std::fs::write(&unfinished, default)
                .and_then(|()| std::fs::rename(&unfinished, path))
                .map_err(|error| {
                    format!("could not write the default to {}: {error}", path.display())
                })?;
            Ok(default.to_string())
        }
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

/// `text` read as TOML; `source` names it in the error, which also gives the line.
fn parse<T: DeserializeOwned>(text: &str, source: &str) -> Result<T, String> {
    toml::from_str(text).map_err(|error| format!("{source} is not valid: {error}"))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::folder::test_folder;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Settings {
        rent: u32,
        tiers: Vec<String>,
    }

    const DEFAULT: &str = "# What a stall costs a day.\nrent = 5\ntiers = [\"hamlet\", \"town\"]\n";

    fn settings() -> ConfigFile<Settings> {
        ConfigFile::named("settings", DEFAULT)
    }

    #[test]
    fn a_missing_file_is_written_from_the_default() {
        let folder = test_folder("config-missing");
        let config = settings();
        config.load_from(&folder).expect("the default loads");
        let default = config.parse_default().expect("the default is valid");
        assert_eq!(config.get(), &default, "the default's values are read");
        let written = std::fs::read_to_string(folder.join("settings.toml")).expect("written");
        assert_eq!(written, DEFAULT, "the file keeps the default's comments");
    }

    #[test]
    fn the_admins_file_wins_over_the_default() {
        let folder = test_folder("config-edited");
        std::fs::write(folder.join("settings.toml"), "rent = 9\ntiers = []\n").expect("written");
        let config = settings();
        config.load_from(&folder).expect("the file loads");
        let expected = Settings {
            rent: 9,
            tiers: Vec::new(),
        };
        assert_eq!(config.get(), &expected, "the file's values are read");
    }

    #[test]
    fn a_broken_file_is_refused_with_its_path_and_line() {
        let folder = test_folder("config-broken");
        let broken = "tiers = []\nrent = \"five\"\n";
        std::fs::write(folder.join("settings.toml"), broken).expect("written");
        let fault = settings()
            .load_from(&folder)
            .expect_err("a string is no rent");
        let named = fault.contains("settings.toml") && fault.contains("line 2");
        assert!(named, "the fault names the file and the line: {fault}");
    }

    #[test]
    fn a_file_or_folder_that_cannot_be_used_is_refused() {
        let folder = test_folder("config-blocked");
        std::fs::create_dir(folder.join("settings.toml")).expect("a folder in the file's place");
        std::fs::write(folder.join("file"), b"").expect("a file in the folder's place");
        for (folder, fault) in [
            (folder.clone(), "could not read"),
            (folder.join("file"), "could not make"),
        ] {
            let found = settings().load_from(&folder).expect_err("it is refused");
            assert!(found.starts_with(fault), "it says so: {found}");
        }
    }

    /// Unix only: Windows lets a read-only folder take new files.
    #[cfg(unix)]
    #[test]
    fn a_default_that_cannot_be_written_is_refused() {
        let folder = test_folder("config-read-only");
        crate::folder::set_mode(&folder, 0o555);
        let loaded = settings().load_from(&folder);
        crate::folder::set_mode(&folder, 0o755);
        let fault = loaded.expect_err("a read-only folder takes no file");
        assert!(fault.starts_with("could not write"), "it says so: {fault}");
    }
}
