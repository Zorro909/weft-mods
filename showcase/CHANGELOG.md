# Changelog

## Unreleased

### Added

- Shots tilt the camera with `.roll(from, to)`, for banking flybys and slanted views.

### Changed

- Built for Weft 26.11.3. Films are sent in a new layout: mods with films have to be
  rebuilt against this version of `showcase_api`.

### Fixed

- Clicks and movement keys no longer break blocks or tug at the player while a film or a
  `/camera` path plays.
