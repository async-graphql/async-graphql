mod cache_control;
mod export_sdl;
mod stringify_exec_doc;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::{self, Display, Formatter, Write},
    sync::Arc,
};

pub use cache_control::CacheControl;
pub use export_sdl::SDLExportOptions;
use indexmap::{map::IndexMap, set::IndexSet};

pub use crate::model::{__DirectiveLocation, location_traits};
use crate::{
    Any, Context, ID, InputType, OutputTypeMarker, Positioned, ServerResult, SubscriptionType, Value, VisitorContext, model::__Schema, parser::types::{BaseType as ParsedBaseType, Field, Type as ParsedType, VariableDefinition}, schema::IntrospectionMode
};

fn strip_brackets(type_name: &str) -> Option<&str> {
    type_name
        .strip_prefix('[')
        .map(|rest| &rest[..rest.len() - 1])
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum MetaTypeName<'a> {
    List(&'a str),
    NonNull(&'a str),
    Named(&'a str),
}

impl Display for MetaTypeName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MetaTypeName::Named(name) => write!(f, "{}", name),
            MetaTypeName::NonNull(name) => write!(f, "{}!", name),
            MetaTypeName::List(name) => write!(f, "[{}]", name),
        }
    }
}

impl MetaTypeName<'_> {
    #[inline]
    pub fn create(type_name: &str) -> MetaTypeName {
        if let Some(type_name) = type_name.strip_suffix('!') {
            MetaTypeName::NonNull(type_name)
        } else if let Some(type_name) = strip_brackets(type_name) {
            MetaTypeName::List(type_name)
        } else {
            MetaTypeName::Named(type_name)
        }
    }

    #[inline]
    pub fn concrete_typename(type_name: &str) -> &str {
        match MetaTypeName::create(type_name) {
            MetaTypeName::List(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::NonNull(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::Named(type_name) => type_name,
        }
    }

    #[inline]
    pub fn is_non_null(&self) -> bool {
        matches!(self, MetaTypeName::NonNull(_))
    }

    #[inline]
    #[must_use]
    pub fn unwrap_non_null(&self) -> Self {
        match self {
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty),
            _ => *self,
        }
    }

    #[inline]
    pub fn is_subtype(&self, sub: &MetaTypeName<'_>) -> bool {
        match (self, sub) {
            (MetaTypeName::NonNull(super_type), MetaTypeName::NonNull(sub_type))
            | (MetaTypeName::Named(super_type), MetaTypeName::NonNull(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => {
                super_type == sub_type
            }
            (MetaTypeName::List(super_type), MetaTypeName::List(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            _ => false,
        }
    }

    #[inline]
    pub fn is_list(&self) -> bool {
        match self {
            MetaTypeName::List(_) => true,
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty).is_list(),
            MetaTypeName::Named(name) => name.ends_with(']'),
        }
    }
}

/// actual directive invocation on SDL definitions
#[derive(Debug, Clone)]
pub struct MetaDirectiveInvocation {
    /// name of directive to invoke
    pub name: String,
    /// actual arguments passed to directive
    pub args: IndexMap<String, Value>,
}

impl MetaDirectiveInvocation {
    pub fn sdl(&self) -> String {
        let formatted_args = if self.args.is_empty() {
            String::new()
        } else {
            format!(
                "({})",
                self.args
                    .iter()
                    .map(|(name, value)| format!("{}: {}", name, value))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!("@{}{}", self.name, formatted_args)
    }
}

/// Input value metadata
#[derive(Clone)]
pub struct MetaInputValue {
    /// The name of the input value
    pub name: String,
    /// The description of the input value
    pub description: Option<String>,
    /// The type of the input value
    pub ty: String,
    /// Field deprecation
    pub deprecation: Deprecation,
    /// The default value of the input value
    pub default_value: Option<String>,
    /// A function that uses to check if the input value should be exported to
    /// schemas
    pub visible: Option<MetaVisibleFn>,
    /// Indicate that an input object is not accessible from a supergraph when
    /// using Apollo Federation
    pub inaccessible: bool,
    /// Arbitrary string metadata that will be propagated to the supergraph when
    /// using Apollo Federation. This attribute is repeatable
    pub tags: Vec<String>,
    /// Indicate that an input object is secret
    pub is_secret: bool,
    /// Custom directive invocations
    pub directive_invocations: Vec<MetaDirectiveInvocation>,
}

type ComputeComplexityFn = fn(
    &VisitorContext<'_>,
    &[Positioned<VariableDefinition>],
    &Field,
    usize,
) -> ServerResult<usize>;

#[derive(Debug, Clone, Default)]
pub enum Deprecation {
    #[default]
    NoDeprecated,
    Deprecated {
        reason: Option<String>,
    },
}

impl Deprecation {
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        matches!(self, Deprecation::Deprecated { .. })
    }

    #[inline]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Deprecation::NoDeprecated => None,
            Deprecation::Deprecated { reason } => reason.as_deref(),
        }
    }
}

/// Field metadata
#[derive(Clone)]
pub struct MetaField {
    /// The name of the field
    pub name: String,
    /// The description of the field
    pub description: Option<String>,
    /// The arguments of the field
    pub args: IndexMap<String, MetaInputValue>,
    /// The type of the field
    pub ty: String,
    /// Field deprecation
    pub deprecation: Deprecation,
    /// Used to create HTTP `Cache-Control` header
    pub cache_control: CacheControl,
    /// Mark a field as owned by another service. This allows service A to use
    /// fields from service B while also knowing at runtime the types of that
    /// field.
    pub external: bool,
    /// Annotate the required input fieldset from a base type for a resolver. It
    /// is used to develop a query plan where the required fields may not be
    /// needed by the client, but the service may need additional information
    /// from other services.
    pub requires: Option<String>,
    /// Annotate the expected returned fieldset from a field on a base type that
    /// is guaranteed to be selectable by the gateway.
    pub provides: Option<String>,
    /// A function that uses to check if the field should be exported to
    /// schemas
    pub visible: Option<MetaVisibleFn>,
    /// Indicate that an object type's field is allowed to be resolved by
    /// multiple subgraphs
    pub shareable: bool,
    /// Indicate that an object is not accessible from a supergraph when using
    /// Apollo Federation
    pub inaccessible: bool,
    /// Arbitrary string metadata that will be propagated to the supergraph when
    /// using Apollo Federation. This attribute is repeatable
    pub tags: Vec<String>,
    /// Mark the field as overriding a field currently present on another
    /// subgraph. It is used to migrate fields between subgraphs.
    pub override_from: Option<String>,
    /// A constant or function to get the complexity
    pub compute_complexity: Option<ComputeComplexityFn>,
    /// Custom directive invocations
    pub directive_invocations: Vec<MetaDirectiveInvocation>,
    /// Indicates to composition that the target element is accessible only to
    /// the authenticated supergraph users with the appropriate JWT scopes
    /// when using Apollo Federation.
    pub requires_scopes: Vec<String>,
}

#[derive(Clone)]
pub struct MetaEnumValue {
    pub name: String,
    pub description: Option<String>,
    pub deprecation: Deprecation,
    pub visible: Option<MetaVisibleFn>,
    pub inaccessible: bool,
    pub tags: Vec<String>,
    pub directive_invocations: Vec<MetaDirectiveInvocation>,
}

type MetaVisibleFn = fn(&Context<'_>) -> bool;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MetaTypeId {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
}

impl MetaTypeId {
    fn create_fake_type(&self, rust_typename: &'static str) -> MetaType {
        match self {
            MetaTypeId::Scalar => MetaType::Scalar {
                name: "".to_string(),
                description: None,
                is_valid: None,
                visible: None,
                inaccessible: false,
                tags: vec![],
                specified_by_url: None,
                directive_invocations: vec![],
                requires_scopes: vec![],
            },
            MetaTypeId::Object => MetaType::Object {
                name: "".to_string(),
                description: None,
                fields: Default::default(),
                cache_control: Default::default(),
                extends: false,
                shareable: false,
                resolvable: true,
                inaccessible: false,
                interface_object: false,
                tags: vec![],
                keys: None,
                visible: None,
                is_subscription: false,
                rust_typename: Some(rust_typename),
                directive_invocations: vec![],
                requires_scopes: vec![],
            },
            MetaTypeId::Interface => MetaType::Interface {
                name: "".to_string(),
                description: None,
                fields: Default::default(),
                possible_types: Default::default(),
                extends: false,
                inaccessible: false,
                tags: vec![],
                keys: None,
                visible: None,
                rust_typename: Some(rust_typename),
                directive_invocations: vec![],
                requires_scopes: vec![],
            },
            MetaTypeId::Union => MetaType::Union {
                name: "".to_string(),
                description: None,
                possible_types: Default::default(),
                visible: None,
                inaccessible: false,
                tags: vec![],
                rust_typename: Some(rust_typename),
                directive_invocations: vec![],
            },
            MetaTypeId::Enum => MetaType::Enum {
                name: "".to_string(),
                description: None,
                enum_values: Default::default(),
                visible: None,
                inaccessible: false,
                tags: vec![],
                rust_typename: Some(rust_typename),
                directive_invocations: vec![],
                requires_scopes: vec![],
            },
            MetaTypeId::InputObject => MetaType::InputObject {
                name: "".to_string(),
                description: None,
                input_fields: Default::default(),
                visible: None,
                inaccessible: false,
                tags: vec![],
                rust_typename: Some(rust_typename),
                oneof: false,
                directive_invocations: vec![],
            },
        }
    }
}

impl Display for MetaTypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MetaTypeId::Scalar => "Scalar",
            MetaTypeId::Object => "Object",
            MetaTypeId::Interface => "Interface",
            MetaTypeId::Union => "Union",
            MetaTypeId::Enum => "Enum",
            MetaTypeId::InputObject => "InputObject",
        })
    }
}

