---
name: add-content
description: Add content to a Weft mod in this repository. Use when adding or reworking a block, item, machine, screen, recipe, command, entity, model or sound of a mod here.
---

# Add content to a mod

Content is done when a player can find it, use it and read about it, and you have seen it in the game. The code is one row of that.

## 1. Find the model

Read `docs/agents/mods.md` ("Where to look") and `docs/agents/mod-design.md`, then the Weft doc for this kind of content and the example file the table in `mods.md` names for it.

Done when you can name the SDK calls you will use and the file in this mod yours sits beside. If the SDK can't express the content, or the change alters what another mod sees, stop: `AGENTS.md` says what to agree with your user first.

## 2. Write it

Follow `docs/agents/code-style.md` and the layout in `mod-design.md`. For a bug in existing content, write the failing test before the fix.

## 3. Everything a player meets

Go through every row. Each is done, or named in your hand-over with the reason it doesn't apply.

| Part | Where |
|---|---|
| Registration | A `mod` line and the `register(r)` call in `lib.rs`; `.tab(..)` so it shows in the mod's creative tab |
| Names and text | `block.<mod>.<name>` or `item.<mod>.<name>`, its `.tooltip`, a key per status and screen label: in every file under `assets/lang/` |
| Textures | `assets/textures/<block, item, entity or gui>/`, named as the builder that reads them expects |
| Model | A Blockbench save under `assets/models/`; `tools/blockbench.sh check <file>` passes |
| Recipe | An entry in `assets/recipes/*.toml` (cogwork: a test fails while a named block has none) |
| Sound | See "Sounds" below |
| Tests | The logic, and what `mod-design.md` pins: saved layout, enum fallbacks, status names, assets |
| The mod's docs | Its page under `<mod>/docs/`, a row in `docs/README.md`, links from the pages of what it works with, and the feature list of the mod's `README.md` |
| Changelog | A bullet under `## Unreleased` in `<mod>/CHANGELOG.md` |
| Film | The mod's film doesn't show new content by itself. Say so at hand-over; a scene for it is the `film` skill |

## 4. Check

`tools/check-mods.sh <mod>`, then `tools/install-mods.sh <mod>`, then the in-game tests as `docs/agents/testing.md` runs them.

## 5. Look at it in the game

Take pictures as `docs/agents/testing.md` ("Pictures") says and read every one. Look for what has come back from players before:

- every face has its texture: no see-through lines, no missing sides;
- parts meet: no gap between neighbours, and no two faces in one plane, which flicker;
- a moving part reaches both ends of its travel and stays outside the block it works on;
- the block faces the way it was placed, with its working side toward the player;
- a screen matches the mock in its doc, in every status.

A still shows no motion: for an animation, take pictures at several moments of it, or record a short film.

Done when every picture has been read and shown to your user, with what you could not judge (motion, sound, feel) named.

## Sounds

1. Add a line to `<mod>/sounds.tsv`: file, variants, seconds, prompt influence, prompt. Its first lines give the format.
2. Write the prompt as the thing that makes the sound: its size, its material, what it does, then "isolated game sound effect, no music". Heavy machinery is low, weighty and large; a hand tool is not.
3. `tools/sfx.sh <mod>` makes every file that is missing; `tools/sfx.sh <mod> <file>` makes one again.
4. Give it a handle and its row in the mod's sound table (`cogwork/src/sounds.rs`): volume, pitch range, and how many blocks it carries. Large machines carry far (40 and more), small parts stay close.
5. You can't hear it. Tell your user which sounds are new; the mod's film records them with the picture.
