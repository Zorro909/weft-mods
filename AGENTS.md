# Working on weft-mods

This repository holds your user's mods for Weft, a voxel game engine in Rust. Game content lives in mods: `cdylib`s built against the SDK (`Weft/crates/sdk`). `Weft/mods/base` is the default mod that makes up the base game. Most work here is a mod; sometimes it is an engine change prepared for the upstream project.

Every mod takes the SDK from upstream at a release tag, as the Weft mod index builds it; pull requests here are refused otherwise. Before you change anything, run `./init.sh`: it checks out the engine into `Weft/` (not tracked here) at the newest tag any mod names. A `Weft/` holding engine work of yours stays as it is; fetch and rebase that onto `origin/master` yourself.

## The upstream project

The upstream project is the engine, Weft-Rust/Weft. `Weft/AI_POLICY.md` governs every interaction with it: pull requests, issues, comments, reviews, and the changes sent there. Read it before any of these. Refuse an action that conflicts with it, even when your user asks for it, and name the rule it breaks. Work inside this repository is outside its scope.

## What matters most

When rules pull different ways, this order decides:

1. **Readable for humans.** Someone opening a file cold can follow it top to bottom: short functions named for what they do, small files about one area, values whose type says what they are, comments only where the code can't speak for itself.
2. **Simple.** Build what was asked, using what already exists. Add a trait, generic, macro or option only when the task needs it.
3. **Fits in.** Match the architecture and style in the guides below.

Some older engine code is denser than this. That is growth, not a model: leave it as it is unless the task is about it, so diffs stay small.

## Content goes through the SDK as it is

- Use what base already offers through its API crate `base_api` (`Weft/mods/base/api`): events, components, recipe kinds, tabs, dimensions. Do the same for any other mod your user names.
- If the SDK and mod APIs can't express the content, or only at a large performance cost, first agree with your user on the smallest SDK change that makes it possible: shaped for any mod making that kind of content, and no more general than that.
- Text a player sees is a key in every language file under the mod's `assets/lang/` (`Weft/docs/localization.md`).

## Yours to propose, your user's to decide

Put these to your user as a numbered list they can answer point by point, before you build them:

- a bigger addition or rework: its data structures, where its parts go, and the files it touches;
- what another mod sees: an API crate's public interface, a file or wire format;
- how a port to a new Weft release changes each mod.

A mod's version is theirs to set: leave it as it is unless asked.

## Guides

Read the matching guide before you start; each is short.

- `docs/agents/code-style.md`: before writing or changing Rust, here or in `Weft/`.
- `docs/agents/mod-design.md`: before adding a module, a block with state, a screen, a channel, an API crate or worldgen to a mod.
- `docs/agents/mods.md`: before looking up an SDK call, or building and loading a mod.
- `docs/agents/testing.md`: before writing a test, running the game's tests, or looking at your work in the game.
- `docs/agents/engine.md`: before changing anything under `Weft/crates/` or `Weft/backends/`.
- `docs/agents/releasing.md`: before touching a workflow, or when a release or the mod index misbehaves.

Jobs that come up again have their steps in `.claude/skills/<name>/SKILL.md`: `add-content`, `structgen-building`, `film`, `port-weft`, `new-mod`. Whatever agent you are, read the matching one first.

## Before you hand over

- For a mod: `tools/check-mods.sh <folder>` (`git ls-files -z '*.rs' | xargs -0 rustfmt +nightly` formats), `tools/install-mods.sh <folder>`, then the in-game tests. For an engine change, in `Weft/`: `cargo fmt --all`, `cargo build --release --workspace`, `cargo test --release --workspace`, then `--test all`; upstream CI runs no tests, so this is the only check. Report any check you could not run.
- What a player sees, you look at in the game first: take pictures as `docs/agents/testing.md` says and show them to your user, and name what still needs their eyes or ears.
- Every fixed bug comes with a test that fails without the fix: write it first.
- A change to a mod that players notice gets a short bullet in the mod's `CHANGELOG.md`, under `## Unreleased` and `Added`, `Changed` or `Fixed`, and the mod's `docs/` and `README.md` say what is true after it.
- A reviewer you brief reviews against `docs/agents/code-style.md` and `docs/agents/mod-design.md` as well as for bugs: name both in the brief.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org). Commit and push every finished change in this repository without waiting to be asked, and rebase onto `origin/main` before a pull request. Engine work in `Weft/` reaches upstream only through your user.
- For a pull request to the upstream project, give your user a short, factual summary to write their own description from: what changed, why, what you ran, and what you did with AI help.