/// A validator for scalar
pub type ScalarValidatorFn = Arc<dyn Fn(&Value) -> bool + Send + Sync>;

/// Type metadata
#[derive(Clone)]
pub enum MetaType {
    /// Scalar
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Scalars>
    Scalar {
        /// The name of the scalar
        name: String,
        /// the description of the scalar
        description: Option<String>,
        /// A function that uses to check if the scalar is valid
        is_valid: Option<ScalarValidatorFn>,
        /// A function that uses to check if the scalar should be exported to
        /// schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that a scalar is not accessible from a supergraph when
        /// using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// Provide a specification URL for this scalar type, it must link to a
        /// human-readable specification of the data format, serialization and
        /// coercion rules for this scalar.
        specified_by_url: Option<String>,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
        /// Indicates to composition that the target element is accessible only
        /// to the authenticated supergraph users with the appropriate
        /// JWT scopes when using Apollo Federation.
        requires_scopes: Vec<String>,
    },
    /// Object
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Objects>
    Object {
        /// The name of the object
        name: String,
        /// The description of the object
        description: Option<String>,
        /// The fields of the object type
        fields: IndexMap<String, MetaField>,
        /// Used to create HTTP `Cache-Control` header
        cache_control: CacheControl,
        /// Indicates that an object definition is an extension of another
        /// definition of that same type.
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#extends>
        extends: bool,
        /// Indicates that an object type's field is allowed to be resolved by
        /// multiple subgraphs (by default in Federation 2, object fields can be
        /// resolved by only one subgraph).
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#shareable>
        shareable: bool,
        /// Indicates that the subgraph does not define a reference resolver
        /// for this object. Objects are assumed to be resolvable by default.
        ///
        /// Most commonly used to reference an entity defined in another
        /// subgraph without contributing fields. Part of the `@key` directive.
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#key>
        resolvable: bool,
        /// The keys of the object type
        ///
        /// Designates an object type as an [entity](https://www.apollographql.com/docs/federation/entities) and specifies
        /// its key fields (a set of fields that the subgraph can use to
        /// uniquely identify any instance of the entity).
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#key>
        keys: Option<Vec<String>>,
        /// A function that uses to check if the object should be exported to
        /// schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that an object is not accessible from a supergraph when
        /// using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// During composition, the fields of every `@interfaceObject` are added
        /// both to their corresponding interface definition and to all
        /// entity types that implement that interface.
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#interfaceobject>
        interface_object: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// Indicates whether it is a subscription object
        is_subscription: bool,
        /// The Rust typename corresponding to the object
        rust_typename: Option<&'static str>,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
        /// Indicates to composition that the target element is accessible only
        /// to the authenticated supergraph users with the appropriate
        /// JWT scopes when using Apollo Federation.
        requires_scopes: Vec<String>,
    },
    /// Interface
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Interfaces>
    Interface {
        /// The name of the interface
        name: String,
        /// The description of the interface
        description: Option<String>,
        /// The fields of the interface
        fields: IndexMap<String, MetaField>,
        /// The object types that implement this interface
        /// Add fields to an entity that's defined in another service
        possible_types: IndexSet<String>,
        /// Indicates that an interface definition is an extension of another
        /// definition of that same type.
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#extends>
        extends: bool,
        /// The keys of the object type
        ///
        /// Designates an object type as an [entity](https://www.apollographql.com/docs/federation/entities) and specifies
        /// its key fields (a set of fields that the subgraph can use to
        /// uniquely identify any instance of the entity).
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#key>
        keys: Option<Vec<String>>,
        /// A function that uses to check if the interface should be exported to
        /// schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that an interface is not accessible from a supergraph when
        /// using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// The Rust typename corresponding to the interface
        rust_typename: Option<&'static str>,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
        /// Indicates to composition that the target element is accessible only
        /// to the authenticated supergraph users with the appropriate
        /// JWT scopes when using Apollo Federation.
        requires_scopes: Vec<String>,
    },
    /// Union
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Unions>
    Union {
        /// The name of the interface
        name: String,
        /// The description of the union
        description: Option<String>,
        /// The object types that could be the union
        possible_types: IndexSet<String>,
        /// A function that uses to check if the union should be exported to
        /// schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that an union is not accessible from a supergraph when
        /// using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// The Rust typename corresponding to the union
        rust_typename: Option<&'static str>,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
    },
    /// Enum
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Enums>
    Enum {
        /// The name of the enum
        name: String,
        /// The description of the enum
        description: Option<String>,
        /// The values of the enum
        enum_values: IndexMap<String, MetaEnumValue>,
        /// A function that uses to check if the enum should be exported to
        /// schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that an enum is not accessible from a supergraph when
        /// using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// The Rust typename corresponding to the enum
        rust_typename: Option<&'static str>,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
        /// Indicates to composition that the target element is accessible only
        /// to the authenticated supergraph users with the appropriate
        /// JWT scopes when using Apollo Federation.
        requires_scopes: Vec<String>,
    },
    /// Input object
    ///
    /// Reference: <https://spec.graphql.org/October2021/#sec-Input-Objects>
    InputObject {
        /// The name of the input object
        name: String,
        /// The description of the input object
        description: Option<String>,
        /// The fields of the input object
        input_fields: IndexMap<String, MetaInputValue>,
        /// A function that uses to check if the input object should be exported
        /// to schemas
        visible: Option<MetaVisibleFn>,
        /// Indicate that a input object is not accessible from a supergraph
        /// when using Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        inaccessible: bool,
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        tags: Vec<String>,
        /// The Rust typename corresponding to the enum
        rust_typename: Option<&'static str>,
        /// Is the oneof input objects
        ///
        /// Reference: <https://github.com/graphql/graphql-spec/pull/825>
        oneof: bool,
        /// custom directive invocations
        directive_invocations: Vec<MetaDirectiveInvocation>,
    },
}

