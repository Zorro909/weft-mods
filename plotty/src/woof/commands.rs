//! `/woof`: your balance, paying another player, and the game masters' giving and taking, the
//! only Woof that comes into the game besides each player's starting balance.

use weft_sdk::{Arg, ServerCommand, permission};
use weftkit::PlayerKey;

use super::{Account, balance, transfer};
use crate::players;

const USAGE: &str = "[name:word] | pay <name:word> <amount:int> | give <name:word> <amount:int> \
                     | take <name:word> <amount:int>";

pub fn register(r: &mut weft_sdk::Registry) {
    r.command("woof", USAGE, run)
        .permission(permission::EVERYONE);
}

fn run(cmd: &mut ServerCommand) {
    let world = cmd.world();
    let level = world.permission(cmd.player());
    let Some(key) = PlayerKey::of(&world, cmd.player()) else {
        return cmd.reply("plotty.only_players", &[]);
    };
    let payment = cmd.given("pay") || cmd.given("give") || cmd.given("take");
    let by_game_master = cmd.given("give") || cmd.given("take");
    if by_game_master && level < permission::GAME_MASTER {
        return cmd.reply("plotty.game_masters_only", &[]);
    }
    if payment {
        pay(cmd, &key);
    } else if cmd.text("name").is_none() {
        let held = balance(&key) as i64;
        cmd.reply("plotty.woof.yours", &[Arg::Int(held)]);
    } else if level < permission::MODERATOR {
        cmd.reply("plotty.moderators_only", &[]);
    } else {
        theirs(cmd);
    }
}

/// Another player's balance, for a moderator.
fn theirs(cmd: &mut ServerCommand) {
    let Some((other, name)) = players::named(cmd) else {
        return;
    };
    let held = balance(&other) as i64;
    cmd.reply("plotty.woof.theirs", &[Arg::Text(&name), Arg::Int(held)]);
}

/// `pay`, `give` or `take`: Woof from `key` to the named player, from outside the game to them,
/// or from them out of the game.
fn pay(cmd: &mut ServerCommand, key: &PlayerKey) {
    let Some((other, name)) = players::named(cmd) else {
        return;
    };
    let Some(amount) = amount(cmd) else {
        return;
    };
    let (from, to, done) = if cmd.given("pay") {
        if other == *key {
            return cmd.reply("plotty.woof.self", &[]);
        }
        let paid = "plotty.woof.paid";
        (Account::Player(key), Account::Player(&other), paid)
    } else if cmd.given("give") {
        (
            Account::Outside,
            Account::Player(&other),
            "plotty.woof.given",
        )
    } else {
        (
            Account::Player(&other),
            Account::Outside,
            "plotty.woof.taken",
        )
    };
    match transfer(from, to, amount) {
        Ok(()) => cmd.reply(done, &[Arg::Int(amount as i64), Arg::Text(&name)]),
        Err(refused) => cmd.reply(refused.key(), &[Arg::Text(&name)]),
    }
}

/// The `amount` parameter, where it is more than nothing.
fn amount(cmd: &mut ServerCommand) -> Option<u64> {
    let amount = cmd.int("amount").unwrap_or(0);
    if amount < 1 {
        cmd.reply("plotty.woof.bad_amount", &[]);
        return None;
    }
    Some(amount as u64)
}
