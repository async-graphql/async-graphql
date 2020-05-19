use crate::pos::Positioned;
use crate::value::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: Positioned<String>,
    pub arguments: Vec<(Positioned<String>, Positioned<Value>)>,
}

impl Directive {
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

pub type FragmentsMap = HashMap<String, Positioned<FragmentDefinition>>;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone)]
pub struct CurrentOperation {
    pub ty: OperationType,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) definitions: Vec<Positioned<Definition>>,
    pub(crate) fragments: FragmentsMap,
    pub(crate) current_operation: Option<CurrentOperation>,
}

impl Document {
    #[inline]
    pub fn definitions(&self) -> &[Positioned<Definition>] {
        &self.definitions
    }

    #[inline]
    pub fn fragments(&self) -> &FragmentsMap {
        &self.fragments
    }

    #[inline]
    pub fn current_operation(&self) -> &CurrentOperation {
        self.current_operation
            .as_ref()
            .expect("Must first call retain_operation")
    }

    pub fn retain_operation(&mut self, operation_name: Option<&str>) -> bool {
        let mut fragments = HashMap::new();

        for definition in self.definitions.drain(..) {
            match definition.node {
                Definition::Operation(operation_definition) if self.current_operation.is_none() => {
                    match operation_definition.node {
                        OperationDefinition::SelectionSet(s) => {
                            self.current_operation = Some(CurrentOperation {
                                ty: OperationType::Query,
                                variable_definitions: Vec::new(),
                                selection_set: s,
                            });
                        }
                        OperationDefinition::Query(query)
                            if query.name.is_none()
                                || operation_name.is_none()
                                || query.name.as_ref().map(|name| name.node.as_str())
                                    == operation_name.as_deref() =>
                        {
                            self.current_operation = Some(CurrentOperation {
                                ty: OperationType::Query,
                                variable_definitions: query.node.variable_definitions,
                                selection_set: query.node.selection_set,
                            });
                        }
                        OperationDefinition::Mutation(mutation)
                            if mutation.name.is_none()
                                || operation_name.is_none()
                                || mutation.name.as_ref().map(|name| name.node.as_str())
                                    == operation_name =>
                        {
                            self.current_operation = Some(CurrentOperation {
                                ty: OperationType::Mutation,
                                variable_definitions: mutation.node.variable_definitions,
                                selection_set: mutation.node.selection_set,
                            });
                        }
                        OperationDefinition::Subscription(subscription)
                            if subscription.name.is_none()
                                || operation_name.is_none()
                                || subscription.name.as_ref().map(|name| name.node.as_str())
                                    == operation_name =>
                        {
                            self.current_operation = Some(CurrentOperation {
                                ty: OperationType::Subscription,
                                variable_definitions: subscription.node.variable_definitions,
                                selection_set: subscription.node.selection_set,
                            });
                        }
                        _ => {}
                    }
                }
                Definition::Operation(_) => {}
                Definition::Fragment(fragment) => {
                    fragments.insert(fragment.name.clone_inner(), fragment);
                }
            }
        }
        self.fragments = fragments;
        self.current_operation.is_some()
    }
}

#[derive(Debug, Clone)]
pub enum Definition {
    Operation(Positioned<OperationDefinition>),
    Fragment(Positioned<FragmentDefinition>),
}

#[derive(Debug, Clone)]
pub enum TypeCondition {
    On(Positioned<String>),
}

#[derive(Debug, Clone)]
pub struct FragmentDefinition {
    pub name: Positioned<String>,
    pub type_condition: Positioned<TypeCondition>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Debug, Clone)]
pub enum OperationDefinition {
    SelectionSet(Positioned<SelectionSet>),
    Query(Positioned<Query>),
    Mutation(Positioned<Mutation>),
    Subscription(Positioned<Subscription>),
}

#[derive(Debug, Clone)]
pub struct Query {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Debug, Clone)]
pub struct Subscription {
    pub name: Option<Positioned<String>>,
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}

#[derive(Debug, Default, Clone)]
pub struct SelectionSet {
    pub items: Vec<Positioned<Selection>>,
}

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub name: Positioned<String>,
    pub var_type: Positioned<Type>,
    pub default_value: Option<Positioned<Value>>,
}

#[derive(Debug, Clone)]
pub enum Selection {
    Field(Positioned<Field>),
    FragmentSpread(Positioned<FragmentSpread>),
    InlineFragment(Positioned<InlineFragment>),
}

#[derive(Debug, Clone)]
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
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

#[derive(Debug, Clone)]
pub struct FragmentSpread {
    pub fragment_name: Positioned<String>,
    pub directives: Vec<Positioned<Directive>>,
}

#[derive(Debug, Clone)]
pub struct InlineFragment {
    pub type_condition: Option<Positioned<TypeCondition>>,
    pub directives: Vec<Positioned<Directive>>,
    pub selection_set: Positioned<SelectionSet>,
}
