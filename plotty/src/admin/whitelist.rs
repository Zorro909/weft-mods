//! `/whitelist`: Plotty edits the engine's own whitelist, `config/whitelist.txt` in the game
//! folder, which a dedicated server reads again at every login. Its lines are player keys, the
//! identity a connection proves, so nobody gets in by taking a listed player's name. Turned off,
//! the file is set aside as `whitelist.txt.off`, which the engine doesn't read.

use std::io;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use weft_sdk::{Arg, ServerCommand, permission};
use weftkit::PlayerKey;

use super::whitelist_file;
use crate::players;

const USAGE: &str = "on | off | list | add <who:targets> | key <key:word> | remove <name:word>";

/// Held while a `/whitelist` command reads and writes the file: two regions may run one each.
static EDITING: Mutex<()> = Mutex::new(());

pub fn register(r: &mut weft_sdk::Registry) {
    r.command("whitelist", USAGE, run)
        .permission(permission::ADMIN);
}

fn run(cmd: &mut ServerCommand) {
    let _editing = EDITING.lock().unwrap_or_else(PoisonError::into_inner);
    let done = if cmd.given("on") {
        turn_on(cmd)
    } else if cmd.given("off") {
        turn_off(cmd)
    } else if cmd.given("list") {
        list(cmd)
    } else if cmd.given("add") {
        add_online(cmd)
    } else if cmd.given("key") {
        add_key(cmd)
    } else {
        remove(cmd)
    };
    if let Err(error) = done {
        let path = file().display().to_string();
        cmd.reply(
            "plotty.whitelist.file_error",
            &[Arg::Text(&path), Arg::Text(&error.to_string())],
        );
    }
}

/// Turns the whitelist on, restoring the set-aside file or seeding it with every known player so
/// nobody already playing is shut out.
fn turn_on(cmd: &mut ServerCommand) -> io::Result<()> {
    if file().exists() {
        cmd.reply("plotty.whitelist.already_on", &[]);
        return Ok(());
    }
    let text = if set_aside().exists() {
        read()?
    } else {
        players::all_keys().iter().fold(String::new(), |text, key| {
            whitelist_file::with(&text, key.as_str())
        })
    };
    let admin = PlayerKey::of(&cmd.world(), cmd.player());
    let Some(text) = whitelist_file::to_turn_on(&text, admin.as_ref().map(PlayerKey::as_str))
    else {
        cmd.reply("plotty.whitelist.empty", &[]);
        return Ok(());
    };
    write(&text)?;
    std::fs::rename(set_aside(), file())?;
    let path = file().display().to_string();
    cmd.reply("plotty.whitelist.on", &[Arg::Text(&path)]);
    Ok(())
}

fn turn_off(cmd: &mut ServerCommand) -> io::Result<()> {
    if !file().exists() {
        cmd.reply("plotty.whitelist.already_off", &[]);
        return Ok(());
    }
    std::fs::rename(file(), set_aside())?;
    cmd.reply("plotty.whitelist.off", &[]);
    Ok(())
}

fn list(cmd: &mut ServerCommand) -> io::Result<()> {
    let text = read()?;
    let keys: Vec<PlayerKey> = whitelist_file::keys(&text)
        .into_iter()
        .filter_map(PlayerKey::new)
        .collect();
    cmd.reply("plotty.whitelist.listed", &[Arg::Int(keys.len() as i64)]);
    for key in keys {
        // The whole key where there's no name, as `remove` takes it.
        let shown = players::known_name(&key).unwrap_or_else(|| key.as_str().to_string());
        cmd.reply("plotty.whitelist.entry", &[Arg::Text(&shown)]);
    }
    Ok(())
}

/// Adds the players a `targets` parameter selected: online ones.
fn add_online(cmd: &mut ServerCommand) -> io::Result<()> {
    let world = cmd.world();
    let keys: Vec<PlayerKey> = cmd
        .targets("who")
        .into_iter()
        .filter_map(|entity| PlayerKey::of(&world, entity))
        .collect();
    if keys.is_empty() {
        cmd.reply("plotty.no_player_online", &[]);
        return Ok(());
    }
    for key in keys {
        add(cmd, &key)?;
    }
    Ok(())
}

/// Adds a key as the server log printed it for someone it turned away.
fn add_key(cmd: &mut ServerCommand) -> io::Result<()> {
    let Some(key) = cmd.text("key").and_then(PlayerKey::new) else {
        cmd.reply("plotty.whitelist.bad_key", &[]);
        return Ok(());
    };
    add(cmd, &key)
}

fn add(cmd: &mut ServerCommand, key: &PlayerKey) -> io::Result<()> {
    write(&whitelist_file::with(&read()?, key.as_str()))?;
    cmd.reply(
        "plotty.whitelist.added",
        &[Arg::Text(&players::name_of(key))],
    );
    Ok(())
}

/// Takes out the player Plotty knows by that name, or the key as the file lists it. One who is
/// online stays until they leave.
fn remove(cmd: &mut ServerCommand) -> io::Result<()> {
    let text = read()?;
    let typed = cmd.text("name").unwrap_or_default().to_string();
    let listed_key = whitelist_file::keys(&text).contains(&typed.as_str());
    let (key, name) = match PlayerKey::new(&typed).filter(|_| listed_key) {
        Some(key) => (key, players::name_of(&key)),
        None => match players::named(cmd) {
            Some(found) => found,
            None => return Ok(()),
        },
    };
    write(&whitelist_file::without(&text, key.as_str()))?;
    cmd.reply("plotty.whitelist.removed", &[Arg::Text(&name)]);
    Ok(())
}

/// The file the engine reads.
fn file() -> PathBuf {
    weftkit::game_folder().join("config").join("whitelist.txt")
}

/// Where the file waits while the whitelist is off.
fn set_aside() -> PathBuf {
    file().with_extension("txt.off")
}

/// The file `/whitelist` changes: the engine's while it is on, else the one set aside, so that
/// adding a key never turns the whitelist on by itself.
fn current() -> PathBuf {
    if file().exists() { file() } else { set_aside() }
}

/// The current file's text; empty where there is none yet.
fn read() -> io::Result<String> {
    match std::fs::read_to_string(current()) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        read => read,
    }
}

/// Writes the current file whole, through a file beside it, so the engine never reads half of
/// one.
fn write(text: &str) -> io::Result<()> {
    let path = current();
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder)?;
    }
    let fresh = path.with_extension("new");
    std::fs::write(&fresh, text)?;
    std::fs::rename(fresh, path)
}
