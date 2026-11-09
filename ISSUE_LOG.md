# Issue log

Problems found while working on these mods that need a change outside this repository: in the
engine, the Weft mod index or the tools around them. Each entry says what breaks, the evidence and
the smallest fix, so it can be turned into an upstream issue. Newest first.

## The mod index moves only the mod's own `weft_sdk` line to a new Weft release

Found 2026-11-08, while adding `weftkit/`. Where: [Weft-Rust/Weft-Mods-Index](https://github.com/Weft-Rust/Weft-Mods-Index), `scripts/patch_sdk.py`. Status: not yet reported.

**What breaks.** When a new Weft release comes out, the index rebuilds each mod's newest version
against it by rewriting one line: the `weft_sdk = { git = "https://github.com/Weft-Rust/Weft",
tag = ".." }` line in the mod's `Cargo.toml`. Every other Weft crate the build takes from git
keeps its old tag, among them `base_api` and the `weft_sdk` of a library the mod uses by path.
Cargo treats the two tags as two sources, so the build holds two copies of `weft_sdk`, and the
SDK's types from one don't match the other's: the mod no longer builds.

This hits every mod that uses `base_api` today, cogwork and structgen among them, as soon as the
index moves them. It would hit every mod that uses `weftkit` the same way.

**Evidence.**

- `scripts/patch_sdk.py:7-12` matches `^(\s*weft_sdk\s*=\s*\{[^}]*git\s*=\s*"https://github\.com/Weft-Rust/Weft"[^}]*tag\s*=\s*")…`
  in the mod's `Cargo.toml` alone and stops unless it matches exactly once. A line under
  `[workspace.dependencies]` matches too.
- `plan.py:49-52` queues the newest version of each mod for every later Weft release.
- The index's README:57-58 describes the rebuild as "its code as it was and only the SDK's tag
  changed".
- `.github/workflows/build.yml:51-73` fetches the whole repository at the mod's commit and builds
  in the mod's folder with `cargo build --release`, so a path dependency outside the folder
  resolves. Only the tag is the problem.
- A dry run: structgen copied to a temporary folder, `patch_sdk.py` run on it with another tag,
  then `cargo tree -d`. It listed both `weft_sdk v26.11.2 (…?tag=v26.11.2)` and
  `weft_sdk v26.11.3 (…?tag=v26.11.3)`, the second through `base_api v26.11.3`. No Weft release
  newer than v26.11.3 exists yet, so the run moved backwards, and that build also failed on API
  differences between the releases: it shows the two copies, not the type clash by itself.
- The index has never moved a mod: at the time of writing it has one commit and an empty `mods/`.

**Smallest fix.** `patch_sdk.py`, or a script beside it, sets the new tag on every dependency
taken from `git = "https://github.com/Weft-Rust/Weft"` in every `Cargo.toml` of the checkout,
and still requires the one `weft_sdk` line in the mod's own `Cargo.toml`, as it does now.

**Here meanwhile.** `tools/ci/check-weft.sh` keeps `weftkit` and every mod that uses it on one
tag, so a move by hand to a new release, of all of them at once, keeps building.
