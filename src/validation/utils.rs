use crate::registry::{Registry, Type, TypeInfo};
use crate::Value;

pub fn is_valid_input_value(registry: &Registry, type_name: &str, value: &Value) -> bool {
    match TypeInfo::create(type_name) {
        TypeInfo::NonNull(type_name) => match value {
            Value::Null => false,
            _ => is_valid_input_value(registry, type_name, value),
        },
        TypeInfo::List(type_name) => match value {
            Value::List(elems) => elems
                .iter()
                .all(|elem| is_valid_input_value(registry, type_name, elem)),
            _ => false,
        },
        TypeInfo::Type(type_name) => {
            if let Value::Null = value {
                return true;
            }

            if let Some(ty) = registry.types.get(type_name) {
                match ty {
                    Type::Scalar { is_valid, .. } => is_valid(value),
                    Type::Enum { enum_values, .. } => match value {
                        Value::Enum(name) => enum_values.contains_key(name.as_str()),
                        _ => false,
                    },
                    Type::InputObject { input_fields, .. } => match value {
                        Value::Object(values) => {
                            for field in input_fields {
                                let value = values.get(field.name).unwrap_or(&Value::Null);
                                if !is_valid_input_value(registry, &field.ty, value) {
                                    return field.default_value.is_some();
                                }
                            }
                            true
                        }
                        _ => false,
                    },
                    _ => false,
                }
            } else {
                unreachable!()
            }
        }
    }
}
