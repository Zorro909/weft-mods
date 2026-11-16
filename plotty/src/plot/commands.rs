//! `/plot`: claiming, sharing and giving up plots, seeing them, and what moderators do with them.

use weft_sdk::{Arg, Output, ServerCommand, World, permission};
use weftkit::{PlayerKey, Pos};

use super::{allowance_of, at, may_claim, owned_by, post};
use crate::city::grid::{CellXZ, Footprint};
use crate::city::{Plot, PlotSize};
use crate::players;
use crate::tables::{CITIES, PLAYERS, PLOTS};
use crate::{dims, protect};

const USAGE: &str = "info | claim | unclaim | trust <name:word> | untrust <name:word> | show | list \
                     | give <name:word> | take | allowance <name:word> <count:int> | bypass";

/// Cells out from the player whose plots `/plot show` outlines.
const SHOW_REACH: i32 = 4;
/// Ticks an outline stays: thirty seconds.
const SHOW_TICKS: u32 = 20 * 30;
/// Outline colors, as 0xRRGGBBAA: yours, someone else's, free.
const COLORS: [u32; 3] = [0x4CC38AFF, 0x6FA8DCFF, 0xF2F2F2FF];

/// Who runs a plot command, and where.
struct Here {
    world: World,
    player: u64,
    key: PlayerKey,
    dim: u64,
    pos: Pos,
}

pub fn register(r: &mut weft_sdk::Registry) {
    r.command("plot", USAGE, run)
        .permission(permission::EVERYONE);
}

fn run(cmd: &mut ServerCommand) {
    let Some(here) = here(cmd) else {
        return cmd.reply("plotty.only_players", &[]);
    };
    let moderator = here.world.permission(here.player) >= permission::MODERATOR;
    let only_moderators = ["give", "take", "allowance", "bypass"];
    if !moderator && only_moderators.iter().any(|word| cmd.given(word)) {
        return cmd.reply("plotty.moderators_only", &[]);
    }
    match () {
        _ if cmd.given("claim") => claim(cmd, &here),
        _ if cmd.given("unclaim") => unclaim(cmd, &here),
        _ if cmd.given("trust") => trust(cmd, &here, true),
        _ if cmd.given("untrust") => trust(cmd, &here, false),
        _ if cmd.given("show") => show(&here),
        _ if cmd.given("list") => list(cmd, &here),
        _ if cmd.given("give") => give(cmd, &here),
        _ if cmd.given("take") => take(cmd, &here),
        _ if cmd.given("allowance") => allowance(cmd),
        _ if cmd.given("bypass") => bypass(cmd, &here),
        _ => info(cmd, &here),
    }
}

fn here(cmd: &ServerCommand) -> Option<Here> {
    let (world, player) = (cmd.world(), cmd.player());
    let key = PlayerKey::of(&world, player)?;
    let pos = Pos::from(world.entity(player)?.pos.block());
    Some(Here {
        world,
        player,
        key,
        dim: dims::key_of(&world),
        pos,
    })
}

fn info(cmd: &mut ServerCommand, here: &Here) {
    let Some((id, plot)) = at(here.dim, here.pos) else {
        return cmd.reply("plotty.plot.none_here", &[]);
    };
    let city = CITIES
        .get(&plot.city)
        .map(|city| city.name)
        .unwrap_or_default();
    let owner = plot.owner.as_ref().map(players::name_of);
    let owner = owner.as_deref().unwrap_or("-");
    let args = [
        Arg::Text(&city),
        Arg::Int(id.into()),
        Arg::Key(post::size_key(plot.size)),
        Arg::Text(owner),
    ];
    cmd.reply("plotty.plot.info", &args);
}

fn claim(cmd: &mut ServerCommand, here: &Here) {
    let Some((id, mut plot)) = at(here.dim, here.pos) else {
        return cmd.reply("plotty.plot.none_here", &[]);
    };
    if plot.owner.is_some() {
        return cmd.reply("plotty.plot.taken", &[]);
    }
    let claimed: Vec<PlotSize> = owned_by(&here.key)
        .into_iter()
        .filter(|(_, owned)| !owned.given)
        .map(|(_, owned)| owned.size)
        .collect();
    if let Err(reason) = may_claim(plot.size, &claimed, allowance_of(&here.key)) {
        return cmd.reply(reason, &[]);
    }
    plot.owner = Some(here.key);
    PLOTS.set(&id, plot);
    cmd.reply("plotty.plot.claimed", &[Arg::Int(id.into())]);
}

fn unclaim(cmd: &mut ServerCommand, here: &Here) {
    let Some((id, plot)) = mine(cmd, here) else {
        return;
    };
    PLOTS.set(
        &id,
        Plot {
            owner: None,
            trusted: Vec::new(),
            given: false,
            ..plot
        },
    );
    cmd.reply("plotty.plot.unclaimed", &[Arg::Int(id.into())]);
}

