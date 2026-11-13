//! What a script keeps on a player: values by key, saved in the store's file under the
//! player's number (`crate::player`).

use rhai::{Dynamic, Engine};

use super::{current, entity};
use crate::player;
use crate::script::engine::{ScriptResult, fail};
use crate::store::STORE;
use crate::value::Value;

pub fn register(engine: &mut Engine) {
    engine.register_fn(
        "data_get",
        |player: i64, key: &str| -> ScriptResult<Dynamic> {
            let number = player::known(&current()?, entity(player));
            let value = number.and_then(|number| STORE.player_get(number, key));
            Ok(value.map_or(Dynamic::UNIT, Value::into_dynamic))
        },
    );
    engine.register_fn(
        "data_set",
        |player: i64, key: &str, value: Dynamic| -> ScriptResult<bool> {
            let Some(value) = Value::from_dynamic(&value) else {
                return fail("data takes numbers, text, yes or no, and lists of them");
            };
            let Some(number) = player::id(&current()?, entity(player)) else {
                return Ok(false);
            };
            let set = STORE.player_set(number, key, value);
            set.map_or_else(|error| fail(error.message()), |()| Ok(true))
        },
    );
    engine.register_fn(
        "data_remove",
        |player: i64, key: &str| -> ScriptResult<bool> {
            let number = player::known(&current()?, entity(player));
            Ok(number.is_some_and(|number| STORE.player_remove(number, key)))
        },
    );
}
