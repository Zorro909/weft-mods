//! What an entity or a block carries: its stacks read, items given and taken.
//!
//! Scripts see a stack as its item's name and a count. What else a stack holds, wear or a
//! name of its own, is not shown; `give` makes new items without any.

use rhai::{Dynamic, Engine, Map};
use weft_sdk::{Inventory, Item, ItemStack, item_info, item_name};

use super::{current, entity, holder};
use crate::script::engine::{ScriptResult, fail};

pub fn register(engine: &mut Engine) {
    engine.register_fn("inventory", |of: Dynamic| -> ScriptResult<Dynamic> {
        let inventory = current()?.inventory(holder(&of)?);
        let stacks = (0..inventory.len()).filter_map(|slot| shown(slot, inventory.get(slot)));
        Ok(Dynamic::from_array(stacks.map(Dynamic::from_map).collect()))
    });
    engine.register_fn("held", |player: i64| -> ScriptResult<Dynamic> {
        let world = current()?;
        let slot = world.selected_slot(entity(player));
        let stack = world.inventory(entity(player)).get(slot);
        let held = (!stack.is_empty()).then(|| item_name(stack.item));
        Ok(held.map_or(Dynamic::UNIT, Dynamic::from))
    });
    engine.register_fn(
        "give",
        |to: Dynamic, item: &str, count: i64| -> ScriptResult<i64> {
            let inventory = current()?.inventory(holder(&to)?);
            Ok(i64::from(give(&inventory, named(item)?, amount(count))))
        },
    );
    engine.register_fn(
        "take",
        |from: Dynamic, item: &str, count: i64| -> ScriptResult<i64> {
            let inventory = current()?.inventory(holder(&from)?);
            Ok(i64::from(take(&inventory, named(item)?, amount(count))))
        },
    );
}

fn named(item: &str) -> ScriptResult<u32> {
    Item::lookup(item).map_or_else(|| fail(&format!("no item is named {item}")), Ok)
}

/// A script's count as a number of items: none for a negative one.
fn amount(count: i64) -> u32 {
    count.clamp(0, i64::from(u32::MAX)) as u32
}

/// A stack as scripts see it; `None` for an empty slot.
fn shown(slot: u32, stack: ItemStack) -> Option<Map> {
    if stack.is_empty() {
        return None;
    }
    let fields = [
        ("slot", Dynamic::from(i64::from(slot))),
        ("item", item_name(stack.item).into()),
        ("count", Dynamic::from(i64::from(stack.count))),
    ];
    Some(
        fields
            .into_iter()
            .map(|(name, value)| (name.into(), value))
            .collect(),
    )
}

/// Puts `count` of `item` into `inventory`, a full stack at a time, and gives how many did
/// not fit.
fn give(inventory: &Inventory, item: u32, count: u32) -> u32 {
    let stack_size = item_info(item).map_or(1, |info| info.max_stack.max(1));
    let mut left = count;
    while left > 0 {
        let giving = left.min(stack_size);
        let back = inventory.insert(ItemStack::new(item, giving)).count;
        left -= giving - back.min(giving);
        if back > 0 {
            break;
        }
    }
    left
}

/// Takes up to `count` of `item` out of `inventory`, first slots first, and gives how many
/// it took.
fn take(inventory: &Inventory, item: u32, count: u32) -> u32 {
    let mut left = count;
    for slot in 0..inventory.len() {
        let stack = inventory.get(slot);
        if left == 0 || stack.is_empty() || stack.item != item {
            continue;
        }
        let taking = left.min(stack.count);
        let rest = ItemStack {
            count: stack.count - taking,
            ..stack
        };
        let emptied = rest.count == 0;
        inventory.set(slot, if emptied { ItemStack::EMPTY } else { rest });
        left -= taking;
    }
    count - left
}
