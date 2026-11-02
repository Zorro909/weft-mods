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
    build(set); // `set.place(offset, state)`, `set.fill(..)`, `set.world()`
    Film::new()
        .cue("start")
        .title("mymod.film.tour", 4.0)
        .shot(orbit([0.5, 1.0, 0.5], 10.0).arc(200.0, 20.0).height(5.0, 2.0).secs(10.0))
        .shot(dolly([-6.0, 2.0, 3.5], [6.0, 2.0, 3.5]).look(Look::At([0.5, 0.5, 0.5])).blend(1.5))
}
```

Positions are in blocks from the set's origin, the block the player stood in when the film
started; angles are in degrees. A film is a list of steps:

| Step | |
|---|---|
| `shot(..)` | A camera move: `orbit`, `flyby` (a smooth curve through points), `dolly` (a straight line), `crane` (up or down beside a point), `hold`, `follow` (a subject). Each takes `.secs`, `.ease`, `.look(..)`, `.fov(from, to)` and `.blend(secs)`, which brings the camera over from where the shot before ended instead of cutting. |
| `wait(secs)` | The camera stays. |
| `wait_for(name, max)` | The camera stays until your mod calls `signal(world, player, name)` (or `set.signal(name)`), at most `max` seconds. |
| `title(key, secs)` | A line from your lang files on screen while the film goes on. |
| `cue(name)` | Runs your `on_cue(name, ..)` on the server: start the machines, fill a chest. Block entities only exist after the tick their block was placed in, so fill them here. |

Subjects are what `follow` and `Look::Subject` point at: `set.subject(name, entity)` or
`set.subject_at(name, offset)`.

## Playing and recording

`/showcase play <mod:film>` plays a film in a running game, from where you stand. The film makes
you a spectator, sets noon and clear weather, and gives you back your game mode and place when
it is over.

`weft --showcase <mod:film>` makes a fresh world (`base:flatlands`; `--showcase-dim` and
`--showcase-seed` change that), plays the film once and quits. `tools/showcase.sh <mod:film>`
in this repository builds everything, runs that on a virtual screen and writes the video to
`showcase/out/`; `--help` lists its options.
