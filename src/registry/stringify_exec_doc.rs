use std::fmt::{Error, Result as FmtResult, Write};

use async_graphql_value::ConstValue;

use crate::parser::types::{
    ExecutableDocument, FragmentDefinition, OperationType, Selection, SelectionSet,
};
use crate::registry::{MetaInputValue, MetaType, MetaTypeName, Registry};
use crate::Variables;

impl Registry {
    pub(crate) fn stringify_exec_doc(
        &self,
        variables: &Variables,
        doc: &ExecutableDocument,
    ) -> Result<String, Error> {
        let mut output = String::new();
        for (name, fragment) in &doc.fragments {
            self.stringify_fragment_definition(
                &mut output,
                variables,
                &*name,
                self.types
                    .get(fragment.node.type_condition.node.on.node.as_str()),
                &fragment.node,
            )?;
        }
        for (name, operation_definition) in doc.operations.iter() {
            write!(&mut output, "{} ", operation_definition.node.ty)?;
            if let Some(name) = name {
                write!(&mut output, "{}", name)?;

                if !operation_definition.node.variable_definitions.is_empty() {
                    output.push('(');
                    for (idx, variable_definition) in operation_definition
                        .node
                        .variable_definitions
                        .iter()
                        .enumerate()
                    {
                        if idx > 0 {
                            output.push_str(", ");
                        }
                        write!(
                            output,
                            "{}: {}",
                            variable_definition.node.name.node,
                            variable_definition.node.var_type.node
                        )?;
                        if let Some(default_value) = &variable_definition.node.default_value {
                            write!(output, " = {}", default_value.node)?;
                        }
                    }
                    output.push(')');
                }

                output.push(' ');
            }
            let root_type = match operation_definition.node.ty {
                OperationType::Query => self.types.get(&self.query_type),
                OperationType::Mutation => self
                    .mutation_type
                    .as_ref()
                    .and_then(|name| self.types.get(name)),
                OperationType::Subscription => self
                    .subscription_type
                    .as_ref()
                    .and_then(|name| self.types.get(name)),
            };
            self.stringify_selection_set(
                &mut output,
                variables,
                &operation_definition.node.selection_set.node,
                root_type,
            )?;
        }
        Ok(output)
    }

    fn stringify_fragment_definition(
        &self,
        output: &mut String,
        variables: &Variables,
        name: &str,
        parent_type: Option<&MetaType>,
        fragment_definition: &FragmentDefinition,
    ) -> FmtResult {
        write!(
            output,
            "fragment {} on {}",
            name, fragment_definition.type_condition.node.on.node
        )?;
        self.stringify_selection_set(
            output,
            variables,
            &fragment_definition.selection_set.node,
            parent_type,
        )?;
        output.push_str("}\n\n");
        Ok(())
    }

    fn stringify_input_value(
        &self,
        output: &mut String,
        meta_input_value: Option<&MetaInputValue>,
        value: &ConstValue,
    ) -> FmtResult {
        if meta_input_value.map(|v| v.is_secret).unwrap_or_default() {
            output.push_str("\"<secret>\"");
            return Ok(());
        }

        match value {
            ConstValue::Object(obj) => {
                let parent_type = meta_input_value.and_then(|input_value| {
                    self.types
                        .get(MetaTypeName::concrete_typename(&input_value.ty))
                });
                if let Some(MetaType::InputObject { input_fields, .. }) = parent_type {
                    output.push('{');
                    for (idx, (key, value)) in obj.iter().enumerate() {
                        if idx > 0 {
                            output.push_str(", ");
                        }
                        write!(output, "{}: ", key)?;
                        self.stringify_input_value(output, input_fields.get(key.as_str()), value)?;
                    }
                    output.push('}');
                } else {
                    write!(output, "{}", value)?;
                }
            }
            _ => write!(output, "{}", value)?,
        }

        Ok(())
    }

