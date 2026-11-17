//! One tick of a built column. It runs while its controller holds crude and, for what a tick's
//! crude makes at its temperature (`products::cut`), every product has room in its taps and its
//! sulfur room in its output, so nothing it makes is lost; gas with no tap of its own vents. A
//! column that runs draws its heat, 40 mB of steam from its heaters or else 300 energy from
//! their burners, and once warm 50 mB of cooling water from its condensers, and distils if its
//! temperature lets it. One that stops takes neither. Last, the heat that came in and went out
//! moves its temperature (`heat::step`).

use strum::VariantArray;
use weft_sdk::{ENERGY, ItemStack, World};

use super::super::column::Layout;
use super::super::heat::{self, Band};
use super::super::products::{self, CRUDE_A_SULFUR, CRUDE_A_TICK, Cut, Product};
use super::super::{Heating, Refinery, Status};
use crate::lib::Pos;
use crate::liquid::{HELD, Held, Holders, LiquidKey, Vessel};

/// Millibuckets of steam a tick heats it, or energy its burners give instead.
const STEAM_A_TICK: u32 = 40;
const BURNER_A_TICK: i64 = 300;
/// Millibuckets of water a tick cools it.
const WATER_A_TICK: u32 = 50;

/// Runs the column of the controller `id` laid out as `layout` one tick.
pub fn tick(world: &World, id: u64, refinery: &mut Refinery, layout: &Layout) {
    let band = heat::band(refinery.warmth);
    let cut = products::cut(band);
    let stop = stopped(world, id, refinery, layout, band, cut);
    let running = stop.is_none();
    let heating = if running {
        draw_heat(world, &layout.heaters)
    } else {
        Heating::Unheated
    };
    let cooled = running
        && band != Band::Cold
        && draw_liquid(world, &layout.condensers, water(), WATER_A_TICK);
    let fed = running && cut.is_some();
    if let (true, Some(cut)) = (running, cut) {
        distil(world, id, refinery, layout, band, cut);
    }
    let heated = heating != Heating::Unheated;
    refinery.warmth = heat::step(
        refinery.warmth,
        heat::Tick {
            heated,
            fed,
            cooled,
        },
    );
    refinery.heating = heating as u32;
    refinery.cooled = u32::from(cooled);
    refinery.set_status(stop.unwrap_or(running_status(band, heated)));
}

/// What keeps the column of the controller `id` at `band` from running a tick, if anything: no
/// crude, or no room for what a tick's `cut` makes. A stopped column takes no heat nor cooling.
fn stopped(
    world: &World,
    id: u64,
    refinery: &mut Refinery,
    layout: &Layout,
    band: Band,
    cut: Option<Cut>,
) -> Option<Status> {
    refinery.blocked = 0;
    if crude_held(world, id) < CRUDE_A_TICK {
        return Some(Status::NoCrude);
    }
    let cut = cut?;
    let sulfur_due =
        products::gives_sulfur(band) && refinery.toward_sulfur + CRUDE_A_TICK >= CRUDE_A_SULFUR;
    if sulfur_due && !sulfur_fits(world, id, true) {
        return Some(Status::Full);
    }
    let product = no_room(world, layout, cut)?;
    refinery.blocked = product as u32;
    Some(Status::TapFull)
}

/// Distils a tick's crude in the column of the controller `id` at `band` into `cut`, which
/// `stopped` found room for.
fn distil(world: &World, id: u64, refinery: &mut Refinery, layout: &Layout, band: Band, cut: Cut) {
    take_crude(world, id);
    for (&product, &made) in Product::VARIANTS.iter().zip(&cut.made) {
        fill(world, layout.taps_of(product), product.key(), made);
    }
    if products::gives_sulfur(band) {
        refinery.toward_sulfur += CRUDE_A_TICK;
    }
    if refinery.toward_sulfur >= CRUDE_A_SULFUR {
        sulfur_fits(world, id, false);
        refinery.toward_sulfur -= CRUDE_A_SULFUR;
    }
}

/// What a column that runs says it does at `band`, `heated` or not.
fn running_status(band: Band, heated: bool) -> Status {
    match band {
        Band::Cold if heated => Status::Heating,
        Band::Cold => Status::NoHeat,
        Band::Under => Status::TooCold,
        Band::In => Status::Distilling,
        Band::Over => Status::TooHot,
    }
}

/// The first product of `cut` its taps have no room for; gas with no taps vents instead.
fn no_room(world: &World, layout: &Layout, cut: Cut) -> Option<Product> {
    Product::VARIANTS
        .iter()
        .copied()
        .zip(cut.made)
        .filter(|&(product, made)| made > 0 && !vents(layout, product))
        .find(|&(product, made)| room(world, layout.taps_of(product), product.key()) < made)
        .map(|(product, _)| product)
}

/// Whether `product` vents where it has no taps: gas alone, as it has no use yet.
fn vents(layout: &Layout, product: Product) -> bool {
    product == Product::Gas && layout.taps_of(product).is_empty()
}

/// Millibuckets of `liquid` the taps at `taps` have room for together.
fn room(world: &World, taps: &[Pos], liquid: LiquidKey) -> u32 {
    let holders = Holders::outside(*world);
    taps.iter()
        .filter_map(|&tap| Vessel::at(world, tap))
        .map(|tap| room_in(&tap.held(&holders), liquid, tap.capacity()))
        .sum()
}

