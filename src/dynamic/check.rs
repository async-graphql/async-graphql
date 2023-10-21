use std::collections::HashSet;

use indexmap::IndexMap;

use crate::dynamic::{
    base::{BaseContainer, BaseField},
    schema::SchemaInner,
    type_ref::TypeRef,
    InputObject, Interface, Object, SchemaError, Type,
};

impl SchemaInner {
    pub(crate) fn check(&self) -> Result<(), SchemaError> {
        self.check_types_exists()?;
        self.check_root_types()?;
        self.check_objects()?;
        self.check_input_objects()?;
        self.check_interfaces()?;
        self.check_unions()?;
        Ok(())
    }

    fn check_root_types(&self) -> Result<(), SchemaError> {
        if let Some(ty) = self.types.get(&self.env.registry.query_type) {
            if !matches!(ty, Type::Object(_)) {
                return Err("The query root must be an object".into());
            }
        }

        if let Some(mutation_type) = &self.env.registry.mutation_type {
            if let Some(ty) = self.types.get(mutation_type) {
                if !matches!(ty, Type::Object(_)) {
                    return Err("The mutation root must be an object".into());
                }
            }
        }

        if let Some(subscription_type) = &self.env.registry.subscription_type {
            if let Some(ty) = self.types.get(subscription_type) {
                if !matches!(ty, Type::Subscription(_)) {
                    return Err("The subsciprtion root must be an subscription object".into());
                }
            }
        }

        Ok(())
    }

    fn check_types_exists(&self) -> Result<(), SchemaError> {
        fn check<I: IntoIterator<Item = T>, T: AsRef<str>>(
            types: &IndexMap<String, Type>,
            type_names: I,
        ) -> Result<(), SchemaError> {
            for name in type_names {
                if !types.contains_key(name.as_ref()) {
                    return Err(format!("Type \"{0}\" not found", name.as_ref()).into());
                }
            }
            Ok(())
        }

        check(
            &self.types,
            std::iter::once(self.env.registry.query_type.as_str())
                .chain(self.env.registry.mutation_type.as_deref()),
        )?;

        for ty in self.types.values() {
            match ty {
                Type::Object(obj) => check(
                    &self.types,
                    obj.fields
                        .values()
                        .map(|field| {
                            std::iter::once(field.ty.type_name())
                                .chain(field.arguments.values().map(|arg| arg.ty.type_name()))
                        })
                        .flatten()
                        .chain(obj.implements.iter().map(AsRef::as_ref)),
                )?,
                Type::InputObject(obj) => {
                    check(
                        &self.types,
                        obj.fields.values().map(|field| field.ty.type_name()),
                    )?;
                }
                Type::Interface(interface) => check(
                    &self.types,
                    interface
                        .fields
                        .values()
                        .map(|field| {
                            std::iter::once(field.ty.type_name())
                                .chain(field.arguments.values().map(|arg| arg.ty.type_name()))
                        })
                        .flatten(),
                )?,
                Type::Union(union) => check(&self.types, &union.possible_types)?,
                Type::Subscription(subscription) => check(
                    &self.types,
                    subscription
                        .fields
                        .values()
                        .map(|field| {
                            std::iter::once(field.ty.type_name())
                                .chain(field.arguments.values().map(|arg| arg.ty.type_name()))
                        })
                        .flatten(),
                )?,
                Type::Scalar(_) | Type::Enum(_) | Type::Upload => {}
            }
        }

        Ok(())
    }

