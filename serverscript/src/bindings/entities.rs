//! Entities: where they are, what they are, their health, and making and removing them.
//!
//! Making one is `summon`: Rhai keeps the word `spawn` to itself.

use rhai::{Array, Dynamic, Engine};
use weft_sdk::abi::entry_kind;
use weft_sdk::{Damage, EntityKind, PLAYER, damage_causes, name_of};

use super::{current, entity, id, number, point};
use crate::script::engine::{ScriptResult, fail};

pub fn register(engine: &mut Engine) {
    register_players(engine);
    register_state(engine);
    register_health(engine);
    register_making(engine);
}

fn register_players(engine: &mut Engine) {
    engine.register_fn("players", || -> ScriptResult<Array> {
        Ok(current()?.players_iter().map(id).collect())
    });
    engine.register_fn("permission", |player: i64| -> ScriptResult<i64> {
        Ok(i64::from(current()?.permission(entity(player))))
    });
}

fn register_state(engine: &mut Engine) {
    engine.register_fn("position", |entity_id: i64| -> ScriptResult<Dynamic> {
        let Some(state) = current()?.entity(entity(entity_id)) else {
            return Ok(Dynamic::UNIT);
        };
        let position = [state.pos.x, state.pos.y, state.pos.z];
        Ok(Dynamic::from_array(position.map(Dynamic::from).to_vec()))
    });
    engine.register_fn(
        "teleport",
        |entity_id: i64, x: Dynamic, y: Dynamic, z: Dynamic| -> ScriptResult<()> {
            let to = point(&x, &y, &z)?;
            current()?.entity_modify(entity(entity_id)).pos(to);
            Ok(())
        },
    );
    engine.register_fn("kind", |entity_id: i64| -> ScriptResult<Dynamic> {
        let state = current()?.entity(entity(entity_id));
        let name = state.map(|state| name_of(entry_kind::ENTITY, state.kind));
        Ok(name.map_or(Dynamic::UNIT, Dynamic::from))
    });
}

fn register_health(engine: &mut Engine) {
    engine.register_fn("health", |entity_id: i64| -> ScriptResult<Dynamic> {
        let state = current()?.entity(entity(entity_id));
        let health = state.map(|state| f64::from(state.health));
        Ok(health.map_or(Dynamic::UNIT, Dynamic::from))
    });
    engine.register_fn(
        "damage",
        |entity_id: i64, amount: Dynamic| -> ScriptResult<bool> {
            let amount = number(&amount)?.max(0.0) as f32;
            let damage = Damage::new(amount, &damage_causes::GENERIC);
            Ok(current()?.damage(entity(entity_id), damage))
        },
    );
    engine.register_fn(
        "heal",
        |entity_id: i64, amount: Dynamic| -> ScriptResult<bool> {
            Ok(current()?.heal(entity(entity_id), number(&amount)? as f32))
        },
    );
}

fn register_making(engine: &mut Engine) {
    engine.register_fn(
        "summon",
        |kind: &str, x: Dynamic, y: Dynamic, z: Dynamic| -> ScriptResult<i64> {
            let found = EntityKind::lookup(kind).filter(|kind| *kind != PLAYER.id());
            let Some(kind) = found else {
                return fail(&format!("no entity to summon is named {kind}"));
            };
            Ok(current()?.spawn(kind, point(&x, &y, &z)?) as i64)
        },
    );
    engine.register_fn("remove", |entity_id: i64| -> ScriptResult<bool> {
        let world = current()?;
        let Some(state) = world.entity(entity(entity_id)) else {
            return Ok(false);
        };
        if state.kind == PLAYER.id() {
            return fail("a player is not removed");
        }
        world.despawn(entity(entity_id));
        Ok(true)
    });
}
