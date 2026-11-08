//! The film mod's side: registering films, building their sets and talking to the camera while
//! they play.

use std::sync::Arc;

use weft_sdk::*;

use crate::film::Film;
use crate::{
    Area, BlockState, CUE, Name, PLAY, Pos, SIGNAL, SUBJECT, Signal, Subject, TIMELINE, Timeline,
    Waiting,
};

/// How deep `Set::level` fills hollows under the ground.
const HOLLOW_DEPTH: i32 = 8;
/// What `Set::level` counts as hollow: air, liquids and plants, the blocks the engine flags
/// replaceable, by the name and tags the engine and the SDK give them.
const HOLLOW: &str = "core:air #liquids #plantable";

/// Where a film is being built and for whom; positions given to it are from its origin.
pub struct Set {
    world: World,
    player: u64,
    origin: Pos,
}

impl Set {
    /// A set around `origin` for `player`, for a mod that builds and sends a film on its own
    /// (`send`); `film` makes them for registered films.
    pub fn new(world: World, player: u64, origin: Pos) -> Set {
        Set {
            world,
            player,
            origin,
        }
    }

    pub fn world(&self) -> World {
        self.world
    }

    /// The player watching, who is the camera.
    pub fn player(&self) -> u64 {
        self.player
    }

    /// The block the film is built around, on top of the ground.
    pub fn origin(&self) -> Pos {
        self.origin
    }

    /// `offset` from the origin, in the world.
    pub fn at(&self, offset: Pos) -> Pos {
        self.origin + offset
    }

    /// Sets the block `offset` from the origin; false if it is not loaded.
    pub fn place(&self, offset: Pos, state: BlockState) -> bool {
        self.world.set_block(self.at(offset), state.id())
    }

    /// Sets every block in `area`, in offsets from the origin.
    pub fn fill(&self, area: Area, state: BlockState) {
        area.blocks().for_each(|offset| {
            self.place(offset, state);
        });
    }

    /// Makes the ground flat under `area` (offsets from the origin), for a set in a natural
    /// world: clears `area`, tops the ground right under it with the block under the origin, and
    /// fills hollows (air, liquids, plants) up to 8 below that with the block under that one.
    /// It happens at the end of the tick, as engine fills do, so that the blocks hear of it only
    /// at its edges: build on it from the next tick, as `FilmOptions::level` and `Waiting` do.
    pub fn level(&self, area: Area) {
        let top = self.get(Pos::new(0, -1, 0));
        let below = self.get(Pos::new(0, -2, 0));
        let ground = area.lo.y - 1;
        let layer = |lo: i32, hi: i32| Area {
            lo: Pos { y: lo, ..area.lo },
            hi: Pos { y: hi, ..area.hi },
        };
        self.world.fill(self.shape(area), BlockState::AIR.id());
        self.world.fill(self.shape(layer(ground, ground)), top.id());
        let hollows = layer(ground - HOLLOW_DEPTH, ground - 1);
        self.world
            .fill(self.shape(hollows), below.id())
            .only(HOLLOW);
    }

    fn get(&self, offset: Pos) -> BlockState {
        self.world.block(self.at(offset)).into()
    }

    /// `area`, in offsets from the origin, as a shape in the world.
    fn shape(&self, area: Area) -> Shape {
        // A shape takes the blocks whose middles are inside it.
        Shape::cuboid(self.at(area.lo), self.at(area.hi) + Pos::new(1, 1, 1))
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
        let origin: [f64; 3] = self.origin.into();
        let pos = [0, 1, 2].map(|i| origin[i] + offset[i]);
        subject(&self.world, self.player, name, 0, pos);
    }
}

/// Lets `player`'s film go on past `Film::wait_for(name)`. A signal sent before the film gets
/// there is kept.
pub fn signal(world: &World, player: u64, name: &str) {
    let name = Name::from(name);
    world.send(player, &SIGNAL, &Signal { name });
}

/// Tells `player`'s camera what the shots that name `name` follow or look at: `entity`, or
/// the world position `pos` where `entity` is 0.
pub fn subject(world: &World, player: u64, name: &str, entity: u64, pos: [f64; 3]) {
    let name = Name::from(name);
    world.send(player, &SUBJECT, &Subject { name, entity, pos });
}

/// Plays `steps` on `player`'s camera now, as the film `film` around `origin`: for films a mod
/// starts on its own (a command, an item). Unlike `/showcase play` it leaves the player's game
/// mode, time and weather as they are, and cues reach `on_cue` only for a film registered under
/// that name.
pub fn send(world: &World, player: u64, film: &str, origin: Pos, steps: &Film) {
    let timeline = Timeline {
        film: film.into(),
        origin,
        steps: steps.steps.clone(),
    };
    world.send(player, &TIMELINE, &timeline.encode());
}

/// Registers the film `name` of this mod (played as `mymod:name`): `build` puts its set into
/// the world and returns its steps, the tick after the film is asked for.
pub fn film<'a>(
    registry: &'a mut Registry,
    name: &str,
    build: impl Fn(&Set) -> Film + Send + Sync + 'static,
) -> FilmOptions<'a> {
    let film = format!("{}:{name}", mod_id());
    let (full, wanted) = (film.clone(), Name::from(film.as_str()));
    let waiting = Arc::new(Waiting::new());
    let asked = waiting.clone();
    registry.on_event(&PLAY, move |batch| {
        for play in batch.iter().filter(|play| play.film == wanted) {
            asked.push(&Set::new(batch.world(), play.player, play.origin), ());
        }
    });
    registry.world_tick(move |world| {
        for (set, ()) in waiting.ready(world) {
            send(world, set.player, &full, set.origin, &build(&set));
        }
    });
    FilmOptions { registry, film }
}

/// More of a film registered with `film`.
pub struct FilmOptions<'a> {
    registry: &'a mut Registry,
    film: String,
}

impl FilmOptions<'_> {
    /// Makes the ground flat under `area` when the film is asked for (`Set::level`), so that
    /// the set is built on it the tick after.
    pub fn level(self, area: Area) -> Self {
        let film = Name::from(self.film.as_str());
        self.registry.on_event(&PLAY, move |batch| {
            for play in batch.iter().filter(|play| play.film == film) {
                Set::new(batch.world(), play.player, play.origin).level(area);
            }
        });
        self
    }

    /// Runs `f` on the server when the film reaches `Film::cue(name)`.
    pub fn on_cue(self, cue: &str, f: impl Fn(&Set) + Send + Sync + 'static) -> Self {
        let (film, cue) = (Name::from(self.film.as_str()), Name::from(cue));
        self.registry.on_event(&CUE, move |batch| {
            for reached in batch.iter().filter(|c| c.film == film && c.name == cue) {
                f(&Set::new(batch.world(), reached.player, reached.origin));
            }
        });
        self
    }
}
