//! A stall's state, on its register's block entity. Its owner's Woof prices are saved in
//! `PRICES`. What the screens show is worked out from them and the register's slots once a
//! second into `STALL`, which isn't saved: whether it stands whole where it may, its offers, and
//! how often each can go through from the stall's side. Its owner is in `owner.rs`.
//!
//! The register holds the items: per offer a sample of what it gives and gets (`gives`, `gets`),
//! then the `stock` it sells from and the `till` payments in items go into.

use weft_sdk::{
    BlockBuilder, Component, Dir, Holds, ItemStack, Registry, SystemBatch, Vec3, World,
    component_flags, value,
};
use weftkit::{PlayerKey, Pos, TypedWorld};

use super::blocks;
use super::offer::{OFFERS, Offer};
use super::owner::{self, owner_at};
use super::pattern::{self, Part};
use super::rent::{self, Term};
use super::screens::SETUP;
use crate::city;
use crate::rules::Zone;
use crate::{dims, woof};

/// Slots a register holds: samples, stock, till.
pub const SLOTS: u32 = 2 * OFFERS as u32 + 2 * STOCK;
/// Slots of the stock, and of the till: three crates of nine.
pub const STOCK: u32 = 27;
/// Ticks between looks at a stall.
const EVERY: u64 = 20;
/// Blocks from a stall a player may change its prices or trade.
pub const REACH: f64 = 8.0;

value! {
    /// Woof prices of one offer: what the stall gives and gets, 0 where that side is items.
    #[derive(Default)]
    pub struct Prices {
        pub gives: u32,
        pub gets: u32,
    }
}

value! {
    /// The Woof prices of a stall's offers, as its owner set them. Saved: its layout is a file
    /// format, and new fields go last.
    #[derive(Default)]
    pub struct OfferPrices {
        pub offers: [Prices; OFFERS],
    }
}

pub static PRICES: Component<OfferPrices> = Component::named("stall_prices");

value! {
    /// What a stall's screens show. `status` is a `Status` by number, `missing` a `Part` by
    /// number where one is missing; `revision` changes whenever an offer does, so a trade
    /// clicked on an offer as it was is refused. Not saved: it holds item ids.
    #[derive(Default)]
    pub struct Stall {
        pub status: u32,
        pub missing: u32,
        pub revision: u32,
        pub prices: [Prices; OFFERS],
        pub samples: [ItemStack; 2 * OFFERS],
        pub left: [u32; OFFERS],
    }
}

pub static STALL: Component<Stall> = Component::named("stall");

/// Whether a stall trades, and why not. Kept in `Stall::status` by number.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Status {
    #[default]
    Open,
    /// A part is missing (`Stall::missing`).
    Missing,
    /// It stands where its owner may not have one: not their plot, nor a place they rent.
    Misplaced,
    /// Its owner's rent of its market place has ended.
    RentEnded,
}

impl Status {
    pub const ALL: [Status; 4] = [
        Status::Open,
        Status::Missing,
        Status::Misplaced,
        Status::RentEnded,
    ];

    /// The status of `stall`; one by a number it doesn't know, `Open`.
    pub fn of(stall: &Stall) -> Status {
        Status::ALL
            .get(stall.status as usize)
            .copied()
            .unwrap_or_default()
    }

    /// The lang key saying so.
    pub fn key(self) -> &'static str {
        match self {
            Status::Open => "plotty.stall.open",
            Status::Missing => "plotty.stall.missing",
            Status::Misplaced => "plotty.stall.misplaced",
            Status::RentEnded => "plotty.stall.rent_ended",
        }
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(&PRICES, OfferPrices::default(), component_flags::SAVE);
    r.entity_component(&STALL, Stall::default(), component_flags::SYNC_MENU);
    r.system(&STALL, tick);
}

