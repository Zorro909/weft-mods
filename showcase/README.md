# Showcase

Films of mods, made in code: a mod builds a set in the world and says where the camera goes,
and this mod plays it, flying the player's camera through orbits, flybys, dollies and cranes.
Two crates:

- `api/` (`showcase_api`): what a film says, and how a mod registers one. Plain data and the
  events and channels between the two mods; no camera logic.
- this folder (`showcase`): the mod that plays films. It has to be loaded wherever a film plays.

## Making a film

Build the films only when asked, so that releases of your mod leave them out:

```toml
[dependencies]
showcase_api = { path = "<path to weft-mods>/showcase/api", optional = true }

[features]
showcase = ["dep:showcase_api"]
```

```rust
#[cfg(not(feature = "showcase"))]
declare_mod!(deps: ["base"], init: init);
#[cfg(feature = "showcase")]
declare_mod!(deps: ["base", "showcase"], init: init);

#[cfg(feature = "showcase")]
mod films;
// and in `init`: #[cfg(feature = "showcase")] films::register(r);
```

```rust
// films.rs
use showcase_api::{Film, Look, Set, dolly, film, orbit};

pub fn register(r: &mut Registry) {
    film(r, "tour", tour).on_cue("start", |set| start_machines(set));
}

fn tour(set: &Set) -> Film {
    set.level([-12, -12], [12, 12], 16); // flat ground as far as the camera goes
    build(set); // `set.place(offset, state)`, `set.fill(..)`, `set.world()`
    Film::new()
        .cue("start")
        .title("mymod.film.tour", 4.0)
        .shot(orbit([0.5, 1.0, 0.5], 10.0).arc(200.0, 20.0).height(5.0, 2.0).secs(10.0))
        .shot(dolly([-6.0, 2.0, 3.5], [6.0, 2.0, 3.5]).look(Look::At([0.5, 0.5, 0.5])).blend(1.5))
}
```

Positions are in blocks from the set's origin, the block the player stood in when the film
started; angles are in degrees. Films are made in a natural world, so `set.level(from, to,
clear)` first makes the ground flat at the origin's height around the set: hills and trees in
that area go, hollows are filled, and the landscape beyond stays as the backdrop. A film is a list of steps:

| Step | |
|---|---|
| `shot(..)` | A camera move: `orbit`, `flyby` (a smooth curve through points), `dolly` (a straight line), `crane` (up or down beside a point), `hold` (still), `follow` (a subject). Each takes `.secs` (5 unless told), `.ease` (`Ease::Smooth` unless told; `Linear`, `In`, `Out`), `.look(..)` or `.look_at(point)`, `.fov(from, to)` (the player's own unless told) and `.blend(secs)`, which brings the camera over from where the shot before ended instead of cutting. Orbits and cranes also take `.arc(from, to)` in degrees, `.radius(from, to)` and `.height(from, to)`. |
| `wait(secs)` | The camera stays. |
| `wait_for(name, max)` | The camera stays until your mod calls `signal(world, player, name)` (or `set.signal(name)`), at most `max` seconds. |
| `title(key, secs)` | A line from your lang files on screen while the film goes on. |
| `cue(name)` | Runs your `on_cue(name, ..)` on the server: start the machines, fill a chest. Block entities only exist after the tick their block was placed in, so fill them here. |
| `menu(name, at, secs)` | Opens the menu `name` (`mymod:press`) on the block `at` in the set, as if the camera had used it, and closes it after `secs`, while the film goes on. Shots after it go on behind the menu. |

Where a shot looks: `Look::At(point)`, `Pan(from, to)`, `Subject(name)`, `Ahead` along the
path (a `hold` keeps looking the way the camera did), fixed `Angles`, or `Along(angles)`, a yaw
and pitch for each point of a flyby that the camera turns through smoothly. Orbits and cranes
look at their center, `follow` at its subject, the others ahead. Subjects are what `follow` and
`Look::Subject` point at: `set.subject(name, entity)` or `set.subject_at(name, offset)`.

A mod that starts a film on its own (a command, an item) builds it the same way and plays it
with `showcase_api::send(world, player, name, origin, &film)`.

## Playing and recording

`/showcase play <mod:film>` plays a film in a running game, from where you stand, with this mod
and yours built with `--features showcase` in the game's `mods/`. The film makes you a
spectator, sets noon and clear weather, and gives you back your game mode and place when it is
over, or after ten seconds if no mod sends the film.

`/showcase gallery <mod>` needs no film: it puts every block of the mod on a plinth in a row
from where you stand and shows each in turn under its name.

`weft --showcase <mod:film>` (or `gallery:<mod>`) makes a fresh normal world from seed 42,
whose spawn is a meadow by a lake, plays the film there once and quits. `--showcase-seed`
picks another seed and `--showcase-dim` another dimension.
`tools/showcase.sh <mod:film>` in this repository builds everything, runs that on a virtual
screen and writes the video, with the game's sound, to `showcase/out/`; `--help` lists its
options and the packages it needs (weston, PipeWire, GStreamer).

## Camera paths by hand

`/camera` makes a path in the game and plays it on your client alone, for animators:

- `/camera point` marks where the camera is and where it looks; `/camera clear` forgets them.
- `/camera play [secs]` flies through the points, turning to look where each was marked.
- `/camera orbit <radius> [secs]` circles the point `radius` blocks in front of you once.

It flies through walls only in spectator mode.