/// Millibuckets of `liquid` a tap of `capacity` holding `held` has room for. A tap takes only
/// the liquid it holds, or any once it is empty: never in place of a leftover of another, as
/// liquids otherwise may, because a tap two columns share can be one's diesel and the other's
/// lubricant, and each would wash the other's away.
fn room_in(held: &Held, liquid: LiquidKey, capacity: u32) -> u32 {
    if held.liquid == liquid {
        capacity.saturating_sub(held.amount)
    } else if held.amount == 0 {
        capacity
    } else {
        0
    }
}

/// Puts `amount` of `liquid` into the taps at `taps`, as far as `room` said they have room.
fn fill(world: &World, taps: &[Pos], liquid: LiquidKey, mut amount: u32) {
    let mut holders = Holders::outside(*world);
    for &tap in taps {
        let Some(tap) = Vessel::at(world, tap).filter(|_| amount > 0) else {
            continue;
        };
        let put = amount.min(room_in(&tap.held(&holders), liquid, tap.capacity()));
        if put > 0 {
            tap.put(&mut holders, liquid, put);
            amount -= put;
        }
    }
}

/// Puts a sulfur into the output of the controller `id`; whether it fit. With `simulate`
/// nothing changes.
fn sulfur_fits(world: &World, id: u64, simulate: bool) -> bool {
    let output = world.inventory(id).group("output");
    let sulfur: ItemStack = crate::coke_oven::SULFUR.stack(1);
    crate::store::insert_into(output, sulfur, simulate).is_empty()
}

/// Millibuckets of crude the controller `id` holds.
fn crude_held(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if held.liquid == crude() {
        held.amount
    } else {
        0
    }
}

/// Takes a tick's crude out of the controller `id`, which holds that much.
fn take_crude(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(CRUDE_A_TICK);
    }
}

/// Heats the column a tick from its `heaters`: by steam where they hold enough together, else by
/// their burners' energy; or not at all.
fn draw_heat(world: &World, heaters: &[Pos]) -> Heating {
    if draw_liquid(world, heaters, crate::steam::key(), STEAM_A_TICK) {
        return Heating::Steam;
    }
    let stores = heaters.iter().map(|&heater| world.block_entity(heater));
    let stored: i64 = stores
        .clone()
        .filter_map(|id| world.component(id, &ENERGY))
        .map(|energy| energy.stored)
        .sum();
    if stored < BURNER_A_TICK {
        return Heating::Unheated;
    }
    let mut owed = BURNER_A_TICK;
    for id in stores {
        if let Some(energy) = world.component_mut(id, &ENERGY) {
            let taken = energy.stored.min(owed);
            energy.stored -= taken;
            owed -= taken;
        }
    }
    Heating::Burner
}

/// Takes `amount` of `liquid` out of the holders at `at` together, if they hold that much;
/// whether it did.
fn draw_liquid(world: &World, at: &[Pos], liquid: LiquidKey, amount: u32) -> bool {
    let ids: Vec<u64> = at.iter().map(|&pos| world.block_entity(pos)).collect();
    let held_there = |id: u64| {
        let held = world.component(id, &HELD).unwrap_or_default();
        if held.liquid == liquid {
            held.amount
        } else {
            0
        }
    };
    if ids.iter().map(|&id| held_there(id)).sum::<u32>() < amount {
        return false;
    }
    let mut owed = amount;
    for &id in &ids {
        let taken = held_there(id).min(owed);
        if let Some(held) = world.component_mut(id, &HELD).filter(|_| taken > 0) {
            held.remove(taken);
            owed -= taken;
        }
    }
    true
}

/// The keys of crude oil and of water, as holders keep liquids.
fn crude() -> LiquidKey {
    crate::liquid::key_named("cogwork:oil")
}

fn water() -> LiquidKey {
    crate::liquid::key_named("base:water")
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIESEL: LiquidKey = 7;
    const LUBRICANT: LiquidKey = 8;
    const TAP: u32 = 1_000;

    #[test]
    fn a_tap_takes_its_own_liquid_or_any_once_empty() {
        let empty = Held::default();
        assert_eq!(room_in(&empty, DIESEL, TAP), TAP, "empty");
        let mut diesel = Held::default();
        diesel.add(DIESEL, 600);
        assert_eq!(room_in(&diesel, DIESEL, TAP), 400, "the rest");
        assert_eq!(room_in(&diesel, LUBRICANT, TAP), 0, "not another");
    }

    #[test]
    fn a_tap_two_columns_share_loses_nothing() {
        // One column's diesel and the other's lubricant, in turns, into the same tap.
        let mut tap = Held::default();
        let mut made = 0;
        for turn in 0..150 {
            let (liquid, amount) = if turn % 2 == 0 {
                (DIESEL, 4)
            } else {
                (LUBRICANT, 2)
            };
            let put = amount.min(room_in(&tap, liquid, TAP));
            if put > 0 {
                tap.add(liquid, put);
                made += put;
            }
        }
        assert_eq!(tap.amount, made, "all it was given is still there");
        assert_eq!(
            tap.liquid, DIESEL,
            "the first one's, the other column waits"
        );
    }

    #[test]
    fn a_running_column_says_where_its_temperature_stands() {
        assert_eq!(running_status(Band::Cold, true), Status::Heating, "warming");
        assert_eq!(running_status(Band::Cold, false), Status::NoHeat, "cold");
        assert_eq!(running_status(Band::Under, true), Status::TooCold, "tar");
        assert_eq!(
            running_status(Band::In, false),
            Status::Distilling,
            "in band"
        );
        assert_eq!(running_status(Band::Over, true), Status::TooHot, "venting");
    }
}
