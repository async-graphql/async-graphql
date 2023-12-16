use darling::{
    ast::{Data, Fields, NestedMeta},
    util::{Ignored, SpannedValue},
    FromDeriveInput, FromField, FromMeta, FromVariant,
};
use inflector::Inflector;
use quote::format_ident;
use syn::{Attribute, Expr, Generics, Ident, Lit, LitBool, LitStr, Meta, Path, Type, Visibility};

use crate::validators::Validators;

#[derive(FromMeta, Clone)]
#[darling(default)]
pub struct CacheControl {
    public: bool,
    private: bool,
    pub no_cache: bool,
    pub max_age: usize,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            private: false,
            no_cache: false,
            max_age: 0,
        }
    }
}

impl CacheControl {
    pub fn is_public(&self) -> bool {
        !self.private && self.public
    }
}

#[derive(Debug)]
pub enum DefaultValue {
    Default,
    Value(Lit),
}

impl FromMeta for DefaultValue {
    fn from_word() -> darling::Result<Self> {
        Ok(DefaultValue::Default)
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        Ok(DefaultValue::Value(value.clone()))
    }
}

#[derive(Debug, Clone)]
pub enum Visible {
    None,
    HiddenAlways,
    FnName(Path),
}

impl FromMeta for Visible {
    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Bool(LitBool { value: true, .. }) => Ok(Visible::None),
            Lit::Bool(LitBool { value: false, .. }) => Ok(Visible::HiddenAlways),
            Lit::Str(str) => Ok(Visible::FnName(syn::parse_str::<Path>(&str.value())?)),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }
}

pub struct PathList(pub Vec<Path>);

impl FromMeta for PathList {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut res = Vec::new();
        for item in items {
            if let NestedMeta::Meta(Meta::Path(p)) = item {
                res.push(p.clone());
            } else {
                return Err(darling::Error::custom("Invalid path list"));
            }
        }
        Ok(PathList(res))
    }
}

#[derive(FromMeta)]
pub struct ConcreteType {
    pub name: String,
    pub params: PathList,
}

#[derive(Debug, Clone, Default)]
pub enum Deprecation {
    #[default]
    NoDeprecated,
    Deprecated {
        reason: Option<String>,
    },
}

