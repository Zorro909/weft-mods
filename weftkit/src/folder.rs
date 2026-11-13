//! Where a mod keeps its files: `<game folder>/<kind>/<mod>`. The game folder is the one the
//! engine was started with `--run-dir` for, as each shard of `--test all` is, or else the
//! folder the game runs in. The SDK says neither to a server, so the engine's own arguments are
//! read, as the engine reads them: the last `--run-dir` counts.

use std::path::PathBuf;

/// `<game folder>/<kind>/<mod>`, for the mod that compiles this.
pub(crate) fn mod_folder(kind: &str) -> PathBuf {
    game_folder(std::env::args())
        .join(kind)
        .join(weft_sdk::mod_id())
}

/// The folder `--run-dir` names in `args`, or the working folder (an empty path) without one.
fn game_folder(args: impl IntoIterator<Item = String>) -> PathBuf {
    let mut args = args.into_iter();
    let mut folder = PathBuf::new();
    while let Some(arg) = args.next() {
        if arg == "--run-dir" {
            folder = args.next().map(PathBuf::from).unwrap_or_default();
        }
    }
    folder
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder_of(args: &[&str]) -> PathBuf {
        game_folder(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn run_dir_names_the_game_folder() {
        let args = [
            "weft",
            "server",
            "--run-dir",
            "test-jobs/2",
            "--test",
            "all",
        ];
        assert_eq!(
            folder_of(&args),
            PathBuf::from("test-jobs/2"),
            "the folder after the flag"
        );
    }

    #[test]
    fn without_run_dir_it_is_the_working_folder() {
        assert_eq!(
            folder_of(&["weft", "server", "--world", "w"]),
            PathBuf::new(),
            "no folder"
        );
        assert_eq!(
            folder_of(&["weft", "--run-dir"]),
            PathBuf::new(),
            "nor without a value"
        );
    }

    #[test]
    fn the_last_run_dir_counts() {
        let args = ["weft", "--run-dir", "a", "--run-dir", "b"];
        assert_eq!(
            folder_of(&args),
            PathBuf::from("b"),
            "as the engine takes it"
        );
    }

    #[test]
    fn a_mods_folder_is_its_kind_then_its_id() {
        let folder = mod_folder("data");
        let id = weft_sdk::mod_id();
        assert_eq!(
            folder,
            PathBuf::from("data").join(id),
            "under the working folder in tests"
        );
    }
}
