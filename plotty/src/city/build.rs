//! The build slot: one build at a time for the whole server. `/city build` (or `found`) checks
//! where it would go, shows its outline and holds the slot; `/city confirm` builds it
//! (`place.rs`), `/city cancel` or two minutes free the slot.

use std::sync::{Mutex, MutexGuard};

use weft_sdk::{Arg, Output, ServerCommand, World, permission};
use weftkit::{PlayerKey, Pos};

use super::grid::{CellXZ, Footprint};
use super::place;
use crate::players;
use crate::tables::{CELLS, CITIES};

/// Seconds a preview holds the slot before anyone may take it.
const HOLD_SECONDS: u64 = 120;
/// Ticks the outline of a preview stays: the same two minutes.
const OUTLINE_TICKS: u32 = 20 * 120;
/// Blocks of the outline above the ground.
const OUTLINE_HEIGHT: i32 = 8;
/// The outline's color, orange, as 0xRRGGBBAA.
const OUTLINE_COLOR: u32 = 0xFF8C2AFF;

/// What a build puts down.
#[derive(Clone, PartialEq, Debug)]
pub enum What {
    /// One of Plotty's structures.
    Kind(super::structures::Kind),
    /// A structgen building on the free plot `plot`: its template's name.
    Building { template: String, plot: u32 },
}

/// The city a build belongs to.
#[derive(Clone, PartialEq, Debug)]
pub enum Target {
    /// It founds a new city of this name.
    New(String),
    Existing(u32),
}

/// A build waiting for its `/city confirm`.
#[derive(Clone, PartialEq, Debug)]
pub struct Plan {
    pub what: What,
    pub target: Target,
    pub dim: u64,
    pub footprint: Footprint,
    pub ground_y: i32,
}

/// The build slot's holder and what they plan.
struct Pending {
    holder: PlayerKey,
    plan: Plan,
    since: u64,
}

static SLOT: Mutex<Option<Pending>> = Mutex::new(None);

/// Holds the slot for `plan` and shows its outline, unless someone else holds it.
pub fn propose(cmd: &mut ServerCommand, holder: PlayerKey, plan: Plan) {
    let world = cmd.world();
    let mut slot = lock();
    if let Some(other) = slot
        .as_ref()
        .filter(|pending| held(pending) && pending.holder != holder)
    {
        let corner = other.plan.footprint.corner.corner(other.plan.ground_y);
        let name = players::name_of(&other.holder);
        cmd.reply(
            "plotty.city.slot_taken",
            &[
                Arg::Text(&name),
                Arg::Int(corner.x.into()),
                Arg::Int(corner.z.into()),
            ],
        );
        return;
    }
    outline(&world, cmd.player(), &plan);
    let corner = plan.footprint.corner.corner(plan.ground_y);
    let word = match &plan.what {
        What::Kind(kind) => kind.word().to_string(),
        What::Building { template, .. } => template.clone(),
    };
    cmd.reply(
        "plotty.city.preview",
        &[
            Arg::Text(&word),
            Arg::Int(corner.x.into()),
            Arg::Int(corner.z.into()),
        ],
    );
    *slot = Some(Pending {
        holder,
        plan,
        since: players::now(),
    });
}

/// Builds the holder's plan and frees the slot.
pub fn confirm(cmd: &mut ServerCommand, holder: &PlayerKey) {
    let plan = {
        let slot = lock();
        match slot.as_ref().filter(|pending| held(pending)) {
            Some(pending) if &pending.holder == holder => pending.plan.clone(),
            Some(_) => return cmd.reply("plotty.city.not_your_preview", &[]),
            None => return cmd.reply("plotty.city.nothing_to_confirm", &[]),
        }
    };
    if place::build(cmd, &plan) {
        *lock() = None;
    }
}

