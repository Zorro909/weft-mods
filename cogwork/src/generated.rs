//! Assets this mod's code writes: the newer pictures (`art`). Their tests write the files with
//! `WRITE_ASSETS=1 cargo test --release`, and otherwise fail on a file that differs from what
//! they would write, so a file changed by hand has to be changed in the code too.

use std::path::{Path, PathBuf};

/// Set to anything, it has the tests write the files instead of comparing them.
pub const WRITE: &str = "WRITE_ASSETS";

/// The mod's file `path` under `assets/`.
pub fn asset(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(path)
}

/// Writes `bytes` to `path` under `WRITE`; otherwise fails unless the file holds them already.
pub fn save_or_check(path: &Path, bytes: &[u8]) {
    if std::env::var_os(WRITE).is_some() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("could not make a folder for an asset");
        }
        std::fs::write(path, bytes).expect("could not write an asset");
    } else {
        let saved = std::fs::read(path).unwrap_or_default();
        assert!(
            saved == bytes,
            "{path:?} differs: {WRITE}=1 writes it again"
        );
    }
}
