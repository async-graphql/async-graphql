use crate::registry::Registry;
use crate::{ErrorWithPosition, GQLInputValue, QueryError, Result};
use fnv::FnvHasher;
use graphql_parser::query::{Field, SelectionSet, Value, VariableDefinition};
use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Variables(BTreeMap<String, Value>);

impl Deref for Variables {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Variables {
    pub fn parse_from_json(data: &[u8]) -> Result<Self> {
        let value = serde_json::from_slice(data)?;
        let gql_value = json_value_to_gql_value(value);
        if let Value::Object(obj) = gql_value {
            Ok(Variables(obj))
        } else {
            Ok(Default::default())
        }
    }
}

fn json_value_to_gql_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(n) => Value::Boolean(n),
        serde_json::Value::Number(n) if n.is_f64() => Value::Float(n.as_f64().unwrap()),
        serde_json::Value::Number(n) => Value::Int((n.as_i64().unwrap() as i32).into()),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(ls) => Value::List(
            ls.into_iter()
                .map(|value| json_value_to_gql_value(value))
                .collect(),
        ),
        serde_json::Value::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(name, value)| (name, json_value_to_gql_value(value)))
                .collect(),
        ),
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Default)]
pub struct Data(HashMap<TypeId, Box<dyn Any + Sync + Send>, BuildHasherDefault<FnvHasher>>);

impl Data {
    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }

    pub fn remove<D: Any + Send + Sync>(&mut self) {
        self.0.remove(&TypeId::of::<D>());
    }
}

pub type ContextSelectionSet<'a> = ContextBase<'a, &'a SelectionSet>;
pub type Context<'a> = ContextBase<'a, &'a Field>;

pub struct ContextBase<'a, T> {
    pub(crate) item: T,
    pub(crate) data: Option<&'a Data>,
    pub(crate) variables: Option<&'a Variables>,
    pub(crate) variable_definitions: Option<&'a [VariableDefinition]>,
    pub(crate) registry: &'a Registry,
}

impl<'a, T> Deref for ContextBase<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a, T> ContextBase<'a, T> {
    #[doc(hidden)]
    pub fn with_item<R>(&self, item: R) -> ContextBase<'a, R> {
        ContextBase {
            item,
            data: self.data,
            variables: self.variables,
            variable_definitions: self.variable_definitions,
            registry: self.registry.clone(),
        }
    }

    pub fn data<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.data.and_then(|data| {
            data.0
                .get(&TypeId::of::<D>())
                .and_then(|d| d.downcast_ref::<D>())
        })
    }
}

impl<'a> ContextBase<'a, &'a Field> {
    fn resolve_input_value(&self, value: Value) -> Result<Value> {
        if let Value::Variable(var_name) = value {
            let def = self
                .variable_definitions
                .and_then(|defs| defs.iter().find(|def| def.name == var_name.as_str()));
            if let Some(def) = def {
                if let Some(var_value) = self.variables.map(|vars| vars.get(&def.name)).flatten() {
                    return Ok(var_value.clone());
                }
            }
            return Err(QueryError::VarNotDefined {
                var_name: var_name.clone(),
            }
            .into());
        } else {
            Ok(value)
        }
    }

    #[doc(hidden)]
    pub fn param_value<T: GQLInputValue, F: FnOnce() -> Value>(
        &self,
        name: &str,
        default: Option<F>,
    ) -> Result<T> {
        match self
            .arguments
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .cloned()
        {
            Some(value) => {
                let value = self.resolve_input_value(value)?;
                let res = GQLInputValue::parse(&value).ok_or_else(|| {
                    QueryError::ExpectedType {
                        expect: T::qualified_type_name(),
                        actual: value,
                    }
                    .with_position(self.item.position)
                })?;
                Ok(res)
            }
            None if default.is_some() => {
                let default = default.unwrap();
                let value = default();
                let res = GQLInputValue::parse(&value).ok_or_else(|| {
                    QueryError::ExpectedType {
                        expect: T::qualified_type_name(),
                        actual: value.clone(),
                    }
                    .with_position(self.item.position)
                })?;
                Ok(res)
            }
            None => {
                let res = GQLInputValue::parse(&Value::Null).ok_or_else(|| {
                    QueryError::ExpectedType {
                        expect: T::qualified_type_name(),
                        actual: Value::Null,
                    }
                    .with_position(self.item.position)
                })?;
                Ok(res)
            }
        }
    }
}