impl MetaType {
    #[inline]
    pub fn type_id(&self) -> MetaTypeId {
        match self {
            MetaType::Scalar { .. } => MetaTypeId::Scalar,
            MetaType::Object { .. } => MetaTypeId::Object,
            MetaType::Interface { .. } => MetaTypeId::Interface,
            MetaType::Union { .. } => MetaTypeId::Union,
            MetaType::Enum { .. } => MetaTypeId::Enum,
            MetaType::InputObject { .. } => MetaTypeId::InputObject,
        }
    }

    #[inline]
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields().and_then(|fields| fields.get(name))
    }

    #[inline]
    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object { fields, .. } => Some(&fields),
            MetaType::Interface { fields, .. } => Some(&fields),
            _ => None,
        }
    }

    #[inline]
    pub fn is_visible(&self, ctx: &Context<'_>) -> bool {
        let visible = match self {
            MetaType::Scalar { visible, .. } => visible,
            MetaType::Object { visible, .. } => visible,
            MetaType::Interface { visible, .. } => visible,
            MetaType::Union { visible, .. } => visible,
            MetaType::Enum { visible, .. } => visible,
            MetaType::InputObject { visible, .. } => visible,
        };
        is_visible(ctx, visible)
    }

    #[inline]
    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar { name, .. } => &name,
            MetaType::Object { name, .. } => name,
            MetaType::Interface { name, .. } => name,
            MetaType::Union { name, .. } => name,
            MetaType::Enum { name, .. } => name,
            MetaType::InputObject { name, .. } => name,
        }
    }

    #[inline]
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            MetaType::Object { .. } | MetaType::Interface { .. } | MetaType::Union { .. }
        )
    }

    #[inline]
    pub fn is_abstract(&self) -> bool {
        matches!(self, MetaType::Interface { .. } | MetaType::Union { .. })
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        matches!(self, MetaType::Enum { .. } | MetaType::Scalar { .. })
    }

    #[inline]
    pub fn is_input(&self) -> bool {
        matches!(
            self,
            MetaType::Enum { .. } | MetaType::Scalar { .. } | MetaType::InputObject { .. }
        )
    }

    #[inline]
    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Union { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Object { name, .. } => name == type_name,
            _ => false,
        }
    }

    #[inline]
    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface { possible_types, .. } => Some(possible_types),
            MetaType::Union { possible_types, .. } => Some(possible_types),
            _ => None,
        }
    }

    pub fn type_overlap(&self, ty: &MetaType) -> bool {
        if std::ptr::eq(self, ty) {
            return true;
        }

        match (self.is_abstract(), ty.is_abstract()) {
            (true, true) => self
                .possible_types()
                .iter()
                .copied()
                .flatten()
                .any(|type_name| ty.is_possible_type(type_name)),
            (true, false) => self.is_possible_type(ty.name()),
            (false, true) => ty.is_possible_type(self.name()),
            (false, false) => false,
        }
    }

    pub fn rust_typename(&self) -> Option<&'static str> {
        match self {
            MetaType::Scalar { .. } => None,
            MetaType::Object { rust_typename, .. } => *rust_typename,
            MetaType::Interface { rust_typename, .. } => *rust_typename,
            MetaType::Union { rust_typename, .. } => *rust_typename,
            MetaType::Enum { rust_typename, .. } => *rust_typename,
            MetaType::InputObject { rust_typename, .. } => *rust_typename,
        }
    }
}

