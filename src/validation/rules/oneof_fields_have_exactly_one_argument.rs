use crate::validation::visitor::{RuleError, Visitor};
use crate::VisitorContext;
use async_graphql_parser::types::Field;
use async_graphql_parser::Positioned;

pub struct OneofFieldsHaveExactlyOneArgument;

impl<'a> Visitor<'a> for OneofFieldsHaveExactlyOneArgument {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(field_def) = parent_type
                .fields()
                .and_then(|fields| fields.get(field.node.name.node.as_str()))
            {
                if field_def.oneof && field.node.arguments.len() != 1 {
                    ctx.errors.push(RuleError::new(
                        vec![field.pos],
                        "Oneof fields requires have exactly one argument".to_string(),
                    ));
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
}