    fn check_objects(&self) -> Result<(), SchemaError> {
        let has_entities = self
            .types
            .iter()
            .filter_map(|(_, ty)| ty.as_object())
            .any(Object::is_entity);

        // https://spec.graphql.org/October2021/#sec-Objects.Type-Validation
        for ty in self.types.values() {
            if let Type::Object(obj) = ty {
                // An Object type must define one or more fields.
                if obj.fields.is_empty()
                    && !(obj.type_name() == self.env.registry.query_type && has_entities)
                {
                    return Err(
                        format!("Object \"{}\" must define one or more fields", obj.name).into(),
                    );
                }

                for field in obj.fields.values() {
                    // The field must not have a name which begins with the characters "__" (two
                    // underscores)
                    if field.name.starts_with("__") {
                        return Err(format!("Field \"{}.{}\" must not have a name which begins with the characters \"__\" (two underscores)", obj.name, field.name).into());
                    }

                    // The field must return a type where IsOutputType(fieldType) returns true.
                    if let Some(ty) = self.types.get(field.ty.type_name()) {
                        if !ty.is_output_type() {
                            return Err(format!(
                                "Field \"{}.{}\" must return a output type",
                                obj.name, field.name
                            )
                            .into());
                        }
                    }

                    for arg in field.arguments.values() {
                        // The argument must not have a name which begins with the characters "__"
                        // (two underscores).
                        if arg.name.starts_with("__") {
                            return Err(format!("Argument \"{}.{}.{}\" must not have a name which begins with the characters \"__\" (two underscores)", obj.name, field.name, arg.name).into());
                        }

                        // The argument must accept a type where
                        // IsInputType(argumentType) returns true.
                        if let Some(ty) = self.types.get(arg.ty.type_name()) {
                            if !ty.is_input_type() {
                                return Err(format!(
                                    "Argument \"{}.{}.{}\" must accept a input type",
                                    obj.name, field.name, arg.name
                                )
                                .into());
                            }
                        }
                    }
                }

                for interface_name in &obj.implements {
                    if let Some(ty) = self.types.get(interface_name) {
                        let interface = ty.as_interface().ok_or_else(|| {
                            format!("Type \"{}\" is not interface", interface_name)
                        })?;
                        check_is_valid_implementation(obj, interface)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn check_input_objects(&self) -> Result<(), SchemaError> {
        // https://spec.graphql.org/October2021/#sec-Input-Objects.Type-Validation
        for ty in self.types.values() {
            if let Type::InputObject(obj) = ty {
                for field in obj.fields.values() {
                    // The field must not have a name which begins with the characters "__" (two
                    // underscores)
                    if field.name.starts_with("__") {
                        return Err(format!("Field \"{}.{}\" must not have a name which begins with the characters \"__\" (two underscores)", obj.name, field.name).into());
                    }

                    // The input field must accept a type where IsInputType(inputFieldType) returns
                    // true.
                    if let Some(ty) = self.types.get(field.ty.type_name()) {
                        if !ty.is_input_type() {
                            return Err(format!(
                                "Field \"{}.{}\" must accept a input type",
                                obj.name, field.name
                            )
                            .into());
                        }
                    }

                    if obj.oneof {
                        // The type of the input field must be nullable.
                        if !field.ty.is_nullable() {
                            return Err(format!(
                                "Field \"{}.{}\" must be nullable",
                                obj.name, field.name
                            )
                            .into());
                        }

                        // The input field must not have a default value.
                        if field.default_value.is_some() {
                            return Err(format!(
                                "Field \"{}.{}\" must not have a default value",
                                obj.name, field.name
                            )
                            .into());
                        }
                    }
                }

                // If an Input Object references itself either directly or
                // through referenced Input Objects, at least one of the
                // fields in the chain of references must be either a
                // nullable or a List type.
                self.check_input_object_reference(&obj.name, &obj, &mut HashSet::new())?;
            }
        }

        Ok(())
    }

    fn check_input_object_reference<'a>(
        &'a self,
        current: &str,
        obj: &'a InputObject,
        ref_chain: &mut HashSet<&'a str>,
    ) -> Result<(), SchemaError> {
        fn typeref_nonnullable_name(ty: &TypeRef) -> Option<&str> {
            match ty {
                TypeRef::NonNull(inner) => match inner.as_ref() {
                    TypeRef::Named(name) => Some(name),
                    _ => None,
                },
                _ => None,
            }
        }

        for field in obj.fields.values() {
            if let Some(this_name) = typeref_nonnullable_name(&field.ty) {
                if this_name == current {
                    return Err(format!("\"{}\" references itself either directly or through referenced Input Objects, at least one of the fields in the chain of references must be either a nullable or a List type.", current).into());
                } else if let Some(obj) = self
                    .types
                    .get(field.ty.type_name())
                    .and_then(Type::as_input_object)
                {
                    // don't visit the reference if we've already visited it in this call chain
                    //  (prevents getting stuck in local cycles and overflowing stack)
                    //  true return from insert indicates the value was not previously there
                    if ref_chain.insert(this_name) {
                        self.check_input_object_reference(current, obj, ref_chain)?;
                        ref_chain.remove(this_name);
                    }
                }
            }
        }

        Ok(())
    }

    fn check_interfaces(&self) -> Result<(), SchemaError> {
        // https://spec.graphql.org/October2021/#sec-Interfaces.Type-Validation
        for ty in self.types.values() {
            if let Type::Interface(interface) = ty {
                for field in interface.fields.values() {
                    // The field must not have a name which begins with the characters "__" (two
                    // underscores)
                    if field.name.starts_with("__") {
                        return Err(format!("Field \"{}.{}\" must not have a name which begins with the characters \"__\" (two underscores)", interface.name, field.name).into());
                    }

                    // The field must return a type where IsOutputType(fieldType) returns true.
                    if let Some(ty) = self.types.get(field.ty.type_name()) {
                        if !ty.is_output_type() {
                            return Err(format!(
                                "Field \"{}.{}\" must return a output type",
                                interface.name, field.name
                            )
                            .into());
                        }
                    }

                    for arg in field.arguments.values() {
                        // The argument must not have a name which begins with the characters "__"
                        // (two underscores).
                        if arg.name.starts_with("__") {
                            return Err(format!("Argument \"{}.{}.{}\" must not have a name which begins with the characters \"__\" (two underscores)", interface.name, field.name, arg.name).into());
                        }

                        // The argument must accept a type where
                        // IsInputType(argumentType) returns true.
                        if let Some(ty) = self.types.get(arg.ty.type_name()) {
                            if !ty.is_input_type() {
                                return Err(format!(
                                    "Argument \"{}.{}.{}\" must accept a input type",
                                    interface.name, field.name, arg.name
                                )
                                .into());
                            }
                        }
                    }

                    // An interface type may declare that it implements one or more unique
                    // interfaces, but may not implement itself.
                    if interface.implements.contains(&interface.name) {
                        return Err(format!(
                            "Interface \"{}\" may not implement itself",
                            interface.name
                        )
                        .into());
                    }

                    // An interface type must be a super-set of all interfaces
                    // it implements
                    for interface_name in &interface.implements {
                        if let Some(ty) = self.types.get(interface_name) {
                            let implemenented_type = ty.as_interface().ok_or_else(|| {
                                format!("Type \"{}\" is not interface", interface_name)
                            })?;
                            check_is_valid_implementation(interface, implemenented_type)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn check_unions(&self) -> Result<(), SchemaError> {
        // https://spec.graphql.org/October2021/#sec-Unions.Type-Validation
        for ty in self.types.values() {
            if let Type::Union(union) = ty {
                // The member types of a Union type must all be Object base
                // types; Scalar, Interface and Union types must not be member
                // types of a Union. Similarly, wrapping types must not be
                // member types of a Union.
                for type_name in &union.possible_types {
                    if let Some(ty) = self.types.get(type_name) {
                        if ty.as_object().is_none() {
                            return Err(format!(
                                "Member \"{}\" of union \"{}\" is not an object",
                                type_name, union.name
                            )
                            .into());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn check_is_valid_implementation(
    implementing_type: &impl BaseContainer,
    implemented_type: &Interface,
) -> Result<(), SchemaError> {
    for field in implemented_type.fields.values() {
        let impl_field = implementing_type.field(&field.name).ok_or_else(|| {
            format!(
                "{} \"{}\" requires field \"{}\" defined by interface \"{}\"",
                implementing_type.graphql_type(),
                implementing_type.name(),
                field.name,
                implemented_type.name
            )
        })?;

        for arg in field.arguments.values() {
            let impl_arg = match impl_field.argument(&arg.name) {
                Some(impl_arg) => impl_arg,
                None if !arg.ty.is_nullable() => {
                    return Err(format!(
                        "Field \"{}.{}\" requires argument \"{}\" defined by interface \"{}.{}\"",
                        implementing_type.name(),
                        field.name,
                        arg.name,
                        implemented_type.name,
                        field.name,
                    )
                    .into());
                }
                None => continue,
            };

            if !arg.ty.is_subtype(&impl_arg.ty) {
                return Err(format!(
                    "Argument \"{}.{}.{}\" is not sub-type of \"{}.{}.{}\"",
                    implemented_type.name,
                    field.name,
                    arg.name,
                    implementing_type.name(),
                    field.name,
                    arg.name
                )
                .into());
            }
        }

        // field must return a type which is equal to or a sub-type of (covariant) the
        // return type of implementedField field’s return type
        if !impl_field.ty().is_subtype(&field.ty) {
            return Err(format!(
                "Field \"{}.{}\" is not sub-type of \"{}.{}\"",
                implementing_type.name(),
                field.name,
                implemented_type.name,
                field.name,
            )
            .into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        dynamic::{
            Field, FieldFuture, InputObject, InputValue, Object, Schema, SchemaBuilder, TypeRef,
        },
        Value,
    };

    fn base_schema() -> SchemaBuilder {
        let query = Object::new("Query").field(Field::new("dummy", TypeRef::named("Int"), |_| {
            FieldFuture::new(async { Ok(Some(Value::from(42))) })
        }));
        Schema::build("Query", None, None).register(query)
    }

    #[test]
    fn test_recursive_input_objects() {
        let top_level = InputObject::new("TopLevel")
            .field(InputValue::new("mid", TypeRef::named_nn("MidLevel")));
        let mid_level = InputObject::new("MidLevel")
            .field(InputValue::new("bottom", TypeRef::named("BotLevel")))
            .field(InputValue::new(
                "list_bottom",
                TypeRef::named_nn_list_nn("BotLevel"),
            ));
        let bot_level = InputObject::new("BotLevel")
            .field(InputValue::new("top", TypeRef::named_nn("TopLevel")));
        let schema = base_schema()
            .register(top_level)
            .register(mid_level)
            .register(bot_level);
        schema.finish().unwrap();
    }

    #[test]
    fn test_recursive_input_objects_bad() {
        let top_level = InputObject::new("TopLevel")
            .field(InputValue::new("mid", TypeRef::named_nn("MidLevel")));
        let mid_level = InputObject::new("MidLevel")
            .field(InputValue::new("bottom", TypeRef::named_nn("BotLevel")));
        let bot_level = InputObject::new("BotLevel")
            .field(InputValue::new("top", TypeRef::named_nn("TopLevel")));
        let schema = base_schema()
            .register(top_level)
            .register(mid_level)
            .register(bot_level);
        schema.finish().unwrap_err();
    }

    #[test]
    fn test_recursive_input_objects_local_cycle() {
        let top_level = InputObject::new("TopLevel")
            .field(InputValue::new("mid", TypeRef::named_nn("MidLevel")));
        let mid_level = InputObject::new("MidLevel")
            .field(InputValue::new("bottom", TypeRef::named_nn("BotLevel")));
        let bot_level = InputObject::new("BotLevel")
            .field(InputValue::new("mid", TypeRef::named_nn("MidLevel")));
        let schema = base_schema()
            .register(top_level)
            .register(mid_level)
            .register(bot_level);
        schema.finish().unwrap_err();
    }
}