pub struct MetaDirective {
    pub name: String,
    pub description: Option<String>,
    pub locations: Vec<__DirectiveLocation>,
    pub args: IndexMap<String, MetaInputValue>,
    pub is_repeatable: bool,
    pub visible: Option<MetaVisibleFn>,
    pub composable: Option<String>,
}

impl MetaDirective {
    pub(crate) fn sdl(&self, options: &SDLExportOptions) -> String {
        let mut sdl = String::new();

        if let Some(description) = &self.description {
            self::export_sdl::write_description(&mut sdl, options, 0, description);
        }

        write!(sdl, "directive @{}", self.name).ok();

        if !self.args.is_empty() {
            let args = self
                .args
                .values()
                .map(|value| self.argument_sdl(value))
                .collect::<Vec<_>>()
                .join(", ");
            write!(sdl, "({})", args).ok();
        }
        let locations = self
            .locations
            .iter()
            .map(|location| location.to_value().to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        write!(sdl, " on {}", locations).ok();
        sdl
    }

    pub(crate) fn argument_sdl(&self, argument: &MetaInputValue) -> String {
        let argument_default = match &argument.default_value {
            Some(default) => format!(" = {default}"),
            None => "".to_string(),
        };

        format!("{}: {}{}", argument.name, argument.ty, argument_default)
    }
}

/// A type registry for build schemas
#[derive(Default)]
pub struct Registry {
    pub types: BTreeMap<String, MetaType>,
    pub directives: BTreeMap<String, MetaDirective>,
    pub implements: HashMap<String, IndexSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
    pub introspection_mode: IntrospectionMode,
    pub enable_federation: bool,
    pub federation_subscription: bool,
    pub ignore_name_conflicts: HashSet<String>,
    pub enable_suggestions: bool,
}

impl Registry {
    pub(crate) fn add_system_types(&mut self) {
        self.add_directive(MetaDirective {
            name: "skip".into(),
            description: Some("Directs the executor to skip this field or fragment when the `if` argument is true.".to_string()),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert("if".to_string(), MetaInputValue {
                    name: "if".to_string(),
                    description: Some("Skipped when true.".to_string()),
                    ty: "Boolean!".to_string(),
                    deprecation: Deprecation::NoDeprecated,
                    default_value: None,
                    visible: None,
                    inaccessible: false,
                    tags: Default::default(),
                    is_secret: false,
                    directive_invocations: vec![]
                });
                args
            },
            is_repeatable: false,
            visible: None,
            composable: None,
        });