impl FromMeta for Deprecation {
    fn from_word() -> darling::Result<Self> {
        Ok(Deprecation::Deprecated { reason: None })
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Bool(LitBool { value: true, .. }) => Ok(Deprecation::Deprecated { reason: None }),
            Lit::Bool(LitBool { value: false, .. }) => Ok(Deprecation::NoDeprecated),
            Lit::Str(str) => Ok(Deprecation::Deprecated {
                reason: Some(str.value()),
            }),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Resolvability {
    #[default]
    Resolvable,
    Unresolvable {
        key: Option<String>,
    },
}

impl FromMeta for Resolvability {
    fn from_word() -> darling::Result<Self> {
        Ok(Resolvability::Unresolvable { key: None })
    }

    fn from_value(value: &Lit) -> darling::Result<Self> {
        match value {
            Lit::Bool(LitBool { value: true, .. }) => Ok(Resolvability::Unresolvable { key: None }),
            Lit::Bool(LitBool { value: false, .. }) => Ok(Resolvability::Resolvable),
            Lit::Str(str) => Ok(Resolvability::Unresolvable {
                key: Some(str.value()),
            }),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }
}

#[derive(FromField)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct SimpleObjectField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,

    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub skip_output: bool,
    // for InputObject
    #[darling(default)]
    pub skip_input: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub deprecation: Deprecation,
    #[darling(default)]
    pub owned: bool,
    #[darling(default)]
    pub cache_control: CacheControl,
    #[darling(default)]
    pub external: bool,
    #[darling(default)]
    pub provides: Option<String>,
    #[darling(default)]
    pub requires: Option<String>,
    #[darling(default)]
    pub shareable: bool,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub override_from: Option<String>,
    #[darling(default)]
    pub guard: Option<Expr>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default, multiple)]
    pub derived: Vec<DerivedField>,
    #[darling(default)]
    pub process_with: Option<Expr>,
    // for InputObject
    #[darling(default)]
    pub default: Option<DefaultValue>,
    #[darling(default)]
    pub default_with: Option<LitStr>,
    #[darling(default)]
    pub validator: Option<Validators>,
    #[darling(default)]
    pub flatten: bool,
    #[darling(default)]
    pub secret: bool,
    #[darling(default, multiple, rename = "directive")]
    pub directives: Vec<Expr>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct SimpleObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, SimpleObjectField>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub fake: bool,
    #[darling(default)]
    pub complex: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub rename_fields: Option<RenameRule>,
    #[darling(default)]
    pub rename_args: Option<RenameRule>,
    #[darling(default)]
    pub cache_control: CacheControl,
    #[darling(default)]
    pub extends: bool,
    #[darling(default)]
    pub shareable: bool,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default)]
    pub interface_object: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default, multiple, rename = "concrete")]
    pub concretes: Vec<ConcreteType>,
    #[darling(default)]
    pub serial: bool,
    #[darling(default, rename = "unresolvable")]
    pub resolvability: Resolvability,
    // for InputObject
    #[darling(default)]
    pub input_name: Option<String>,
    #[darling(default)]
    pub guard: Option<Expr>,
    #[darling(default, multiple, rename = "directive")]
    pub directives: Vec<Expr>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Argument {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: Option<DefaultValue>,
    pub default_with: Option<LitStr>,
    pub validator: Option<Validators>,
    #[darling(default)]
    pub process_with: Option<Expr>,
    pub key: bool, // for entity
    pub visible: Option<Visible>,
    pub inaccessible: bool,
    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,
    pub secret: bool,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Object {
    pub internal: bool,
    pub name: Option<String>,
    pub name_type: bool,
    pub rename_fields: Option<RenameRule>,
    pub rename_args: Option<RenameRule>,
    pub cache_control: CacheControl,
    pub extends: bool,
    pub shareable: bool,
    pub inaccessible: bool,
    pub interface_object: bool,
    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,
    pub use_type_description: bool,
    pub visible: Option<Visible>,
    pub serial: bool,
    #[darling(default, rename = "unresolvable")]
    pub resolvability: Resolvability,
    #[darling(multiple, rename = "concrete")]
    pub concretes: Vec<ConcreteType>,
    #[darling(default)]
    pub guard: Option<Expr>,
    #[darling(default, multiple, rename = "directive")]
    pub directives: Vec<Expr>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct ObjectField {
    pub skip: bool,
    pub entity: bool,
    pub name: Option<String>,
    pub deprecation: Deprecation,
    pub cache_control: CacheControl,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
    pub shareable: bool,
    pub inaccessible: bool,
    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,
    pub override_from: Option<String>,
    pub guard: Option<Expr>,
    pub visible: Option<Visible>,
    pub complexity: Option<Expr>,
    #[darling(default, multiple)]
    pub derived: Vec<DerivedField>,
    pub flatten: bool,
    #[darling(default, multiple, rename = "directive")]
    pub directives: Vec<Expr>,
}

#[derive(FromMeta, Default, Clone)]
#[darling(default)]
/// Derivied fields arguments: are used to generate derivied fields.
pub struct DerivedField {
    pub name: Option<Ident>,
    pub into: Option<String>,
    pub with: Option<Path>,
    #[darling(default)]
    pub owned: Option<bool>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct Enum {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<EnumItem, Ignored>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub rename_items: Option<RenameRule>,
    #[darling(default)]
    pub remote: Option<Type>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
}

#[derive(FromVariant)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct EnumItem {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub fields: Fields<Ignored>,

    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub deprecation: Deprecation,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct Union {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<UnionItem, Ignored>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    // for OneofObject
    #[darling(default)]
    pub input_name: Option<String>,
}

#[derive(FromVariant)]
#[darling(attributes(graphql))]
pub struct UnionItem {
    pub ident: Ident,
    pub fields: Fields<syn::Type>,

    #[darling(default)]
    pub flatten: bool,
}

#[derive(FromField)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct InputObjectField {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub vis: Visibility,
    pub attrs: Vec<Attribute>,

    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub default: Option<DefaultValue>,
    #[darling(default)]
    pub default_with: Option<LitStr>,
    #[darling(default)]
    pub validator: Option<Validators>,
    #[darling(default)]
    pub flatten: bool,
    #[darling(default)]
    pub skip: bool,
    #[darling(default)]
    pub skip_input: bool,
    #[darling(default)]
    pub process_with: Option<Expr>,
    // for SimpleObject
    #[darling(default)]
    pub skip_output: bool,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub secret: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct InputObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, InputObjectField>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub input_name: Option<String>,
    #[darling(default)]
    pub rename_fields: Option<RenameRule>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default, multiple, rename = "concrete")]
    pub concretes: Vec<ConcreteType>,
    #[darling(default)]
    pub validator: Option<Expr>,
    // for SimpleObject
    #[darling(default)]
    pub complex: bool,
    #[darling(default)]
    pub shareable: bool,
}

