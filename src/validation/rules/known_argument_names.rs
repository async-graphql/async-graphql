use async_graphql_value::Value;
use indexmap::map::IndexMap;

use crate::{
    parser::types::{Directive, Field},
    registry::MetaInputValue,
    validation::{
        suggestion::make_suggestion,
        visitor::{Visitor, VisitorContext},
    },
    Name, Positioned,
};

enum ArgsType<'a> {
    Directive(&'a str),
    Field {
        field_name: &'a str,
        type_name: &'a str,
    },
}

#[derive(Default)]
pub struct KnownArgumentNames<'a> {
    current_args: Option<(&'a IndexMap<String, MetaInputValue>, ArgsType<'a>)>,
}

impl KnownArgumentNames<'_> {
    fn get_suggestion(&self, name: &str) -> String {
        make_suggestion(
            " Did you mean",
            self.current_args
                .iter()
                .map(|(args, _)| args.iter().map(|arg| arg.0.as_str()))
                .flatten(),
            name,
        )
        .unwrap_or_default()
    }
}

impl<'a> Visitor<'a> for KnownArgumentNames<'a> {
    fn enter_directive(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        directive: &'a Positioned<Directive>,
    ) {
        self.current_args = ctx
            .registry
            .directives
            .get(directive.node.name.node.as_str())
            .map(|d| (&d.args, ArgsType::Directive(&directive.node.name.node)));
    }

    fn exit_directive(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _directive: &'a Positioned<Directive>,
    ) {
        self.current_args = None;
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        _value: &'a Positioned<Value>,
    ) {
        if let Some((args, arg_type)) = &self.current_args {
            if !args.contains_key(name.node.as_str()) {
                match arg_type {
                    ArgsType::Field {
                        field_name,
                        type_name,
                    } => {
                        ctx.report_error(
                            vec![name.pos],
                            format!(
                                "Unknown argument \"{}\" on field \"{}\" of type \"{}\".{}",
                                name,
                                field_name,
                                type_name,
                                if ctx.registry.enable_suggestions {
                                    self.get_suggestion(name.node.as_str())
                                } else {
                                    String::new()
                                }
                            ),
                        );
                    }
                    ArgsType::Directive(directive_name) => {
                        ctx.report_error(
                            vec![name.pos],
                            format!(
                                "Unknown argument \"{}\" on directive \"{}\".{}",
                                name,
                                directive_name,
                                self.get_suggestion(name.node.as_str())
                            ),
                        );
                    }
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(schema_field) = parent_type.field_by_name(&field.node.name.node) {
                self.current_args = Some((
                    &schema_field.args,
                    ArgsType::Field {
                        field_name: &field.node.name.node,
                        type_name: ctx.parent_type().unwrap().name(),
                    },
                ));
            }
        }
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Positioned<Field>) {
        self.current_args = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory<'a>() -> KnownArgumentNames<'a> {
        KnownArgumentNames::default()
    }

    #[test]
    fn single_arg_is_known() {
        expect_passes_rule!(
            factory,
            r#"
          fragment argOnRequiredArg on Dog {
            doesKnowCommand(dogCommand: SIT)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn multiple_args_are_known() {
        expect_passes_rule!(
            factory,
            r#"
          fragment multipleArgs on ComplicatedArgs {
            multipleReqs(req1: 1, req2: 2)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn ignores_args_of_unknown_fields() {
        expect_passes_rule!(
            factory,
            r#"
          fragment argOnUnknownField on Dog {
            unknownField(unknownArg: SIT)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn multiple_args_in_reverse_order_are_known() {
        expect_passes_rule!(
            factory,
            r#"
          fragment multipleArgsReverseOrder on ComplicatedArgs {
            multipleReqs(req2: 2, req1: 1)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn no_args_on_optional_arg() {
        expect_passes_rule!(
            factory,
            r#"
          fragment noArgOnOptionalArg on Dog {
            isHousetrained
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn args_are_known_deeply() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              doesKnowCommand(dogCommand: SIT)
            }
            human {
              pet {
                ... on Dog {
                  doesKnowCommand(dogCommand: SIT)
                }
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn directive_args_are_known() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog @skip(if: true)
          }
        "#,
        );
    }

    #[test]
    fn undirective_args_are_invalid() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog @skip(unless: true)
          }
        "#,
        );
    }

    #[test]
    fn invalid_arg_name() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidArgName on Dog {
            doesKnowCommand(unknown: true)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn unknown_args_amongst_known_args() {
        expect_fails_rule!(
            factory,
            r#"
          fragment oneGoodArgOneInvalidArg on Dog {
            doesKnowCommand(whoknows: 1, dogCommand: SIT, unknown: true)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn unknown_args_deeply() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog {
              doesKnowCommand(unknown: true)
            }
            human {
              pet {
                ... on Dog {
                  doesKnowCommand(unknown: true)
                }
              }
            }
          }
        "#,
        );
    }
}
