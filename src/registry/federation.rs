use crate::registry::{MetaField, MetaInputValue, MetaType, Registry};
use crate::{Any, Type};
use indexmap::IndexMap;
use itertools::Itertools;
use std::fmt::Write;

impl Registry {
    pub fn create_federation_sdl(&self) -> String {
        let mut sdl = String::new();
        for ty in self.types.values() {
            if ty.name().starts_with("__") {
                continue;
            }
            const FEDERATION_TYPES: &[&str] = &["_Any", "_Entity", "_Service"];
            if FEDERATION_TYPES.contains(&ty.name()) {
                continue;
            }
            self.create_federation_type(ty, &mut sdl);
        }
        sdl
    }

    pub fn create_federation_types(&mut self) {
        Any::create_type_info(self);

        self.types.insert(
            "_Service".to_string(),
            MetaType::Object {
                name: "_Service".to_string(),
                description: None,
                fields: {
                    let mut fields = IndexMap::new();
                    fields.insert(
                        "sdl".to_string(),
                        MetaField {
                            name: "sdl".to_string(),
                            description: None,
                            args: Default::default(),
                            ty: "String".to_string(),
                            deprecation: None,
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                        },
                    );
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                keys: None,
            },
        );

        self.create_entity_type();

        let query_root = self.types.get_mut(&self.query_type).unwrap();
        if let MetaType::Object { fields, .. } = query_root {
            fields.insert(
                "_service".to_string(),
                MetaField {
                    name: "_service".to_string(),
                    description: None,
                    args: Default::default(),
                    ty: "_Service!".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                },
            );

            fields.insert(
                "_entities".to_string(),
                MetaField {
                    name: "_entities".to_string(),
                    description: None,
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "representations",
                            MetaInputValue {
                                name: "representations",
                                description: None,
                                ty: "[_Any!]!".to_string(),
                                default_value: None,
                                validator: None,
                            },
                        );
                        args
                    },
                    ty: "[_Entity]!".to_string(),
                    deprecation: None,
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                },
            );
        }
    }

    fn create_federation_fields<'a, I: Iterator<Item = &'a MetaField>>(sdl: &mut String, it: I) {
        for field in it {
            if field.name.starts_with("__") || matches!(&*field.name, "_service" | "_entities") {
                continue;
            }

            if !field.args.is_empty() {
                write!(
                    sdl,
                    "\t{}({}): {}",
                    field.name,
                    field
                        .args
                        .values()
                        .map(|arg| federation_input_value(arg))
                        .join(", "),
                    field.ty
                )
                .ok();
            } else {
                write!(sdl, "\t{}: {}", field.name, field.ty).ok();
            }

            if field.external {
                write!(sdl, " @external").ok();
            }
            if let Some(requires) = field.requires {
                write!(sdl, " @requires(fields: \"{}\")", requires).ok();
            }
            if let Some(provides) = field.provides {
                write!(sdl, " @provides(fields: \"{}\")", provides).ok();
            }
            writeln!(sdl).ok();
        }
    }

    fn create_federation_type(&self, ty: &MetaType, sdl: &mut String) {
        match ty {
            MetaType::Scalar { name, .. } => {
                const SYSTEM_SCALARS: &[&str] = &["Int", "Float", "String", "Boolean", "ID", "Any"];
                if !SYSTEM_SCALARS.contains(&name.as_str()) {
                    writeln!(sdl, "scalar {}", name).ok();
                }
            }
            MetaType::Object {
                name,
                fields,
                extends,
                keys,
                ..
            } => {
                if name == &self.query_type && fields.len() == 4 {
                    // Is empty query root, only __schema, __type, _service, _entities fields
                    return;
                }
                if let Some(subscription_type) = &self.subscription_type {
                    if name == subscription_type {
                        return;
                    }
                }
                if *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "type {} ", name).ok();
                if let Some(implements) = self.implements.get(name) {
                    if !implements.is_empty() {
                        write!(sdl, "implements {}", implements.iter().join(" & ")).ok();
                    }
                }
                if let Some(keys) = keys {
                    for key in keys {
                        write!(sdl, "@key(fields: \"{}\") ", key).ok();
                    }
                }
                writeln!(sdl, "{{").ok();
                Self::create_federation_fields(sdl, fields.values());
                writeln!(sdl, "}}").ok();
            }
            MetaType::Interface {
                name,
                fields,
                extends,
                keys,
                ..
            } => {
                if *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "interface {} ", name).ok();
                if let Some(keys) = keys {
                    for key in keys {
                        write!(sdl, "@key(fields: \"{}\") ", key).ok();
                    }
                }
                writeln!(sdl, "{{").ok();
                Self::create_federation_fields(sdl, fields.values());
                writeln!(sdl, "}}").ok();
            }
            MetaType::Enum {
                name, enum_values, ..
            } => {
                write!(sdl, "enum {} ", name).ok();
                writeln!(sdl, "{{").ok();
                for value in enum_values.values() {
                    writeln!(sdl, "{}", value.name).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::InputObject {
                name, input_fields, ..
            } => {
                write!(sdl, "input {} ", name).ok();
                writeln!(sdl, "{{").ok();
                for field in input_fields.values() {
                    writeln!(sdl, "{}", federation_input_value(&field)).ok();
                }
                writeln!(sdl, "}}").ok();
            }
            MetaType::Union {
                name,
                possible_types,
                ..
            } => {
                writeln!(
                    sdl,
                    "union {} = {}",
                    name,
                    possible_types.iter().join(" | ")
                )
                .ok();
            }
        }
    }
}

fn federation_input_value(input_value: &MetaInputValue) -> String {
    if let Some(default_value) = &input_value.default_value {
        format!(
            "{}: {} = {}",
            input_value.name, input_value.ty, default_value
        )
    } else {
        format!("{}: {}", input_value.name, input_value.ty)
    }
}
