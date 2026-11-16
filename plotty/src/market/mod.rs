//! Market stalls: a player builds one of Plotty's blocks on their own plot or a market place they
//! rent, and it trades for them
//! with any other player, online or not. Up to eight offers sell items or buy them, each side
//! items or Golden Woof.
//!
//! - `blocks.rs`: the blocks, in three woods; `pattern.rs`: how they make a stall.
//! - `stall.rs`: a stall's state on its register, and who owns it.
//! - `offer.rs`: what an offer is and when it can go through; `stacks.rs`: counting items.
//! - `screens.rs`: the owner's and the customers' screens; `trade.rs`: one trade.
//! - `rent.rs`: the places of a market square, rented for a stall.

mod blocks;
mod offer;
mod pattern;
mod rent;
mod screens;
mod stacks;
mod stall;
mod trade;

pub use blocks::part_of;
pub use rent::{Rent, any_rented, forget as forget_rents, owners_term};
pub use stall::StallEntry;
pub use trade::tell_sales;

pub fn register(r: &mut weft_sdk::Registry) {
    blocks::register(r);
    stall::register(r);
    screens::register(r);
    trade::register(r);
    rent::register(r);
}
