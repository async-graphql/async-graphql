use crate::{Result, Scalar, Value};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ID(String);

impl Deref for ID {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Scalar for ID {
    fn type_name() -> &'static str {
        "ID"
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::Int(n) => Some(ID(n.as_i64().unwrap().to_string())),
            Value::String(s) => Some(ID(s.clone())),
            _ => None,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}
