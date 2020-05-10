use crate::pos::Positioned;
use crate::value::Value;
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
    pub name: Positioned<String>,
    pub arguments: Vec<(Positioned<String>, Positioned<Value>)>,
}

impl Directive {
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.as_str() == name)
            .map(|item| &item.1)
    }
}

#[derive(Clone, Debug)]
pub struct Document {
    pub definitions: Vec<Positioned<Definition>>,
}

#[derive(Clone, Debug)]
pub enum Definition {
    Operation(Positioned<OperationDefinition>),
    Fragment(Positioned<FragmentDefinition>),
}

#[derive(Clone, Debug)]
pub enum TypeCondition {
    On(Positioned<String>),
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition {
    pub name: Positioned<String>,
    pub type_condition: Positioned<TypeCondition>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub enum OperationDefinition {
    SelectionSet(Positioned<SelectionSet>),
    Query(Positioned<Query>),
    Mutation(Positioned<Mutation>),
    Subscription(Positioned<Subscription>),
}

#[derive(Clone, Debug)]
pub struct Query {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Mutation {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Clone, Debug)]
pub struct Subscription {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Clone, Debug, Default)]
pub struct SelectionSet {
    pub items: Vec<Positioned<Selection>>,
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub name: Positioned<String>,
    pub var_type: Positioned<Type>,
    pub default_value: Option<Positioned<Value>>,
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Positioned<Field>),
    FragmentSpread(Positioned<FragmentSpread>),
    InlineFragment(Positioned<InlineFragment>),
}

#[derive(Clone, Debug)]
pub struct Field {
    pub alias: Option<Positioned<String>>,
    pub name: Positioned<String>,
    pub arguments: Vec<(Positioned<String>, Positioned<Value>)>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

impl Field {
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.as_str() == name)
            .map(|item| &item.1)
    }
}

#[derive(Clone, Debug)]
pub struct FragmentSpread {
    pub fragment_name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment {
    pub type_condition: Option<Positioned<TypeCondition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}
