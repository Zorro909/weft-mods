# Changelog

## Unreleased

### Changed

- The mod's description opens with a shorter summary, which the Weft mod index also shows.

## 0.3.0 - 2026-11-08

### Added

- Shots tilt the camera with `.roll(from, to)`, for banking flybys and slanted views.
- `FilmOptions::level(area)` levels a film's ground, and `Waiting` holds a set a mod plays
  on its own until the ground under it is level.

### Changed

- Built for Weft 26.11.3. Films are sent in a new layout: mods with films have to be
  rebuilt against this version of `showcase_api`.
- `Set::level` fills with the engine's area fills at the end of the tick, and a film's set is
  built in the tick after: films level their ground with `.level(area)` when they register,
  not in their `build`.

### Fixed

- Clicks and movement keys no longer break blocks or tug at the player while a film or a
  `/camera` path plays.
- Levelling the ground for a large set no longer runs past the engine's limit of block updates
  in a tick, which dropped the rest of them.
