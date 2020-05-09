use crate::parser::span::Spanned;
use crate::parser::value::Value;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    Named(String),
    List(Box<Type>),
    NonNull(Box<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named(name) => write!(f, "{}", name),
            Type::List(ty) => write!(f, "[{}]", ty),
            Type::NonNull(ty) => write!(f, "{}!", ty),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Directive {
    pub name: Spanned<String>,
    pub arguments: Vec<(Spanned<String>, Spanned<Value>)>,
}

impl Directive {
    pub fn get_argument(&self, name: &str) -> Option<&Spanned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.as_str() == name)
            .map(|item| &item.1)
    }
}

#[derive(Clone, Debug)]
pub struct Document {
    pub definitions: Vec<Spanned<Definition>>,
}

#[derive(Clone, Debug)]
pub enum Definition {
    Operation(Spanned<OperationDefinition>),
    Fragment(Spanned<FragmentDefinition>),
}

#[derive(Clone, Debug)]
pub enum TypeCondition {
    On(Spanned<String>),
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition {
    pub name: Spanned<String>,
    pub type_condition: Spanned<TypeCondition>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub enum OperationDefinition {
    SelectionSet(Spanned<SelectionSet>),
    Query(Spanned<Query>),
    Mutation(Spanned<Mutation>),
    Subscription(Spanned<Subscription>),
}

#[derive(Clone, Debug)]
pub struct Query {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Vec<Spanned<VariableDefinition>>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Mutation {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Vec<Spanned<VariableDefinition>>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Subscription {
    pub name: Option<Spanned<String>>,
    pub variable_definitions: Vec<Spanned<VariableDefinition>>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

#[derive(Clone, Debug, Default)]
pub struct SelectionSet {
    pub items: Vec<Spanned<Selection>>,
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub name: Spanned<String>,
    pub var_type: Spanned<Type>,
    pub default_value: Option<Spanned<Value>>,
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Spanned<Field>),
    FragmentSpread(Spanned<FragmentSpread>),
    InlineFragment(Spanned<InlineFragment>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub alias: Option<Spanned<String>>,
    pub name: Spanned<String>,
    pub arguments: Vec<(Spanned<String>, Spanned<Value>)>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}

impl Field {
    pub fn get_argument(&self, name: &str) -> Option<&Spanned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.as_str() == name)
            .map(|item| &item.1)
    }
}

#[derive(Clone, Debug)]
pub struct FragmentSpread {
    pub fragment_name: Spanned<String>,
    pub directives: Vec<Spanned<Directive>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment {
    pub type_condition: Option<Spanned<TypeCondition>>,
    pub directives: Vec<Spanned<Directive>>,
    pub selection_set: Spanned<SelectionSet>,
}
