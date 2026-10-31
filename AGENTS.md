# Working on weft-mods

This repository holds your user's mods for Weft, a voxel game engine in Rust that sits in the submodule `Weft/`. Game content lives in mods: `cdylib`s built against the SDK (`Weft/crates/sdk`). `Weft/mods/base` is the default mod that makes up the base game. Most work here is a mod; sometimes it is an engine change prepared for the upstream project.

Before you change anything, update the engine to upstream's latest commit with `git submodule update --remote Weft`, then rebuild the engine and the mods here as `docs/agents/mods.md` says. If `Weft/` holds engine work of yours, fetch and rebase that onto `origin/master` instead.

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

- Format your own lines with `tools/fmt-changed.sh` rather than `cargo fmt`, which would reformat whole files. It formats the repository you run it in, so for engine changes run `../tools/fmt-changed.sh` from inside `Weft/`. For a whole branch, pass the commit it started from. Check the diff afterwards.
- Every fixed bug comes with a test that fails without the fix.
- For a mod, run the checks in `docs/agents/mods.md`. For an engine change, run in `Weft/`: `cargo build --release --workspace`, `cargo test --release --workspace`, then `--test all` under Xvfb as `docs/agents/testing.md` runs it; upstream CI runs no tests, so this is the only check. Report any check you could not run, and say when something visual needs a human to look at it in the game.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org).
- For a pull request to the upstream project, give your user a short, factual summary to write their own description from: what changed, why, what you ran, and what you did with AI help.
