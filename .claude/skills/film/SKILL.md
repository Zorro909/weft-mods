---
name: film
description: Write, extend or record a showcase film of a mod. Use when asked for a film, video or showcase of mod content, to add a scene, or to render or re-encode one.
---

# Film a mod

A film is Rust code in the mod, behind its `showcase` feature, played by the showcase mod and recorded by `tools/showcase.sh`. `showcase/README.md` ("Making a film") has the API: the feature and `declare_mod!` lines, steps, shots and looks. `cogwork/src/films/` is a film built on a set, `structgen/src/films/` one flown through the world as it generates.

## 1. Write

- `films/mod.rs` registers the film, named `showcase`, and reads as its table of contents: one function per section, `fn section(film: Film) -> Film`, chained in order.
- A section's file builds its part of the set (`build(set)`) and holds its positions as `pub const`s that its shots share.
- Every title is a lang key under `# Film titles` in `en_us.lang`: `<mod>.film.<name>`. A title sits just before the shot it captions and stays 3 to 7 seconds.
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

## 3. Record

```
tools/showcase.sh --gpu <address> <mod>:showcase -- --view 32
```

- The video is `showcase/out/<mod>-showcase.mp4`, about real time plus the build; run it in the background.
- `--view 32` is the view distance for a film of a landscape; leave it out for a set.
- For a film without music, write `volume_music = 0` into `showcase/out/run/config/core.txt` before recording.
- `showcase/out/` is not tracked. No video goes into the repository.

## 4. Review the take

Cut frames from every section and read them:

```
ffmpeg -ss <seconds> -i showcase/out/<mod>-showcase.mp4 -frames:v 1 /tmp/<name>.png
```

For a film over a minute, give each section to a subagent with its time range and the list under "What makes a film good", and have it name the second of every fault. Fix, record again, and number the takes.

Done when every section's frames have been read, no fault from the list is left, and your user has the path of the video with the takes' differences in a line each. Motion and sound stay theirs to judge: say so.

## Re-encoding

For sharing, on request: `ffmpeg -i in.mp4 -c:v libvpx-vp9 -crf 32 -b:v 0 -an out.webm` gives a small file without sound; leave `-an` out to keep it.
