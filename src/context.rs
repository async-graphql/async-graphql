use crate::registry::Registry;
use crate::{ErrorWithPosition, GQLInputValue, QueryError, Result};
use fnv::FnvHasher;
use graphql_parser::query::{Field, SelectionSet, Value};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Variables(HashMap<String, serde_json::Value>);

impl Deref for Variables {
    type Target = HashMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    #[doc(hidden)]
    pub fn param_value<T: GQLInputValue>(&self, name: &str) -> Result<T> {
        let value = self
            .arguments
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .cloned();

        if let Some(Value::Variable(var_name)) = &value {
            if let Some(vars) = &self.variables {
                if let Some(var_value) = vars.get(&*var_name).cloned() {
                    let res =
                        GQLInputValue::parse_from_json(var_value.clone()).ok_or_else(|| {
                            QueryError::ExpectedJsonType {
                                expect: T::qualified_type_name(),
                                actual: var_value,
                            }
                            .with_position(self.item.position)
                        })?;
                    return Ok(res);
                }
            }

            return Err(QueryError::VarNotDefined {
                var_name: var_name.clone(),
            }
            .into());
        };

        let value = value.unwrap_or(Value::Null);
        let res = GQLInputValue::parse(value.clone()).ok_or_else(|| {
            QueryError::ExpectedType {
                expect: T::qualified_type_name(),
                actual: value,
            }
            .with_position(self.item.position)
        })?;
        Ok(res)
    }
}
