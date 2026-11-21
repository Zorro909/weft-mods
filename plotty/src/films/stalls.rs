//! The film's stalls: the camera's own on its plot, put together part by part on camera, and its
//! neighbours' on market places they rent, each stocked and priced as its owner would through
//! the setup screen. The camera then buys at Mira's through Plotty's own trade.

use showcase_api::Set;
use weft_sdk::{Dir, Item, ItemStack, World, warn};
use weftkit::{BlockState, PlayerKey, Pos, TypedWorld};

use super::town::{GROUND, MARKET, NEIGHBOURS};
use crate::city::grid::CellXZ;
use crate::city::structures::PLACES;
use crate::market::pattern::{self, Part};
use crate::market::stall::{self, PRICES, STALL};
use crate::market::trade::{self, Trade};
use crate::market::{Rent, StallEntry};
use crate::tables::{RENTS, STALLS};
use crate::{dims, players};

/// One side of an offer: items, by name and count, or Golden Woof.
#[derive(Clone, Copy)]
enum Side {
    Items(&'static str, u32),
    Woof(u32),
}

/// A stall of the film: its wood, where its register stands and which way it faces, whose it
/// is (a neighbour's key, or the camera's where `None`), its offers and its stock.
struct FilmStall {
    wood: &'static str,
    register: Pos,
    front: Dir,
    owner: Option<&'static str>,
    offers: &'static [(Side, Side)],
    stock: &'static [(&'static str, u32)],
}

/// The register of the camera's stall, in the middle of its plot, facing the plaza.
pub const YOUR_REGISTER: Pos = Pos::new(-137, GROUND + 1, -11);
/// Mira's register, on the first place of the market's south side, facing its middle.
pub const MIRAS_REGISTER: Pos = place_register(3);

const YOURS: FilmStall = FilmStall {
    wood: "spruce",
    register: YOUR_REGISTER,
    front: Dir::North,
    owner: None,
    offers: &[
        (Side::Items("base:torch", 16), Side::Woof(4)),
        (Side::Items("base:glass", 8), Side::Items("base:coal", 4)),
        (Side::Woof(6), Side::Items("base:wheat", 16)),
    ],
    stock: &[("base:torch", 64), ("base:torch", 40), ("base:glass", 64)],
};

/// The neighbours' stalls, on the three places along the market's south side.
const MARKET_STALLS: [FilmStall; 3] = [
    FilmStall {
        wood: "oak",
        register: MIRAS_REGISTER,
        front: Dir::North,
        owner: Some(NEIGHBOURS[0].0),
        offers: &[
            (Side::Items("base:bread", 4), Side::Woof(5)),
            (Side::Items("base:apple", 8), Side::Woof(4)),
            (Side::Woof(10), Side::Items("base:wheat", 16)),
        ],
        stock: &[("base:bread", 64), ("base:bread", 20), ("base:apple", 48)],
    },
    FilmStall {
        wood: "spruce",
        register: place_register(4),
        front: Dir::North,
        owner: Some(NEIGHBOURS[1].0),
        offers: &[
            (Side::Items("base:iron_ingot", 2), Side::Woof(12)),
            (
                Side::Items("base:iron_pickaxe", 1),
                Side::Items("base:iron_ingot", 4),
            ),
        ],
        stock: &[
            ("base:iron_ingot", 30),
            ("base:iron_pickaxe", 1),
            ("base:iron_pickaxe", 1),
        ],
    },
    FilmStall {
        wood: "birch",
        register: place_register(5),
        front: Dir::North,
        owner: Some(NEIGHBOURS[2].0),
        offers: &[
            (Side::Items("base:oak_log", 16), Side::Woof(6)),
            (Side::Items("base:birch_planks", 32), Side::Woof(5)),
        ],
        stock: &[("base:oak_log", 64), ("base:birch_planks", 64)],
    },
];

/// Days the neighbours rent their places for.
const RENT_DAYS: u64 = 7;

/// Where the register stands on place `index` of the market's south side: in the middle of its
/// front row, toward the market's middle, the crates behind it.
const fn place_register(index: usize) -> Pos {
    let [x, z] = PLACES[index];
    Pos::new(MARKET[0] * 16 + x + 2, GROUND + 1, MARKET[1] * 16 + z + 1)
}

/// Puts the parts `parts` of the camera's stall: its crates, counters, posts, awning and
/// register, a step at a time on camera.
pub fn assemble(set: &Set, parts: &[Part]) {
    place(&set.world(), &YOURS, parts);
}

