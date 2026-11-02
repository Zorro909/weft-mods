//! Server half: `/showcase play` makes the player the camera and asks the film's mod to build
//! its set; the client reports the start, cues and end of the film here. A film no mod answers
//! for in time gives the player back what they had.

use std::sync::Mutex;

use showcase_api::{CUE, Cue, Name, PLAY, Play, name};
use weft_sdk::abi::game_mode;
use weft_sdk::*;

value! {
    /// A film reached `Film::cue(name)`.
    pub struct Reached { pub film: Name, pub origin: [i32; 3], pub name: Name }
}
pub static REACHED: Channel<Reached> = Channel::named("reached");
/// The film arrived and plays.
pub static STARTED: Channel<()> = Channel::named("started");
/// The film is over: the player gets back their game mode and place.
pub static FINISHED: Channel<()> = Channel::named("finished");
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

/// What each watching player had before the film: game mode and where they stood.
static BEFORE: Mutex<Vec<(u64, u32, [f64; 3])>> = Mutex::new(Vec::new());
/// Films asked for that haven't started: the player, the film, and the tick it was asked at.
static ASKED: Mutex<Vec<(u64, Name, u64)>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    r.command("showcase", "play <film:word> | gallery <mod:word>", |cmd| {
        let (world, player) = (cmd.world(), cmd.player());
        let Some(origin) = prepare(&world, player) else {
            cmd.reply("showcase.only_players", &[]);
            return;
        };
        if let Some(film) = cmd.text("film").map(str::to_string) {
            world.emit(
                &PLAY,
                Play {
                    film: name(&film),
                    player,
                    origin,
                },
            );
            ask(player, name(&film), world.time());
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
    r.packet(&FINISHED, |p, ()| restore(&p.world(), p.player()));
}

/// Makes the player the camera: a spectator at noon in clear weather. Returns the block they
/// stand in, which the film is built around; `None` for no player.
fn prepare(world: &World, player: u64) -> Option<[i32; 3]> {
    let me = world.entity(player).filter(|_| player != 0)?;
    remember(player, world.game_mode(player), me.pos);
    world.set_game_mode(player, game_mode::SPECTATOR);
    world.set_day_time(NOON);
    world.set_weather(false, false, CLEAR_TICKS);
    Some(me.pos.map(|v| v.floor() as i32))
}

fn ask(player: u64, film: Name, tick: u64) {
    answered(player);
    lock(&ASKED).push((player, film, tick));
}

fn answered(player: u64) {
    lock(&ASKED).retain(|a| a.0 != player);
}

/// Gives the players here whose film never came back what they had, and tells them.
fn give_up(world: &World) {
    let now = world.time();
    let mut late = Vec::new();
    lock(&ASKED).retain(|&(player, film, tick)| {
        // `players()` leaves out spectators, which the film made them.
        let gone = now >= tick + ANSWER_TICKS && world.entity(player).is_some();
        if gone {
            late.push((player, film));
        }
        !gone
    });
    for (player, film) in late {
        restore(world, player);
        world.send(player, &MISSING, &Missing { film });
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn remember(player: u64, mode: u32, pos: [f64; 3]) {
    let mut before = lock(&BEFORE);
    // A film started over another keeps what the player had before the first.
    if !before.iter().any(|b| b.0 == player) {
        before.push((player, mode, pos));
    }
}

fn restore(world: &World, player: u64) {
    let mut before = lock(&BEFORE);
    let Some(i) = before.iter().position(|b| b.0 == player) else {
        return;
    };
    let (_, mode, pos) = before.swap_remove(i);
    world.set_game_mode(player, mode);
    world.teleport(player, None, pos);
}