#[derive(FromVariant)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct OneofObjectField {
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub fields: Fields<syn::Type>,

    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub validator: Option<Validators>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub secret: bool,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct OneofObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<OneofObjectField, Ignored>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub input_name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub rename_fields: Option<RenameRule>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default, multiple, rename = "concrete")]
    pub concretes: Vec<ConcreteType>,
}

#[derive(FromMeta)]
pub struct InterfaceFieldArgument {
    pub name: String,
    #[darling(default)]
    pub desc: Option<String>,
    pub ty: Type,
    #[darling(default)]
    pub default: Option<DefaultValue>,
    #[darling(default)]
    pub default_with: Option<LitStr>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub secret: bool,
}

#[derive(FromMeta)]
pub struct InterfaceField {
    pub name: SpannedValue<String>,
    pub ty: Type,
    #[darling(default)]
    pub method: Option<String>,
    #[darling(default)]
    pub desc: Option<String>,
    #[darling(default, multiple, rename = "arg")]
    pub args: Vec<InterfaceFieldArgument>,
    #[darling(default)]
    pub deprecation: Deprecation,
    #[darling(default)]
    pub external: bool,
    #[darling(default)]
    pub provides: Option<String>,
    #[darling(default)]
    pub requires: Option<String>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub shareable: bool,
    #[darling(default)]
    pub override_from: Option<String>,
}