    fn stringify_selection_set(
        &self,
        output: &mut String,
        variables: &Variables,
        selection_set: &SelectionSet,
        parent_type: Option<&MetaType>,
    ) -> FmtResult {
        output.push_str("{ ");
        for (idx, selection) in selection_set.items.iter().map(|s| &s.node).enumerate() {
            if idx > 0 {
                output.push(' ');
            }
            match selection {
                Selection::Field(field) => {
                    if let Some(alias) = &field.node.alias {
                        write!(output, "{}:", alias.node)?;
                    }
                    write!(output, "{}", field.node.name.node)?;
                    if !field.node.arguments.is_empty() {
                        output.push('(');
                        for (idx, (name, argument)) in field.node.arguments.iter().enumerate() {
                            let meta_input_value = parent_type
                                .and_then(|parent_type| {
                                    parent_type.field_by_name(field.node.name.node.as_str())
                                })
                                .and_then(|field| field.args.get(name.node.as_str()));
                            if idx > 0 {
                                output.push_str(", ");
                            }
                            write!(output, "{}: ", name)?;
                            let value = argument
                                .node
                                .clone()
                                .into_const_with(|name| variables.get(&name).cloned().ok_or(()))
                                .unwrap_or_default();
                            self.stringify_input_value(output, meta_input_value, &value)?;
                        }
                        output.push(')');
                    }
                    if !field.node.selection_set.node.items.is_empty() {
                        output.push(' ');
                        let parent_type = parent_type
                            .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()))
                            .and_then(|field| {
                                self.types.get(MetaTypeName::concrete_typename(&field.ty))
                            });
                        self.stringify_selection_set(
                            output,
                            variables,
                            &field.node.selection_set.node,
                            parent_type,
                        )?;
                    }
                }
                Selection::FragmentSpread(fragment_spread) => {
                    write!(output, "... {}", fragment_spread.node.fragment_name.node)?;
                }
                Selection::InlineFragment(inline_fragment) => {
                    output.push_str("... ");
                    let parent_type = if let Some(name) = &inline_fragment.node.type_condition {
                        write!(output, "on {} ", name.node.on.node)?;
                        self.types.get(name.node.on.node.as_str())
                    } else {
                        None
                    };
                    self.stringify_selection_set(
                        output,
                        variables,
                        &inline_fragment.node.selection_set.node,
                        parent_type,
                    )?;
                }
            }
        }
        output.push_str(" }");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_query;
    use crate::*;

    #[test]
    fn test_stringify() {
        let registry = Registry::default();
        let doc = parse_query(
            r#"
            query Abc {
              a b c(a:1,b:2) {
                d e f
              }
            }
        "#,
        )
        .unwrap();
        assert_eq!(
            registry
                .stringify_exec_doc(&Default::default(), &doc)
                .unwrap(),
            r#"query Abc { a b c(a: 1, b: 2) { d e f } }"#
        );

        let doc = parse_query(
            r#"
            query Abc($a:Int) {
              value(input:$a)
            }
        "#,
        )
        .unwrap();
        assert_eq!(
            registry
                .stringify_exec_doc(
                    &Variables::from_value(value! ({
                        "a": 10,
                    })),
                    &doc
                )
                .unwrap(),
            r#"query Abc { value(input: 10) }"#
        );
    }

    #[test]
    fn test_stringify_secret() {
        #[derive(InputObject)]
        #[graphql(internal)]
        struct MyInput {
            v1: i32,
            #[graphql(secret)]
            v2: i32,
            v3: MyInput2,
        }

        #[derive(InputObject)]
        #[graphql(internal)]
        struct MyInput2 {
            v4: i32,
            #[graphql(secret)]
            v5: i32,
        }

        struct Query;

        #[Object(internal)]
        #[allow(unreachable_code, unused_variables)]
        impl Query {
            async fn value(&self, a: i32, #[graphql(secret)] b: i32, c: MyInput) -> i32 {
                todo!()
            }
        }

        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let registry = schema.registry();
        let s = registry
            .stringify_exec_doc(
                &Default::default(),
                &parse_query(
                    r#"
            {
                value(a: 10, b: 20, c: { v1: 1, v2: 2, v3: { v4: 4, v5: 5}})
            }
        "#,
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            s,
            r#"query { value(a: 10, b: "<secret>", c: {v1: 1, v2: "<secret>", v3: {v4: 4, v5: "<secret>"}}) }"#
        );
    }
}
