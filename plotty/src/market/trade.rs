//! One trade at a stall, when a customer clicks an offer's button: every check and every move
//! in one server callback in the stall's region, so the customer's inventory, the stall's stock
//! and till, and both balances change together or not at all. The answer comes above the
//! hotbar.

use weft_sdk::{Arg, Channel, Inventory, ItemStack, Output, Packet, Registry, World, value};
use weftkit::{PlayerKey, Pos};

use super::offer::{Goods, Holdings, OFFERS, Offer};
use super::owner::owner_at;
use super::pattern::Part;
use super::stacks;
use super::stall::{self, STALL, Status};
use crate::tables::SALES;
use crate::woof::{self, Account, Refused};

value! {
    /// A customer trades offer `offer` of the register `register` once, as they saw it at
    /// `revision`.
    pub struct Trade {
        pub register: u64,
        pub offer: u32,
        pub revision: u32,
    }
}

pub static TRADE: Channel<Trade> = Channel::named("stall_trade");

/// The customer's own slots a trade takes from and gives into.
const YOUR_GROUPS: [&str; 2] = ["hotbar", "main"];

pub fn register(r: &mut Registry) {
    r.packet(&TRADE, answer);
}

/// Why a trade didn't happen: a lang key, and the part a closed stall lacks for its argument.
pub struct Refusal {
    pub key: &'static str,
    part: Option<Part>,
}

impl From<&'static str> for Refusal {
    fn from(key: &'static str) -> Refusal {
        Refusal { key, part: None }
    }
}

fn answer(packet: &Packet, trade: Trade) {
    let world = packet.world();
    let player = packet.player();
    let Err(Refusal { key, part }) = attempt(&world, player, trade) else {
        world.hint("plotty.trade.done", &[]).to(player);
        return;
    };
    let args: Vec<Arg> = part.map(|part| Arg::Key(part.key())).into_iter().collect();
    world.hint(key, &args).to(player);
}

/// Tells a player who just joined how often their stalls traded since they last did.
pub fn tell_sales(world: &World, player: u64, key: &PlayerKey) {
    let sales = SALES.get(key).unwrap_or(0);
    if sales > 0 {
        world
            .chat("plotty.trade.since_join", &[Arg::Int(sales.into())])
            .to(player);
        SALES.remove(key);
    }
}

/// Trades once, or says why not.
pub fn attempt(world: &World, player: u64, trade: Trade) -> Result<(), Refusal> {
    let gone = "plotty.trade.gone";
    let register = world.entity(trade.register).ok_or(gone)?;
    let you = world.entity(player).ok_or(gone)?;
    if !stall::within_reach(register.pos, you.pos) {
        return Err("plotty.trade.far".into());
    }
    let pos = Pos::from(register.block_pos());
    let stall = world.component_mut(trade.register, &STALL).ok_or(gone)?;
    stall::look(world, trade.register, pos, stall);
    let status = Status::of(stall);
    if status != Status::Open {
        return Err(Refusal {
            key: status.key(),
            part: Some(stall::missing(stall)),
        });
    }
    if stall.revision != trade.revision {
        return Err("plotty.trade.changed".into());
    }
    let no_offer = "plotty.trade.no_offer";
    let index = trade.offer as usize;
    let offer = (index < OFFERS)
        .then(|| stall::offer_of(stall, index))
        .flatten()
        .ok_or(no_offer)?;
    let owner = owner_at(world, pos).ok_or(gone)?;
    let customer = PlayerKey::of(world, player).ok_or(gone)?;
    if owner == customer {
        return Err("plotty.trade.own_stall".into());
    }
    exchange(world, trade.register, player, offer, [owner, customer])?;
    SALES.update(&owner, |sales| *sales = sales.saturating_add(1));
    stall::look(world, trade.register, pos, stall);
    Ok(())
}

/// Moves both sides of `offer` between `register` and `player` (`owner` and `customer`).
fn exchange(
    world: &World,
    register: u64,
    player: u64,
    offer: Offer,
    [owner, customer]: [PlayerKey; 2],
) -> Result<(), &'static str> {
    let stall_items = world.inventory(register);
    let your_items = world.inventory(player);
    let mut stock = stall::slots(&stall_items, "stock");
    let mut till = stall::slots(&stall_items, "till");
    let mut yours = your_slots(&your_items);
    let held = Holdings {
        stock: &stock,
        till: &till,
        owner_woof: woof::balance(&owner),
        yours: &yours,
        your_woof: woof::balance(&customer),
        max_stack: stall::max_stack,
    };
    offer.check(&held).map_err(|short| short.key())?;
    pay_woof(offer, &owner, &customer)?;
    // The customer pays first, so what they get may go where their payment was.
    if let Goods::Items(kind) = offer.gets {
        stacks::take(&mut yours, kind, kind.count);
        stacks::put(&mut till, kind, kind.count, stall::max_stack(kind));
    }
    if let Goods::Items(kind) = offer.gives {
        stacks::take(&mut stock, kind, kind.count);
        stacks::put(&mut yours, kind, kind.count, stall::max_stack(kind));
    }
    write(&stall_items, "stock", &stock);
    write(&stall_items, "till", &till);
    write_yours(&your_items, &yours);
    Ok(())
}

/// The Woof side of `offer`, if it has one: from the owner's balance or into it.
fn pay_woof(offer: Offer, owner: &PlayerKey, customer: &PlayerKey) -> Result<(), &'static str> {
    let (from, to, price, short) = match (offer.gives, offer.gets) {
        (Goods::Woof(price), _) => (owner, customer, price, "plotty.trade.owner_woof"),
        (_, Goods::Woof(price)) => (customer, owner, price, "plotty.trade.your_woof"),
        _ => return Ok(()),
    };
    woof::transfer(Account::Player(from), Account::Player(to), price).map_err(|refused| {
        match refused {
            Refused::Short => short,
            Refused::TooMuch => "plotty.trade.too_much",
        }
    })
}

/// The customer's hotbar and main inventory, one after the other.
fn your_slots(inventory: &Inventory) -> Vec<ItemStack> {
    YOUR_GROUPS
        .iter()
        .flat_map(|group| stall::slots(inventory, group))
        .collect()
}

/// Puts `slots`, as `your_slots` read them, back.
fn write_yours(inventory: &Inventory, slots: &[ItemStack]) {
    let mut rest = slots;
    for name in YOUR_GROUPS {
        let len = inventory.group(name).len() as usize;
        let (these, after) = rest.split_at(len.min(rest.len()));
        write(inventory, name, these);
        rest = after;
    }
}

/// Sets the slots of `group` that differ from `slots`.
fn write(inventory: &Inventory, group: &str, slots: &[ItemStack]) {
    let group = inventory.group(group);
    for (slot, stack) in slots.iter().enumerate() {
        let slot = slot as u32;
        if group.get(slot) != *stack {
            group.set(slot, *stack);
        }
    }
}
