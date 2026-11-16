//! Where a mod keeps its files: `<game folder>/<kind>/<mod>`. The game folder is the one the
//! engine was started with `--run-dir` for, as each shard of `--test all` is, or else the
//! folder the game runs in. The SDK says neither to a server, so the engine's own arguments are
//! read, as the engine reads them: the last `--run-dir` counts.

use std::path::PathBuf;

/// `<game folder>/<kind>/<mod>`, for the mod that compiles this.
pub(crate) fn mod_folder(kind: &str) -> PathBuf {
    game_folder().join(kind).join(weft_sdk::mod_id())
}

/// The game folder: what `--run-dir` names, or the working folder (an empty path) without one.
/// A mod reaches the engine's own files from it, such as `config/whitelist.txt`.
pub fn game_folder() -> PathBuf {
    game_folder_in(std::env::args())
}

/// The folder `--run-dir` names in `args`, or the working folder (an empty path) without one.
fn game_folder_in(args: impl IntoIterator<Item = String>) -> PathBuf {
    let args: Vec<String> = args.into_iter().collect();
    let last = args.iter().rposition(|arg| arg == "--run-dir");
    last.and_then(|at| args.get(at + 1))
        .map(PathBuf::from)
        .unwrap_or_default()
}

/// An empty folder of its own under the system's temporary folder.
#[cfg(test)]
pub(crate) fn test_folder(name: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("weftkit-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).expect("a test folder can be made");
    folder
}

/// Gives `path` the Unix permissions `mode`.
#[cfg(all(test, unix))]
pub(crate) fn set_mode(path: &std::path::Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let permissions = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(path, permissions).expect("the permissions change");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_run_dir_names_the_game_folder() {
        let cases: [(&[&str], &str); 4] = [
            (
                &[
                    "weft",
                    "server",
                    "--run-dir",
                    "test-jobs/2",
                    "--test",
                    "all",
                ],
                "test-jobs/2",
            ),
            (&["weft", "--run-dir", "a", "--run-dir", "b"], "b"),
            (&["weft", "server", "--world", "w"], ""),
            (&["weft", "--run-dir"], ""),
        ];
        for (args, folder) in cases {
            let found = game_folder_in(args.iter().map(|arg| arg.to_string()));
            assert_eq!(
                found,
                PathBuf::from(folder),
                "the last one, if any: {args:?}"
            );
        }
    }

    #[test]
    fn a_mods_folder_is_its_kind_then_its_id() {
        let expected = PathBuf::from("data").join(weft_sdk::mod_id());
        assert_eq!(
            mod_folder("data"),
            expected,
            "under the working folder in tests"
        );
    }
}
