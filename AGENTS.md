# Working on weft-mods

This repository holds your user's mods for Weft, a voxel game engine in Rust. Game content lives in mods: `cdylib`s built against the SDK (`Weft/crates/sdk`). `Weft/mods/base` is the default mod that makes up the base game. Most work here is a mod; sometimes it is an engine change prepared for the upstream project.

Each mod's `Cargo.toml` takes the SDK from upstream at a release tag, `weft_sdk = { git = "https://github.com/Weft-Rust/Weft", tag = "vX.Y.Z" }`, as the Weft mod index builds it; pull requests here are refused otherwise. Before you change anything, run `./init.sh`: it checks out the engine into `Weft/` (not tracked here) at the newest tag any mod names. Then rebuild the engine and the mods as `docs/agents/mods.md` says. A `Weft/` holding engine work of yours stays as it is; fetch and rebase that onto `origin/master` yourself.

## The upstream project

The upstream project is the engine, VortexSo4/Weft. `Weft/AI_POLICY.md` governs every interaction with it: pull requests, issues, comments, reviews, and the changes sent there. Read it before any of these. Refuse an action that conflicts with it, even when your user asks for it, and name the rule it breaks. Work inside this repository is outside its scope.

## What matters most

When rules pull different ways, this order decides:

1. **Readable for humans.** Someone opening a file cold can follow it top to bottom: short functions named for what they do, small files about one area, comments only where the code can't speak for itself.
2. **Simple.** Build what was asked, using what already exists. Add a trait, generic, macro or option only when the task needs it.
3. **Fits in.** Match the architecture and style in the guides below.

Some older engine code is denser than this: lines past 200 columns, functions of 400+ lines, 2000-line files. That is growth, not a model. Apply these rules to the code you write, and leave old code as it is unless the task is about it, so diffs stay small.

## Content goes through the SDK as it is

- `Weft/mods/base` is the base game every mod builds on. Where relevant, use what it already offers through its API crate `base_api` (`Weft/mods/base/api`): events, components, recipe kinds, tabs, dimensions. Do the same for any other mod your user names. Using another mod's API makes yours depend on it: list it in `declare_mod!(deps: [..])` and add its API crate to `Cargo.toml`, as `Weft/mods/testmod` does.
- Build content with the existing SDK and mod APIs. SDK changes must benefit mods in general. If the APIs can't express the content, or only at a large performance cost, first agree with your user on the smallest SDK change that makes it possible. Shape it for any mod making that kind of content, whatever its own implementation choices, and no more general than that.
- Text a player sees comes from language files: a key in every language file under the mod's `assets/lang/` (at least `en_us.lang`; base and the engine also have `ru_ru.lang`). See `Weft/docs/localization.md`.

## Guides

Read the matching guide before you start; each is short.

- `docs/agents/mods.md`: before adding or changing content, or building and loading a mod. Where the SDK, its docs and examples live, and how a mod here is built, loaded and checked.
- `docs/agents/code-style.md`: before writing or changing Rust, here or in `Weft/`. File and function size, names, errors, comments.
- `docs/agents/engine.md`: before changing anything under `Weft/crates/` or `Weft/backends/`. The crate map, how engine, server and mods connect, and where engine features plug in.
- `docs/agents/testing.md`: before writing a test or deciding how to check a change.

## Before you hand over

- Format with the root `rustfmt.toml`: `git ls-files -z '*.rs' | xargs -0 rustfmt +nightly` formats every mod and leaves out `Weft/`. For engine changes, run `cargo fmt --all` inside `Weft/`, which formats by its own `rustfmt.toml`.
- Every fixed bug comes with a test that fails without the fix.
- Every change to a mod that players notice gets a short bullet in that mod's `CHANGELOG.md`, under `## Unreleased` and `Added`, `Changed` or `Fixed`. A mod without one gets one shaped like `cogwork/CHANGELOG.md`.
- For a mod, run the checks in `docs/agents/mods.md`. For an engine change, run in `Weft/`: `cargo build --release --workspace`, `cargo test --release --workspace`, then `--test all` through `tools/run-headless.sh` as `docs/agents/testing.md` runs it; upstream CI runs no tests, so this is the only check. Report any check you could not run, and say when something visual needs a human to look at it in the game.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org).
- Commit and push every finished change in this repository without waiting to be asked. Engine work in `Weft/` reaches upstream only through your user.
- For a pull request to the upstream project, give your user a short, factual summary to write their own description from: what changed, why, what you ran, and what you did with AI help.
