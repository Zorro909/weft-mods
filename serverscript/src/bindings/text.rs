//! Text a script shows to players.

use rhai::Engine;
use weft_sdk::{Arg, Output, Who};

use super::{current, entity};
use crate::script::engine::ScriptResult;

/// The lang key that shows a script's text as it is.
pub const TEXT: &str = "serverscript.text";

pub fn register(engine: &mut Engine) {
    engine.register_fn("hint", |player: i64, text: &str| -> ScriptResult<()> {
        current()?.hint(TEXT, &[Arg::Text(text)]).to(entity(player));
        Ok(())
    });
    engine.register_fn("chat", |player: i64, text: &str| -> ScriptResult<()> {
        current()?.chat(TEXT, &[Arg::Text(text)]).to(entity(player));
        Ok(())
    });
    engine.register_fn("title", |player: i64, text: &str| -> ScriptResult<()> {
        current()?
            .title(TEXT, &[Arg::Text(text)])
            .to(entity(player));
        Ok(())
    });
    engine.register_fn("broadcast", |text: &str| -> ScriptResult<()> {
        current()?.chat(TEXT, &[Arg::Text(text)]).to(Who::All);
        Ok(())
    });
}