/// The neighbours rent their places and put up their stalls, while the camera is away.
pub fn market(set: &Set) {
    let world = set.world();
    let dim = dims::key_of(&world);
    let until = players::now() + RENT_DAYS * 24 * 60 * 60;
    let corner = CellXZ::new(MARKET[0], MARKET[1]);
    for (index, stall) in (3..).zip(&MARKET_STALLS) {
        place(&world, stall, &Part::ALL);
        if let Some(renter) = stall.owner.and_then(PlayerKey::new) {
            RENTS.set(&(dim, corner, index), Rent { renter, until });
        }
    }
}

/// The neighbours' stalls get their owners, offers and stock: a tick after their registers were
/// placed, when their block entities are there.
pub fn stock_market(set: &Set) {
    let world = set.world();
    for stall in &MARKET_STALLS {
        if let Some(owner) = stall.owner.and_then(PlayerKey::new) {
            fill(&world, stall, owner);
        }
    }
}

/// The camera's stall gets its offers and stock, as its owner sets them up.
pub fn stock_yours(set: &Set) {
    let world = set.world();
    if let Some(you) = PlayerKey::of(&world, set.player()) {
        fill(&world, &YOURS, you);
    }
}

/// The camera buys Mira's first offer once, as the shop's button does.
pub fn buy(set: &Set) {
    let world = set.world();
    let id = world.block_entity(MIRAS_REGISTER);
    let Some(stall) = world.component_mut(id, &STALL) else {
        return;
    };
    stall::look(&world, id, MIRAS_REGISTER, stall);
    let trade = Trade {
        register: id,
        offer: 0,
        revision: stall.revision,
    };
    if let Err(refused) = trade::attempt(&world, set.player(), trade) {
        warn!("plotty film: the trade was refused: {}", refused.key);
    }
}

/// Places `parts` of `stall`.
fn place(world: &World, stall: &FilmStall, parts: &[Part]) {
    for (pos, part) in pattern::layout(stall.register, stall.front) {
        if !parts.contains(&part) {
            continue;
        }
        let name = match part {
            Part::Register => format!("register[facing={}]", stall.front.name()),
            Part::Counter => "counter".to_string(),
            Part::Crate => "crate".to_string(),
            Part::Post => "post".to_string(),
            Part::Awning => "awning".to_string(),
        };
        let name = format!("plotty:{}_stall_{name}", stall.wood);
        match BlockState::named(&name) {
            Some(state) => {
                world.set_block_typed(pos, state);
            }
            None => warn!("plotty film: no block {name}"),
        }
    }
}

/// Gives `stall` its owner, its offers' samples and Woof prices, and its stock.
fn fill(world: &World, stall: &FilmStall, owner: PlayerKey) {
    let dim = dims::key_of(world);
    STALLS.set(&(dim, stall.register), StallEntry { owner });
    let id = world.block_entity(stall.register);
    let inventory = world.inventory(id);
    let (gives, gets) = (inventory.group("gives"), inventory.group("gets"));
    let mut prices = world.component(id, &PRICES).unwrap_or_default();
    for (index, (give, get)) in stall.offers.iter().enumerate() {
        let slot = index as u32;
        gives.set(slot, sample(*give));
        gets.set(slot, sample(*get));
        prices.offers[index].gives = woof(*give);
        prices.offers[index].gets = woof(*get);
    }
    if let Some(saved) = world.component_mut(id, &PRICES) {
        *saved = prices;
    }
    let stock = inventory.group("stock");
    for (slot, (item, count)) in stall.stock.iter().enumerate() {
        stock.set(slot as u32, items(item, *count));
    }
}

/// The sample a side puts into its slot: its items, or nothing for Woof.
fn sample(side: Side) -> ItemStack {
    match side {
        Side::Items(item, count) => items(item, count),
        Side::Woof(_) => ItemStack::EMPTY,
    }
}

/// The Woof price a side sets: 0 for items.
fn woof(side: Side) -> u32 {
    match side {
        Side::Items(..) => 0,
        Side::Woof(price) => price,
    }
}

fn items(name: &str, count: u32) -> ItemStack {
    match Item::lookup(name) {
        Some(item) => ItemStack::new(item, count),
        None => {
            warn!("plotty film: no item {name}");
            ItemStack::EMPTY
        }
    }
}
