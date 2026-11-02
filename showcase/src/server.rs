//! Server half: `/showcase play` makes the player the camera and asks the film's mod to build
//! its set; the client reports cues and the end of the film here.

use std::sync::Mutex;

use showcase_api::{CUE, Cue, Name, PLAY, Play, name};
use weft_sdk::abi::game_mode;
use weft_sdk::*;

value! {
    /// A film reached `Film::cue(name)`.
    pub struct Reached { pub film: Name, pub origin: [i32; 3], pub name: Name }
}
pub static REACHED: Channel<Reached> = Channel::named("reached");
/// The film is over: the player gets back their game mode and place.
pub static FINISHED: Channel<()> = Channel::named("finished");

/// Noon, so that a set stands in full light.
const NOON: u64 = 6000;
/// Ten minutes of clear weather, longer than any film.
const CLEAR_TICKS: u32 = 20 * 60 * 10;

/// What each watching player had before the film: game mode and where they stood.
static BEFORE: Mutex<Vec<(u64, u32, [f64; 3])>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry) {
    r.command("showcase", "play <film:word>", |cmd| {
        let film = cmd.text("film").unwrap_or_default().to_string();
        let (world, player) = (cmd.world(), cmd.player());
        let Some(me) = world.entity(player).filter(|_| player != 0) else {
            cmd.reply("showcase.only_players", &[]);
            return;
        };
        remember(player, world.game_mode(player), me.pos);
        world.set_game_mode(player, game_mode::SPECTATOR);
        world.set_day_time(NOON);
        world.set_weather(false, false, CLEAR_TICKS);
        let origin = me.pos.map(|v| v.floor() as i32);
        world.emit(
            &PLAY,
            Play {
                film: name(&film),
                player,
                origin,
            },
        );
        cmd.reply("showcase.asked", &[Arg::Text(&film)]);
    });
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

fn remember(player: u64, mode: u32, pos: [f64; 3]) {
    let mut before = BEFORE.lock().unwrap_or_else(|e| e.into_inner());
    // A film started over another keeps what the player had before the first.
    if !before.iter().any(|b| b.0 == player) {
        before.push((player, mode, pos));
    }
}

fn restore(world: &World, player: u64) {
    let mut before = BEFORE.lock().unwrap_or_else(|e| e.into_inner());
    let Some(i) = before.iter().position(|b| b.0 == player) else {
        return;
    };
    let (_, mode, pos) = before.swap_remove(i);
    world.set_game_mode(player, mode);
    world.teleport(player, None, pos);
}
