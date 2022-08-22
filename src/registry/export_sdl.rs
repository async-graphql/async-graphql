use std::fmt::Write;

use crate::registry::{Deprecation, MetaField, MetaInputValue, MetaType, Registry};

const SYSTEM_SCALARS: &[&str] = &["Int", "Float", "String", "Boolean", "ID"];
const FEDERATION_SCALARS: &[&str] = &["Any"];

/// Options for SDL export
#[derive(Debug, Copy, Clone, Default)]
pub struct SDLExportOptions {
    sorted_fields: bool,
    sorted_arguments: bool,
    sorted_enum_values: bool,
    federation: bool,
    prefer_single_line_descriptions: bool,
}

impl SDLExportOptions {
    /// Create a `SDLExportOptions`
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Export sorted fields
    #[inline]
    #[must_use]
    pub fn sorted_fields(self) -> Self {
        Self {
            sorted_fields: true,
            ..self
        }
    }

    /// Export sorted field arguments
    #[inline]
    #[must_use]
    pub fn sorted_arguments(self) -> Self {
        Self {
            sorted_arguments: true,
            ..self
        }
    }

    /// Export sorted enum items
    #[inline]
    #[must_use]
    pub fn sorted_enum_items(self) -> Self {
        Self {
            sorted_enum_values: true,
            ..self
        }
    }

    /// Export as Federation SDL(Schema Definition Language)
    #[inline]
    #[must_use]
    pub fn federation(self) -> Self {
        Self {
            federation: true,
            ..self
        }
    }

    /// When possible, write one-line instead of three-line descriptions
    #[inline]
    #[must_use]
    pub fn prefer_single_line_descriptions(self) -> Self {
        Self {
            prefer_single_line_descriptions: true,
            ..self
        }
    }
}

impl Registry {
    pub(crate) fn export_sdl(&self, options: SDLExportOptions) -> String {
        let mut sdl = String::new();

        let has_oneof = self
            .types
            .values()
            .any(|ty| matches!(ty, MetaType::InputObject { oneof: true, .. }));

        if has_oneof {
            sdl.write_str("directive @oneOf on INPUT_OBJECT\n\n").ok();
        }

        for ty in self.types.values() {
            if ty.name().starts_with("__") {
                continue;
            }

            if options.federation {
                const FEDERATION_TYPES: &[&str] = &["_Any", "_Entity", "_Service"];
                if FEDERATION_TYPES.contains(&ty.name()) {
                    continue;
                }
            }

            self.export_type(ty, &mut sdl, &options);
            writeln!(sdl).ok();
        }

        if !options.federation {
            writeln!(sdl, "schema {{").ok();
            writeln!(sdl, "\tquery: {}", self.query_type).ok();
            if let Some(mutation_type) = self.mutation_type.as_deref() {
                writeln!(sdl, "\tmutation: {}", mutation_type).ok();
            }
            if let Some(subscription_type) = self.subscription_type.as_deref() {
                writeln!(sdl, "\tsubscription: {}", subscription_type).ok();
            }
            writeln!(sdl, "}}").ok();
        }

        sdl
    }

    fn export_fields<'a, I: Iterator<Item = &'a MetaField>>(
        sdl: &mut String,
        it: I,
        options: &SDLExportOptions,
    ) {
        let mut fields = it.collect::<Vec<_>>();

        if options.sorted_fields {
            fields.sort_by(|a, b| a.name.cmp(&b.name));
        }

        for field in fields {
            if field.name.starts_with("__")
                || (options.federation && matches!(&*field.name, "_service" | "_entities"))
            {
                continue;
            }

            if let Some(description) = field.description {
                export_description(sdl, options, false, description);
            }

            if !field.args.is_empty() {
                write!(sdl, "\t{}(", field.name).ok();

                let mut args = field.args.values().collect::<Vec<_>>();
                if options.sorted_arguments {
                    args.sort_by(|a, b| a.name.cmp(b.name));
                }

                for (i, arg) in args.into_iter().enumerate() {
                    if i != 0 {
                        sdl.push_str(", ");
                    }
                    sdl.push_str(&export_input_value(arg));

                    if options.federation && arg.inaccessible {
                        write!(sdl, " @inaccessible").ok();
                    }
                }
                write!(sdl, "): {}", field.ty).ok();
            } else {
                write!(sdl, "\t{}: {}", field.name, field.ty).ok();
            }

            write_deprecated(sdl, &field.deprecation);

            if options.federation {
                if field.external {
                    write!(sdl, " @external").ok();
                }
                if let Some(requires) = field.requires {
                    write!(sdl, " @requires(fields: \"{}\")", requires).ok();
                }
                if let Some(provides) = field.provides {
                    write!(sdl, " @provides(fields: \"{}\")", provides).ok();
                }
                if field.shareable {
                    write!(sdl, " @shareable").ok();
                }
                if field.inaccessible {
                    write!(sdl, " @inaccessible").ok();
                }
                if let Some(from) = field.override_from {
                    write!(sdl, " @override(from: \"{}\")", from).ok();
                }
            }

            writeln!(sdl).ok();
        }
    }

