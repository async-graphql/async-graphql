use crate::{GQLInputValue, Result};
use fnv::FnvHasher;
use graphql_parser::query::{Field, SelectionSet, Value};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize, Default)]
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

pub type ContextSelectionSet<'a> = Context<'a, &'a SelectionSet>;
pub type ContextField<'a> = Context<'a, &'a Field>;

pub struct Context<'a, T> {
    pub(crate) item: T,
    pub(crate) data: Option<&'a Data>,
    pub(crate) variables: Option<&'a Variables>,
}

impl<'a, T> Deref for Context<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a, T> Context<'a, T> {
    pub fn with_item<R>(&self, item: R) -> Context<'a, R> {
        Context {
            item,
            data: self.data,
            variables: self.variables,
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

impl<'a> Context<'a, &'a Field> {
    pub fn param_value<T: GQLInputValue>(&self, name: &str) -> Result<T> {
        let value = self
            .arguments
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .cloned()
            .unwrap_or(Value::Null);
        GQLInputValue::parse(value)
    }
}
