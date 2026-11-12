//! A command's usage string, as the engine writes them (`Weft/crates/engine/src/commands.rs`):
//! literal words, `<name:type>` and `[name:type]` parameters, alternatives apart by `|`.
//!
//! The engine reads what a player types after a command registered at start. A command a script
//! added since is run through `/s`, whose arguments are read here: numbers, words, text and
//! player names. The engine's parser is not in the SDK, so what needs the world to read (a
//! selector, a `~` position, an item's name) is only read for a command of its own.

use crate::value::Value;

/// What a parameter takes.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A word of the usage itself: given or not.
    Literal,
    /// A whole number, or one of these words.
    Int(Vec<String>),
    Float,
    Word,
    /// The rest of the line.
    Text,
    Player,
    /// An item, block, entity kind, dimension or attribute, by its full name.
    Named,
    /// Three coordinates.
    Pos,
    /// A player name or a selector: entities.
    Targets,
    /// `day`, `noon`, `night`, `midnight` or ticks.
    Time,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub kind: Kind,
    pub optional: bool,
}

/// Why `/s` could not read what was typed; `subject` is the parameter or the words it is about.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgError {
    Missing(String),
    NotANumber(String),
    /// Another word than the usage's literal.
    Expected(String),
    /// A kind only the engine reads.
    Unreadable(String),
    TooMany(String),
}

impl ArgError {
    /// The lang key of its message, which takes the subject as `{0}`.
    pub fn key(&self) -> &'static str {
        match self {
            ArgError::Missing(_) => "serverscript.args.missing",
            ArgError::NotANumber(_) => "serverscript.args.not_a_number",
            ArgError::Expected(_) => "serverscript.args.expected",
            ArgError::Unreadable(_) => "serverscript.args.unreadable",
            ArgError::TooMany(_) => "serverscript.args.too_many",
        }
    }

    pub fn subject(&self) -> &str {
        let (ArgError::Missing(subject)
        | ArgError::NotANumber(subject)
        | ArgError::Expected(subject)
        | ArgError::Unreadable(subject)
        | ArgError::TooMany(subject)) = self;
        subject
    }
}

/// A usage string taken apart.
#[derive(Clone, Debug, PartialEq)]
pub struct Usage {
    alternatives: Vec<Vec<Param>>,
}

impl Usage {
    /// The usage `text` says, or what is wrong with it.
    pub fn parse(text: &str) -> Result<Usage, String> {
        let alternatives: Result<Vec<Vec<Param>>, String> = text
            .split('|')
            .map(|alternative| alternative.split_whitespace().map(param).collect())
            .collect();
        Ok(Usage {
            alternatives: alternatives?,
        })
    }

    /// Every parameter of every alternative, the first of each name.
    pub fn params(&self) -> Vec<&Param> {
        let mut params: Vec<&Param> = Vec::new();
        for param in self.alternatives.iter().flatten() {
            if params.iter().all(|seen| seen.name != param.name) {
                params.push(param);
            }
        }
        params
    }

    /// What `line` gives each parameter of the first alternative it fits; the first
    /// alternative's complaint if it fits none.
    pub fn read(&self, line: &str) -> Result<Vec<(String, Value)>, ArgError> {
        let mut readings = self.alternatives.iter().map(|params| read(params, line));
        let first = readings.next().unwrap_or(Ok(Vec::new()));
        match first {
            Ok(values) => Ok(values),
            Err(error) => readings.find_map(Result::ok).ok_or(error),
        }
    }
}

fn param(token: &str) -> Result<Param, String> {
    let (close, optional) = match token.chars().next() {
        Some('<') => ('>', false),
        Some('[') => (']', true),
        _ => {
            return Ok(Param {
                name: token.to_string(),
                kind: Kind::Literal,
                optional: false,
            });
        }
    };
    let inner = token[1..]
        .strip_suffix(close)
        .ok_or_else(|| format!("bad parameter {token}"))?;
    let (name, kind) = inner
        .split_once(':')
        .ok_or_else(|| format!("parameter {token} needs a type"))?;
    Ok(Param {
        name: name.to_string(),
        kind: kind_named(kind)?,
        optional,
    })
}

fn kind_named(kind: &str) -> Result<Kind, String> {
    let listed = |prefix: &str| {
        kind.strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(')'))
    };
    if let Some(words) = listed("int(") {
        return Ok(Kind::Int(
            words
                .split(',')
                .map(|word| word.trim().to_string())
                .collect(),
        ));
    }
    if listed("word(").is_some() {
        return Ok(Kind::Word);
    }
    match kind {
        "int" => Ok(Kind::Int(Vec::new())),
        "float" => Ok(Kind::Float),
        "word" => Ok(Kind::Word),
        "text" => Ok(Kind::Text),
        "player" => Ok(Kind::Player),
        "item" | "block" | "entity" | "dim" | "attribute" => Ok(Kind::Named),
        "pos" => Ok(Kind::Pos),
        "targets" => Ok(Kind::Targets),
        "time" => Ok(Kind::Time),
        _ => Err(format!("unknown parameter type {kind}")),
    }
}

/// What `line` gives `params`, in their order. An optional parameter may only be left out at
/// the end.
fn read(params: &[Param], line: &str) -> Result<Vec<(String, Value)>, ArgError> {
    let mut rest = line.trim();
    let mut values = Vec::new();
    for param in params {
        let (word, after) = match param.kind {
            Kind::Text => (rest, ""),
            _ => rest.split_once(char::is_whitespace).unwrap_or((rest, "")),
        };
        if word.is_empty() {
            if param.optional {
                continue;
            }
            return Err(ArgError::Missing(param.name.clone()));
        }
        values.push((param.name.clone(), value(param, word)?));
        rest = after.trim_start();
    }
    if rest.is_empty() {
        Ok(values)
    } else {
        Err(ArgError::TooMany(rest.to_string()))
    }
}

