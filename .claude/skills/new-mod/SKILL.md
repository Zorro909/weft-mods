---
name: new-mod
description: Start a new mod or shared library in this repository. Use when asked to create a mod, split one off, or add a crate the mods share.
---

# Start a new mod

A mod is a folder at the repository root whose `Cargo.toml` builds a `cdylib`; `init.sh`, `tools/install-mods.sh` and `tools/check-mods.sh` find it by that. Read `docs/agents/mods.md` and `docs/agents/mod-design.md` first. Agree the mod's id with your user: it is the crate name, the folder name, and the prefix of every name the mod adds.

## 1. The crate

Start from a copy of `Weft/examples/examplemod`; the top of its `lib.rs` lists what to rename. Then make it one of this repository's:

| Part | What |
|---|---|
| `Cargo.toml` | The `path` dependency becomes `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = ".." }` at the tag the other mods name, with `base_api` the same way when used. Copy `rust-version`, `[workspace]` and `[profile.release]` from `structgen/Cargo.toml`; `version = "0.1.0"` |
| What the copy brought | Delete its `.github/` folder, its example model and texture, and the commented examples you don't use |
| `.gitignore` | `/target`, as the other mods have |
| `Cargo.lock` | Built once and committed |
| `src/lib.rs` | `declare_mod!` and an `init` calling one `register(r)` per area, as `cogwork/src/lib.rs` |
| `assets/lang/en_us.lang` | `mod.<id>.name` and `mod.<id>.description`. The first sentence of the description is the summary the mod index shows: one sentence that says what the mod adds |
| `assets/textures/gui/icon.png` | The mod's icon on the Mods screen and in the root README |
| `CHANGELOG.md` | Shaped like `cogwork/CHANGELOG.md`, starting with `## Unreleased` |
| `README.md` | An overview: what it adds, how to build it, a table of its code. Detail on each block and item goes into `<id>/docs/` |

## 2. The repository

| Part | What |
|---|---|
| Workflows | The folder in the `mods:` list of `release.yml`, `check.yml` and `index-pr.yml` under `.github/workflows/` |
| Root `README.md` | A row in "The mods" and the folder in "Repository layout"; the downloads section is CI's |
| `sonar-project.properties` | Files that only register content or call straight into the engine, under `sonar.coverage.exclusions` |

## 3. Check

`tools/check-mods.sh <id>`, then `tools/install-mods.sh <id>`: done when it prints `loaded mod <id>`, and `tools/ci/check-weft.sh` passes.

## Optional parts

- **An API crate** for other mods, `<id>/api`: the workspace lines in `docs/agents/mods.md`, the design in `mod-design.md` ("An API crate"). Its interface is your user's to decide.
- **Films**: the `showcase` feature and `src/films/`, as the `film` skill and `showcase/README.md` describe.
- **A shared library** instead of a mod, as `weftkit/`: no `cdylib`, so no workflow lists and no release; it takes Weft at the tag of every mod that uses it.
