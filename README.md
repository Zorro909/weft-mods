<img src="assets/icon.svg" width="64" height="64" alt="">

# weft-mods

Mods for [Weft](https://github.com/Weft-Rust/Weft), a voxel sandbox engine written in Rust.
Each mod is a native library built against Weft's SDK, and adds to the base game that Weft
ships as its own mod.

## The mods

| | Mod | What it adds |
|---|---|---|
| <img src="cogwork/assets/textures/gui/icon.png" width="48" height="48" alt=""> | [Cogwork](cogwork/README.md) | Item tubes you can see into, engines, and machines that work for you: an excavator that mines out a whole area, an assembly bench that crafts on its own, and a leveler that makes land flat. |
| <img src="structgen/assets/textures/gui/icon.png" width="48" height="48" alt=""> | [Structgen](structgen/README.md) | Hamlets, villages and towns across the overworld, each laid out afresh from the land, built of timber-framed buildings in the engine's structure files. |
| | [Showcase](showcase/README.md) | A tool for mod makers: plays films that mods make of their content in code, and records them to video. |

Each mod's README gives an overview; Cogwork's and Structgen's `docs/` folders describe every
feature, block and item in detail.

## Playing

1. Get Weft from its [releases](https://github.com/Weft-Rust/Weft/releases).
2. Download a mod's zip from the [downloads](#downloads) below, built against the same Weft.
3. Put the zip as it is into the game's `mods/` folder and start the game.

Every mod here needs the base game, which every Weft has, and none needs another. Showcase is
only needed to play a film.

## Building from source

Each mod's `Cargo.toml` takes Weft's SDK from GitHub at a Weft release tag. With Rust and Weft's
build requirements installed:

```
./start.sh              # checks out and builds the engine and every mod, then starts the game
./start.sh cogwork      # only this mod
```

`start.sh` first runs `./init.sh`, which clones the engine into `Weft/` at the newest release
any mod builds against, or moves it there. A mod loads only into an engine built from the same
Weft release, so whenever that tag moves, rebuild the engine and the mods.
`tools/install-mods.sh` does the same without starting the game, and confirms that the game loads
each mod; [`docs/agents/mods.md`](docs/agents/mods.md) has the build, load and check commands.

## Repository layout

| Path | What it holds |
|---|---|
| `cogwork/`, `structgen/`, `showcase/` | One mod each: its crate, `assets/`, README, and `docs/` where it has them |
| `Weft/` | The engine, checked out by `init.sh` and not tracked here |
| `weftkit/` | A library the mods share: typed block positions, areas and block states |
| `docs/agents/` | Guides for working on this repository: code style, mod design, mods, testing, engine, releasing |
| `.claude/skills/` | Step lists for jobs that come up again: adding content, drawing a structgen building, filming, porting to a new Weft, starting a mod |
| `tools/` | Build, install and load check (`install-mods.sh`), formatting, tests and clippy (`check-mods.sh`), the headless test run, film recording (`showcase.sh`), structure photos, sounds, Blockbench models and CI scripts |
| `init.sh`, `start.sh` | Check out the engine; build, install and start everything |

## Contributing

Changes come as pull requests to `main`. Commits follow
[Conventional Commits](https://www.conventionalcommits.org), and Rust is formatted by the root
`rustfmt.toml` (`git ls-files -z '*.rs' | xargs -0 rustfmt +nightly`). Every mod takes Weft at a
release tag, as the [Weft mod index](https://github.com/Weft-Rust/Weft-Mods-Index) builds it:
pull requests that don't are refused. `tools/check-mods.sh` checks every mod's formatting, unit
tests and clippy; CI runs it too, and builds every mod on Windows, Linux and macOS.

## Releases

Every push to `main` releases the mods that changed, through
[weft-mod-action](https://github.com/Zorro909/weft-mod-action):

- A mod whose version has no release yet is released as that version: a full release
  (`X.Y.Z`) or a release candidate (`X.Y.Z-rc.N`).
- A mod that changed without a new version gets a release candidate from CI,
  `X.Y.(Z+1)-rc.<run>.g<commit>`.

Full releases can go to the [Weft mod index](https://github.com/Weft-Rust/Weft-Mods-Index), which
builds them itself and lists them in the Weft launcher.

The section below is kept up to date by CI; see
[`docs/agents/releasing.md`](docs/agents/releasing.md) for the details.

<!-- mod-releases:start -->
## Downloads

Put a mod's zip as it is into the game's `mods/` folder; each one holds the Windows, Linux and
macOS builds. A mod loads only into a game with the same mod interface as the Weft it was built
against. Release candidates (`X.Y.Z-rc.…`) are builds of `main` on the way to the next release.

| Mod | Release | Release candidate |
|---|---|---|
| cogwork | [0.4.0](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.4.0), Weft 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) | – |
| showcase | [0.3.1](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.3.1), Weft 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) | – |
| structgen | [0.4.0](https://github.com/Zorro909/weft-mods/releases/tag/structgen-v0.4.0), Weft 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) | – |
| structures | [0.1.3](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.3), Weft 26.11.2 [2db18c2](https://github.com/Weft-Rust/Weft/commit/2db18c2387c1bb09ecb25871072d4d82c3ee0ab4) | [0.1.4-rc.24.gfbcd03f](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.4-rc.24.gfbcd03f), Weft unknown |

## Versions

### cogwork

| Version | Released | Weft |
|---|---|---|
| [0.4.0](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.4.0) | 2026-11-12 | 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) |
| [0.3.0](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.3.0) | 2026-11-08 | 26.11.3 [80d1f65](https://github.com/Weft-Rust/Weft/commit/80d1f6504545b1ce3f6afab0b1df975630a3c7e5) |
| [0.2.2](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.2.2) | 2026-11-05 | 26.11.2 [2db18c2](https://github.com/Weft-Rust/Weft/commit/2db18c2387c1bb09ecb25871072d4d82c3ee0ab4) |
| [0.2.1](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.2.1) | 2026-11-04 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.2.0](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.2.0) | 2026-11-04 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.2](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.1.2) | 2026-11-02 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.1](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.1.1) | 2026-11-02 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.0](https://github.com/Zorro909/weft-mods/releases/tag/cogwork-v0.1.0) | 2026-11-02 | 26.10.4 [5faf2e3](https://github.com/Weft-Rust/Weft/commit/5faf2e3e57697dd3476b03a300f1231b84f6419e) |

### showcase

| Version | Released | Weft |
|---|---|---|
| [0.3.1](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.3.1) | 2026-11-12 | 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) |
| [0.3.0](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.3.0) | 2026-11-08 | 26.11.3 [80d1f65](https://github.com/Weft-Rust/Weft/commit/80d1f6504545b1ce3f6afab0b1df975630a3c7e5) |
| [0.2.1](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.2.1) | 2026-11-05 | 26.11.2 [2db18c2](https://github.com/Weft-Rust/Weft/commit/2db18c2387c1bb09ecb25871072d4d82c3ee0ab4) |
| [0.2.0](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.2.0) | 2026-11-04 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.0](https://github.com/Zorro909/weft-mods/releases/tag/showcase-v0.1.0) | 2026-11-02 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |

### structgen

| Version | Released | Weft |
|---|---|---|
| [0.4.0](https://github.com/Zorro909/weft-mods/releases/tag/structgen-v0.4.0) | 2026-11-12 | 26.11.4 [2e2014f](https://github.com/Weft-Rust/Weft/commit/2e2014f2714943c07d33279a70cdc5ed44e8d601) |
| [0.3.0](https://github.com/Zorro909/weft-mods/releases/tag/structgen-v0.3.0) | 2026-11-09 | 26.11.3 [80d1f65](https://github.com/Weft-Rust/Weft/commit/80d1f6504545b1ce3f6afab0b1df975630a3c7e5) |
| [0.2.0](https://github.com/Zorro909/weft-mods/releases/tag/structgen-v0.2.0) | 2026-11-08 | 26.11.3 [80d1f65](https://github.com/Weft-Rust/Weft/commit/80d1f6504545b1ce3f6afab0b1df975630a3c7e5) |

### structures

| Version | Released | Weft |
|---|---|---|
| [0.1.3](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.3) | 2026-11-05 | 26.11.2 [2db18c2](https://github.com/Weft-Rust/Weft/commit/2db18c2387c1bb09ecb25871072d4d82c3ee0ab4) |
| [0.1.2](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.2) | 2026-11-02 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.1](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.1) | 2026-11-02 | 26.11.1 [ff9c712](https://github.com/Weft-Rust/Weft/commit/ff9c7128511fc0dce7719b67c9a59274d70bdd5a) |
| [0.1.0](https://github.com/Zorro909/weft-mods/releases/tag/structures-v0.1.0) | 2026-11-02 | 26.10.4 [5faf2e3](https://github.com/Weft-Rust/Weft/commit/5faf2e3e57697dd3476b03a300f1231b84f6419e) |

<!-- mod-releases:end -->