/// Frees the slot: the holder's own preview, or anyone's for a game master.
pub fn cancel(cmd: &mut ServerCommand, holder: &PlayerKey) {
    let world = cmd.world();
    let mut slot = lock();
    let may = slot.as_ref().is_some_and(|pending| {
        &pending.holder == holder || world.permission(cmd.player()) >= permission::GAME_MASTER
    });
    if !may {
        return cmd.reply("plotty.city.nothing_to_cancel", &[]);
    }
    *slot = None;
    cmd.reply("plotty.city.cancelled", &[]);
}

/// Where `side` cells may go around the block `pos` for a city of ground `ground_y`, or why not:
/// cells nobody has, next to exactly one city (`founding`: next to none needed).
pub fn footprint_for(
    dim: u64,
    pos: Pos,
    side: i32,
    founding: bool,
) -> Result<(Footprint, Option<u32>), &'static str> {
    let footprint = Footprint::around(pos, side);
    let city_of = |cell: CellXZ| CELLS.get(&(dim, cell)).map(|cell| cell.city);
    match fit(footprint, founding, city_of) {
        Ok(city) => Ok((footprint, city)),
        Err(problem) => Err(problem),
    }
}

/// Whether `footprint` is free, and the one city it joins; `city_of` says whose a cell is.
fn fit(
    footprint: Footprint,
    founding: bool,
    city_of: impl Fn(CellXZ) -> Option<u32>,
) -> Result<Option<u32>, &'static str> {
    if footprint.cells().any(|cell| city_of(cell).is_some()) {
        return Err("plotty.city.taken");
    }
    let mut cities: Vec<u32> = footprint.ring().into_iter().filter_map(city_of).collect();
    cities.sort_unstable();
    cities.dedup();
    match (founding, cities.as_slice()) {
        (true, _) => Ok(None),
        (false, [city]) => Ok(Some(*city)),
        (false, []) => Err("plotty.city.not_next_to_city"),
        (false, _) => Err("plotty.city.two_cities"),
    }
}

/// The ground height of city `city`.
pub fn ground_of(city: u32) -> Option<i32> {
    CITIES.get(&city).map(|city| city.ground_y)
}

/// Shows `plan`'s outline to `player`.
fn outline(world: &World, player: u64, plan: &Plan) {
    let area = plan
        .footprint
        .area(plan.ground_y, plan.ground_y + OUTLINE_HEIGHT);
    let high = area.hi.add_x(1).add_y(1).add_z(1);
    world
        .draw_box(None, area.lo, high, OUTLINE_COLOR)
        .to(player)
        .ticks(OUTLINE_TICKS);
}

/// Whether a preview still holds the slot.
fn held(pending: &Pending) -> bool {
    players::now() < pending.since + HOLD_SECONDS
}

fn lock() -> MutexGuard<'static, Option<Pending>> {
    SLOT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: Footprint = Footprint {
        corner: CellXZ::new(0, 0),
        side: 1,
    };

    #[test]
    fn a_build_joins_the_one_city_beside_it() {
        let city_of = |cell: CellXZ| (cell == CellXZ::new(1, 0)).then_some(7);
        assert_eq!(fit(ONE, false, city_of), Ok(Some(7)), "city 7 is east");
    }

    #[test]
    fn a_build_must_touch_a_city_but_a_founding_need_not() {
        assert_eq!(
            fit(ONE, false, |_| None),
            Err("plotty.city.not_next_to_city"),
            "alone"
        );
        assert_eq!(fit(ONE, true, |_| None), Ok(None), "a new city");
    }

    #[test]
    fn taken_cells_and_two_cities_are_refused() {
        let taken = |cell: CellXZ| (cell == CellXZ::new(0, 0)).then_some(1);
        assert_eq!(
            fit(ONE, false, taken),
            Err("plotty.city.taken"),
            "already a city's"
        );
        let between = |cell: CellXZ| match cell.x {
            1 => Some(1),
            -1 => Some(2),
            _ => None,
        };
        assert_eq!(
            fit(ONE, false, between),
            Err("plotty.city.two_cities"),
            "between two"
        );
    }
}
