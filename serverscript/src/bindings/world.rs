//! The blocks of the world and its time of day.

use rhai::{Dynamic, Engine};
use weft_sdk::{Block, block_name};

use super::{block_pos, current};
use crate::script::engine::{ScriptResult, fail};

pub fn register(engine: &mut Engine) {
    engine.register_fn(
        "block",
        |x: Dynamic, y: Dynamic, z: Dynamic| -> ScriptResult<Dynamic> {
            let state = current()?.block_loaded(block_pos(&x, &y, &z)?);
            Ok(state.map_or(Dynamic::UNIT, |state| block_name(state).into()))
        },
    );
    engine.register_fn(
        "set_block",
        |x: Dynamic, y: Dynamic, z: Dynamic, name: &str| -> ScriptResult<bool> {
            let Some(state) = Block::lookup(name) else {
                return fail(&format!("no block is named {name}"));
            };
            Ok(current()?.set_block(block_pos(&x, &y, &z)?, state))
        },
    );
    engine.register_fn("time", || -> ScriptResult<i64> {
        Ok(current()?.day_time() as i64)
    });
    engine.register_fn("set_time", |time: i64| -> ScriptResult<()> {
        current()?.set_day_time(time.max(0) as u64);
        Ok(())
    });
}
