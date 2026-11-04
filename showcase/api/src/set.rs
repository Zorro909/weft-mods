//! The film mod's side: registering films, building their sets and talking to the camera while
//! they play.

use weft_sdk::abi::block_flags::{LIQUID, REPLACEABLE};
use weft_sdk::*;

use crate::film::Film;
use crate::{
    Area, BlockState, CUE, Name, PLAY, Pos, SIGNAL, SUBJECT, Signal, Subject, TIMELINE, Timeline,
};

/// How deep `Set::level` fills hollows under the ground.
const HOLLOW_DEPTH: i32 = 8;

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
        self.world.set(self.at(offset).into(), state.id())
    }

    /// Sets every block in `area`, in offsets from the origin.
    pub fn fill(&self, area: Area, state: BlockState) {
        area.blocks().for_each(|offset| {
            self.place(offset, state);
        });
    }

    /// Makes the ground flat under `area` (offsets from the origin), for a set in a natural
    /// world: clears `area`, tops the ground right under it with the block under the origin, and
    /// fills hollows below that, up to 8 deep, with the block under that one.
    pub fn level(&self, area: Area) {
        let top = self.get(Pos::new(0, -1, 0));
        let below = self.get(Pos::new(0, -2, 0));
        self.fill(area, BlockState::AIR);
        let ground = area.lo.y - 1;
        for (x, z) in area.columns() {
            self.place(Pos::new(x, ground, z), top);
            self.fill_hollow(Pos::new(x, ground - 1, z), below);
        }
    }

    fn get(&self, offset: Pos) -> BlockState {
        self.world.get(self.at(offset).into()).into()
    }

    /// Fills the column down from `top` to the first block that isn't air, a plant or a liquid.
    fn fill_hollow(&self, top: Pos, state: BlockState) {
        for y in (top.y - HOLLOW_DEPTH + 1..=top.y).rev() {
            let offset = Pos { y, ..top };
            if self.world.flags(self.at(offset).into()) & (REPLACEABLE | LIQUID) == 0 {
                return;
            }
            self.place(offset, state);
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
/// the world and returns its steps.
pub fn film<'a>(
    registry: &'a mut Registry,
    name: &str,
    build: impl Fn(&Set) -> Film + Send + Sync + 'static,
) -> FilmOptions<'a> {
    let film = format!("{}:{name}", mod_id());
    let (full, wanted) = (film.clone(), Name::from(film.as_str()));
    registry.on(&PLAY, move |batch| {
        for play in batch.iter().filter(|play| play.film == wanted) {
            let set = Set::new(batch.world(), play.player, play.origin);
            send(&set.world, set.player, &full, set.origin, &build(&set));
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
    /// Runs `f` on the server when the film reaches `Film::cue(name)`.
    pub fn on_cue(self, cue: &str, f: impl Fn(&Set) + Send + Sync + 'static) -> Self {
        let (film, cue) = (Name::from(self.film.as_str()), Name::from(cue));
        self.registry.on(&CUE, move |batch| {
            for reached in batch.iter().filter(|c| c.film == film && c.name == cue) {
                f(&Set::new(batch.world(), reached.player, reached.origin));
            }
        });
        self
    }
}
