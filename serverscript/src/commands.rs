//! The commands scripts declare, and the two of the mod itself.
//!
//! The game takes commands only while a mod registers, so each script command is registered
//! under its own name when the server starts, with its usage and permission level. With the
//! setting Development Mode on, `/scripts reload` reads the scripts again and `/s <name> ..`
//! runs any command of theirs, one added since the start too.

use std::path::{Path, PathBuf};

use rhai::{Dynamic, Map};
use weft_sdk::{Arg, Config, Registry, ServerCommand, info, permission, warn};

use crate::bindings::{self, TEXT};
use crate::script::{self, Scripts};
use crate::usage::{Kind, Param, Usage};
use crate::value::Value;

/// Lets `/scripts reload` and `/s` run. The server's own setting: its players see it.
pub static DEVELOPMENT_MODE: Config<bool> = Config::named("development_mode");

/// The setting, which the client declares as the server does: a server's clients must have the
/// same settings to join.
pub fn register_setting(r: &mut Registry) {
    r.config(&DEVELOPMENT_MODE).bool(false).server();
}

/// Registers the commands of `scripts`, read from `folder`, and the mod's own.
pub fn register(r: &mut Registry, scripts: &Scripts, folder: PathBuf) {
    for command in scripts.commands() {
        let (name, usage) = (command.name.clone(), command.usage.clone());
        r.command(&command.name, &command.usage_text, move |cmd| {
            run_own(cmd, &name, &usage)
        })
        .permission(command.permission);
    }
    r.command("s", "<name:word> [args:text]", run_any);
    r.command("scripts", "reload", move |cmd| reload(cmd, &folder))
        .permission(permission::ADMIN);
}

/// Whether Development Mode is on; tells the player when it isn't.
fn developing(cmd: &mut ServerCommand) -> bool {
    let on = DEVELOPMENT_MODE.get();
    if !on {
        cmd.reply("serverscript.development_mode_off", &[]);
    }
    on
}

/// A command under its own name: the engine read its arguments by `usage`, the one it was
/// registered with, whatever the script says since a reload.
fn run_own(cmd: &mut ServerCommand, name: &str, usage: &Usage) {
    let params = usage.params().into_iter();
    let arguments = params.filter_map(|param| read(cmd, param));
    run(
        cmd,
        name,
        arguments
            .map(|(name, value)| (name.into(), value))
            .collect(),
    );
}

/// What the engine read for `param`, as scripts see it.
fn read(cmd: &ServerCommand, param: &Param) -> Option<(String, Dynamic)> {
    let name = param.name.as_str();
    let text = || cmd.text(name).map(|text| Dynamic::from(text.to_string()));
    let value = match param.kind {
        Kind::Literal => cmd.given(name).then(|| true.into()),
        Kind::Int(_) | Kind::Time => cmd.int(name).map(Dynamic::from).or_else(text),
        Kind::Float => cmd.float(name).map(Dynamic::from),
        Kind::Word | Kind::Text | Kind::Player | Kind::Named => text(),
        Kind::Pos => cmd
            .pos(name)
            .map(|pos| [pos.x, pos.y, pos.z].map(Dynamic::from).to_vec().into()),
        Kind::Targets if cmd.given(name) => {
            let targets = cmd
                .targets(name)
                .into_iter()
                .map(|id| Dynamic::from(id as i64));
            Some(Dynamic::from_array(targets.collect()))
        }
        Kind::Targets => None,
    };
    value.map(|value| (param.name.clone(), value))
}

/// `/s <name> [args]`: any script command, its arguments read by the mod.
fn run_any(cmd: &mut ServerCommand) {
    if !developing(cmd) {
        return;
    }
    let name = cmd.text("name").unwrap_or_default().to_string();
    let line = cmd.text("args").unwrap_or_default().to_string();
    let Some(scripts) = script::current() else {
        return;
    };
    let Some(command) = scripts.command(&name) else {
        cmd.reply("serverscript.unknown_command", &[Arg::Text(&name)]);
        return;
    };
    if cmd.world().permission(cmd.player()) < command.permission {
        cmd.reply("serverscript.no_permission", &[Arg::Text(&name)]);
        return;
    }
    match command.usage.read(&line) {
        Ok(values) => {
            let named = |(name, value): (String, Value)| (name.into(), value.into_dynamic());
            run(cmd, &name, values.into_iter().map(named).collect());
        }
        Err(error) => {
            cmd.reply(error.key(), &[Arg::Text(error.subject())]);
            cmd.reply(
                "serverscript.usage",
                &[Arg::Text(&name), Arg::Text(&command.usage_text)],
            );
        }
    }
}

/// Runs the script command `name` with `arguments` for whoever typed `cmd`, who reads the text
/// it gives back.
fn run(cmd: &mut ServerCommand, name: &str, mut arguments: Map) {
    let Some(scripts) = script::current() else {
        return;
    };
    let Some(command) = scripts.command(name) else {
        cmd.reply("serverscript.unknown_command", &[Arg::Text(name)]);
        return;
    };
    arguments.insert("sender".into(), Dynamic::from(cmd.player() as i64));
    let result = bindings::within(cmd.world(), || scripts.run(command, arguments));
    match result.map(Dynamic::into_string) {
        Ok(Ok(text)) => cmd.reply(TEXT, &[Arg::Text(&text)]),
        Ok(Err(_)) => {}
        Err(error) => {
            warn!("{error}");
            cmd.reply("serverscript.command_failed", &[Arg::Text(name)]);
        }
    }
}

/// `/scripts reload`: reads the folder again and tells what it found.
fn reload(cmd: &mut ServerCommand, folder: &Path) {
    if !developing(cmd) {
        return;
    }
    let (scripts, problems) = Scripts::load(folder, bindings::register);
    for problem in &problems {
        warn!("{problem}");
        cmd.reply("serverscript.problem", &[Arg::Text(problem)]);
    }
    let (files, commands) = (scripts.len() as i64, scripts.commands().len() as i64);
    info!(
        "reloaded {files} scripts with {commands} commands from {}",
        folder.display()
    );
    cmd.reply(
        "serverscript.reloaded",
        &[Arg::Int(files), Arg::Int(commands)],
    );
    script::replace(scripts);
}