fn value(param: &Param, word: &str) -> Result<Value, ArgError> {
    let name = || param.name.clone();
    match &param.kind {
        Kind::Literal if word == param.name => Ok(Value::Bool(true)),
        Kind::Literal => Err(ArgError::Expected(name())),
        Kind::Int(words) if words.iter().any(|listed| listed == word) => {
            Ok(Value::Text(word.to_string()))
        }
        Kind::Int(_) => word
            .parse()
            .map(Value::Int)
            .map_err(|_| ArgError::NotANumber(name())),
        Kind::Float => word
            .parse()
            .map(Value::Float)
            .map_err(|_| ArgError::NotANumber(name())),
        Kind::Word | Kind::Text | Kind::Player => Ok(Value::Text(word.to_string())),
        Kind::Named | Kind::Pos | Kind::Targets | Kind::Time => Err(ArgError::Unreadable(name())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(text: &str) -> Usage {
        Usage::parse(text).expect("the usage is well written")
    }

    fn text(value: &str) -> Value {
        Value::Text(value.to_string())
    }

    #[test]
    fn a_usage_is_taken_apart_as_the_engine_does() {
        let params = usage("give <who:player> [count:int] | list");
        let kinds: Vec<(&str, &Kind, bool)> = params
            .params()
            .into_iter()
            .map(|param| (param.name.as_str(), &param.kind, param.optional))
            .collect();
        let expected = [
            ("give", &Kind::Literal, false),
            ("who", &Kind::Player, false),
            ("count", &Kind::Int(Vec::new()), true),
            ("list", &Kind::Literal, false),
        ];
        assert_eq!(kinds, expected);
    }

    #[test]
    fn every_type_of_the_engine_is_known() {
        let all = "<a:int> <b:float> <c:word> <d:word(x,y)> <e:word(#core:rules)> <f:item> \
                   <g:block> <h:entity> <i:dim> <j:attribute> <k:player> <l:pos> <m:targets> \
                   <n:time> <o:int(infinite)> <p:text>";
        assert_eq!(usage(all).params().len(), 16);
    }

    #[test]
    fn a_badly_written_usage_says_why() {
        assert_eq!(
            Usage::parse("<count:number>"),
            Err("unknown parameter type number".into())
        );
        assert_eq!(
            Usage::parse("<count>"),
            Err("parameter <count> needs a type".into())
        );
        assert_eq!(
            Usage::parse("<count:int"),
            Err("bad parameter <count:int".into())
        );
    }

    #[test]
    fn a_name_in_two_alternatives_is_one_parameter() {
        assert_eq!(usage("set <n:int> | add <n:int>").params().len(), 3);
    }

    #[test]
    fn typed_words_become_values() {
        let read = usage("<amount:int> <share:float> <who:player>").read("5 0.5  ada");
        let expected = vec![
            ("amount".to_string(), Value::Int(5)),
            ("share".to_string(), Value::Float(0.5)),
            ("who".to_string(), text("ada")),
        ];
        assert_eq!(read, Ok(expected));
    }

    #[test]
    fn text_takes_the_rest_of_the_line() {
        let read = usage("<who:word> <message:text>").read("ada good  morning");
        let expected = vec![
            ("who".to_string(), text("ada")),
            ("message".to_string(), text("good  morning")),
        ];
        assert_eq!(read, Ok(expected));
    }

    #[test]
    fn an_empty_usage_takes_nothing() {
        assert_eq!(usage("").read("  "), Ok(Vec::new()));
        assert_eq!(usage("").read("now"), Err(ArgError::TooMany("now".into())));
    }

    #[test]
    fn an_optional_parameter_may_be_left_out() {
        assert_eq!(usage("[amount:int] [note:text]").read(""), Ok(Vec::new()));
        assert_eq!(
            usage("<amount:int>").read(""),
            Err(ArgError::Missing("amount".into()))
        );
    }

    #[test]
    fn a_word_that_is_no_number_is_refused() {
        let read = usage("<amount:int>").read("many");
        assert_eq!(read, Err(ArgError::NotANumber("amount".into())));
        assert_eq!(
            usage("<share:float>").read("x"),
            Err(ArgError::NotANumber("share".into()))
        );
    }

    #[test]
    fn a_listed_word_stands_in_for_a_number() {
        let read = usage("<seconds:int(infinite)>").read("infinite");
        assert_eq!(read, Ok(vec![("seconds".to_string(), text("infinite"))]));
    }

    #[test]
    fn the_alternative_that_fits_is_read() {
        let both = usage("set <n:int> | clear");
        assert_eq!(
            both.read("clear"),
            Ok(vec![("clear".to_string(), Value::Bool(true))])
        );
        assert_eq!(
            both.read("drop"),
            Err(ArgError::Expected("set".into())),
            "the first complains"
        );
    }

    #[test]
    fn what_only_the_engine_reads_is_refused() {
        for kind in ["pos", "targets", "item", "time"] {
            let read = usage(&format!("<it:{kind}>")).read("something");
            assert_eq!(
                read,
                Err(ArgError::Unreadable("it".into())),
                "{kind} needs the world"
            );
        }
    }

    #[test]
    fn every_error_has_a_name() {
        let errors = [
            ArgError::Missing("a".into()),
            ArgError::NotANumber("a".into()),
            ArgError::Expected("a".into()),
            ArgError::Unreadable("a".into()),
            ArgError::TooMany("a".into()),
        ];
        for error in errors {
            assert!(
                crate::translated(error.key()),
                "{} has no translation",
                error.key()
            );
            assert_eq!(error.subject(), "a");
        }
    }
}