        self.add_directive(MetaDirective {
            name: "include".into(),
            description: Some("Directs the executor to include this field or fragment only when the `if` argument is true.".to_string()),
            locations: vec![
                __DirectiveLocation::FIELD,
                __DirectiveLocation::FRAGMENT_SPREAD,
                __DirectiveLocation::INLINE_FRAGMENT
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert("if".to_string(), MetaInputValue {
                    name: "if".to_string(),
                    description: Some("Included when true.".to_string()),
                    ty: "Boolean!".to_string(),
                    deprecation: Deprecation::NoDeprecated,
                    default_value: None,
                    visible: None,
                    inaccessible: false,
                    tags: Default::default(),
                    is_secret: false,
                    directive_invocations: vec![]
                });
                args
            },
            is_repeatable: false,
            visible: None,
            composable: None,
        });

        self.add_directive(MetaDirective {
            name: "deprecated".into(),
            description: Some(
                "Marks an element of a GraphQL schema as no longer supported.".into(),
            ),
            locations: vec![
                __DirectiveLocation::FIELD_DEFINITION,
                __DirectiveLocation::ARGUMENT_DEFINITION,
                __DirectiveLocation::INPUT_FIELD_DEFINITION,
                __DirectiveLocation::ENUM_VALUE,
            ],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "reason".into(),
                    MetaInputValue {
                        name: "reason".into(),
                        description: Some(
                            "A reason for why it is deprecated, formatted using Markdown syntax"
                                .into(),
                        ),
                        ty: "String".into(),
                        deprecation: Deprecation::NoDeprecated,
                        default_value: Some(r#""No longer supported""#.into()),
                        visible: None,
                        inaccessible: false,
                        tags: Default::default(),
                        is_secret: false,
                        directive_invocations: vec![],
                    },
                );
                args
            },
            is_repeatable: false,
            visible: None,
            composable: None,
        });

        self.add_directive(MetaDirective {
            name: "specifiedBy".into(),
            description: Some("Provides a scalar specification URL for specifying the behavior of custom scalar types.".into()),
            locations: vec![__DirectiveLocation::SCALAR],
            args: {
                let mut args = IndexMap::new();
                args.insert(
                    "url".into(),
                    MetaInputValue {
                        name: "url".into(),
                        description: Some("URL that specifies the behavior of this scalar.".into()),
                        ty: "String!".into(),
                        deprecation: Deprecation::NoDeprecated,
                        default_value: None,
                        visible: None,
                        inaccessible: false,
                        tags: Default::default(),
                        is_secret: false,
                        directive_invocations: vec![],
                    },
                );
                args
            },
            is_repeatable: false,
            visible: None,
            composable: None,
        });

        self.add_directive(MetaDirective {
            name: "oneOf".into(),
            description: Some(
                "Indicates that an Input Object is a OneOf Input Object (and thus requires \
                exactly one of its field be provided)"
                    .to_string(),
            ),
            locations: vec![__DirectiveLocation::INPUT_OBJECT],
            args: Default::default(),
            is_repeatable: false,
            visible: None,
            composable: None,
        });

