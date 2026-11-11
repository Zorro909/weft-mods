---
name: add-content
description: Use when adding or reworking a block, item, machine, screen, recipe, command, entity, model or sound of a mod.
---

# Add content to a mod

## 1. Find the model

Read `docs/agents/mods.md` ("Where to look") and `docs/agents/mod-design.md`, then the Weft doc for this kind of content and the example file `mods.md` names for it.

## 2. Propose a bigger addition first

A new machine or module, new saved state, or a change to code several areas share: before writing it, give your user

- the data structures: components and their fields, enums, channels;
- where each part goes: the modules, and what is shared or moves;
- the files you will add and change.

Done when they have answered. A small addition inside an existing area needs no proposal.

## 3. Write it

By `docs/agents/code-style.md`, with the parts below that the content has:

| Part | Where |
|---|---|
| Registration | A `mod` line and the `register(r)` call in `lib.rs`; `.tab(..)` so it shows in the mod's creative tab |
| Names and text | `block.<mod>.<name>` or `item.<mod>.<name>`, its `.tooltip`, a key per status and screen label: in every file under `assets/lang/` |
| Textures | `assets/textures/<block, item, entity or gui>/`, named as the builder that reads them expects |
| Model | A Blockbench save under `assets/models/`; `tools/blockbench.sh check <file>` passes |
| Recipe | An entry in `assets/recipes/*.toml` (cogwork: a test fails while a named block has none) |
| Sound | "Sounds" below |
| Tests | The logic, and what `mod-design.md` pins: saved layout, enum fallbacks, status names, assets |
| The mod's docs | Its page under `<mod>/docs/` with a row in `docs/README.md`, links from the pages of what it works with, and the feature list of the mod's `README.md` |
| Changelog | A bullet under `## Unreleased` in `<mod>/CHANGELOG.md` |

## 4. Check and look

`tools/check-mods.sh <mod>`, `tools/install-mods.sh <mod>`, the in-game tests, then pictures of it in the game (`docs/agents/testing.md`). In the pictures, look for what players have sent back before:

- every face has its texture: no see-through lines, no missing sides;
- parts meet: no gap between neighbours, and no two faces in one plane, which flicker;
- a moving part reaches both ends of its travel and stays outside the block it works on;
- the block faces the way it was placed, with its working side toward the player;
- a screen matches the mock in its doc, in every status.

A still shows no motion: take pictures at several moments of an animation, or record a short film.

## Sounds

1. Add a line to `<mod>/sounds.tsv`: file, variants, seconds, prompt influence, prompt. Its first lines give the format.
2. Write the prompt as the thing that makes the sound: its size, its material, what it does, then "isolated game sound effect, no music". Heavy machinery is low, weighty and large; a hand tool is not.
3. `tools/sfx.sh <mod>` makes every file that is missing; `tools/sfx.sh <mod> <file>` makes one again.
4. Give it a handle and its row in the mod's sound table (`cogwork/src/sounds.rs`): volume, pitch range, and how many blocks it carries. Large machines carry far (40 and more), small parts stay close.
