use crate::parser::query::{Directive, Field};
use crate::registry::MetaTypeName;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct ProvidedNonNullArguments;

impl<'a> Visitor<'a> for ProvidedNonNullArguments {
    fn enter_directive(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        directive: &'a Positioned<Directive>,
    ) {
        if let Some(schema_directive) = ctx.registry.directives.get(directive.name.as_str()) {
            for arg in schema_directive.args.values() {
                if MetaTypeName::create(&arg.ty).is_non_null()
                    && arg.default_value.is_none()
                    && directive
                        .arguments
                        .iter()
                        .find(|(name, _)| name.node == arg.name)
                        .is_none()
                {
                    ctx.report_error(vec![directive.position()],
                            format!(
                                "Directive \"@{}\" argument \"{}\" of type \"{}\" is required but not provided",
                                directive.name, arg.name, arg.ty
                            ));
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(schema_field) = parent_type.field_by_name(&field.name) {
                for arg in schema_field.args.values() {
                    if MetaTypeName::create(&arg.ty).is_non_null()
                        && arg.default_value.is_none()
                        && field
                            .arguments
                            .iter()
                            .find(|(name, _)| name.node == arg.name)
                            .is_none()
                    {
                        ctx.report_error(vec![field.position()],
                             format!(
                                 r#"Field "{}" argument "{}" of type "{}" is required but not provided"#,
                                 field.name, arg.name, parent_type.name()
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
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory() -> ProvidedNonNullArguments {
        ProvidedNonNullArguments
    }

    #[test]
    fn ignores_unknown_arguments() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              isHousetrained(unknownArgument: true)
            }
          }
        "#,
        );
    }

    #[test]
    fn arg_on_optional_arg() {
        expect_passes_rule!(
            factory,
            r#"
            {
              dog {
                isHousetrained(atOtherHomes: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn no_arg_on_optional_arg() {
        expect_passes_rule!(
            factory,
            r#"
            {
              dog {
                isHousetrained
              }
            }
        "#,
        );
    }

    #[test]
    fn multiple_args() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs(req1: 1, req2: 2)
              }
            }
        "#,
        );
    }

    #[test]
    fn multiple_args_reverse_order() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs(req2: 2, req1: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn no_args_on_multiple_optional() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOpts
              }
            }
        "#,
        );
    }

    #[test]
    fn one_arg_on_multiple_optional() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOpts(opt1: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn second_arg_on_multiple_optional() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOpts(opt2: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn muliple_reqs_on_mixed_list() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOptAndReq(req1: 3, req2: 4)
              }
            }
        "#,
        );
    }

    #[test]
    fn multiple_reqs_and_one_opt_on_mixed_list() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOptAndReq(req1: 3, req2: 4, opt1: 5)
              }
            }
        "#,
        );
    }

    #[test]
    fn all_reqs_on_opts_on_mixed_list() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleOptAndReq(req1: 3, req2: 4, opt1: 5, opt2: 6)
              }
            }
        "#,
        );
    }

    #[test]
    fn missing_one_non_nullable_argument() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs(req2: 2)
              }
            }
        "#,
        );
    }

    #[test]
    fn missing_multiple_non_nullable_arguments() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs
              }
            }
        "#,
        );
    }

    #[test]
    fn incorrect_value_and_missing_argument() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs(req1: "one")
              }
            }
        "#,
        );
    }

    #[test]
    fn ignores_unknown_directives() {
        expect_passes_rule!(
            factory,
            r#"
            {
              dog @unknown
            }
        "#,
        );
    }

    #[test]
    fn with_directives_of_valid_types() {
        expect_passes_rule!(
            factory,
            r#"
            {
              dog @include(if: true) {
                name
              }
              human @skip(if: false) {
                name
              }
            }
        "#,
        );
    }

    #[test]
    fn with_directive_with_missing_types() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog @include {
                name @skip
              }
            }
        "#,
        );
    }
}
