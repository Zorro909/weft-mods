//! What a script keeps: values by key under a data key, `<script>:<name>`, as the game keeps
//! an entity's values under a component. A data key is there once for everyone, or once on
//! each player when the call names one.
//!
//! What is there once is reached from every region at the same time, so none of its calls
//! replaces what another wrote: a value is added to, appended to, or set where there was
//! none. A player's is also set and removed: only the region the player is in reaches it.

use rhai::{Dynamic, Engine, Map, NativeCallContext};

use super::{current, entity};
use crate::name::qualified;
use crate::player;
use crate::script::engine::{ScriptResult, fail};
use crate::store::{Place, SHARED, STORE, StoreError};
use crate::value::Value;

pub fn register(engine: &mut Engine) {
    register_reads(engine);
    register_merges(engine);
    register_claims(engine);
    register_replacing(engine);
}

/// The full name of the data key a script calls `name`: its own, unless it says whose.
fn component(context: &NativeCallContext, name: &str) -> ScriptResult<String> {
    let script = context.tag().and_then(|tag| tag.clone().into_string().ok());
    let Some(script) = script else {
        return fail("data is reached from a function, not from the top of a script");
    };
    qualified(&script, name).map_or_else(
        || fail("a data key is a-z, 0-9 and _, after script: for another script's"),
        Ok,
    )
}

fn kept(value: &Dynamic) -> ScriptResult<Value> {
    Value::from_dynamic(value).map_or_else(
        || fail("data takes numbers, text, yes or no, and lists of them"),
        Ok,
    )
}

fn refused<T>(error: StoreError) -> ScriptResult<T> {
    fail(error.message())
}

/// The number `player`'s data is kept under, if they have any and are near.
fn known(player: i64) -> ScriptResult<Option<u64>> {
    Ok(player::known(&current()?, entity(player)))
}

/// The number `player`'s data is kept under, given now if they had none; `None` when the
/// player is not near.
fn numbered(player: i64) -> ScriptResult<Option<u64>> {
    Ok(player::id(&current()?, entity(player)))
}

fn value(owner: u64, component: &str, key: &str) -> Dynamic {
    let place = Place {
        owner,
        component,
        key,
    };
    STORE.get(place).map_or(Dynamic::UNIT, Value::into_dynamic)
}

fn all(owner: u64, component: &str) -> Map {
    let entries = STORE.all(owner, component).into_iter();
    entries
        .map(|(key, value)| (key.into(), value.into_dynamic()))
        .collect()
}

fn register_reads(engine: &mut Engine) {
    engine.register_fn(
        "data_get",
        |context: NativeCallContext, name: &str, key: &str| -> ScriptResult<Dynamic> {
            Ok(value(SHARED, &component(&context, name)?, key))
        },
    );
    engine.register_fn(
        "data_get",
        |context: NativeCallContext, player: i64, name: &str, key: &str| -> ScriptResult<Dynamic> {
            let component = component(&context, name)?;
            let owner = known(player)?;
            Ok(owner.map_or(Dynamic::UNIT, |owner| value(owner, &component, key)))
        },
    );
    engine.register_fn(
        "data_all",
        |context: NativeCallContext, name: &str| -> ScriptResult<Map> {
            Ok(all(SHARED, &component(&context, name)?))
        },
    );
    engine.register_fn(
        "data_all",
        |context: NativeCallContext, player: i64, name: &str| -> ScriptResult<Map> {
            let component = component(&context, name)?;
            let owner = known(player)?;
            Ok(owner.map_or_else(Map::new, |owner| all(owner, &component)))
        },
    );
}

fn add(owner: u64, component: &str, key: &str, amount: &Dynamic) -> ScriptResult<Dynamic> {
    let place = Place {
        owner,
        component,
        key,
    };
    let total = STORE.add(place, &kept(amount)?).or_else(refused)?;
    Ok(total.into_dynamic())
}

fn append(owner: u64, component: &str, key: &str, item: &Dynamic) -> ScriptResult<Dynamic> {
    let place = Place {
        owner,
        component,
        key,
    };
    let length = STORE.append(place, kept(item)?).or_else(refused)?;
    Ok(Dynamic::from(length as i64))
}

/// `data_add` and `data_append`. For a player who is not near they give nothing.
fn register_merges(engine: &mut Engine) {
    type Merge = fn(u64, &str, &str, &Dynamic) -> ScriptResult<Dynamic>;
    let merges: [(&str, Merge); 2] = [("data_add", add), ("data_append", append)];
    for (call, merge) in merges {
        engine.register_fn(
            call,
            move |context: NativeCallContext,
                  name: &str,
                  key: &str,
                  value: Dynamic|
                  -> ScriptResult<Dynamic> {
                merge(SHARED, &component(&context, name)?, key, &value)
            },
        );
        engine.register_fn(
            call,
            move |context: NativeCallContext,
                  player: i64,
                  name: &str,
                  key: &str,
                  value: Dynamic|
                  -> ScriptResult<Dynamic> {
                let component = component(&context, name)?;
                let Some(owner) = numbered(player)? else {
                    return Ok(Dynamic::UNIT);
                };
                merge(owner, &component, key, &value)
            },
        );
    }
}

fn claim(owner: u64, component: &str, key: &str, value: &Dynamic) -> ScriptResult<bool> {
    let place = Place {
        owner,
        component,
        key,
    };
    STORE.set_if_absent(place, kept(value)?).or_else(refused)
}

fn register_claims(engine: &mut Engine) {
    engine.register_fn(
        "data_set_if_absent",
        |context: NativeCallContext, name: &str, key: &str, value: Dynamic| -> ScriptResult<bool> {
            claim(SHARED, &component(&context, name)?, key, &value)
        },
    );
    engine.register_fn(
        "data_set_if_absent",
        |context: NativeCallContext,
         player: i64,
         name: &str,
         key: &str,
         value: Dynamic|
         -> ScriptResult<bool> {
            let component = component(&context, name)?;
            let Some(owner) = numbered(player)? else {
                return Ok(false);
            };
            claim(owner, &component, key, &value)
        },
    );
}

/// `data_set` and `data_remove`, which only a player's data has.
fn register_replacing(engine: &mut Engine) {
    engine.register_fn(
        "data_set",
        |context: NativeCallContext,
         player: i64,
         name: &str,
         key: &str,
         value: Dynamic|
         -> ScriptResult<bool> {
            let component = component(&context, name)?;
            let value = kept(&value)?;
            let Some(owner) = numbered(player)? else {
                return Ok(false);
            };
            let place = Place {
                owner,
                component: &component,
                key,
            };
            STORE.set(place, value).map_or_else(refused, |()| Ok(true))
        },
    );
    engine.register_fn(
        "data_remove",
        |context: NativeCallContext, player: i64, name: &str, key: &str| -> ScriptResult<bool> {
            let component = component(&context, name)?;
            let Some(owner) = known(player)? else {
                return Ok(false);
            };
            let place = Place {
                owner,
                component: &component,
                key,
            };
            Ok(STORE.remove(place))
        },
    );
}
