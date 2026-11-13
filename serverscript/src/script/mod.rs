//! The scripts a server runs: every `.rhai` file of its scripts folder, compiled, with the
//! commands they declare and the functions the game calls in them.
//!
//! A script's function is called with a fresh scope and sees nothing outside itself, so a script
//! holds no state between calls: regions tick in parallel and any thread may run it. What lasts
//! lives in `crate::store`.
//!
//! A script is known by its file's name without the ending, which is what its data keys are
//! named after (`crate::name`): each call is tagged with it, for `crate::bindings::data`.

pub mod engine;

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use rhai::{AST, CallFnOptions, Dynamic, Engine, Map, Scope};

use self::engine::{Declarations, Declared};
use crate::lock;
use crate::name::is_plain;
use crate::usage::Usage;

/// What a script file's name ends in.
const EXTENSION: &str = "rhai";

/// The highest permission level there is: the host's.
const MAX_PERMISSION: i64 = 4;

/// The mod's own commands (`crate::commands`), which no script command may be named like.
const RESERVED: [&str; 2] = ["s", "scripts"];

/// The scripts in use, swapped whole by a reload.
static CURRENT: Mutex<Option<Arc<Scripts>>> = Mutex::new(None);

/// The scripts in use; `None` where the game was started without a scripts folder.
pub fn current() -> Option<Arc<Scripts>> {
    lock(&CURRENT).clone()
}

pub fn replace(scripts: Scripts) {
    *lock(&CURRENT) = Some(Arc::new(scripts));
}

/// One script file.
struct Script {
    /// Its file name without the ending: lowercase letters, digits and `_`.
    name: String,
    ast: AST,
    functions: BTreeSet<String>,
}

/// A command a script declared, run by its function `cmd_<name>`.
#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub usage_text: String,
    pub usage: Usage,
    /// The `permission` level it asks for.
    pub permission: u32,
    /// Which of `Scripts::scripts` declared it.
    script: usize,
}

pub struct Scripts {
    engine: Engine,
    scripts: Vec<Script>,
    commands: Vec<Command>,
}

impl Scripts {
    /// The scripts of `folder`, in the order of their names, and what was wrong with the ones
    /// left out. `extend` adds the calls scripts make into the game.
    pub fn load(folder: &Path, extend: fn(&mut Engine)) -> (Scripts, Vec<String>) {
        let Ok(files) = std::fs::read_dir(folder) else {
            let problem = format!("the scripts folder {} can't be read", folder.display());
            return (Scripts::compile(&[], extend).0, vec![problem]);
        };
        let mut paths: Vec<_> = files.flatten().map(|file| file.path()).collect();
        paths.retain(|path| path.extension().is_some_and(|ending| ending == EXTENSION));
        paths.sort();
        let mut sources = Vec::new();
        let mut problems = Vec::new();
        for path in paths {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            match std::fs::read_to_string(&path) {
                Ok(source) => sources.push((name, source)),
                Err(error) => problems.push(format!("{name}: {error}")),
            }
        }
        let (scripts, compile_problems) = Scripts::compile(&sources, extend);
        problems.extend(compile_problems);
        (scripts, problems)
    }

    /// Scripts from `(name, source)` pairs, and what was wrong with the ones left out.
    pub fn compile(
        sources: &[(String, String)],
        extend: fn(&mut Engine),
    ) -> (Scripts, Vec<String>) {
        let declarations = Declarations::default();
        let mut engine = engine::build(&declarations);
        extend(&mut engine);
        let mut scripts = Scripts {
            engine,
            scripts: Vec::new(),
            commands: Vec::new(),
        };
        let mut problems = Vec::new();
        for (name, source) in sources {
            match scripts.add(name, source, &declarations) {
                Ok(declared) => scripts.declare(declared, &mut problems),
                Err(problem) => problems.push(format!("{name}: {problem}")),
            }
        }
        (scripts, problems)
    }

