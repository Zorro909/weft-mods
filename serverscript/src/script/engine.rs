//! The script engine without the game: its limits, and the `command(..)` line of a script.
//! `crate::bindings` adds what scripts call.

use std::sync::{Arc, Mutex};

use rhai::module_resolvers::DummyModuleResolver;
use rhai::{Engine, EvalAltResult};

use crate::lock;

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
    // Rhai's own resolver reads any `.rhai` file an `import` names: scripts read no files.
    engine.set_module_resolver(DummyModuleResolver::new());
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
    engine
}
