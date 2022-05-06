use std::collections::HashSet;

use async_graphql_value::{ConstValue, Value};

use crate::{context::QueryPathNode, registry, QueryPathSegment};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

pub fn is_valid_input_value(
    registry: &registry::Registry,
    type_name: &str,
    value: &ConstValue,
    path_node: QueryPathNode,
) -> Option<String> {
    match registry::MetaTypeName::create(type_name) {
        registry::MetaTypeName::NonNull(type_name) => match value {
            ConstValue::Null => Some(valid_error(
                &path_node,
                format!("expected type \"{}\"", type_name),
            )),
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::List(type_name) => match value {
            ConstValue::List(elems) => elems.iter().enumerate().find_map(|(idx, elem)| {
                is_valid_input_value(
                    registry,
                    type_name,
                    elem,
                    QueryPathNode {
                        parent: Some(&path_node),
                        segment: QueryPathSegment::Index(idx),
                    },
                )
            }),
            ConstValue::Null => None,
            _ => is_valid_input_value(registry, type_name, value, path_node),
        },
        registry::MetaTypeName::Named(type_name) => {
            if let ConstValue::Null = value {
                return None;
            }

            match registry
                .types
                .get(type_name)
                .unwrap_or_else(|| panic!("Type `{}` not defined", type_name))
            {
                registry::MetaType::Scalar { is_valid, .. } => {
                    if is_valid(&value) {
                        None
                    } else {
                        Some(valid_error(
                            &path_node,
                            format!("expected type \"{}\"", type_name),
                        ))
                    }
                }
                registry::MetaType::Enum {
                    enum_values,
                    name: enum_name,
                    ..
                } => match value {
                    ConstValue::Enum(name) => {
                        if !enum_values.contains_key(name.as_str()) {
                            Some(valid_error(
                                &path_node,
                                format!(
                                    "enumeration type \"{}\" does not contain the value \"{}\"",
                                    enum_name, name
                                ),
                            ))
                        } else {
                            None
                        }
                    }
                    ConstValue::String(name) => {
                        if !enum_values.contains_key(name.as_str()) {
                            Some(valid_error(
                                &path_node,
                                format!(
                                    "enumeration type \"{}\" does not contain the value \"{}\"",
                                    enum_name, name
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
                registry::MetaType::InputObject {
                    input_fields,
                    name: object_name,
                    oneof,
                    ..
                } => match value {
                    ConstValue::Object(values) => {
                        if *oneof {
                            if values.len() != 1 {
                                return Some(valid_error(
                                    &path_node,
                                    "Oneof input objects requires have exactly one field"
                                        .to_string(),
                                ));
                            }

                            if let ConstValue::Null = values[0] {
                                return Some(valid_error(
                                    &path_node,
                                    "Oneof Input Objects require that exactly one field must be supplied and that field must not be null"
                                        .to_string(),
                                ));
                            }
                        }

                        let mut input_names =
                            values.keys().map(AsRef::as_ref).collect::<HashSet<_>>();

                        for field in input_fields.values() {
                            input_names.remove(field.name);
                            if let Some(value) = values.get(field.name) {
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
                                        field.name, object_name,
                                    ),
                                ));
                            }
                        }

                        if let Some(name) = input_names.iter().next() {
                            return Some(valid_error(
                                &path_node,
                                format!("unknown field \"{}\" of type \"{}\"", name, object_name),
                            ));
                        }

                        None
                    }
                    _ => None,
                },
                _ => None,
            }
        }
    }
}