    fn export_type(&self, ty: &MetaType, sdl: &mut String, options: &SDLExportOptions) {
        match ty {
            MetaType::Scalar {
                name,
                description,
                inaccessible,
                ..
            } => {
                let mut export_scalar = !SYSTEM_SCALARS.contains(&name.as_str());
                if options.federation && FEDERATION_SCALARS.contains(&name.as_str()) {
                    export_scalar = false;
                }
                if export_scalar {
                    if let Some(description) = description {
                        export_description(sdl, options, true, description);
                    }
                    write!(sdl, "scalar {} ", name).ok();

                    if options.federation && *inaccessible {
                        write!(sdl, "@inaccessible ").ok();
                    }
                    writeln!(sdl).ok();
                }
            }
            MetaType::Object {
                name,
                fields,
                extends,
                keys,
                description,
                shareable,
                inaccessible,
                ..
            } => {
                if Some(name.as_str()) == self.subscription_type.as_deref()
                    && options.federation
                    && !self.federation_subscription
                {
                    return;
                }

                if name.as_str() == self.query_type && options.federation {
                    let mut field_count = 0;
                    for field in fields.values() {
                        if field.name.starts_with("__")
                            || (options.federation
                                && matches!(&*field.name, "_service" | "_entities"))
                        {
                            continue;
                        }
                        field_count += 1;
                    }
                    if field_count == 0 {
                        // is empty query root type
                        return;
                    }
                }

                if let Some(description) = description {
                    export_description(sdl, options, true, description);
                }

                if options.federation && *extends {
                    write!(sdl, "extend ").ok();
                }

                write!(sdl, "type {} ", name).ok();
                self.write_implements(sdl, name);

                if options.federation {
                    if let Some(keys) = keys {
                        for key in keys {
                            write!(sdl, "@key(fields: \"{}\") ", key).ok();
                        }
                    }
                    if *shareable {
                        write!(sdl, "@shareable ").ok();
                    }

                    if *inaccessible {
                        write!(sdl, "@inaccessible ").ok();
                    }
                }

                writeln!(sdl, "{{").ok();
                Self::export_fields(sdl, fields.values(), options);
                writeln!(sdl, "}}").ok();
            }
            MetaType::Interface {
                name,
                fields,
                extends,
                keys,
                description,
                inaccessible,
                ..
            } => {
                if let Some(description) = description {
                    export_description(sdl, options, true, description);
                }

                if options.federation && *extends {
                    write!(sdl, "extend ").ok();
                }
                write!(sdl, "interface {} ", name).ok();

                if options.federation {
                    if let Some(keys) = keys {
                        for key in keys {
                            write!(sdl, "@key(fields: \"{}\") ", key).ok();
                        }
                    }
                    if *inaccessible {
                        write!(sdl, "@inaccessible ").ok();
                    }
                }
                self.write_implements(sdl, name);

                writeln!(sdl, "{{").ok();
                Self::export_fields(sdl, fields.values(), options);
                writeln!(sdl, "}}").ok();
            }
            MetaType::Enum {
                name,
                enum_values,
                description,
                inaccessible,
                ..
            } => {
                if let Some(description) = description {
                    export_description(sdl, options, true, description);
                }

                write!(sdl, "enum {} ", name).ok();
                if options.federation && *inaccessible {
                    write!(sdl, "@inaccessible ").ok();
                }
                writeln!(sdl, "{{").ok();

                let mut values = enum_values.values().collect::<Vec<_>>();
                if options.sorted_enum_values {
                    values.sort_by(|a, b| a.name.cmp(&b.name));
                }

                for value in values {
                    write!(sdl, "\t{}", value.name).ok();
                    write_deprecated(sdl, &value.deprecation);
                    if options.federation && value.inaccessible {
                        write!(sdl, " @inaccessible").ok();
                    }
                    writeln!(sdl).ok();
                }

                writeln!(sdl, "}}").ok();
            }
            MetaType::InputObject {
                name,
                input_fields,
                description,
                inaccessible,
                oneof,
                ..
            } => {
                if let Some(description) = description {
                    export_description(sdl, options, true, description);
                }

                write!(sdl, "input {} ", name).ok();

                if *oneof {
                    write!(sdl, "@oneof ").ok();
                }
                if options.federation && *inaccessible {
                    write!(sdl, "@inaccessible ").ok();
                }
                writeln!(sdl, "{{").ok();

                let mut fields = input_fields.values().collect::<Vec<_>>();
                if options.sorted_fields {
                    fields.sort_by(|a, b| a.name.cmp(b.name));
                }

                for field in fields {
                    if let Some(description) = field.description {
                        export_description(sdl, options, false, description);
                    }
                    write!(sdl, "\t{} ", export_input_value(&field)).ok();
                    if options.federation && field.inaccessible {
                        write!(sdl, "@inaccessible ").ok();
                    }
                    writeln!(sdl).ok();
                }

                writeln!(sdl, "}}").ok();
            }
            MetaType::Union {
                name,
                possible_types,
                description,
                inaccessible,
                ..
            } => {
                if let Some(description) = description {
                    export_description(sdl, options, true, description);
                }

                write!(sdl, "union {} ", name).ok();
                if options.federation && *inaccessible {
                    write!(sdl, "@inaccessible ").ok();
                }
                write!(sdl, "=").ok();

                for (idx, ty) in possible_types.iter().enumerate() {
                    if idx == 0 {
                        write!(sdl, " {}", ty).ok();
                    } else {
                        write!(sdl, " | {}", ty).ok();
                    }
                }
                writeln!(sdl).ok();
            }
        }
    }