/// Adds a player to the plot's trusted, or takes them off.
fn trust(cmd: &mut ServerCommand, here: &Here, add: bool) {
    let Some((id, mut plot)) = mine(cmd, here) else {
        return;
    };
    let Some((key, name)) = players::named(cmd) else {
        return;
    };
    plot.trusted.retain(|trusted| *trusted != key);
    if add {
        plot.trusted.push(key);
    }
    PLOTS.set(&id, plot);
    let reply = if add {
        "plotty.plot.trusted"
    } else {
        "plotty.plot.untrusted"
    };
    cmd.reply(reply, &[Arg::Text(&name)]);
}

/// Outlines the plots around the player for a while, in the color of whose they are.
fn show(here: &Here) {
    let centre = CellXZ::of(here.pos);
    let near = Footprint {
        corner: CellXZ::new(centre.x - SHOW_REACH, centre.z - SHOW_REACH),
        side: 2 * SHOW_REACH + 1,
    };
    let mut shown: Vec<u32> = Vec::new();
    for cell in near.cells() {
        let Some((id, plot)) = at(here.dim, cell.corner(here.pos.y)) else {
            continue;
        };
        if shown.contains(&id) {
            continue;
        }
        shown.push(id);
        let color = match &plot.owner {
            Some(owner) if plot.lets_build(&here.key) || *owner == here.key => COLORS[0],
            Some(_) => COLORS[1],
            None => COLORS[2],
        };
        let area = plot.footprint.area(here.pos.y - 2, here.pos.y + 6);
        let high = area.hi.add_x(1).add_y(1).add_z(1);
        here.world
            .draw_box(None, area.lo, high, color)
            .to(here.player)
            .ticks(SHOW_TICKS);
    }
}

fn list(cmd: &mut ServerCommand, here: &Here) {
    let owned = owned_by(&here.key);
    cmd.reply(
        "plotty.plot.yours",
        &[
            Arg::Int(owned.len() as i64),
            Arg::Int(allowance_of(&here.key).into()),
        ],
    );
    for (id, plot) in owned {
        let city = CITIES
            .get(&plot.city)
            .map(|city| city.name)
            .unwrap_or_default();
        let corner = plot.footprint.corner.corner(0);
        let args = [
            Arg::Int(id.into()),
            Arg::Text(&city),
            Arg::Key(post::size_key(plot.size)),
            Arg::Int(corner.x.into()),
            Arg::Int(corner.z.into()),
        ];
        cmd.reply("plotty.plot.yours_entry", &args);
    }
}

/// A moderator gives the plot here, of any size, to a player, outside their allowance.
fn give(cmd: &mut ServerCommand, here: &Here) {
    let Some((id, plot)) = at(here.dim, here.pos) else {
        return cmd.reply("plotty.plot.none_here", &[]);
    };
    let Some((key, name)) = players::named(cmd) else {
        return;
    };
    PLOTS.set(
        &id,
        Plot {
            owner: Some(key),
            trusted: Vec::new(),
            given: true,
            ..plot
        },
    );
    cmd.reply(
        "plotty.plot.given",
        &[Arg::Int(id.into()), Arg::Text(&name)],
    );
}

/// A moderator takes the plot here back from its owner.
fn take(cmd: &mut ServerCommand, here: &Here) {
    let Some((id, plot)) = at(here.dim, here.pos) else {
        return cmd.reply("plotty.plot.none_here", &[]);
    };
    PLOTS.set(
        &id,
        Plot {
            owner: None,
            trusted: Vec::new(),
            given: false,
            ..plot
        },
    );
    cmd.reply("plotty.plot.taken_back", &[Arg::Int(id.into())]);
}

fn allowance(cmd: &mut ServerCommand) {
    let Some((key, name)) = players::named(cmd) else {
        return;
    };
    let count = cmd.int("count").unwrap_or(0).clamp(0, 64) as u32;
    PLAYERS.update(&key, |entry| entry.plot_allowance = Some(count));
    cmd.reply(
        "plotty.plot.allowance_set",
        &[Arg::Text(&name), Arg::Int(count.into())],
    );
}

fn bypass(cmd: &mut ServerCommand, here: &Here) {
    let reply = if protect::toggle_bypass(here.key) {
        "plotty.bypass.on"
    } else {
        "plotty.bypass.off"
    };
    cmd.reply(reply, &[]);
}

/// The plot here, where the player owns it.
fn mine(cmd: &mut ServerCommand, here: &Here) -> Option<(u32, Plot)> {
    let Some((id, plot)) = at(here.dim, here.pos) else {
        cmd.reply("plotty.plot.none_here", &[]);
        return None;
    };
    if plot.owner.as_ref() != Some(&here.key) {
        cmd.reply("plotty.plot.not_yours", &[]);
        return None;
    }
    Some((id, plot))
}
