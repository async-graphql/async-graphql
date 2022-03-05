use async_graphql_parser::types::Field;
use async_graphql_parser::Positioned;

use crate::validation::visitor::{RuleError, Visitor};
use crate::{Value, VisitorContext};

pub struct OneofFieldsHaveExactlyOneArgument;

impl<'a> Visitor<'a> for OneofFieldsHaveExactlyOneArgument {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(field_def) = parent_type
                .fields()
                .and_then(|fields| fields.get(field.node.name.node.as_str()))
            {
                if field_def.oneof {
                    if field.node.arguments.len() != 1 {
                        ctx.errors.push(RuleError::new(
                            vec![field.pos],
                            "Oneof fields requires have exactly one argument".to_string(),
                        ));
                        return;
                    }

                    let value = field.node.arguments[0]
                        .1
                        .node
                        .clone()
                        .into_const_with(|var_name| {
                            ctx.variables
                                .and_then(|variables| variables.get(&var_name))
                                .map(Clone::clone)
                                .ok_or(())
                        })
                        .ok();
                    if let Some(Value::Null) = value {
                        ctx.errors.push(RuleError::new(
                            vec![field.pos],
                            "Oneof Fields require that exactly one argument must be supplied and that argument must not be null".to_string(),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory() -> OneofFieldsHaveExactlyOneArgument {
        OneofFieldsHaveExactlyOneArgument
    }

    #[test]
    fn oneof_field() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            oneofField(a: 10)
          }
        "#,
        );
    }

    #[test]
    fn oneof_not_exactly_one_argument() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            oneofField(a: 10, b: "abc")
          }
        "#,
        );

        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            oneofField
          }
        "#,
        );
    }

    #[test]
    fn oneof_arguments_not_be_null() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            oneofField(a: null)
          }
        "#,
        );
    }
}
