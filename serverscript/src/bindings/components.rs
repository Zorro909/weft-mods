//! What a script keeps: components, read and set as a mod reads and sets the game's.
//! `component(holder, name)` gives a component's value and `set_component(holder, name, value)`
//! sets it, on a player or on a name of the script's choosing.
//!
//! A component's name is its script's, `<script>:<name>` (`crate::name`), and its value is
//! anything `crate::value` holds. They are the mod's own, kept in `crate::store`: the game
//! knows nothing of them.

use rhai::{Dynamic, Engine, NativeCallContext};
use weft_sdk::World;

use super::{current, entity};
use crate::name::qualified;
use crate::player;
use crate::script::engine::{ScriptResult, fail};
use crate::store::{Holder, STORE, turn};
use crate::value::Value;

pub fn register(engine: &mut Engine) {
    engine.register_fn(
        "component",
        |context: NativeCallContext, holder: Dynamic, name: &str| -> ScriptResult<Dynamic> {
            let component = full_name(&context, name)?;
            let Some(holder) = named_holder(&holder, player::known)? else {
                return Ok(Dynamic::UNIT);
            };
            let value = STORE.get(&holder, &component);
            Ok(value.map_or(Dynamic::UNIT, Value::into_dynamic))
        },
    );
    engine.register_fn(
        "set_component",
        |context: NativeCallContext,
         holder: Dynamic,
         name: &str,
         value: Dynamic|
         -> ScriptResult<bool> {
            let component = full_name(&context, name)?;
            let value = kept(&value)?;
            let Some(holder) = named_holder(&holder, player::id)? else {
                return Ok(false);
            };
            STORE.set(holder, &component, value);
            Ok(true)
        },
    );
}

/// The full name of the component a script calls `name`: its own, unless it says whose.
fn full_name(context: &NativeCallContext, name: &str) -> ScriptResult<String> {
    let script = context.tag().and_then(|tag| tag.clone().into_string().ok());
    let Some(script) = script else {
        return fail("components are reached from a function, not from the top of a script");
    };
    qualified(&script, name).map_or_else(
        || fail("a component's name is a-z, 0-9 and _, after script: for another script's"),
        Ok,
    )
}

/// The value a script set: `None` for nothing, `()`, which takes the component off.
fn kept(value: &Dynamic) -> ScriptResult<Option<Value>> {
    if value.is_unit() {
        return Ok(None);
    }
    Value::from_dynamic(value).map_or_else(
        || fail("a component holds numbers, text, yes or no, and lists and maps of them 32 deep"),
        |value| Ok(Some(value)),
    )
}

/// The holder a script named: a name as it is, or a player by the number `number_of` finds;
/// `None` for a player who is not near.
///
/// A name's components are every region's, so the script takes the turn at them (`turn`).
fn named_holder(
    named: &Dynamic,
    number_of: fn(&World, u64) -> Option<u64>,
) -> ScriptResult<Option<Holder>> {
    if named.is_string() {
        turn::take();
        let name = named.clone().into_string().unwrap_or_default();
        return Ok(Some(Holder::Named(name)));
    }
    let Ok(player) = named.as_int() else {
        return fail("a component is on a player, or on a name given as text");
    };
    let number = number_of(&current()?, entity(player));
    Ok(number.map(Holder::Player))
}
