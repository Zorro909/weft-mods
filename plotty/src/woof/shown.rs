//! The balance in a panel beside the inventory. The table lives on the server, so each
//! player carries a copy in the `WOOF` component, which only its own client is sent; a look at
//! every player twice a second keeps it equal to the table, wherever the payment came from. It
//! goes through the component's system, not `World::players`, which leaves out spectators.

use weft_sdk::{Arg, Component, Menu, PLAYER, Registry, S, SystemBatch, component_flags};
use weftkit::PlayerKey;

/// A player's balance, as their client sees it.
static WOOF: Component<u64> = Component::named("woof");

/// Base's inventory, the menu a player opens on their own; base_api doesn't name it.
static INVENTORY: Menu = Menu::named("base:inventory");

/// Ticks between looks.
const EVERY: u64 = 10;

pub fn register(r: &mut Registry) {
    r.entity_component(&WOOF, 0, component_flags::SYNC_OWNER);
    r.edit(&PLAYER).attach(&WOOF);
    r.system(&WOOF, refresh);
    r.screen_part(&INVENTORY, "side", 0, |ui| {
        ui.panel("panel", S::row().gap(3.0).pad(4.0), |ui| {
            ui.image("plotty:gui/woof", 16.0, 16.0);
            ui.column(S::column(), |ui| {
                ui.text_styled(S::column().class("dim"), "plotty.woof.name", &[]);
                ui.text_bind(|view, out| {
                    let held = view.component(&WOOF).unwrap_or(0);
                    out.set("plotty.woof.amount", &[Arg::Int(held as i64)]);
                });
            });
        });
    });
}

/// Copies each player's balance into their component where it changed.
fn refresh(batch: &mut SystemBatch<u64>) {
    let world = batch.world();
    if world.game_time() % EVERY != 0 {
        return;
    }
    for (player, shown, _) in batch.iter_mut() {
        let Some(key) = PlayerKey::of(&world, player) else {
            continue;
        };
        let held = super::balance(&key);
        if *shown != held {
            *shown = held;
        }
    }
}
