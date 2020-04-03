use crate::context::Data;
use crate::extensions::BoxExtension;
use crate::mutation_resolver::do_mutation_resolve;
use crate::registry::CacheControl;
use crate::{do_resolve, ContextBase, Error, Result, Schema};
use crate::{ObjectType, QueryError, Variables};
use bytes::Bytes;
use graphql_parser::query::{
    Definition, Document, OperationDefinition, SelectionSet, VariableDefinition,
};
use graphql_parser::Pos;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

/// Query response
pub struct QueryResponse {
    /// Data of query result
    pub data: serde_json::Value,

    /// Extensions result
    pub extensions: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Query builder
pub struct QueryBuilder<Query, Mutation, Subscription> {
    pub(crate) schema: Schema<Query, Mutation, Subscription>,
    pub(crate) extensions: Vec<BoxExtension>,
    pub(crate) document: Document,
    pub(crate) operation_name: Option<String>,
    pub(crate) variables: Variables,
    pub(crate) ctx_data: Option<Data>,
    pub(crate) cache_control: CacheControl,
}

impl<Query, Mutation, Subscription> QueryBuilder<Query, Mutation, Subscription> {
    fn current_operation(&self) -> Option<(&SelectionSet, &[VariableDefinition], bool)> {
        for definition in &self.document.definitions {
            match definition {
                Definition::Operation(operation_definition) => match operation_definition {
                    OperationDefinition::SelectionSet(s) => {
                        return Some((s, &[], true));
                    }
                    OperationDefinition::Query(query)
                        if query.name.is_none()
                            || query.name.as_deref() == self.operation_name.as_deref() =>
                    {
                        return Some((&query.selection_set, &query.variable_definitions, true));
                    }
                    OperationDefinition::Mutation(mutation)
                        if mutation.name.is_none()
                            || mutation.name.as_deref() == self.operation_name.as_deref() =>
                    {
                        return Some((
                            &mutation.selection_set,
                            &mutation.variable_definitions,
                            false,
                        ));
                    }
                    OperationDefinition::Subscription(subscription)
                        if subscription.name.is_none()
                            || subscription.name.as_deref() == self.operation_name.as_deref() =>
                    {
                        return None;
                    }
                    _ => {}
                },
                Definition::Fragment(_) => {}
            }
        }
        None
    }

    /// Specify the operation name.
    pub fn operator_name<T: Into<String>>(self, name: T) -> Self {
        QueryBuilder {
            operation_name: Some(name.into()),
            ..self
        }
    }

    /// Specify the variables.
    pub fn variables(self, variables: Variables) -> Self {
        QueryBuilder { variables, ..self }
    }

    /// Add a context data that can be accessed in the `Context`, you access it with `Context::data`.
    pub fn data<D: Any + Send + Sync>(mut self, data: D) -> Self {
        if let Some(ctx_data) = &mut self.ctx_data {
            ctx_data.insert(data);
        } else {
            let mut ctx_data = Data::default();
            ctx_data.insert(data);
            self.ctx_data = Some(ctx_data);
        }
        self
    }

    /// Detects whether any parameter contains the Upload type
    pub fn is_upload(&self) -> bool {
        if let Some((_, variable_definitions, _)) = self.current_operation() {
            for d in variable_definitions {
                if let Some(ty) = self
                    .schema
                    .0
                    .registry
                    .basic_type_by_parsed_type(&d.var_type)
                {
                    if ty.name() == "Upload" {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Set upload files
    pub fn set_upload(
        &mut self,
        var_path: &str,
        filename: &str,
        content_type: Option<&str>,
        content: Bytes,
    ) {
        self.variables
            .set_upload(var_path, filename, content_type, content);
    }

    /// Execute the query.
    pub async fn execute(self) -> Result<QueryResponse>
    where
        Query: ObjectType + Send + Sync,
        Mutation: ObjectType + Send + Sync,
    {
        let resolve_id = AtomicUsize::default();
        let mut fragments = HashMap::new();
        let (selection_set, variable_definitions, is_query) =
            self.current_operation().ok_or_else(|| Error::Query {
                pos: Pos::default(),
                path: None,
                err: QueryError::MissingOperation,
            })?;

        for definition in &self.document.definitions {
            if let Definition::Fragment(fragment) = &definition {
                fragments.insert(fragment.name.clone(), fragment.clone());
            }
        }

        let ctx = ContextBase {
            path_node: None,
            resolve_id: &resolve_id,
            extensions: &self.extensions,
            item: selection_set,
            variables: &self.variables,
            variable_definitions,
            registry: &self.schema.0.registry,
            data: &self.schema.0.data,
            ctx_data: self.ctx_data.as_ref(),
            fragments: &fragments,
        };

        self.extensions.iter().for_each(|e| e.execution_start());
        let data = if is_query {
            do_resolve(&ctx, &self.schema.0.query).await?
        } else {
            do_mutation_resolve(&ctx, &self.schema.0.mutation).await?
        };
        self.extensions.iter().for_each(|e| e.execution_end());

        let res = QueryResponse {
            data,
            extensions: if !self.extensions.is_empty() {
                Some(
                    self.extensions
                        .iter()
                        .map(|e| (e.name().to_string(), e.result()))
                        .collect::<serde_json::Map<_, _>>(),
                )
            } else {
                None
            },
        };
        Ok(res)
    }

    /// Get cache control value
    pub fn cache_control(&self) -> CacheControl {
        self.cache_control
    }
}