        // create system scalars
        <bool as InputType>::create_type_info(self);
        <i32 as InputType>::create_type_info(self);
        <f32 as InputType>::create_type_info(self);
        <String as InputType>::create_type_info(self);
        <ID as InputType>::create_type_info(self);
    }

    pub fn create_input_type<T, F>(&mut self, type_id: MetaTypeId, mut f: F) -> String
    where
        T: InputType,
        F: FnMut(&mut Registry) -> MetaType,
    {
        self.create_type(&mut f, &T::type_name(), std::any::type_name::<T>(), type_id);
        T::qualified_type_name()
    }

    pub fn create_output_type<T, F>(&mut self, type_id: MetaTypeId, mut f: F) -> String
    where
        T: OutputTypeMarker + ?Sized,
        F: FnMut(&mut Registry) -> MetaType,
    {
        self.create_type(&mut f, &<T as OutputTypeMarker>::type_name(), std::any::type_name::<T>(), type_id);
        <T as OutputTypeMarker>::qualified_type_name()
    }

    pub fn create_subscription_type<T, F>(&mut self, mut f: F) -> String
    where
        T: SubscriptionType + ?Sized,
        F: FnMut(&mut Registry) -> MetaType,
    {
        self.create_type(
            &mut f,
            &T::type_name(),
            std::any::type_name::<T>(),
            MetaTypeId::Object,
        );
        T::qualified_type_name()
    }

    fn create_type(
        &mut self,
        f: &mut dyn FnMut(&mut Registry) -> MetaType,
        name: &str,
        rust_typename: &'static str,
        type_id: MetaTypeId,
    ) {
        match self.types.get(name) {
            Some(ty) => {
                if let Some(prev_typename) = ty.rust_typename() {
                    if prev_typename == "__fake_type__" {
                        return;
                    }

                    if rust_typename != prev_typename && !self.ignore_name_conflicts.contains(name)
                    {
                        panic!(
                            "`{}` and `{}` have the same GraphQL name `{}`",
                            prev_typename, rust_typename, name,
                        );
                    }

                    if ty.type_id() != type_id {
                        panic!(
                            "Register `{}` as `{}`, but it is already registered as `{}`",
                            name,
                            type_id,
                            ty.type_id()
                        );
                    }
                }
            }
            None => {
                // Inserting a fake type before calling the function allows recursive types to
                // exist.
                self.types
                    .insert(name.to_string(), type_id.create_fake_type(rust_typename));
                let ty = f(self);
                *self.types.get_mut(name).unwrap() = ty;
            }
        }
    }

    pub fn create_fake_output_type<T: OutputTypeMarker>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn create_fake_input_type<T: InputType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn create_fake_subscription_type<T: SubscriptionType>(&mut self) -> MetaType {
        T::create_type_info(self);
        self.types
            .get(&*T::type_name())
            .cloned()
            .expect("You definitely encountered a bug!")
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
        self.directives
            .insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = IndexSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn add_keys(&mut self, ty: &str, keys: impl Into<String>) {
        let all_keys = match self.types.get_mut(ty) {
            Some(MetaType::Object { keys: all_keys, .. }) => all_keys,
            Some(MetaType::Interface { keys: all_keys, .. }) => all_keys,
            _ => return,
        };
        if let Some(all_keys) = all_keys {
            all_keys.push(keys.into());
        } else {
            *all_keys = Some(vec![keys.into()]);
        }
    }

    pub fn concrete_type_by_name(&self, type_name: &str) -> Option<&MetaType> {
        self.types.get(MetaTypeName::concrete_typename(type_name))
    }

    pub fn concrete_type_by_parsed_type(&self, query_type: &ParsedType) -> Option<&MetaType> {
        match &query_type.base {
            ParsedBaseType::Named(name) => self.types.get(name.as_str()),
            ParsedBaseType::List(ty) => self.concrete_type_by_parsed_type(ty),
        }
    }

    pub(crate) fn has_entities(&self) -> bool {
        self.types.values().any(|ty| match ty {
            MetaType::Object {
                keys: Some(keys),
                resolvable: true,
                ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        })
    }

    /// Each type annotated with @key should be added to the _Entity union.
    /// If no types are annotated with the key directive, then the _Entity union
    /// and Query._entities field should be removed from the schema.
    ///
    /// [Reference](https://www.apollographql.com/docs/federation/federation-spec/#resolve-requests-for-entities).
    fn create_entity_type_and_root_field(&mut self) {
        let possible_types: IndexSet<String> = self
            .types
            .values()
            .filter_map(|ty| match ty {
                MetaType::Object {
                    name,
                    keys: Some(keys),
                    resolvable: true,
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                MetaType::Interface {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                _ => None,
            })
            .collect();

        if let MetaType::Object { fields, .. } = self
            .types
            .get_mut(&self.query_type)
            .expect("missing query type")
        {
            fields.insert(
                "_service".to_string(),
                MetaField {
                    name: "_service".to_string(),
                    description: None,
                    args: Default::default(),
                    ty: "_Service!".to_string(),
                    deprecation: Default::default(),
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                    shareable: false,
                    inaccessible: false,
                    tags: Default::default(),
                    override_from: None,
                    visible: None,
                    compute_complexity: None,
                    directive_invocations: vec![],
                    requires_scopes: vec![],
                },
            );
        }

        if !possible_types.is_empty() {
            self.types.insert(
                "_Entity".to_string(),
                MetaType::Union {
                    name: "_Entity".to_string(),
                    description: None,
                    possible_types,
                    visible: None,
                    inaccessible: false,
                    tags: Default::default(),
                    rust_typename: Some("async_graphql::federation::Entity"),
                    directive_invocations: vec![],
                },
            );

            if let MetaType::Object { fields, .. } = self.types.get_mut(&self.query_type).unwrap() {
                fields.insert(
                    "_entities".to_string(),
                    MetaField {
                        name: "_entities".to_string(),
                        description: None,
                        args: {
                            let mut args = IndexMap::new();
                            args.insert(
                                "representations".to_string(),
                                MetaInputValue {
                                    name: "representations".to_string(),
                                    description: None,
                                    ty: "[_Any!]!".to_string(),
                                    deprecation: Deprecation::NoDeprecated,
                                    default_value: None,
                                    visible: None,
                                    inaccessible: false,
                                    tags: Default::default(),
                                    is_secret: false,
                                    directive_invocations: vec![],
                                },
                            );
                            args
                        },
                        ty: "[_Entity]!".to_string(),
                        deprecation: Default::default(),
                        cache_control: Default::default(),
                        external: false,
                        requires: None,
                        provides: None,
                        shareable: false,
                        visible: None,
                        inaccessible: false,
                        tags: Default::default(),
                        override_from: None,
                        compute_complexity: None,
                        directive_invocations: vec![],
                        requires_scopes: vec![],
                    },
                );
            }
        }
    }

    pub(crate) fn create_introspection_types(&mut self) {
        __Schema::create_type_info(self);

        if let Some(MetaType::Object { fields, .. }) = self.types.get_mut(&self.query_type) {
            fields.insert(
                "__schema".to_string(),
                MetaField {
                    name: "__schema".to_string(),
                    description: Some("Access the current type schema of this server.".to_string()),
                    args: Default::default(),
                    ty: "__Schema".to_string(),
                    deprecation: Default::default(),
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                    shareable: false,
                    inaccessible: false,
                    tags: Default::default(),
                    visible: None,
                    compute_complexity: None,
                    override_from: None,
                    directive_invocations: vec![],
                    requires_scopes: vec![],
                },
            );

            fields.insert(
                "__type".to_string(),
                MetaField {
                    name: "__type".to_string(),
                    description: Some("Request the type information of a single type.".to_string()),
                    args: {
                        let mut args = IndexMap::new();
                        args.insert(
                            "name".to_string(),
                            MetaInputValue {
                                name: "name".to_string(),
                                description: None,
                                ty: "String!".to_string(),
                                deprecation: Deprecation::NoDeprecated,
                                default_value: None,
                                visible: None,
                                inaccessible: false,
                                tags: Default::default(),
                                is_secret: false,
                                directive_invocations: vec![],
                            },
                        );
                        args
                    },
                    ty: "__Type".to_string(),
                    deprecation: Default::default(),
                    cache_control: Default::default(),
                    external: false,
                    requires: None,
                    provides: None,
                    shareable: false,
                    inaccessible: false,
                    tags: Default::default(),
                    override_from: None,
                    visible: None,
                    compute_complexity: None,
                    directive_invocations: vec![],
                    requires_scopes: vec![],
                },
            );
        }
    }

    pub(crate) fn create_federation_types(&mut self) {
        <Any as InputType>::create_type_info(self);

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
                            deprecation: Default::default(),
                            cache_control: Default::default(),
                            external: false,
                            requires: None,
                            provides: None,
                            shareable: false,
                            visible: None,
                            inaccessible: false,
                            tags: Default::default(),
                            override_from: None,
                            compute_complexity: None,
                            directive_invocations: vec![],
                            requires_scopes: vec![],
                        },
                    );
                    fields
                },
                cache_control: Default::default(),
                extends: false,
                shareable: false,
                resolvable: true,
                interface_object: false,
                keys: None,
                visible: None,
                inaccessible: false,
                tags: Default::default(),
                is_subscription: false,
                rust_typename: Some("async_graphql::federation::Service"),
                directive_invocations: vec![],
                requires_scopes: vec![],
            },
        );

        self.create_entity_type_and_root_field();
    }

    pub fn names(&self) -> Vec<String> {
        let mut names = HashSet::new();

        for d in self.directives.values() {
            names.insert(d.name.to_string());
            names.extend(d.args.values().map(|arg| arg.name.to_string()));
        }

        for ty in self.types.values() {
            match ty {
                MetaType::Scalar { name, .. } | MetaType::Union { name, .. } => {
                    names.insert(name.clone());
                }
                MetaType::Object { name, fields, .. }
                | MetaType::Interface { name, fields, .. } => {
                    names.insert(name.clone());
                    names.extend(
                        fields
                            .values()
                            .map(|field| {
                                std::iter::once(field.name.clone())
                                    .chain(field.args.values().map(|arg| arg.name.to_string()))
                            })
                            .flatten(),
                    );
                }
                MetaType::Enum {
                    name, enum_values, ..
                } => {
                    names.insert(name.clone());
                    names.extend(enum_values.values().map(|value| value.name.to_string()));
                }
                MetaType::InputObject {
                    name, input_fields, ..
                } => {
                    names.insert(name.clone());
                    names.extend(input_fields.values().map(|field| field.name.to_string()));
                }
            }
        }

        names.into_iter().collect()
    }

    pub fn set_description(&mut self, name: impl AsRef<str>, desc: impl Into<String>) {
        let desc = desc.into();
        match self.types.get_mut(name.as_ref()) {
            Some(MetaType::Scalar { description, .. }) => *description = Some(desc),
            Some(MetaType::Object { description, .. }) => *description = Some(desc),
            Some(MetaType::Interface { description, .. }) => *description = Some(desc),
            Some(MetaType::Union { description, .. }) => *description = Some(desc),
            Some(MetaType::Enum { description, .. }) => *description = Some(desc),
            Some(MetaType::InputObject { description, .. }) => *description = Some(desc),
            None => {}
        }
    }

    pub fn remove_unused_types(&mut self) {
        let mut used_types = BTreeSet::new();
        let mut unused_types = BTreeSet::new();

        fn traverse_field<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            field: &'a MetaField,
        ) {
            traverse_type(
                types,
                used_types,
                MetaTypeName::concrete_typename(&field.ty),
            );
            for arg in field.args.values() {
                traverse_input_value(types, used_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            traverse_type(
                types,
                used_types,
                MetaTypeName::concrete_typename(&input_value.ty),
            );
        }

        fn traverse_type<'a>(
            types: &'a BTreeMap<String, MetaType>,
            used_types: &mut BTreeSet<&'a str>,
            type_name: &'a str,
        ) {
            if used_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                used_types.insert(type_name);
                match ty {
                    MetaType::Object { fields, .. } => {
                        for field in fields.values() {
                            traverse_field(types, used_types, field);
                        }
                    }
                    MetaType::Interface {
                        fields,
                        possible_types,
                        ..
                    } => {
                        for field in fields.values() {
                            traverse_field(types, used_types, field);
                        }
                        for type_name in possible_types.iter() {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::Union { possible_types, .. } => {
                        for type_name in possible_types.iter() {
                            traverse_type(types, used_types, type_name);
                        }
                    }
                    MetaType::InputObject { input_fields, .. } => {
                        for field in input_fields.values() {
                            traverse_input_value(types, used_types, field);
                        }
                    }
                    _ => {}
                }
            }
        }

        for directive in self.directives.values() {
            for arg in directive.args.values() {
                traverse_input_value(&self.types, &mut used_types, arg);
            }
        }

        for type_name in Some(&self.query_type)
            .into_iter()
            .chain(self.mutation_type.iter())
            .chain(self.subscription_type.iter())
        {
            traverse_type(&self.types, &mut used_types, type_name);
        }

        for ty in self.types.values().filter(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        }) {
            traverse_type(&self.types, &mut used_types, ty.name());
        }

        for ty in self.types.values() {
            let name = ty.name();
            if !is_system_type(name) && !used_types.contains(name) {
                unused_types.insert(name.to_string());
            }
        }

        for type_name in unused_types {
            self.types.remove(&type_name);
        }
    }

    pub fn find_visible_types(&self, ctx: &Context<'_>) -> HashSet<&str> {
        let mut visible_types = HashSet::new();

        fn traverse_field<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            field: &'a MetaField,
        ) {
            if !is_visible(ctx, &field.visible) {
                return;
            }

            traverse_type(
                ctx,
                types,
                visible_types,
                MetaTypeName::concrete_typename(&field.ty),
            );
            for arg in field.args.values() {
                traverse_input_value(ctx, types, visible_types, arg);
            }
        }

        fn traverse_input_value<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            input_value: &'a MetaInputValue,
        ) {
            if !is_visible(ctx, &input_value.visible) {
                return;
            }

            traverse_type(
                ctx,
                types,
                visible_types,
                MetaTypeName::concrete_typename(&input_value.ty),
            );
        }

        fn traverse_type<'a>(
            ctx: &Context<'_>,
            types: &'a BTreeMap<String, MetaType>,
            visible_types: &mut HashSet<&'a str>,
            type_name: &'a str,
        ) {
            if visible_types.contains(type_name) {
                return;
            }

            if let Some(ty) = types.get(type_name) {
                if !ty.is_visible(ctx) {
                    return;
                }

                visible_types.insert(type_name);
                match ty {
                    MetaType::Object { fields, .. } => {
                        for field in fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                    }
                    MetaType::Interface {
                        fields,
                        possible_types,
                        ..
                    } => {
                        for field in fields.values() {
                            traverse_field(ctx, types, visible_types, field);
                        }
                        for type_name in possible_types.iter() {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::Union { possible_types, .. } => {
                        for type_name in possible_types.iter() {
                            traverse_type(ctx, types, visible_types, type_name);
                        }
                    }
                    MetaType::InputObject { input_fields, .. } => {
                        for field in input_fields.values() {
                            traverse_input_value(ctx, types, visible_types, field);
                        }
                    }
                    _ => {}
                }
            }
        }

        for directive in self.directives.values() {
            if is_visible(ctx, &directive.visible) {
                for arg in directive.args.values() {
                    traverse_input_value(ctx, &self.types, &mut visible_types, arg);
                }
            }
        }

        for type_name in Some(&self.query_type)
            .into_iter()
            .chain(self.mutation_type.iter())
            .chain(self.subscription_type.iter())
        {
            traverse_type(ctx, &self.types, &mut visible_types, type_name);
        }

        for ty in self.types.values().filter(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            }
            | MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        }) {
            traverse_type(ctx, &self.types, &mut visible_types, ty.name());
        }

        for ty in self.types.values() {
            if let MetaType::Interface { possible_types, .. } = ty {
                if ty.is_visible(ctx) && !visible_types.contains(ty.name()) {
                    for type_name in possible_types.iter() {
                        if visible_types.contains(type_name.as_str()) {
                            traverse_type(ctx, &self.types, &mut visible_types, ty.name());
                            break;
                        }
                    }
                }
            }
        }

        self.types
            .values()
            .filter_map(|ty| {
                let name = ty.name();
                if is_system_type(name) || visible_types.contains(name) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub(crate) fn is_visible(ctx: &Context<'_>, visible: &Option<MetaVisibleFn>) -> bool {
    match visible {
        Some(f) => f(ctx),
        None => true,
    }
}

fn is_system_type(name: &str) -> bool {
    if name.starts_with("__") {
        return true;
    }

    name == "Boolean" || name == "Int" || name == "Float" || name == "String" || name == "ID"
}

#[cfg(test)]
mod test {
    use crate::registry::MetaDirectiveInvocation;

    #[test]
    fn test_directive_invocation_dsl() {
        let expected = r#"@testDirective(int_value: 1, str_value: "abc")"#;
        assert_eq!(
            expected.to_string(),
            MetaDirectiveInvocation {
                name: "testDirective".to_string(),
                args: [
                    ("int_value".to_string(), 1u32.into()),
                    ("str_value".to_string(), "abc".into())
                ]
                .into(),
            }
            .sdl()
        )
    }
}
