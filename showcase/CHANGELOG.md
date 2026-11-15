# Changelog

## Unreleased

### Added

- `--showcase-rehearse <folder>` goes through a film without recording it: pictures along
  every shot, and a report of where the camera is inside a block or turns too fast and which
  titles stay too short or too long.
- `Film::weather(..)` sets fog, sky colour, sunlight, clouds and acoustics from that step on,
  fading in and crossfading into the next; `clear_weather()` and the film's end lift it.
  `Weather::dusk()`, `mist()` and `underground()` are ready to use.
- `Film::screen(shader, strength, secs)` puts on a screen effect over time, `fade_out(secs)`
  and `fade_in(secs)` fade to black and back between shots, and `showcase_api::screen` has
  fades to black and white and warm and cool grades.
- `showcase:demo`, a short film of these steps.

### Changed

- Built for Weft 26.11.5.
- The gallery fades in from black and out to it.
- Films show what is over the screen under water, and other mods' screen effects, as the
  player would see them there.
- Films are sent in a new layout: mods with films have to be rebuilt against this version of
  `showcase_api`.

## 0.3.1 - 2026-11-12

### Added

- `Ease::Speeds { from, to }` sets a shot's speed at its start and end, so a camera flying
  on from one shot to the next keeps moving instead of stopping where they meet.

### Fixed

- A long flight is loaded along its whole path, and a film opens with its first view loaded.

### Changed

- Built for Weft 26.11.4.
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