    /// Compiles a script and runs its top level once, which is where it declares commands.
    fn add(
        &mut self,
        name: &str,
        source: &str,
        sink: &Declarations,
    ) -> Result<Vec<Declared>, String> {
        if !is_plain(name) {
            return Err("a script's file name may only have a-z, 0-9 and _".into());
        }
        let ast = self
            .engine
            .compile(source)
            .map_err(|error| error.to_string())?;
        *lock(sink) = Some(Vec::new());
        let ran = self.engine.run_ast(&ast);
        let declared = lock(sink).take().unwrap_or_default();
        ran.map_err(|error| error.to_string())?;
        let functions = ast
            .iter_functions()
            .map(|function| function.name.to_string())
            .collect();
        self.scripts.push(Script {
            name: name.to_string(),
            ast,
            functions,
        });
        Ok(declared)
    }

    /// Takes in the commands the script added last declared, but those that can't work.
    fn declare(&mut self, declared: Vec<Declared>, problems: &mut Vec<String>) {
        let index = self.scripts.len() - 1;
        for line in declared {
            match self.checked(&line, index) {
                Ok(command) => self.commands.push(command),
                Err(problem) => {
                    problems.push(format!(
                        "{}: command {}: {problem}",
                        self.scripts[index].name, line.name
                    ));
                }
            }
        }
    }

    fn checked(&self, line: &Declared, script: usize) -> Result<Command, String> {
        if !is_plain(&line.name) {
            return Err("its name may only have a-z, 0-9 and _".into());
        }
        if RESERVED.contains(&line.name.as_str()) {
            return Err("the name is the mod's own command".into());
        }
        if self.command(&line.name).is_some() {
            return Err("it is declared twice".into());
        }
        if !(0..=MAX_PERMISSION).contains(&line.permission) {
            return Err(format!("permission is 0 to {MAX_PERMISSION}"));
        }
        if !self.scripts[script]
            .functions
            .contains(&function_of(&line.name))
        {
            return Err(format!(
                "the script has no fn {}(cmd)",
                function_of(&line.name)
            ));
        }
        Ok(Command {
            name: line.name.clone(),
            usage_text: line.usage.clone(),
            usage: Usage::parse(&line.usage)?,
            permission: line.permission as u32,
            script,
        })
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn command(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|command| command.name == name)
    }

    /// How many script files are loaded.
    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    /// Whether any script has the function `hook`.
    pub fn has(&self, hook: &str) -> bool {
        self.scripts
            .iter()
            .any(|script| script.functions.contains(hook))
    }

    /// Calls `hook` with `argument` in every script that has it: what each gave back, or what
    /// went wrong in it.
    pub fn notify(&self, hook: &str, argument: &Map) -> Vec<Result<Dynamic, String>> {
        let having = self
            .scripts
            .iter()
            .filter(|script| script.functions.contains(hook));
        having
            .map(|script| self.call(script, hook, argument.clone()))
            .collect()
    }

    /// Runs `command` with `argument`: what its function gave back, or what went wrong in it.
    pub fn run(&self, command: &Command, argument: Map) -> Result<Dynamic, String> {
        self.call(
            &self.scripts[command.script],
            &function_of(&command.name),
            argument,
        )
    }

    fn call(&self, script: &Script, function: &str, argument: Map) -> Result<Dynamic, String> {
        // The top level ran when the script loaded: only the function runs now.
        let options = CallFnOptions::new()
            .eval_ast(false)
            .with_tag(script.name.clone());
        let mut scope = Scope::new();
        self.engine
            .call_fn_with_options(options, &mut scope, &script.ast, function, (argument,))
            .map_err(|error| format!("{}: {function}: {error}", script.name))
    }
}

/// The function that runs the command `name`.
fn function_of(name: &str) -> String {
    format!("cmd_{name}")
}

#[cfg(test)]
mod tests;