    fn write_implements(&self, sdl: &mut String, name: &str) {
        if let Some(implements) = self.implements.get(name) {
            if !implements.is_empty() {
                write!(
                    sdl,
                    "implements {} ",
                    implements
                        .iter()
                        .map(AsRef::as_ref)
                        .collect::<Vec<&str>>()
                        .join(" & ")
                )
                .ok();
            }
        }
    }
}

fn export_description(
    sdl: &mut String,
    options: &SDLExportOptions,
    top_level: bool,
    description: &str,
) {
    if options.prefer_single_line_descriptions && !description.contains('\n') {
        let tab = if top_level { "" } else { "\t" };
        let description = description.replace('"', r#"\""#);
        writeln!(sdl, "{}\"{}\"", tab, description).ok();
    } else if top_level {
        writeln!(sdl, "\"\"\"\n{}\n\"\"\"", description).ok();
    } else {
        let description = description.replace('\n', "\n\t");
        writeln!(sdl, "\t\"\"\"\n\t{}\n\t\"\"\"", description).ok();
    }
}

fn export_input_value(input_value: &MetaInputValue) -> String {
    if let Some(default_value) = &input_value.default_value {
        format!(
            "{}: {} = {}",
            input_value.name, input_value.ty, default_value
        )
    } else {
        format!("{}: {}", input_value.name, input_value.ty)
    }
}

fn write_deprecated(sdl: &mut String, deprecation: &Deprecation) {
    if let Deprecation::Deprecated { reason } = deprecation {
        let _ = match reason {
            Some(reason) => write!(sdl, " @deprecated(reason: \"{}\")", escape_string(reason)).ok(),
            None => write!(sdl, " @deprecated").ok(),
        };
    }
}

fn escape_string(s: &str) -> String {
    let mut res = String::new();

    for c in s.chars() {
        let ec = match c {
            '\\' => Some("\\\\"),
            '\x08' => Some("\\b"),
            '\x0c' => Some("\\f"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\t' => Some("\\t"),
            _ => None,
        };
        match ec {
            Some(ec) => {
                res.write_str(ec).ok();
            }
            None => {
                res.write_char(c).ok();
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(
            escape_string("1\\\x08d\x0c3\n4\r5\t6"),
            "1\\\\\\bd\\f3\\n4\\r5\\t6"
        );
    }
}
