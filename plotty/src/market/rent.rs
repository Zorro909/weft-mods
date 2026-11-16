//! Market places: the six outlined places of a market square, each with a rent post. A player
//! rents one for 1, 7 or 30 real days and may put a stall on it; the rent leaves the game. When
//! the time is up the stall closes, but its renter may still empty it, break it or renew; after
//! the grace days the place is free again.

use serde::{Deserialize, Serialize};
use weft_sdk::{Arg, Block, Output, Registry, ServerCommand, Use, World, permission};
use weftkit::{PlayerKey, Pos};

use crate::city::grid::CellXZ;
use crate::city::structures::{PLACE_SIZE, PLACES};
use crate::city::{CellKind, cell_at};
use crate::tables::{self, RENTS, STALLS};
use crate::woof::{self, Account, Refused};
use crate::{dims, players, settings};

pub static RENT_POST: Block = Block::named("rent_post");

/// Seconds in a day.
const DAY: u64 = 24 * 60 * 60;
/// The days a place is rented for, matching `settings::rent_woof`.
const TERMS: [u64; 3] = [1, 7, 30];

/// A place: the dimension, the market square's corner cell, and which of its places.
pub type PlaceKey = (u64, CellXZ, u8);

/// Who rents a place, and until when, in seconds since 1970.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Rent {
    pub renter: PlayerKey,
    pub until: u64,
}

/// Where a place's rent stands at a moment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Term {
    /// Nobody rents it, or a rent ended more than the grace days ago.
    Free,
    /// Rented, with this many seconds left.
    Running(u64),
    /// The rent ended; the place is free in this many seconds.
    Ended(u64),
}

/// Where a rent `until` stands at `now`, with `grace` seconds after it ends.
pub fn term(until: u64, now: u64, grace: u64) -> Term {
    if now < until {
        Term::Running(until - now)
    } else if now < until + grace {
        Term::Ended(until + grace - now)
    } else {
        Term::Free
    }
}

/// Which place of a market square holds the block `local` blocks from its corner, `margin`
/// blocks around each place counting too.
pub fn index_at(local: [i32; 2], margin: i32) -> Option<u8> {
    let [width, depth] = PLACE_SIZE;
    PLACES
        .iter()
        .position(|[x, z]| {
            (x - margin..x + width + margin).contains(&local[0])
                && (z - margin..z + depth + margin).contains(&local[1])
        })
        .map(|index| index as u8)
}

pub fn register(r: &mut Registry) {
    r.block(&RENT_POST)
        .texture("rent_post")
        .unbreakable()
        .on_use(tell)
        .tab(&crate::TAB);
    r.command(
        "market",
        "rent <days:word(1,7,30)> | places | info",
        command,
    )
    .permission(permission::EVERYONE);
}

/// The place holding the block `pos`, or by it with `margin` blocks around each place.
fn place_near(dim: u64, pos: Pos, margin: i32) -> Option<PlaceKey> {
    let cell = cell_at(dim, pos).filter(|cell| cell.kind == CellKind::Market)?;
    let corner = cell.corner.corner(pos.y);
    let index = index_at([pos.x - corner.x, pos.z - corner.z], margin)?;
    Some((dim, cell.corner, index))
}

/// The place holding the block `pos`.
pub fn place_at(dim: u64, pos: Pos) -> Option<PlaceKey> {
    place_near(dim, pos, 0)
}

/// Who rents the place `key` and where the rent stands now; `None` where it is free.
pub fn rent_of(key: &PlaceKey) -> Option<(PlayerKey, Term)> {
    let rent = RENTS.get(key)?;
    match term(rent.until, players::now(), grace()) {
        Term::Free => None,
        running => Some((rent.renter, running)),
    }
}

/// Where the rent of the place holding the block `pos` stands, if `owner` rents it.
pub fn owners_term(dim: u64, pos: Pos, owner: &PlayerKey) -> Option<Term> {
    let (renter, term) = rent_of(&place_at(dim, pos)?)?;
    (renter == *owner).then_some(term)
}

/// Whether any place of the market square with corner cell `corner` is rented.
pub fn any_rented(dim: u64, corner: CellXZ) -> bool {
    (0..PLACES.len() as u8).any(|index| rent_of(&(dim, corner, index)).is_some())
}

/// Forgets the rents of the market square with corner cell `corner`.
pub fn forget(dim: u64, corner: CellXZ) {
    (0..PLACES.len() as u8).for_each(|index| RENTS.remove(&(dim, corner, index)));
}

fn grace() -> u64 {
    u64::from(settings::get().rent_grace_days) * DAY
}

fn command(cmd: &mut ServerCommand) {
    let world = cmd.world();
    let player = cmd.player();
    let (Some(you), Some(state)) = (PlayerKey::of(&world, player), world.entity(player)) else {
        return cmd.reply("plotty.only_players", &[]);
    };
    let pos = Pos::from(state.pos.block());
    let dim = dims::key_of(&world);
    if cmd.given("places") {
        return list(cmd, &you);
    }
    let _changing = tables::changing();
    let Some(place) = place_at(dim, pos) else {
        return cmd.reply("plotty.rent.no_place", &[]);
    };
    if cmd.given("rent") {
        rent(cmd, place, &you);
    } else {
        say(&world, player, place);
    }
}

