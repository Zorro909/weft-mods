//! What scripts call that reaches into the game: text for players, entities, and the data kept
//! on them.
//!
//! A script's call has no world of its own to act on, so the world of the callback that runs the
//! script is noted for its thread while it runs (`within`). A world is the region the callback
//! ticks in: an entity in another region is not there to read or change.

use std::cell::Cell;

use rhai::{Array, Dynamic, Engine};
use weft_sdk::{Arg, Output, Vec3, Who, World, info};

use crate::data;
use crate::script::engine::{ScriptResult, fail};
use crate::store::STORE;
use crate::value::Value;

/// The lang key that shows a script's text as it is.
pub const TEXT: &str = "serverscript.text";

thread_local! {
    /// The world of the callback this thread runs a script for.
    static WORLD: Cell<Option<World>> = const { Cell::new(None) };
}

/// Puts back the world noted before, also when the script panicked.
struct Restore(Option<World>);

impl Drop for Restore {
    fn drop(&mut self) {
        WORLD.set(self.0);
    }
}

/// Runs `script` with `world` as the one its calls act on.
pub fn within<T>(world: World, script: impl FnOnce() -> T) -> T {
    STORE.enter(world.session());
    let _restore = Restore(WORLD.replace(Some(world)));
    script()
}

fn world() -> ScriptResult<World> {
    WORLD
        .get()
        .map_or_else(|| fail("this needs the game: call it from a function"), Ok)
}

/// A script's whole number as the entity id it stands for.
fn entity(id: i64) -> u64 {
    id as u64
}

fn number(value: &Dynamic) -> ScriptResult<f64> {
    let whole = value.as_int().map(|number| number as f64);
    whole
        .or_else(|_| value.as_float())
        .or_else(|_| fail("a coordinate is a number"))
}

fn kept(value: &Dynamic) -> ScriptResult<Value> {
    let value = Value::from_dynamic(value);
    value.map_or_else(
        || fail("data takes numbers, text, yes or no, and lists of them"),
        Ok,
    )
}

pub fn register(engine: &mut Engine) {
    engine.on_print(|text| info!("{text}"));
    engine.register_fn("log", |text: &str| info!("{text}"));
    register_text(engine);
    register_entities(engine);
    register_data(engine);
}

fn register_text(engine: &mut Engine) {
    engine.register_fn("hint", |player: i64, text: &str| -> ScriptResult<()> {
        world()?.hint(TEXT, &[Arg::Text(text)]).to(entity(player));
        Ok(())
    });
    engine.register_fn("chat", |player: i64, text: &str| -> ScriptResult<()> {
        world()?.chat(TEXT, &[Arg::Text(text)]).to(entity(player));
        Ok(())
    });
    engine.register_fn("title", |player: i64, text: &str| -> ScriptResult<()> {
        world()?.title(TEXT, &[Arg::Text(text)]).to(entity(player));
        Ok(())
    });
    engine.register_fn("broadcast", |text: &str| -> ScriptResult<()> {
        world()?.chat(TEXT, &[Arg::Text(text)]).to(Who::All);
        Ok(())
    });
}

fn register_entities(engine: &mut Engine) {
    engine.register_fn("players", || -> ScriptResult<Array> {
        let players: Array = world()?
            .players_iter()
            .map(|id| Dynamic::from(id as i64))
            .collect();
        Ok(players)
    });
    engine.register_fn("position", |id: i64| -> ScriptResult<Dynamic> {
        let Some(state) = world()?.entity(entity(id)) else {
            return Ok(Dynamic::UNIT);
        };
        let position = [state.pos.x, state.pos.y, state.pos.z];
        Ok(Dynamic::from_array(position.map(Dynamic::from).to_vec()))
    });
    engine.register_fn(
        "teleport",
        |id: i64, x: Dynamic, y: Dynamic, z: Dynamic| -> ScriptResult<()> {
            let to = Vec3::new(number(&x)?, number(&y)?, number(&z)?);
            world()?.entity_modify(entity(id)).pos(to);
            Ok(())
        },
    );
    engine.register_fn("permission", |player: i64| -> ScriptResult<i64> {
        Ok(i64::from(world()?.permission(entity(player))))
    });
}

fn register_data(engine: &mut Engine) {
    engine.register_fn("data_get", |id: i64, key: &str| -> ScriptResult<Dynamic> {
        let value = data::get(&world()?, entity(id), key);
        Ok(value.map_or(Dynamic::UNIT, Value::into_dynamic))
    });
    engine.register_fn(
        "data_set",
        |id: i64, key: &str, value: Dynamic| -> ScriptResult<bool> {
            Ok(data::set(&world()?, entity(id), key, kept(&value)?))
        },
    );
    engine.register_fn("data_remove", |id: i64, key: &str| -> ScriptResult<bool> {
        Ok(data::remove(&world()?, entity(id), key))
    });
}
