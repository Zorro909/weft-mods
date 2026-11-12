//! A value scripts keep, in the shared store or in an entity's data.

use rhai::{Array, Dynamic};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
    List(Vec<Value>),
}

impl Value {
    /// What a script handed over; `None` for what can't be kept (a map, a function, nothing).
    pub fn from_dynamic(dynamic: &Dynamic) -> Option<Value> {
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
        let list = dynamic.clone().try_cast::<Array>()?;
        let values: Option<Vec<Value>> = list.iter().map(Value::from_dynamic).collect();
        values.map(Value::List)
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
        }
    }
}

#[cfg(test)]
mod tests {
    use rhai::Map;

    use super::*;

    #[test]
    fn every_kind_survives_a_script_and_back() {
        let values = [
            Value::Int(-7),
            Value::Float(2.5),
            Value::Bool(true),
            Value::Text("home".into()),
            Value::List(vec![
                Value::Int(1),
                Value::List(vec![Value::Text("a".into())]),
            ]),
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
        assert_eq!(
            Value::from_dynamic(&Dynamic::from_map(Map::new())),
            None,
            "a map is not kept"
        );
        let mixed: Array = vec![Dynamic::from(1_i64), Dynamic::UNIT];
        assert_eq!(
            Value::from_dynamic(&Dynamic::from_array(mixed)),
            None,
            "nor a list holding one"
        );
    }
}