/// Rents `place` to `you` for the days the command names, or adds them to your rent.
fn rent(cmd: &mut ServerCommand, place: PlaceKey, you: &PlayerKey) {
    let days = cmd.text("days").and_then(|days| days.parse::<u64>().ok());
    let Some(term) = TERMS.iter().position(|term| Some(*term) == days) else {
        return cmd.reply("plotty.rent.bad_days", &[]);
    };
    let now = players::now();
    let current = rent_of(&place);
    match &current {
        Some((renter, _)) if renter != you => return cmd.reply("plotty.rent.taken", &[]),
        None if stall_on(place) => return cmd.reply("plotty.rent.occupied", &[]),
        _ => {}
    }
    let price = settings::get().rent_woof[term];
    if let Err(refused) = woof::transfer(Account::Player(you), Account::Outside, price) {
        let key = match refused {
            Refused::Short => "plotty.woof.short",
            Refused::TooMuch => "plotty.trade.too_much",
        };
        return cmd.reply(key, &[]);
    }
    let from = match current {
        Some((_, Term::Running(left))) => now + left,
        _ => now,
    };
    let until = from + TERMS[term] * DAY;
    RENTS.set(
        &place,
        Rent {
            renter: *you,
            until,
        },
    );
    let [days, hours] = days_and_hours(until - now);
    cmd.reply("plotty.rent.rented", &[Arg::Int(days), Arg::Int(hours)]);
}

/// Whether a stall's register stands on `place`.
fn stall_on((dim, corner, index): PlaceKey) -> bool {
    let mut found = false;
    STALLS.for_each(|(stall_dim, pos), _| {
        let local = [pos.x - corner.corner(0).x, pos.z - corner.corner(0).z];
        found |= stall_dim == dim && index_at(local, 0) == Some(index);
    });
    found
}

/// The places `you` rents, with the time left.
fn list(cmd: &mut ServerCommand, you: &PlayerKey) {
    let mut places = Vec::new();
    RENTS.for_each(|key, rent| {
        if rent.renter == *you {
            places.push(key);
        }
    });
    let yours: Vec<(PlaceKey, Term)> = places
        .into_iter()
        .filter_map(|key| Some((key, rent_of(&key)?.1)))
        .collect();
    cmd.reply("plotty.rent.yours", &[Arg::Int(yours.len() as i64)]);
    for ((_, corner, index), term) in yours {
        let at = corner.corner(0);
        let (key, [days, hours]) = match term {
            Term::Ended(left) => ("plotty.rent.yours_ended", days_and_hours(left)),
            Term::Running(left) => ("plotty.rent.yours_running", days_and_hours(left)),
            Term::Free => continue,
        };
        let args = [
            Arg::Int(i64::from(index) + 1),
            Arg::Int(at.x.into()),
            Arg::Int(at.z.into()),
            Arg::Int(days),
            Arg::Int(hours),
        ];
        cmd.reply(key, &args);
    }
}

/// A rent post tells its place's rent.
fn tell(using: &mut Use) -> bool {
    let world = using.world();
    let pos = Pos::from(using.pos());
    if let Some(place) = place_near(dims::key_of(&world), pos, 1) {
        say(&world, using.player(), place);
    }
    true
}

/// Tells `player` who rents `place` and for how long, or what renting it costs.
fn say(world: &World, player: u64, place: PlaceKey) {
    let number = Arg::Int(i64::from(place.2) + 1);
    let Some((renter, term)) = rent_of(&place) else {
        let [day, week, month] = settings::get().rent_woof.map(|woof| Arg::Int(woof as i64));
        world
            .chat("plotty.rent.free", &[number, day, week, month])
            .to(player);
        return;
    };
    let name = players::name_of(&renter);
    let (key, [days, hours]) = term_words(term);
    world
        .chat(key, &[Arg::Text(&name), Arg::Int(days), Arg::Int(hours)])
        .to(player);
}

/// The lang key for a rent's term and its days and hours.
fn term_words(term: Term) -> (&'static str, [i64; 2]) {
    match term {
        Term::Running(left) => ("plotty.rent.running", days_and_hours(left)),
        Term::Ended(left) => ("plotty.rent.ended", days_and_hours(left)),
        Term::Free => ("plotty.rent.free_now", [0, 0]),
    }
}

/// `seconds` as whole days and the hours left over.
fn days_and_hours(seconds: u64) -> [i64; 2] {
    let hours = seconds / 3600;
    [(hours / 24) as i64, (hours % 24) as i64]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rent_runs_then_has_its_grace() {
        let grace = 3 * DAY;
        assert_eq!(term(100, 40, grace), Term::Running(60), "before its end");
        assert_eq!(term(100, 100, grace), Term::Ended(grace), "just over");
        assert_eq!(
            term(100, 100 + grace, grace),
            Term::Free,
            "after the grace days"
        );
    }

    #[test]
    fn places_are_found_with_their_margin() {
        let [x, z] = PLACES[4];
        assert_eq!(index_at([x, z], 0), Some(4), "its first block");
        assert_eq!(index_at([x - 1, z], 0), None, "its edge is no part of it");
        assert_eq!(index_at([x - 1, z], 1), Some(4), "but near it");
        assert_eq!(index_at([16, 16], 1), None, "the middle of the square");
    }

    #[test]
    fn time_left_reads_in_days_and_hours() {
        assert_eq!(
            days_and_hours(DAY * 5 + 3 * 3600 + 59),
            [5, 3],
            "minutes drop"
        );
    }
}
