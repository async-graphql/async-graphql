macro_rules! impl_set_description {
    () => {
        /// Set the description
        #[inline]
        pub fn description(self, description: impl Into<String>) -> Self {
            Self {
                description: Some(description.into()),
                ..self
            }
        }
    };
}

macro_rules! impl_set_deprecation {
    () => {
        /// Set the description
        #[inline]
        pub fn deprecation(self, reason: Option<&str>) -> Self {
            Self {
                deprecation: Deprecation::Deprecated {
                    reason: reason.map(Into::into),
                },
                ..self
            }
        }
    };
}

macro_rules! impl_set_extends {
    () => {
        /// Indicates that an object or interface definition is an extension of another
        /// definition of that same type.
        #[inline]
        pub fn extends(self) -> Self {
            Self {
                extends: true,
                ..self
            }
        }
    };
}

macro_rules! impl_set_inaccessible {
    () => {
        /// Indicate that an enum is not accessible from a supergraph when using
        /// Apollo Federation
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#inaccessible>
        #[inline]
        pub fn inaccessible(self) -> Self {
            Self {
                inaccessible: true,
                ..self
            }
        }
    };
}

macro_rules! impl_set_interface_object {
    () => {
        /// During composition, the fields of every `@interfaceObject` are added
        /// both to their corresponding interface definition and to all
        /// entity types that implement that interface.
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#interfaceobject>
        #[inline]
        pub fn interface_object(self) -> Self {
            Self {
                interface_object: true,
                ..self
            }
        }
    };
}

macro_rules! impl_set_tags {
    () => {
        /// Arbitrary string metadata that will be propagated to the supergraph
        /// when using Apollo Federation. This attribute is repeatable
        ///
        /// Reference: <https://www.apollographql.com/docs/federation/federated-types/federated-directives/#applying-metadata>
        #[inline]
        pub fn tags<I: IntoIterator<Item = T>, T: Into<String>>(self, tags: I) -> Self {
            Self {
                tags: tags.into_iter().map(Into::into).collect(),
                ..self
            }
        }
    };
}

macro_rules! impl_set_external {
    () => {
        /// Mark a field as owned by another service. This allows service A to use
        /// fields from service B while also knowing at runtime the types of that
        /// field.
        #[inline]
        pub fn external(self) -> Self {
            Self {
                external: true,
                ..self
            }
        }
    };
}

macro_rules! impl_set_requires {
    () => {
        /// Annotate the required input fieldset from a base type for a resolver. It
        /// is used to develop a query plan where the required fields may not be
        /// needed by the client, but the service may need additional information
        /// from other services.
        #[inline]
        pub fn requires(self, fields: impl Into<String>) -> Self {
            Self {
                requires: Some(fields.into()),
                ..self
            }
        }
    };
}

macro_rules! impl_set_provides {
    () => {
        /// Annotate the expected returned fieldset from a field on a base type that
        /// is guaranteed to be selectable by the gateway.
        #[inline]
        pub fn provides(self, fields: impl Into<String>) -> Self {
            Self {
                provides: Some(fields.into()),
                ..self
            }
        }
    };
}

macro_rules! impl_set_shareable {
    () => {
        /// Indicate that an object type's field is allowed to be resolved by
        /// multiple subgraphs
        #[inline]
        pub fn shareable(self) -> Self {
            Self {
                shareable: true,
                ..self
            }
        }
    };
}

macro_rules! impl_set_override_from {
    () => {
        /// Indicate that an object type's field is allowed to be resolved by
        /// multiple subgraphs
        #[inline]
        pub fn override_from(self, name: impl Into<String>) -> Self {
            Self {
                override_from: Some(name.into()),
                ..self
            }
        }
    };
}
