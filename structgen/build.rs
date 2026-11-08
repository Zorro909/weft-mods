//! Compiles every `assets/structures/<name>.wst` into the mod as `EMBEDDED` (read in
//! `worldgen`): the SDK gives a mod no way to read its own files while the game runs.

use std::path::{Path, PathBuf};

fn main() {
    let dir = Path::new("assets/structures");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries = String::new();
    for path in structure_files(dir) {
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        let valid = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        assert!(
            valid,
            "{}: a structure file is named in lowercase letters, digits and underscores",
            path.display()
        );
        let full = std::fs::canonicalize(&path)
            .unwrap_or_else(|e| panic!("could not resolve {}: {e}", path.display()));
        entries.push_str(&format!(
            "    ({name:?}, include_bytes!({:?})),\n",
            full.to_string_lossy()
        ));
    }
    let code = format!(
        "/// Every structure file compiled in: its name and its contents.\nconst EMBEDDED: &[(&str, &[u8])] = &[\n{entries}];\n"
    );
    let out =
        PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR for build scripts"))
            .join("embedded.rs");
    std::fs::write(&out, code).unwrap_or_else(|e| panic!("could not write {}: {e}", out.display()));
}

/// The `.wst` files in `dir`, sorted so every build registers them in the same order.
fn structure_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "wst"))
        .collect();
    files.sort();
    files
}
