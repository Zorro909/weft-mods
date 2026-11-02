//! The film mod's side: registering films, building their sets and talking to the camera while
//! they play.

use weft_sdk::*;

use crate::film::Film;
use crate::{CUE, PLAY, SIGNAL, SUBJECT, Signal, Subject, TIMELINE, name, name_text, wire};

/// Where a film is being built and for whom; positions given to it are from its origin.
pub struct Set {
    world: World,
    player: u64,
    origin: [i32; 3],
}

impl Set {
    pub fn world(&self) -> World {
        self.world
    }

    /// The player watching, who is the camera.
    pub fn player(&self) -> u64 {
        self.player
    }

    /// The block the film is built around, on top of the ground.
    pub fn origin(&self) -> [i32; 3] {
        self.origin
    }

    /// `offset` from the origin, in the world.
    pub fn at(&self, offset: [i32; 3]) -> [i32; 3] {
        [
            self.origin[0] + offset[0],
            self.origin[1] + offset[1],
            self.origin[2] + offset[2],
        ]
    }

    /// Sets the block `offset` from the origin; false if it is not loaded.
    pub fn place(&self, offset: [i32; 3], state: u32) -> bool {
        self.world.set(self.at(offset), state)
    }

    /// Fills the box between two offsets, both included.
    pub fn fill(&self, from: [i32; 3], to: [i32; 3], state: u32) {
        let lo = [0, 1, 2].map(|i| from[i].min(to[i]));
        let hi = [0, 1, 2].map(|i| from[i].max(to[i]));
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    self.place([x, y, z], state);
                }
            }
        }
    }

    /// See `signal`.
    pub fn signal(&self, name: &str) {
        signal(&self.world, self.player, name);
    }

    /// See `subject`.
    pub fn subject(&self, name: &str, entity: u64) {
        subject(&self.world, self.player, name, entity, [0.0; 3]);
    }

    /// A subject that is a place, `offset` from the origin.
    pub fn subject_at(&self, name: &str, offset: [f64; 3]) {
        let pos = [0, 1, 2].map(|i| self.origin[i] as f64 + offset[i]);
        subject(&self.world, self.player, name, 0, pos);
    }
}

/// Lets `player`'s film go on past `Film::wait_for(name)`. A signal sent before the film gets
/// there is kept.
pub fn signal(world: &World, player: u64, name: &str) {
    world.send(
        player,
        &SIGNAL,
        &Signal {
            name: crate::name(name),
        },
    );
}

/// Tells `player`'s camera what the shots that name `name` follow or look at: `entity`, or
/// the world position `pos` where `entity` is 0.
pub fn subject(world: &World, player: u64, name: &str, entity: u64, pos: [f64; 3]) {
    world.send(
        player,
        &SUBJECT,
        &Subject {
            name: crate::name(name),
            entity,
            pos,
        },
    );
}

/// Registers the film `name` of this mod (played as `mymod:name`): `build` puts its set into
/// the world and returns its steps.
pub fn film<'a>(
    r: &'a mut Registry,
    name: &str,
    build: impl Fn(&Set) -> Film + Send + Sync + 'static,
) -> FilmOptions<'a> {
    let full = format!("{}:{name}", mod_id());
    let mine = full.clone();
    r.on(&PLAY, move |batch| {
        for play in batch.iter().filter(|p| name_text(&p.film) == mine) {
            let set = Set {
                world: batch.world(),
                player: play.player,
                origin: play.origin,
            };
            let film = build(&set);
            set.world.send(
                set.player,
                &TIMELINE,
                &wire::encode(&mine, set.origin, &film),
            );
        }
    });
    FilmOptions { r, film: full }
}

/// More of a film registered with `film`.
pub struct FilmOptions<'a> {
    r: &'a mut Registry,
    film: String,
}

impl FilmOptions<'_> {
    /// Runs `f` on the server when the film reaches `Film::cue(name)`.
    pub fn on_cue(self, cue: &str, f: impl Fn(&Set) + Send + Sync + 'static) -> Self {
        let (film, cue) = (name(&self.film), name(cue));
        self.r.on(&CUE, move |batch| {
            for c in batch.iter().filter(|c| c.film == film && c.name == cue) {
                f(&Set {
                    world: batch.world(),
                    player: c.player,
                    origin: c.origin,
                });
            }
        });
        self
    }
}
