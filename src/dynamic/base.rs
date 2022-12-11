use crate::dynamic::{Field, InputValue, Interface, InterfaceField, Object, TypeRef};

pub(crate) trait BaseField {
    fn ty(&self) -> &TypeRef;

    fn argument(&self, name: &str) -> Option<&InputValue>;
}

pub(crate) trait BaseContainer {
    type FieldType: BaseField;

    fn name(&self) -> &str;

    fn graphql_type(&self) -> &str;

    fn field(&self, name: &str) -> Option<&Self::FieldType>;
}

impl BaseField for Field {
    #[inline]
    fn ty(&self) -> &TypeRef {
        &self.ty
    }

    #[inline]
    fn argument(&self, name: &str) -> Option<&InputValue> {
        self.arguments.get(name)
    }
}

impl BaseContainer for Object {
    type FieldType = Field;

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn graphql_type(&self) -> &str {
        "Object"
    }

    #[inline]
    fn field(&self, name: &str) -> Option<&Self::FieldType> {
        self.fields.get(name)
    }
}

impl BaseField for InterfaceField {
    #[inline]
    fn ty(&self) -> &TypeRef {
        &self.ty
    }

    #[inline]
    fn argument(&self, name: &str) -> Option<&InputValue> {
        self.arguments.get(name)
    }
}

impl BaseContainer for Interface {
    type FieldType = InterfaceField;

    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn graphql_type(&self) -> &str {
        "Interface"
    }

    #[inline]
    fn field(&self, name: &str) -> Option<&Self::FieldType> {
        self.fields.get(name)
    }
}
