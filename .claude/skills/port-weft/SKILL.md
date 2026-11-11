---
name: port-weft
description: Use when moving the mods to a newer Weft release or to upstream master.
---

# Port to a new Weft

A port has two halves with your user between them: what changed and what to do about it, then the work.

## 1. See what changed

1. The tag the mods are on is the `weft_sdk` line of their `Cargo.toml`. Fetch upstream in `Weft/` (`git -C Weft fetch --tags origin`) and name the target: a release tag, or a commit of `origin/master`.
2. Read the changes between the two, in this order:
   - `git -C Weft log --oneline <from>..<to> -- crates/sdk crates/abi mods/base/api docs`
   - `git -C Weft diff <from>..<to> -- docs`, where upstream describes its own changes;
   - `git -C Weft diff <from>..<to> -- crates/sdk mods/base/api`.
3. Check each gap the mods work around: `grep -rn "SDK" --include='*.rs' cogwork structgen showcase weftkit` finds the comments that name one. A gap the new release closes is an improvement.

## 2. Propose

For each mod, a numbered list in three groups:

- **Breaking**: what no longer compiles, and what compiles but now behaves differently.
- **Improvement**: code a new call makes shorter, a workaround that can go.
- **New and useful**: what the mod could now do that it couldn't.

Add how the port is split into pull requests. So far: a port branch (`chore/weft-master`) that takes one pull request per mod and reaches `main` once every mod builds, since `weftkit` and all its users move together.

Done when your user has answered the list.

## 3. Port

1. Move every Weft crate line of the mod, of its API crate and of `weftkit` to the new tag, then `./init.sh`. For a commit that has no tag yet, leave the tags and build against `Weft/` with the `.cargo/config.toml` patch in `docs/agents/mods.md`.
2. Fix what broke, then the improvements your user took. Keep the mod's structure and the diff small: a port is no rework.
3. Bring the known failures in `docs/agents/testing.md` to the new release: run each test named there alone, and replace the list with what fails now.
4. A bullet under `Changed` in each mod's `CHANGELOG.md`: the Weft release it now builds against, and what a player notices.

Done when every mod loads into the new engine and the checks pass or each failure is named with its cause.
