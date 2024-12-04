//! Executable document-related GraphQL types.

use serde::{Deserialize, Serialize};

use super::*;

/// An executable GraphQL file or request string.
///
/// [Reference](https://spec.graphql.org/October2021/#ExecutableDocument).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableDocument {
    /// The operations of the document.
    pub operations: DocumentOperations,
    /// The fragments of the document.
    pub fragments: HashMap<Name, Positioned<FragmentDefinition>>,
}

/// The operations of a GraphQL document.
///
/// There is either one anonymous operation or many named operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentOperations {
    /// The document contains a single anonymous operation.
    Single(Positioned<OperationDefinition>),
    /// The document contains many named operations.
    Multiple(HashMap<Name, Positioned<OperationDefinition>>),
}

impl DocumentOperations {
    /// Iterate over the operations of the document.
    #[must_use]
    pub fn iter(&self) -> OperationsIter<'_> {
        OperationsIter(match self {
            Self::Single(op) => OperationsIterInner::Single(Some(op)),
            Self::Multiple(ops) => OperationsIterInner::Multiple(ops.iter()),
        })
    }
}

// TODO: This is not implemented as I would like to later implement IntoIterator
// for DocumentOperations (not a reference) without having a breaking change.
//
// impl<'a> IntoIterator for &'a DocumentOperations {
//    type Item = &'a Positioned<OperationDefinition>;
//    type IntoIter = OperationsIter<'a>;
//
//    fn into_iter(self) -> Self::IntoIter {
//        self.iter()
//    }
//}

/// An iterator over the operations of a document.
#[derive(Debug, Clone)]
pub struct OperationsIter<'a>(OperationsIterInner<'a>);

impl<'a> Iterator for OperationsIter<'a> {
    type Item = (Option<&'a Name>, &'a Positioned<OperationDefinition>);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            OperationsIterInner::Single(op) => op.take().map(|op| (None, op)),
            OperationsIterInner::Multiple(iter) => iter.next().map(|(name, op)| (Some(name), op)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }
}

impl std::iter::FusedIterator for OperationsIter<'_> {}

impl ExactSizeIterator for OperationsIter<'_> {
    fn len(&self) -> usize {
        match &self.0 {
            OperationsIterInner::Single(opt) => usize::from(opt.is_some()),
            OperationsIterInner::Multiple(iter) => iter.len(),
        }
    }
}

#[derive(Debug, Clone)]
enum OperationsIterInner<'a> {
    Single(Option<&'a Positioned<OperationDefinition>>),
    Multiple(hash_map::Iter<'a, Name, Positioned<OperationDefinition>>),
}

/// A GraphQL operation, such as `mutation($content:String!) { makePost(content:
/// $content) { id } }`.
///
/// [Reference](https://spec.graphql.org/October2021/#OperationDefinition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDefinition {
    /// The type of operation.
    pub ty: OperationType,
    /// The variable definitions.
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    /// The operation's directives.
    pub directives: Vec<Positioned<Directive>>,
    /// The operation's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

/// A variable definition inside a list of variable definitions, for example
/// `$name:String!`.
///
/// [Reference](https://spec.graphql.org/October2021/#VariableDefinition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDefinition {
    /// The name of the variable, without the preceding `$`.
    pub name: Positioned<Name>,
    /// The type of the variable.
    pub var_type: Positioned<Type>,
    /// The variable's directives.
    pub directives: Vec<Positioned<Directive>>,
    /// The optional default value of the variable.
    pub default_value: Option<Positioned<ConstValue>>,
}

impl VariableDefinition {
    /// Get the default value of the variable; this is `default_value` if it is
    /// present, `Value::Null` if it is nullable and `None` otherwise.
    #[must_use]
    pub fn default_value(&self) -> Option<&ConstValue> {
        self.default_value.as_ref().map(|value| &value.node).or({
            if self.var_type.node.nullable {
                Some(&ConstValue::Null)
            } else {
                None
            }
        })
    }
}

