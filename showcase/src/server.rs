//! Server half: `/showcase play` makes the player the camera and asks the film's mod to build
//! its set; the client reports the start, cues and end of the film here. A film no mod answers
//! for in time gives the player back what they had.

use std::sync::Mutex;

use showcase_api::{CUE, Cue, Name, PLAY, Play, Pos};
use weft_sdk::abi::game_mode;
use weft_sdk::*;

use crate::lock;

value! {
    /// A film reached `Film::cue(name)`.
    pub struct Reached { pub film: Name, pub origin: Pos, pub name: Name }
}
pub static REACHED: Channel<Reached> = Channel::named("reached");
/// The film arrived and plays.
pub static STARTED: Channel<()> = Channel::named("started");
/// The film is over: the player gets back their game mode and place.
pub static FINISHED: Channel<()> = Channel::named("finished");
value! {
    /// A film's `Film::menu` step: open `menu` on the block at `at` for this player.
    pub struct OpenMenu { pub menu: Name, pub at: Pos }
}
pub static OPEN_MENU: Channel<OpenMenu> = Channel::named("open_menu");
value! {
    /// No mod sent `film` in time.
    pub struct Missing { pub film: Name }
}
pub static MISSING: Channel<Missing> = Channel::named("missing");

/// Ticks the film's mod has to send it: ten seconds.
const ANSWER_TICKS: u64 = 20 * 10;

/// Noon, so that a set stands in full light.
const NOON: u64 = 6000;
/// Ten minutes of clear weather, longer than any film.
const CLEAR_TICKS: u32 = 20 * 60 * 10;

/// What a watching player had before the film, given back at its end.
struct Before {
    player: u64,
    game_mode: u32,
    pos: [f64; 3],
}

/// A film asked for that hasn't started yet.
struct Asked {
    player: u64,
    film: Name,
    /// The tick it was asked at.
    tick: u64,
}

static BEFORE: Mutex<Vec<Before>> = Mutex::new(Vec::new());
static ASKED: Mutex<Vec<Asked>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    r.command("showcase", "play <film:word> | gallery <mod:word>", |cmd| {
        let (world, player) = (cmd.world(), cmd.player());
        let Some(origin) = prepare(&world, player) else {
            cmd.reply("showcase.only_players", &[]);
            return;
        };
        if let Some(film) = cmd.text("film").map(str::to_string) {
            ask(&world, player, origin, &film);
            cmd.reply("showcase.asked", &[Arg::Text(&film)]);
        } else if let Some(mod_id) = cmd.text("mod").map(str::to_string) {
            let shown = crate::gallery::show(&world, player, origin, &mod_id);
            if shown == 0 {
                restore(&world, player);
                cmd.reply("showcase.gallery.empty", &[Arg::Text(&mod_id)]);
            }
        }
    });
    r.packet(&STARTED, |p, ()| answered(p.player()));
    r.world_tick(give_up);
    r.packet(&REACHED, |p, reached| {
        let cue = Cue {
            film: reached.film,
            player: p.player(),
            origin: reached.origin,
            name: reached.name,
        };
        p.world().emit(&CUE, cue);
    });
    r.packet(&OPEN_MENU, |p, open| {
        open_menu(&p.world(), p.player(), open)
    });
    r.packet(&FINISHED, |p, ()| restore(&p.world(), p.player()));
}

fn open_menu(world: &World, player: u64, open: OpenMenu) {
    let menu: &str = (&open.menu).into();
    let Some(id) = Menu::lookup(menu) else {
        warn!("showcase: a film opens the menu {menu}, which no mod has");
        return;
    };
    world.open_menu_at(player, Id::<Menu>::from(id), open.at.into());
}

/// Makes the player the camera: a spectator at noon in clear weather. Returns the block they
/// stand in, which the film is built around; `None` for no player.
fn prepare(world: &World, player: u64) -> Option<Pos> {
    let me = world.entity(player).filter(|_| player != 0)?;
    remember(Before {
        player,
        game_mode: world.game_mode(player),
        pos: me.pos,
    });
    world.set_game_mode(player, game_mode::SPECTATOR);
    world.set_day_time(NOON);
    world.set_weather(false, false, CLEAR_TICKS);
    Some(block_at(me.pos).into())
}

/// Asks the mod that owns `film` to build it around `origin` and send it to `player`.
fn ask(world: &World, player: u64, origin: Pos, film: &str) {
    let film = Name::from(film);
    world.emit(
        &PLAY,
        Play {
            film,
            player,
            origin,
        },
    );
    answered(player);
    let tick = world.time();
    lock(&ASKED).push(Asked { player, film, tick });
}

fn answered(player: u64) {
    lock(&ASKED).retain(|asked| asked.player != player);
}

/// Gives the players here whose film never came back what they had, and tells them.
fn give_up(world: &World) {
    let now = world.time();
    // Players in this world; `players()` would leave out spectators, which the film made them.
    let here = |player: u64| world.entity(player).is_some();
    let late: Vec<Asked> = lock(&ASKED)
        .extract_if(.., |asked| {
            now >= asked.tick + ANSWER_TICKS && here(asked.player)
        })
        .collect();
    for Asked { player, film, .. } in late {
        restore(world, player);
        world.send(player, &MISSING, &Missing { film });
    }
}

fn remember(before: Before) {
    let mut all = lock(&BEFORE);
    // A film started over another keeps what the player had before the first.
    if !all.iter().any(|b| b.player == before.player) {
        all.push(before);
    }
}

fn restore(world: &World, player: u64) {
    let mut all = lock(&BEFORE);
    let Some(i) = all.iter().position(|b| b.player == player) else {
        return;
    };
    let before = all.swap_remove(i);
    world.set_game_mode(player, before.game_mode);
    world.teleport(player, None, before.pos);
}
