//! Executable document-related GraphQL types.

use super::*;

/// An executable GraphQL file or request string.
///
/// [Reference](https://spec.graphql.org/June2018/#ExecutableDocument).
#[derive(Debug, Clone)]
pub struct ExecutableDocument {
    /// The definitions of the document.
    pub definitions: Vec<ExecutableDefinition>,
}

impl ExecutableDocument {
    /// Convert the document into an [`ExecutableDocumentData`](struct.ExecutableDocumentData.html).
    /// Will return `None` if there is no operation in the document.
    ///
    /// The `operation_name` parameter, if set, makes sure that the main operation of the document,
    /// if named, will have that name.
    #[must_use]
    pub fn into_data(self, operation_name: Option<&str>) -> Option<ExecutableDocumentData> {
        let mut operation = None;
        let mut fragments = HashMap::new();

        for definition in self.definitions {
            match definition {
                ExecutableDefinition::Operation(op)
                    if operation_name
                        .zip(op.node.name.as_ref())
                        .map_or(false, |(required_name, op_name)| {
                            required_name != op_name.node
                        }) => {}
                ExecutableDefinition::Operation(op) => {
                    operation.get_or_insert(op);
                }
                ExecutableDefinition::Fragment(fragment) => {
                    fragments.insert(fragment.node.name.node.clone(), fragment);
                }
            }
        }
        operation.map(|operation| ExecutableDocumentData {
            operation,
            fragments,
        })
    }
}

/// The data of an executable document. This is a [`ExecutableDocument`](struct.ExecutableDocument.html) with at least
/// one operation, and any number of fragments.
#[derive(Debug, Clone)]
pub struct ExecutableDocumentData {
    /// The main operation of the document.
    pub operation: Positioned<OperationDefinition>,
    /// The fragments of the document.
    pub fragments: HashMap<Name, Positioned<FragmentDefinition>>,
}

/// An executable definition in a query; a query, mutation, subscription or fragment definition.
///
/// [Reference](https://spec.graphql.org/June2018/#ExecutableDefinition).
#[derive(Debug, Clone)]
pub enum ExecutableDefinition {
    /// The definition of an operation.
    Operation(Positioned<OperationDefinition>),
    /// The definition of a fragment.
    Fragment(Positioned<FragmentDefinition>),
}

impl ExecutableDefinition {
    /// Get the position of the definition.
    #[must_use]
    pub fn pos(&self) -> Pos {
        match self {
            Self::Operation(op) => op.pos,
            Self::Fragment(frag) => frag.pos,
        }
    }

    /// Get a reference to the directives of the definition.
    #[must_use]
    pub fn directives(&self) -> &Vec<Positioned<Directive>> {
        match self {
            Self::Operation(op) => &op.node.directives,
            Self::Fragment(frag) => &frag.node.directives,
        }
    }
    /// Get a mutable reference to the directives of the definition.
    #[must_use]
    pub fn directives_mut(&mut self) -> &mut Vec<Positioned<Directive>> {
        match self {
            Self::Operation(op) => &mut op.node.directives,
            Self::Fragment(frag) => &mut frag.node.directives,
        }
    }
}

/// A GraphQL operation, such as `mutation($content:String!) { makePost(content: $content) { id } }`.
///
/// [Reference](https://spec.graphql.org/June2018/#OperationDefinition).
#[derive(Debug, Clone)]
pub struct OperationDefinition {
    /// The type of operation.
    pub ty: OperationType,
    /// The name of the operation.
    pub name: Option<Positioned<Name>>,
    /// The variable definitions.
    pub variable_definitions: Vec<Positioned<VariableDefinition>>,
    /// The operation's directives.
    pub directives: Vec<Positioned<Directive>>,
    /// The operation's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

/// A variable definition inside a list of variable definitions, for example `$name:String!`.
///
/// [Reference](https://spec.graphql.org/June2018/#VariableDefinition).
#[derive(Debug, Clone)]
pub struct VariableDefinition {
    /// The name of the variable, without the preceding `$`.
    pub name: Positioned<Name>,
    /// The type of the variable.
    pub var_type: Positioned<Type>,
    /// The optional default value of the variable.
    pub default_value: Option<Positioned<ConstValue>>,
}

impl VariableDefinition {
    /// Get the default value of the variable; this is `default_value` if it is present,
    /// `Value::Null` if it is nullable and `None` otherwise.
    #[must_use]
    pub fn default_value(&self) -> Option<&ConstValue> {
        self.default_value
            .as_ref()
            .map(|value| &value.node)
            .or_else(|| {
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
/// [Reference](https://spec.graphql.org/June2018/#SelectionSet).
#[derive(Debug, Default, Clone)]
pub struct SelectionSet {
    /// The fields to be selected.
    pub items: Vec<Positioned<Selection>>,
}

/// A part of an object to be selected; a single field, a fragment spread or an inline fragment.
///
/// [Reference](https://spec.graphql.org/June2018/#Selection).
#[derive(Debug, Clone)]
pub enum Selection {
    /// Select a single field, such as `name` or `weightKilos: weight(unit: KILOGRAMS)`.
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

/// A field being selected on an object, such as `name` or `weightKilos: weight(unit: KILOGRAMS)`.
///
/// [Reference](https://spec.graphql.org/June2018/#Field).
#[derive(Debug, Clone)]
pub struct Field {
    /// The optional field alias.
    pub alias: Option<Positioned<Name>>,
    /// The name of the field.
    pub name: Positioned<Name>,
    /// The arguments to the field, empty if no arguments are provided.
    pub arguments: Vec<(Positioned<Name>, Positioned<Value>)>,
    /// The directives in the field selector.
    pub directives: Vec<Positioned<Directive>>,
    /// The subfields being selected in this field, if it is an object. Empty if no fields are
    /// being selected.
    pub selection_set: Positioned<SelectionSet>,
}

impl Field {
    /// Get the response key of the field. This is the alias if present and the name otherwise.
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
/// [Reference](https://spec.graphql.org/June2018/#FragmentSpread).
#[derive(Debug, Clone)]
pub struct FragmentSpread {
    /// The name of the fragment being selected.
    pub fragment_name: Positioned<Name>,
    /// The directives in the fragment selector.
    pub directives: Vec<Positioned<Directive>>,
}

/// An inline fragment selector, such as `... on User { name }`.
///
/// [Reference](https://spec.graphql.org/June2018/#InlineFragment).
#[derive(Debug, Clone)]
pub struct InlineFragment {
    /// The type condition.
    pub type_condition: Option<Positioned<TypeCondition>>,
    /// The directives in the inline fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The selected fields of the fragment.
    pub selection_set: Positioned<SelectionSet>,
}

/// The definition of a fragment, such as `fragment userFields on User { name age }`.
///
/// [Reference](https://spec.graphql.org/June2018/#FragmentDefinition).
#[derive(Debug, Clone)]
pub struct FragmentDefinition {
    /// The name of the fragment. Any name is allowed except `on`.
    pub name: Positioned<Name>,
    /// The type this fragment operates on.
    pub type_condition: Positioned<TypeCondition>,
    /// Directives in the fragment.
    pub directives: Vec<Positioned<Directive>>,
    /// The fragment's selection set.
    pub selection_set: Positioned<SelectionSet>,
}

/// A type a fragment can apply to (`on` followed by the type).
///
/// [Reference](https://spec.graphql.org/June2018/#TypeCondition).
#[derive(Debug, Clone)]
pub struct TypeCondition {
    /// The type this fragment applies to.
    pub on: Positioned<Name>,
}