/// The register's own part of its block: its state, items and owner.
pub fn finish_register(builder: BlockBuilder<'_>) -> BlockBuilder<'_> {
    builder
        .entity(|entity| entity.attach(&PRICES).attach(&STALL).inventory(SLOTS))
        .opens(&SETUP)
        .on_place(owner::placed)
        .on_break(owner::broken)
}

/// The way the register at `register` faces.
pub fn front_of(world: &World, register: Pos) -> Dir {
    world
        .block_typed(register)
        .prop("facing")
        .and_then(Dir::from_name)
        .unwrap_or(Dir::North)
}

/// Looks at every stall once a second: whether it stands whole and where it may, and what its
/// offers are and allow.
fn tick(batch: &mut SystemBatch<Stall>) {
    let world = batch.world();
    for (id, stall, _) in batch.iter_mut() {
        if (world.game_time() + id) % EVERY != 0 {
            continue;
        }
        let Some(entity) = world.entity(id) else {
            continue;
        };
        look(&world, id, Pos::from(entity.block_pos()), stall);
    }
}

/// Brings `stall`, the register `id` at `pos`, up to date.
pub fn look(world: &World, id: u64, pos: Pos, stall: &mut Stall) {
    let owner = owner_at(world, pos);
    let status = status_of(world, pos, owner.as_ref(), stall);
    stall.status = status as u32;
    stall.prices = world.component(id, &PRICES).unwrap_or_default().offers;
    let inventory = world.inventory(id);
    let mut samples = slots(&inventory, "gives");
    samples.extend(slots(&inventory, "gets"));
    let samples = samples
        .into_iter()
        .chain(std::iter::repeat(ItemStack::EMPTY));
    for (kept, sample) in stall.samples.iter_mut().zip(samples) {
        *kept = sample;
    }
    stall.revision = revision(stall);
    let stock = slots(&inventory, "stock");
    let till = slots(&inventory, "till");
    let owner_woof = owner.as_ref().map_or(0, woof::balance);
    for index in 0..OFFERS {
        stall.left[index] = offer_of(stall, index).map_or(0, |offer| {
            let times = offer.times_for_stall(&stock, &till, owner_woof, max_stack);
            times.min(u64::from(u32::MAX)) as u32
        });
    }
}

/// Whether the stall at `pos` stands whole where `owner` may have it; sets `stall.missing` to
/// the first part missing.
fn status_of(world: &World, pos: Pos, owner: Option<&PlayerKey>, stall: &mut Stall) -> Status {
    let checked = pattern::check(pos, front_of(world, pos), |at| {
        blocks::part_of(world.block(weft_sdk::BlockPos::from(at)))
    });
    match checked {
        Err((part, _)) => {
            stall.missing = Part::ALL.iter().position(|p| *p == part).unwrap_or(0) as u32;
            Status::Missing
        }
        Ok(()) => standing(world, pos, owner),
    }
}

/// The part `stall` lacks, where its status says one is missing.
pub fn missing(stall: &Stall) -> Part {
    Part::ALL
        .get(stall.missing as usize)
        .copied()
        .unwrap_or(Part::Register)
}

/// Offer `index` of `stall` as its samples and prices make it.
pub fn offer_of(stall: &Stall, index: usize) -> Option<Offer> {
    let prices = stall.prices[index];
    let samples = [stall.samples[index], stall.samples[OFFERS + index]];
    Offer::from(samples, [prices.gives, prices.gets])
}

/// A number that changes with any offer: its samples or prices.
fn revision(stall: &Stall) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    let mut mix = |value: u32| hash = (hash ^ value).wrapping_mul(0x0100_0193);
    for sample in stall.samples {
        [sample.item, sample.count, sample.data]
            .into_iter()
            .for_each(&mut mix);
    }
    for prices in stall.prices {
        [prices.gives, prices.gets].into_iter().for_each(&mut mix);
    }
    hash
}

/// Whether a player at `player` is near enough the stall at `register` to use it.
pub fn within_reach(register: Vec3, player: Vec3) -> bool {
    let (dx, dy, dz) = (
        register.x - player.x,
        register.y - player.y,
        register.z - player.z,
    );
    (dx * dx + dy * dy + dz * dz).sqrt() <= REACH
}

/// The most a stack of `kind` holds.
pub fn max_stack(kind: ItemStack) -> u32 {
    weft_sdk::MAX_STACK.on_stack(&kind).unwrap_or(64)
}

/// The stacks of one slot group of `inventory`.
pub fn slots(inventory: &weft_sdk::Inventory, group: &str) -> Vec<ItemStack> {
    let group = inventory.group(group);
    (0..group.len()).map(|slot| group.get(slot)).collect()
}

/// Whether a whole stall at `pos` of `owner` may trade: on a plot they own or are trusted on,
/// or on a market place while their rent runs.
fn standing(world: &World, pos: Pos, owner: Option<&PlayerKey>) -> Status {
    let Some(owner) = owner else {
        return Status::Misplaced;
    };
    let dim = dims::key_of(world);
    if city::zone(dim, pos, Some(owner)) == Some(Zone::OwnPlot) {
        return Status::Open;
    }
    match rent::owners_term(dim, pos, owner) {
        Some(Term::Running(_)) => Status::Open,
        Some(_) => Status::RentEnded,
        None => Status::Misplaced,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for status in Status::ALL {
            assert!(crate::translated(status.key()), "{status:?}");
        }
        for part in Part::ALL {
            assert!(crate::translated(part.key()), "{part:?}");
        }
    }

    #[test]
    fn an_unknown_number_reads_as_open() {
        let stall = Stall {
            status: 99,
            ..Stall::default()
        };
        assert_eq!(Status::of(&stall), Status::Open, "from a newer plotty");
    }

    #[test]
    fn the_saved_prices_lie_in_pairs() {
        use std::mem::{offset_of, size_of};
        assert_eq!(offset_of!(Prices, gets), 4, "after gives");
        assert_eq!(size_of::<OfferPrices>(), 8 * OFFERS, "eight pairs");
    }

    #[test]
    fn the_revision_follows_the_offers() {
        let mut stall = Stall::default();
        let before = revision(&stall);
        stall.prices[3].gets = 40;
        assert_ne!(revision(&stall), before, "a price changed");
    }
}
