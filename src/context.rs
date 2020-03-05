use crate::registry::Registry;
use crate::{ErrorWithPosition, GQLInputValue, GQLType, QueryError, Result};
use fnv::FnvHasher;
use graphql_parser::query::{
    Directive, Field, FragmentDefinition, Selection, SelectionSet, Value, VariableDefinition,
};
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

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Variables {
    pub(crate) fn parse_from_json(value: serde_json::Value) -> Result<Self> {
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

#[derive(Default)]
pub struct Data(HashMap<TypeId, Box<dyn Any + Sync + Send>, BuildHasherDefault<FnvHasher>>);

impl Data {
    pub fn insert<D: Any + Send + Sync>(&mut self, data: D) {
        self.0.insert(TypeId::of::<D>(), Box::new(data));
    }
}

pub type ContextSelectionSet<'a> = ContextBase<'a, &'a SelectionSet>;
pub type Context<'a> = ContextBase<'a, &'a Field>;

pub struct ContextBase<'a, T> {
    pub(crate) item: T,
    pub(crate) variables: Option<&'a Variables>,
    pub(crate) variable_definitions: Option<&'a [VariableDefinition]>,
    pub(crate) registry: &'a Registry,
    pub(crate) data: &'a Data,
    pub(crate) fragments: &'a HashMap<String, &'a FragmentDefinition>,
}

impl<'a, T> Deref for ContextBase<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

pub struct FieldIter<'a, T> {
    ctx: &'a ContextBase<'a, T>,
    stack: Vec<std::slice::Iter<'a, Selection>>,
}

impl<'a, T> Iterator for FieldIter<'a, T> {
    type Item = Result<&'a Field>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(it) = self.stack.last_mut() {
            if let Some(selection) = it.next() {
                match selection {
                    Selection::Field(field) => {
                        return Some(Ok(field));
                    }
                    Selection::FragmentSpread(fragment_spread) => {
                        let skip = match self.ctx.is_skip(&fragment_spread.directives) {
                            Ok(skip) => skip,
                            Err(err) => return Some(Err(err)),
                        };
                        if skip {
                            continue;
                        }
                        if let Some(fragment) =
                            self.ctx.fragments.get(&fragment_spread.fragment_name)
                        {
                            self.stack.push(fragment.selection_set.items.iter());
                        } else {
                            return Some(Err(QueryError::UnknownFragment {
                                name: fragment_spread.fragment_name.clone(),
                            }
                            .into()));
                        }
                    }
                    Selection::InlineFragment(_) => {}
                }
            } else {
                self.stack.pop();
            }
        }
        None
    }
}

impl<'a, T> ContextBase<'a, T> {
    #[doc(hidden)]
    pub fn with_item<R>(&self, item: R) -> ContextBase<'a, R> {
        ContextBase {
            item,
            variables: self.variables,
            variable_definitions: self.variable_definitions,
            registry: self.registry.clone(),
            data: self.data,
            fragments: self.fragments,
        }
    }

    pub fn data<D: Any + Send + Sync>(&self) -> Option<&D> {
        self.data
            .0
            .get(&TypeId::of::<D>())
            .and_then(|d| d.downcast_ref::<D>())
    }

    fn resolve_input_value(&self, value: Value) -> Result<Value> {
        if let Value::Variable(var_name) = value {
            let def = self
                .variable_definitions
                .and_then(|defs| defs.iter().find(|def| def.name == var_name.as_str()));
            if let Some(def) = def {
                if let Some(var_value) = self.variables.map(|vars| vars.get(&def.name)).flatten() {
                    return Ok(var_value.clone());
                } else if let Some(default) = &def.default_value {
                    return Ok(default.clone());
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
    pub fn fields(&'a self, selection_set: &'a SelectionSet) -> FieldIter<'a, T> {
        FieldIter {
            ctx: self,
            stack: vec![selection_set.items.iter()],
        }
    }

    #[doc(hidden)]
    pub fn is_skip(&self, directives: &[Directive]) -> Result<bool> {
        for directive in directives {
            if directive.name == "skip" {
                if let Some(value) = directive
                    .arguments
                    .iter()
                    .find(|(name, _)| name == "if")
                    .map(|(_, value)| value)
                {
                    let value = self.resolve_input_value(value.clone())?;
                    let res: bool = GQLInputValue::parse(&value).ok_or_else(|| {
                        QueryError::ExpectedType {
                            expect: bool::qualified_type_name(),
                            actual: value,
                        }
                        .with_position(directive.position)
                    })?;
                    if res {
                        return Ok(true);
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@skip",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .with_position(directive.position)
                    .into());
                }
            } else if directive.name == "include" {
                if let Some(value) = directive
                    .arguments
                    .iter()
                    .find(|(name, _)| name == "if")
                    .map(|(_, value)| value)
                {
                    let value = self.resolve_input_value(value.clone())?;
                    let res: bool = GQLInputValue::parse(&value).ok_or_else(|| {
                        QueryError::ExpectedType {
                            expect: bool::qualified_type_name(),
                            actual: value,
                        }
                        .with_position(directive.position)
                    })?;
                    if !res {
                        return Ok(true);
                    }
                } else {
                    return Err(QueryError::RequiredDirectiveArgs {
                        directive: "@include",
                        arg_name: "if",
                        arg_type: "Boolean!",
                    }
                    .with_position(directive.position)
                    .into());
                }
            } else {
                return Err(QueryError::UnknownDirective {
                    name: directive.name.clone(),
                }
                .with_position(directive.position)
                .into());
            }
        }

        Ok(false)
    }
}

impl<'a> ContextBase<'a, &'a Field> {
    #[doc(hidden)]
    pub fn param_value<T: GQLInputValue, F: FnOnce() -> Value>(
        &self,
        name: &str,
        default: F,
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
            None => {
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
        }
    }
}
