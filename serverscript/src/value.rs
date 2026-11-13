//! A value a script's component holds.

use std::collections::BTreeMap;

use rhai::{Array, Dynamic, Map};

/// How deep lists and maps may lie in each other. Reading one calls itself once per level, so
/// a deeper one is refused, from a script as from the file, before it costs the stack.
pub const MAX_DEPTH: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    List(Vec<Value>),
    /// Values by name, `#{ at: [1, 2, 3], returns: 4 }`: what a struct is to the game's
    /// components.
    Map(BTreeMap<String, Value>),
}

impl Value {
    /// What a script handed over; `None` for what can't be kept (a function, nothing, lists
    /// and maps deeper than `MAX_DEPTH`).
    pub fn from_dynamic(dynamic: &Dynamic) -> Option<Value> {
        Value::nested(dynamic, MAX_DEPTH)
    }

    /// `from_dynamic`, with `deeper` more levels of lists and maps allowed.
    fn nested(dynamic: &Dynamic, deeper: usize) -> Option<Value> {
        if let Ok(number) = dynamic.as_int() {
            return Some(Value::Int(number));
        }
        if let Ok(number) = dynamic.as_float() {
            return Some(Value::Float(number));
        }
        if let Ok(flag) = dynamic.as_bool() {
            return Some(Value::Bool(flag));
        }
        if dynamic.is_string() {
            return dynamic.clone().into_string().ok().map(Value::Text);
        }
        // Borrowed, not cloned: a clone would walk every level before any is counted.
        let deeper = deeper.checked_sub(1)?;
        if let Some(list) = dynamic.read_lock::<Array>() {
            let values = list.iter().map(|value| Value::nested(value, deeper));
            return values.collect::<Option<_>>().map(Value::List);
        }
        let map = dynamic.read_lock::<Map>()?;
        let entry = |(name, value): (&_, &Dynamic)| {
            Some((format!("{name}"), Value::nested(value, deeper)?))
        };
        map.iter().map(entry).collect::<Option<_>>().map(Value::Map)
    }

    pub fn into_dynamic(self) -> Dynamic {
        match self {
            Value::Int(number) => number.into(),
            Value::Float(number) => number.into(),
            Value::Bool(flag) => flag.into(),
            Value::Text(text) => text.into(),
            Value::List(values) => {
                Dynamic::from_array(values.into_iter().map(Value::into_dynamic).collect())
            }
            Value::Map(entries) => {
                let entry = |(name, value): (String, Value)| (name.into(), value.into_dynamic());
                Dynamic::from_map(entries.into_iter().map(entry).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_survives_a_script_and_back() {
        let camp = [
            ("at".to_string(), Value::List(vec![Value::Float(0.5)])),
            ("returns".to_string(), Value::Int(3)),
        ];
        let values = [
            Value::Int(-7),
            Value::Float(2.5),
            Value::Bool(true),
            Value::Text("home".into()),
            Value::List(vec![
                Value::Int(1),
                Value::List(vec![Value::Text("a".into())]),
            ]),
            Value::Map(camp.into()),
        ];
        for value in values {
            let back = Value::from_dynamic(&value.clone().into_dynamic());
            assert_eq!(back, Some(value), "a value comes back as it went");
        }
    }

    #[test]
    fn what_cannot_be_kept_is_refused() {
        assert_eq!(
            Value::from_dynamic(&Dynamic::UNIT),
            None,
            "nothing is no value"
        );
        let mixed: Array = vec![Dynamic::from(1_i64), Dynamic::UNIT];
        assert_eq!(
            Value::from_dynamic(&Dynamic::from_array(mixed)),
            None,
            "nor a list holding nothing"
        );
        let mut map = Map::new();
        map.insert("gone".into(), Dynamic::UNIT);
        assert_eq!(
            Value::from_dynamic(&Dynamic::from_map(map)),
            None,
            "nor a map holding nothing"
        );
    }

    #[test]
    fn lists_in_lists_are_kept_only_so_deep() {
        let nested = |levels| {
            (0..levels).fold(Dynamic::from(1_i64), |inner, _| {
                Dynamic::from_array(vec![inner])
            })
        };
        assert!(Value::from_dynamic(&nested(MAX_DEPTH)).is_some());
        assert_eq!(Value::from_dynamic(&nested(MAX_DEPTH + 1)), None);
    }
}
