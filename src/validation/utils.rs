use crate::context::QueryPathNode;
use crate::parser::query::OperationDefinition;
use crate::{registry, Pos, QueryPathSegment, Value};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope<'a> {
    Operation(Option<&'a str>),
    Fragment(&'a str),
}

fn valid_error(path_node: &QueryPathNode, msg: String) -> String {
    format!("\"{}\", {}", path_node, msg)
}

pub fn referenced_variables(value: &Value) -> Vec<&str> {
    let mut vars = Vec::new();
    referenced_variables_to_vec(value, &mut vars);
    vars
}

fn referenced_variables_to_vec<'a>(value: &'a Value, vars: &mut Vec<&'a str>) {
    match value {
        Value::Variable(name) => {
            vars.push(name);
        }
        Value::List(values) => values
            .iter()
            .for_each(|value| referenced_variables_to_vec(value, vars)),
        Value::Object(obj) => obj
            .values()
            .for_each(|value| referenced_variables_to_vec(value, vars)),
        _ => {}
    }
}

pub fn operation_name(operation_definition: &OperationDefinition) -> (Option<&str>, Pos) {
    match operation_definition {
        OperationDefinition::SelectionSet(selection_set) => (None, selection_set.position()),
        OperationDefinition::Query(query) => {
            (query.name.as_ref().map(|n| n.as_str()), query.position())
        }
        OperationDefinition::Mutation(mutation) => (
            mutation.name.as_ref().map(|n| n.as_str()),
            mutation.position(),
        ),
        OperationDefinition::Subscription(subscription) => (
            subscription.name.as_ref().map(|n| n.as_str()),
            subscription.position(),
        ),
    }
}

pub fn is_valid_input_value(
    registry: &registry::Registry,
    type_name: &str,
    value: &Value,
    path_node: QueryPathNode,
) -> Option<String> {
    if let Value::Variable(_) = value {
        return None;
    }

    match registry::MetaTypeName::create(type_name) {
        registry::MetaTypeName::NonNull(type_name) => match value {
            Value::Null => Some(valid_error(
                &path_node,
                format!("expected type \"{}\"", type_name),
            )),
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::List(type_name) => match value {
            Value::List(elems) => {
                for (idx, elem) in elems.iter().enumerate() {
                    if let Some(reason) = is_valid_input_value(
                        registry,
                        type_name,
                        elem,
                        QueryPathNode {
                            parent: Some(&path_node),
                            segment: QueryPathSegment::Index(idx),
                        },
                    ) {
                        return Some(reason);
                    }
                }
                None
            }
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::Named(type_name) => {
            if let Value::Null = value {
                return None;
            }

            if let Some(ty) = registry.types.get(type_name) {
                match ty {
                    registry::MetaType::Scalar { is_valid, .. } => {
                        if !is_valid(value) {
                            Some(valid_error(
                                &path_node,
                                format!("expected type \"{}\"", type_name),
                            ))
                        } else {
                            None
                        }
                    }
                    registry::MetaType::Enum { enum_values, .. } => match value {
                        Value::Enum(name) => {
                            if !enum_values.contains_key(name.as_str()) {
                                Some(valid_error(
                                    &path_node,
                                    format!(
                                        "enumeration type \"{}\" does not contain the value \"{}\"",
                                        ty.name(),
                                        name
                                    ),
                                ))
                            } else {
                                None
                            }
                        }
                        _ => Some(valid_error(
                            &path_node,
                            format!("expected type \"{}\"", type_name),
                        )),
                    },
                    registry::MetaType::InputObject { input_fields, .. } => match value {
                        Value::Object(values) => {
                            let mut input_names = values
                                .keys()
                                .map(|name| name.as_ref())
                                .collect::<HashSet<_>>();

                            for field in input_fields.values() {
                                input_names.remove(field.name);
                                if let Some(value) = values.get(field.name) {
                                    if let Some(validator) = &field.validator {
                                        if let Some(reason) = validator.is_valid(value) {
                                            return Some(valid_error(
                                                &QueryPathNode {
                                                    parent: Some(&path_node),
                                                    segment: QueryPathSegment::Name(field.name),
                                                },
                                                reason,
                                            ));
                                        }
                                    }

                                    if let Some(reason) = is_valid_input_value(
                                        registry,
                                        &field.ty,
                                        value,
                                        QueryPathNode {
                                            parent: Some(&path_node),
                                            segment: QueryPathSegment::Name(field.name),
                                        },
                                    ) {
                                        return Some(reason);
                                    }
                                } else if registry::MetaTypeName::create(&field.ty).is_non_null()
                                    && field.default_value.is_none()
                                {
                                    return Some(valid_error(
                                            &path_node,
                                            format!(
                                                "field \"{}\" of type \"{}\" is required but not provided",
                                                field.name,
                                                ty.name(),
                                            ),
                                        ));
                                }
                            }

                            if let Some(name) = input_names.iter().next() {
                                return Some(valid_error(
                                    &path_node,
                                    format!("unknown field \"{}\" of type \"{}\"", name, ty.name()),
                                ));
                            }

                            None
                        }
                        _ => None,
                    },
                    _ => None,
                }
            } else {
                unreachable!()
            }
        }
    }
}