/// A set of fields to be selected, for example `{ name age }`.
///
/// [Reference](https://spec.graphql.org/October2021/#SelectionSet).
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SelectionSet {
    /// The fields to be selected.
    pub items: Vec<Positioned<Selection>>,
}

/// A part of an object to be selected; a single field, a fragment spread or an
/// inline fragment.
///
/// [Reference](https://spec.graphql.org/October2021/#Selection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Selection {
    /// Select a single field, such as `name` or `weightKilos: weight(unit:
    /// KILOGRAMS)`.
    Field(Positioned<Field>),
    /// Select using a fragment.
    FragmentSpread(Positioned<FragmentSpread>),
    /// Select using an inline fragment.
    InlineFragment(Positioned<InlineFragment>),
}

impl Selection {
    /// Get a reference to the directives of the selection.
    #[must_use]
    pub fn directives(&self) -> &Vec<Positioned<Directive>> {
        match self {
            Self::Field(field) => &field.node.directives,
            Self::FragmentSpread(spread) => &spread.node.directives,
            Self::InlineFragment(fragment) => &fragment.node.directives,
        }
    }
    /// Get a mutable reference to the directives of the selection.
    #[must_use]
    pub fn directives_mut(&mut self) -> &mut Vec<Positioned<Directive>> {
        match self {
            Self::Field(field) => &mut field.node.directives,
            Self::FragmentSpread(spread) => &mut spread.node.directives,
            Self::InlineFragment(fragment) => &mut fragment.node.directives,
        }
    }
}

/// A field being selected on an object, such as `name` or `weightKilos:
/// weight(unit: KILOGRAMS)`.
///
/// [Reference](https://spec.graphql.org/October2021/#Field).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    /// The optional field alias.
    pub alias: Option<Positioned<Name>>,
    /// The name of the field.
    pub name: Positioned<Name>,
    /// The arguments to the field, empty if no arguments are provided.
    pub arguments: Vec<(Positioned<Name>, Positioned<Value>)>,
    /// The directives in the field selector.
    pub directives: Vec<Positioned<Directive>>,
    /// The subfields being selected in this field, if it is an object. Empty if
    /// no fields are being selected.
    pub selection_set: Positioned<SelectionSet>,
}

impl Field {
    /// Get the response key of the field. This is the alias if present and the
    /// name otherwise.
    #[must_use]
    pub fn response_key(&self) -> &Positioned<Name> {
        self.alias.as_ref().unwrap_or(&self.name)
    }

    /// Get the value of the argument with the specified name.
    #[must_use]
    pub fn get_argument(&self, name: &str) -> Option<&Positioned<Value>> {
        self.arguments
            .iter()
            .find(|item| item.0.node == name)
            .map(|item| &item.1)
    }
}

/// A fragment selector, such as `... userFields`.
///
/// [Reference](https://spec.graphql.org/October2021/#FragmentSpread).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentSpread {
    /// The name of the fragment being selected.
    pub fragment_name: Positioned<Name>,
    /// The directives in the fragment selector.
    pub directives: Vec<Positioned<Directive>>,
}

/// An inline fragment selector, such as `... on User { name }`.
///
/// [Reference](https://spec.graphql.org/October2021/#InlineFragment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineFragment {
    /// The type condition.
    pub type_condition: Option<Positioned<TypeCondition>>,
    /// The directives in the inline fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The selected fields of the fragment.
    pub selection_set: Positioned<SelectionSet>,
}

/// The definition of a fragment, such as `fragment userFields on User { name
/// age }`.
///
/// [Reference](https://spec.graphql.org/October2021/#FragmentDefinition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentDefinition {
    /// The type this fragment operates on.
    pub type_condition: Positioned<TypeCondition>,
    /// Directives in the fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The fragment's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

/// A type a fragment can apply to (`on` followed by the type).
///
/// [Reference](https://spec.graphql.org/October2021/#TypeCondition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCondition {
    /// The type this fragment applies to.
    pub on: Positioned<Name>,
}
