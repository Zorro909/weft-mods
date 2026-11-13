//! The script engine without the game: its limits, and what scripts call that needs no world.
//! `crate::bindings` adds the calls that do.

use std::sync::{Arc, Mutex};

use rhai::{Dynamic, Engine, EvalAltResult};

use crate::lock;
use crate::store::STORE;
use crate::value::Value;

/// Operations one handler may run before it is stopped, so a loop that never ends costs a few
/// milliseconds of one tick and not the server.
const MAX_OPERATIONS: u64 = 100_000;

/// A script's `command(name, usage, permission)` line.
#[derive(Clone, Debug, PartialEq)]
pub struct Declared {
    pub name: String,
    pub usage: String,
    pub permission: i64,
}

/// Where `command(..)` lines go while a script loads; `None` once it has.
pub type Declarations = Arc<Mutex<Option<Vec<Declared>>>>;

pub type ScriptResult<T> = Result<T, Box<EvalAltResult>>;

/// An error a script sees, with `message`.
pub fn fail<T>(message: &str) -> ScriptResult<T> {
    Err(message.into())
}

/// An engine whose `command(..)` lines land in `declarations`.
pub fn build(declarations: &Declarations) -> Engine {
    let mut engine = Engine::new();
    engine.set_max_operations(MAX_OPERATIONS);
    let declarations = declarations.clone();
    engine.register_fn(
        "command",
        move |name: &str, usage: &str, permission: i64| {
            let mut declarations = lock(&declarations);
            let Some(declared) = declarations.as_mut() else {
                return fail("command(..) belongs at the top of a script, outside its functions");
            };
            declared.push(Declared {
                name: name.to_string(),
                usage: usage.to_string(),
                permission,
            });
            Ok(())
        },
    );
    register_store(&mut engine);
    engine
}

fn register_store(engine: &mut Engine) {
    engine.register_fn("store_get", |key: &str| {
        STORE.get(key).map_or(Dynamic::UNIT, Value::into_dynamic)
    });
    engine.register_fn("store_add", |key: &str, amount: Dynamic| {
        let Some(amount) = Value::from_dynamic(&amount) else {
            return fail("store_add needs a number");
        };
        let total = STORE
            .add(key, &amount)
            .or_else(|error| fail(error.message()))?;
        Ok(total.into_dynamic())
    });
    engine.register_fn("store_append", |key: &str, value: Dynamic| {
        let Some(value) = Value::from_dynamic(&value) else {
            return fail("store_append takes numbers, text, yes or no, and lists of them");
        };
        let length = STORE
            .append(key, value)
            .or_else(|error| fail(error.message()))?;
        Ok(length as i64)
    });
    engine.register_fn("store_set_if_absent", |key: &str, value: Dynamic| {
        let Some(value) = Value::from_dynamic(&value) else {
            return fail("store_set_if_absent takes numbers, text, yes or no, and lists of them");
        };
        STORE
            .set_if_absent(key, value)
            .or_else(|error| fail(error.message()))
    });
}
