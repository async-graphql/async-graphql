use crate::{
    dynamic::{Enum, InputObject, Interface, Object, Scalar, SchemaError, Subscription, Union},
    registry::Registry,
    Upload,
};

/// A GraphQL type
#[derive(Debug)]
pub enum Type {
    /// Scalar
    Scalar(Scalar),
    /// Object
    Object(Object),
    /// Input object
    InputObject(InputObject),
    /// Enum
    Enum(Enum),
    /// Interface
    Interface(Interface),
    /// Union
    Union(Union),
    /// Subscription
    Subscription(Subscription),
    /// Upload
    Upload,
}

impl Type {
    pub(crate) fn name(&self) -> &str {
        match self {
            Type::Scalar(scalar) => &scalar.name,
            Type::Object(object) => &object.name,
            Type::InputObject(input_object) => &input_object.name,
            Type::Enum(e) => &e.name,
            Type::Interface(interface) => &interface.name,
            Type::Union(union) => &union.name,
            Type::Subscription(subscription) => &subscription.name,
            Type::Upload => "Upload",
        }
    }

    #[inline]
    pub(crate) fn as_object(&self) -> Option<&Object> {
        if let Type::Object(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn as_interface(&self) -> Option<&Interface> {
        if let Type::Interface(interface) = self {
            Some(interface)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn as_input_object(&self) -> Option<&InputObject> {
        if let Type::InputObject(obj) = self {
            Some(obj)
        } else {
            None
        }
    }

    #[inline]
    pub(crate) fn as_subscription(&self) -> Option<&Subscription> {
        if let Type::Subscription(subscription) = self {
            Some(subscription)
        } else {
            None
        }
    }

    pub(crate) fn is_output_type(&self) -> bool {
        match self {
            Type::Scalar(_) => true,
            Type::Object(_) => true,
            Type::InputObject(_) => false,
            Type::Enum(_) => true,
            Type::Interface(_) => true,
            Type::Union(_) => true,
            Type::Subscription(_) => false,
            Type::Upload => false,
        }
    }

    pub(crate) fn is_input_type(&self) -> bool {
        match self {
            Type::Scalar(_) => true,
            Type::Object(_) => false,
            Type::InputObject(_) => true,
            Type::Enum(_) => true,
            Type::Interface(_) => false,
            Type::Union(_) => false,
            Type::Subscription(_) => false,
            Type::Upload => true,
        }
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        if registry.types.contains_key(self.name()) {
            return Err(format!("Type \"{0}\" already exists", self.name()).into());
        }

        match self {
            Type::Scalar(scalar) => scalar.register(registry),
            Type::Object(object) => object.register(registry),
            Type::InputObject(input_object) => input_object.register(registry),
            Type::Enum(e) => e.register(registry),
            Type::Interface(interface) => interface.register(registry),
            Type::Union(union) => union.register(registry),
            Type::Subscription(subscription) => subscription.register(registry),
            Type::Upload => {
                <Upload as crate::InputType>::create_type_info(registry);
                Ok(())
            }
        }
    }
}

impl From<Scalar> for Type {
    #[inline]
    fn from(scalar: Scalar) -> Self {
        Type::Scalar(scalar)
    }
}

impl From<Object> for Type {
    #[inline]
    fn from(obj: Object) -> Self {
        Type::Object(obj)
    }
}

impl From<InputObject> for Type {
    #[inline]
    fn from(obj: InputObject) -> Self {
        Type::InputObject(obj)
    }
}

impl From<Enum> for Type {
    #[inline]
    fn from(e: Enum) -> Self {
        Type::Enum(e)
    }
}

impl From<Interface> for Type {
    #[inline]
    fn from(interface: Interface) -> Self {
        Type::Interface(interface)
    }
}

impl From<Union> for Type {
    #[inline]
    fn from(union: Union) -> Self {
        Type::Union(union)
    }
}

impl From<Subscription> for Type {
    #[inline]
    fn from(subscription: Subscription) -> Self {
        Type::Subscription(subscription)
    }
}
