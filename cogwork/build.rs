//! Turns the recipe files in `assets/recipes/` into the `RECIPES` table `src/recipes.rs`
//! registers, so a mistake in one fails the build, naming the file and the recipe. The files'
//! format is described at the top of `assets/recipes/cogs.toml`.

use std::fmt::Write;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

const RECIPES: &str = "assets/recipes";

fn main() {
    println!("cargo::rerun-if-changed={RECIPES}");
    let mut files: Vec<PathBuf> = std::fs::read_dir(RECIPES)
        .unwrap_or_else(|error| panic!("cannot read {RECIPES}: {error}"))
        .map(|entry| entry.expect("cannot list the recipe files").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .collect();
    files.sort();
    let mut table = String::from("&[\n");
    for path in &files {
        for recipe in recipes_in(path) {
            writeln!(table, "    {recipe},").unwrap();
        }
    }
    table.push(']');
    let out = Path::new(&std::env::var("OUT_DIR").unwrap()).join("recipes.rs");
    std::fs::write(&out, table)
        .unwrap_or_else(|error| panic!("cannot write {}: {error}", out.display()));
}

/// Every recipe in the file at `path`, as the Rust that makes its `Recipe`.
fn recipes_in(path: &Path) -> Vec<String> {
    let name = path.display();
    let text =
        std::fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {name}: {error}"));
    let file: Table = text
        .parse()
        .unwrap_or_else(|error| panic!("{name} is not TOML: {error}"));
    let mut out = Vec::new();
    for (kind, list) in &file {
        let Some(list) = list.as_array() else {
            panic!("{name}: `{kind}` must be a list of recipes, `[[{kind}]]`");
        };
        for recipe in list {
            let Some(recipe) = recipe.as_table() else {
                panic!("{name}: a `{kind}` entry is not a table");
            };
            let made = match kind.as_str() {
                "shaped" => shaped(recipe),
                "shapeless" => shapeless(recipe),
                _ => Err(format!(
                    "`{kind}` is no kind of recipe: shaped or shapeless"
                )),
            };
            out.push(made.unwrap_or_else(|error| panic!("{name}: {error}")));
        }
    }
    out
}

fn shaped(recipe: &Table) -> Result<String, String> {
    let result = result(recipe, &["result", "count", "pattern", "key"])?;
    let pattern = strings(recipe, "pattern", &result)?;
    let width = pattern.first().map_or(0, |row| row.chars().count());
    let rows_fit = pattern.iter().all(|row| row.chars().count() == width);
    if pattern.is_empty() || pattern.len() > 3 || !(1..=3).contains(&width) || !rows_fit {
        return Err(format!(
            "{result}: the pattern must be 1 to 3 rows of 1 to 3, all as wide"
        ));
    }
    let keys = keys(recipe, &pattern, &result)?;
    let count = count(recipe, &result)?;
    Ok(format!(
        "Recipe::Shaped {{ result: {result:?}, count: {count}, pattern: &{pattern:?}, key: &[{}] }}",
        keys.join(", ")
    ))
}

/// What each letter of `pattern` is, from the recipe's `key` table, as the Rust of each pair.
/// Every key is one letter of the pattern, and every letter of the pattern has a key.
fn keys(recipe: &Table, pattern: &[String], result: &str) -> Result<Vec<String>, String> {
    let Some(key) = recipe.get("key").and_then(Value::as_table) else {
        return Err(format!(
            "{result}: no `key` table naming what the pattern's letters are"
        ));
    };
    let mut keys = Vec::new();
    for (letter, ingredient) in key {
        let mut chars = letter.chars();
        let (Some(symbol), None, Some(ingredient)) =
            (chars.next(), chars.next(), ingredient.as_str())
        else {
            return Err(format!(
                "{result}: key `{letter}` must be one letter naming an item"
            ));
        };
        if !pattern.iter().any(|row| row.contains(symbol)) {
            return Err(format!("{result}: key `{symbol}` is not in the pattern"));
        }
        keys.push(format!("({symbol:?}, {ingredient:?})"));
    }
    let mut letters = pattern.iter().flat_map(|row| row.chars());
    match letters.find(|&symbol| symbol != ' ' && !key.contains_key(&symbol.to_string())) {
        Some(symbol) => Err(format!(
            "{result}: the pattern's `{symbol}` is not in the key"
        )),
        None => Ok(keys),
    }
}

fn shapeless(recipe: &Table) -> Result<String, String> {
    let result = result(recipe, &["result", "count", "inputs"])?;
    let inputs = strings(recipe, "inputs", &result)?;
    if !(1..=9).contains(&inputs.len()) {
        return Err(format!("{result}: a crafting grid takes 1 to 9 inputs"));
    }
    let count = count(recipe, &result)?;
    Ok(format!(
        "Recipe::Shapeless {{ result: {result:?}, count: {count}, inputs: &{inputs:?} }}"
    ))
}

/// The recipe's result, once it is sure to have no fields but `fields`, so a misspelt one
/// isn't silently left out.
fn result(recipe: &Table, fields: &[&str]) -> Result<String, String> {
    let result = recipe.get("result").and_then(Value::as_str);
    let Some(result) = result.filter(|r| !r.is_empty()) else {
        return Err("a recipe without a `result`".to_string());
    };
    match recipe
        .keys()
        .find(|field| !fields.contains(&field.as_str()))
    {
        Some(field) => Err(format!("{result}: no field `{field}`; it has {fields:?}")),
        None => Ok(result.to_string()),
    }
}

/// How many the recipe makes: `count`, 1 if left out.
fn count(recipe: &Table, result: &str) -> Result<u32, String> {
    match recipe.get("count") {
        None => Ok(1),
        Some(count) => count
            .as_integer()
            .and_then(|number| u32::try_from(number).ok())
            .filter(|&number| number >= 1)
            .ok_or_else(|| format!("{result}: `count` must be a whole number from 1")),
    }
}

/// The list of strings `field`.
fn strings(recipe: &Table, field: &str, result: &str) -> Result<Vec<String>, String> {
    let list = recipe.get(field).and_then(Value::as_array);
    let strings = list.and_then(|list| list.iter().map(string).collect());
    strings.ok_or_else(|| format!("{result}: `{field}` must be a list of strings"))
}

fn string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
}
