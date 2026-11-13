//! What scripts call that reaches into the game: text for players (`text`), blocks and the
//! time of day (`world`), entities (`entities`), what they carry (`inventory`), and the
//! components scripts keep (`components`).
//!
//! A script's call has no world of its own to act on, so the world of the callback that runs the
//! script is noted for its thread while it runs (`within`). A world is the region the callback
//! ticks in: a block or an entity in another region is not there to read or change.

mod components;
mod entities;
mod inventory;
mod text;
mod world;

use std::cell::Cell;

use rhai::{Array, Dynamic, Engine};
use weft_sdk::{BlockPos, Holder, Vec3, World, info};

pub use self::text::TEXT;
use crate::script::engine::{ScriptResult, fail};
use crate::store::turn;

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
    let _restore = Restore(WORLD.replace(Some(world)));
    script()
}

pub fn register(engine: &mut Engine) {
    engine.on_print(|text| info!("{text}"));
    engine.register_fn("log", |text: &str| info!("{text}"));
    text::register(engine);
    world::register(engine);
    entities::register(engine);
    inventory::register(engine);
    components::register(engine);
}

/// The world a script's call acts on. A script that reaches into it is done with what names
/// hold: its turn at that ends here (`turn`).
fn current() -> ScriptResult<World> {
    turn::end();
    WORLD
        .get()
        .map_or_else(|| fail("this needs the game: call it from a function"), Ok)
}

/// A script's whole number as the entity id it stands for.
fn entity(id: i64) -> u64 {
    id as u64
}

/// An entity id as scripts hold it.
fn id(entity: u64) -> Dynamic {
    Dynamic::from(entity as i64)
}

fn number(value: &Dynamic) -> ScriptResult<f64> {
    let whole = value.as_int().map(|number| number as f64);
    whole
        .or_else(|_| value.as_float())
        .or_else(|_| fail("a coordinate is a number"))
}

/// The point a script's three coordinates name.
fn point(x: &Dynamic, y: &Dynamic, z: &Dynamic) -> ScriptResult<Vec3> {
    Ok(Vec3::new(number(x)?, number(y)?, number(z)?))
}

/// The block a script's three coordinates lie in.
fn block_pos(x: &Dynamic, y: &Dynamic, z: &Dynamic) -> ScriptResult<BlockPos> {
    let block = |coordinate: &Dynamic| number(coordinate).map(|number| number.floor() as i32);
    Ok(BlockPos {
        x: block(x)?,
        y: block(y)?,
        z: block(z)?,
    })
}

/// What a script names as carrying things: an entity by its id, or a block by `[x, y, z]`.
fn holder(named: &Dynamic) -> ScriptResult<Holder> {
    if let Ok(entity_id) = named.as_int() {
        return Ok(entity(entity_id).into());
    }
    let coordinates = named.clone().try_cast::<Array>().unwrap_or_default();
    let [x, y, z] = coordinates.as_slice() else {
        return fail("this takes an entity, or a block as [x, y, z]");
    };
    Ok(block_pos(x, y, z)?.into())
}