#[derive(FromVariant)]
pub struct InterfaceMember {
    pub ident: Ident,
    pub fields: Fields<syn::Type>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct Interface {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<InterfaceMember, Ignored>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub rename_fields: Option<RenameRule>,
    #[darling(default)]
    pub rename_args: Option<RenameRule>,
    #[darling(default, multiple, rename = "field")]
    pub fields: Vec<InterfaceField>,
    #[darling(default)]
    pub extends: bool,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Scalar {
    pub internal: bool,
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    pub use_type_description: bool,
    pub visible: Option<Visible>,
    pub inaccessible: bool,
    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,
    pub specified_by_url: Option<String>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Subscription {
    pub internal: bool,
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    pub rename_fields: Option<RenameRule>,
    pub rename_args: Option<RenameRule>,
    pub use_type_description: bool,
    pub extends: bool,
    pub visible: Option<Visible>,
    #[darling(default)]
    pub guard: Option<Expr>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct SubscriptionFieldArgument {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub default: Option<DefaultValue>,
    pub default_with: Option<LitStr>,
    pub validator: Option<Validators>,
    #[darling(default)]
    pub process_with: Option<Expr>,
    pub visible: Option<Visible>,
    pub secret: bool,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct SubscriptionField {
    pub skip: bool,
    pub name: Option<String>,
    pub deprecation: Deprecation,
    pub guard: Option<Expr>,
    pub visible: Option<Visible>,
    pub complexity: Option<Expr>,
}

#[derive(FromField)]
pub struct MergedObjectField {
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct MergedObject {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, MergedObjectField>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub cache_control: CacheControl,
    #[darling(default)]
    pub extends: bool,
    #[darling(default)]
    pub shareable: bool,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default)]
    pub interface_object: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub serial: bool,
    #[darling(default, multiple, rename = "directive")]
    pub directives: Vec<Expr>,
}

#[derive(FromField)]
pub struct MergedSubscriptionField {
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct MergedSubscription {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, MergedSubscriptionField>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub extends: bool,
}

#[derive(Debug, Copy, Clone, FromMeta)]
pub enum RenameRule {
    #[darling(rename = "lowercase")]
    Lower,
    #[darling(rename = "UPPERCASE")]
    Upper,
    #[darling(rename = "PascalCase")]
    Pascal,
    #[darling(rename = "camelCase")]
    Camel,
    #[darling(rename = "snake_case")]
    Snake,
    #[darling(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnake,
}

impl RenameRule {
    fn rename(&self, name: impl AsRef<str>) -> String {
        match self {
            Self::Lower => name.as_ref().to_lowercase(),
            Self::Upper => name.as_ref().to_uppercase(),
            Self::Pascal => name.as_ref().to_pascal_case(),
            Self::Camel => name.as_ref().to_camel_case(),
            Self::Snake => name.as_ref().to_snake_case(),
            Self::ScreamingSnake => name.as_ref().to_screaming_snake_case(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RenameTarget {
    Type,
    EnumItem,
    Field,
    Argument,
}

impl RenameTarget {
    fn rule(&self) -> RenameRule {
        match self {
            RenameTarget::Type => RenameRule::Pascal,
            RenameTarget::EnumItem => RenameRule::ScreamingSnake,
            RenameTarget::Field => RenameRule::Camel,
            RenameTarget::Argument => RenameRule::Camel,
        }
    }

    pub fn rename(&self, name: impl AsRef<str>) -> String {
        self.rule().rename(name)
    }
}

pub trait RenameRuleExt {
    fn rename(&self, name: impl AsRef<str>, target: RenameTarget) -> String;
}

impl RenameRuleExt for Option<RenameRule> {
    fn rename(&self, name: impl AsRef<str>, target: RenameTarget) -> String {
        self.unwrap_or(target.rule()).rename(name)
    }
}

#[derive(FromDeriveInput)]
#[darling(forward_attrs(doc))]
pub struct Description {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,

    #[darling(default)]
    pub internal: bool,
}

#[derive(Debug)]
pub enum NewTypeName {
    New(String),
    Rust,
    Original,
}

impl Default for NewTypeName {
    fn default() -> Self {
        Self::Original
    }
}

impl FromMeta for NewTypeName {
    fn from_word() -> darling::Result<Self> {
        Ok(Self::Rust)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::New(value.to_string()))
    }

    fn from_bool(value: bool) -> darling::Result<Self> {
        if value {
            Ok(Self::Rust)
        } else {
            Ok(Self::Original)
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(attributes(graphql), forward_attrs(doc))]
pub struct NewType {
    pub ident: Ident,
    pub generics: Generics,
    pub attrs: Vec<Attribute>,
    pub data: Data<Ignored, syn::Type>,

    #[darling(default)]
    pub internal: bool,
    #[darling(default)]
    pub name: NewTypeName,
    #[darling(default)]
    pub visible: Option<Visible>,
    #[darling(default)]
    pub inaccessible: bool,
    #[darling(default, multiple, rename = "tag")]
    pub tags: Vec<String>,
    #[darling(default)]
    pub specified_by_url: Option<String>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct ComplexObject {
    pub internal: bool,
    pub rename_fields: Option<RenameRule>,
    pub rename_args: Option<RenameRule>,
    pub guard: Option<Expr>,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct ComplexObjectField {
    pub skip: bool,
    pub name: Option<String>,
    pub deprecation: Deprecation,
    pub cache_control: CacheControl,
    pub external: bool,
    pub provides: Option<String>,
    pub requires: Option<String>,
    pub shareable: bool,
    pub inaccessible: bool,
    #[darling(multiple, rename = "tag")]
    pub tags: Vec<String>,
    pub override_from: Option<String>,
    pub guard: Option<Expr>,
    pub visible: Option<Visible>,
    pub complexity: Option<Expr>,
    #[darling(multiple)]
    pub derived: Vec<DerivedField>,
    pub flatten: bool,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct Directive {
    pub internal: bool,
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    pub visible: Option<Visible>,
    pub repeatable: bool,
    pub rename_args: Option<RenameRule>,
    #[darling(multiple, rename = "location")]
    pub locations: Vec<DirectiveLocation>,
}

#[derive(Debug, Copy, Clone, FromMeta, strum::Display)]
#[darling(rename_all = "PascalCase")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectiveLocation {
    Field,
}

#[derive(FromMeta, Default)]
#[darling(default)]
pub struct TypeDirective {
    pub internal: bool,
    pub name: Option<String>,
    #[darling(default)]
    pub name_type: bool,
    pub visible: Option<Visible>,
    pub repeatable: bool,
    pub rename_args: Option<RenameRule>,
    #[darling(multiple, rename = "location")]
    pub locations: Vec<TypeDirectiveLocation>,
    #[darling(default)]
    pub composable: Option<String>,
}

#[derive(Debug, Copy, Clone, FromMeta, strum::Display)]
#[darling(rename_all = "PascalCase")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeDirectiveLocation {
    FieldDefinition,
    Object,
}

impl TypeDirectiveLocation {
    pub fn location_trait_identifier(&self) -> Ident {
        format_ident!("Directive_At_{}", self.to_string())
    }
}
