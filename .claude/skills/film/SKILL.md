---
name: film
description: Use when writing, extending or recording a showcase film or video of a mod.
---

# Film a mod

A film is Rust code in the mod, behind its `showcase` feature, played by the showcase mod and recorded by `tools/showcase.sh`. `showcase/README.md` ("Making a film") has the API. `cogwork/src/films/` is a film built on a set, `structgen/src/films/` one flown through the world as it generates.

## 1. Write

- `films/mod.rs` registers the film, named `showcase`, and reads as its table of contents: one function per section, `fn section(film: Film) -> Film`, chained in order.
- A section's file builds its part of the set (`build(set)`) and holds its positions as `pub const`s that its shots share.
- Every title is a lang key under `# Film titles` in `en_us.lang`, `<mod>.film.<name>`. It sits just before the shot it captions and stays 3 to 7 seconds.
- Films are shot in the natural world of seed 42. A set takes `.level(area)` for flat ground and is built in offsets from the origin; the landscape beyond stays as the backdrop.
- A block entity exists from the tick after its block was placed: stock and start machines in a cue handler, the film's first step.

## 2. What makes a film good

- **Content in use.** Machines working, buildings in their landscape. A row of blocks on plinths is the gallery, which a film never looks like.
- **Started on camera.** A machine that runs out of input waits for the camera: a lever, or a cue just before its shot.
- **Facing the camera.** The working side of a machine, clear of frames and gantries.
- **One direction.** A camera move keeps its heading; it never goes back the way it came.
- **No clipping, no hard cuts.** The camera passes through doors and windows, never through blocks. Shots that follow each other join with `.blend(..)` or meet at one speed with `Ease::Speeds { from, to }`; smooth ends between joined shots read as stalls.
- **Screens.** Show a machine's screen with `Film::menu`.
- **Titles low** in the picture, where they hide little.

For a flight through buildings: join shots along one heading, with a lead-in point along the tangent of the orbit that follows; keep the stretches between points of a flyby similar in length; keep turns under about 75 degrees a second.

## 3. Rehearse

```
tools/showcase.sh --gpu <address> --rehearse <mod>:showcase -- --view 32
```

A rehearsal goes through the film in a minute or two without recording. It prints its report and ends with 3 while the report has faults: where the camera is inside a block, where it turns faster than 75 degrees a second, which titles stay under 3 or over 7 seconds. Its pictures, along every shot at most five seconds apart, are the film's storyboard: `showcase/out/<mod>-<film>-rehearsal/sheets/`.

Read every sheet against the list above: framing, what stands in the way, which side of a machine shows, where the titles sit. Machines have run only for the rehearsal's minute, so what they do, and motion, sound and menus, are for the take.

Done when the report has no fault and no sheet shows one. Only then record: a take costs the film's length each time.

## 4. Record

```
tools/showcase.sh --gpu <address> <mod>:showcase -- --view 32
```

- The video is `showcase/out/<mod>-showcase.mp4`. Recording takes the film's length plus the build: run it in the background.
- `--view 32` is the view distance for a film of a landscape; leave it out for a set.
- For a film without music, write `volume_music = 0` into `showcase/out/run/config/core.txt` before recording.
- `showcase/out/` is not tracked. No video goes into the repository.

## 5. Review the take

```
tools/film-sheet.sh showcase/out/<mod>-showcase.mp4
tools/film-sheet.sh --every 0.5 --from 80 --to 90 showcase/out/<mod>-showcase.mp4 <folder>
```

The first cuts the take into sheets of a frame every five seconds, each under its time; the second looks closely at one stretch, for motion. Read what the rehearsal could not show: machines working when the camera arrives (their screens say "Waiting" when not), moving parts, menus, and `ERROR [mod:<mod>]` lines in `showcase/out/run/logs/latest.log`.

For a film over a minute, give each section to a subagent with its sheets and the list above, and have it name the second of every fault. Fix, rehearse, record again, and number the takes.

Done when no fault from the list is left in any section's frames, and your user has the path of the video.
