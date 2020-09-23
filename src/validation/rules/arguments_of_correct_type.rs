use crate::context::QueryPathNode;
use crate::parser::types::{Directive, Field, Name, Value};
use crate::registry::MetaInputValue;
use crate::validation::utils::is_valid_input_value;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::{Positioned, QueryPathSegment};
use indexmap::map::IndexMap;

#[derive(Default)]
pub struct ArgumentsOfCorrectType<'a> {
    current_args: Option<&'a IndexMap<&'static str, MetaInputValue>>,
}

impl<'a> Visitor<'a> for ArgumentsOfCorrectType<'a> {
    fn enter_directive(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        directive: &'a Positioned<Directive>,
    ) {
        self.current_args = ctx
            .registry
            .directives
            .get(directive.node.name.node.as_str())
            .map(|d| &d.args);
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
        value: &'a Positioned<Value>,
    ) {
        if let Some(arg) = self
            .current_args
            .and_then(|args| args.get(name.node.as_str()).map(|input| input))
        {
            let value = value
                .node
                .clone()
                .into_const_with(|var_name| {
                    ctx.variables
                        .and_then(|variables| variables.0.get(&var_name))
                        .map(Clone::clone)
                        .ok_or(())
                })
                .ok();

            if let Some(validator) = &arg.validator {
                if let Some(value) = &value {
                    if let Err(reason) = validator.is_valid(value) {
                        ctx.report_error(
                            vec![name.pos],
                            format!("Invalid value for argument \"{}\", {}", arg.name, reason),
                        );
                        return;
                    }
                }
            }

            if let Some(reason) = value.and_then(|value| {
                is_valid_input_value(
                    ctx.registry,
                    ctx.variables,
                    &arg.ty,
                    &value,
                    QueryPathNode {
                        parent: None,
                        segment: QueryPathSegment::Name(arg.name),
                    },
                )
            }) {
                ctx.report_error(
                    vec![name.pos],
                    format!("Invalid value for argument {}", reason),
                );
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        self.current_args = ctx
            .parent_type()
            .and_then(|p| p.field_by_name(&field.node.name.node))
            .map(|f| &f.args);
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Positioned<Field>) {
        self.current_args = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory<'a>() -> ArgumentsOfCorrectType<'a> {
        ArgumentsOfCorrectType::default()
    }

    #[test]
    fn good_null_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: null)
              }
            }
        "#,
        );
    }

    #[test]
    fn null_into_int() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                nonNullIntArgField(nonNullIntArg: null)
              }
            }
        "#,
        );
    }

    #[test]
    fn good_int_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: 2)
              }
            }
        "#,
        );
    }

    #[test]
    fn good_boolean_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                booleanArgField(booleanArg: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn good_string_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringArgField(stringArg: "foo")
              }
            }
        "#,
        );
    }

    #[test]
    fn good_float_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                floatArgField(floatArg: 1.1)
              }
            }
        "#,
        );
    }

    #[test]
    fn int_into_float() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                floatArgField(floatArg: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn int_into_id() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                idArgField(idArg: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn string_into_id() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                idArgField(idArg: "someIdString")
              }
            }
        "#,
        );
    }

    #[test]
    fn good_enum_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: SIT)
              }
            }
        "#,
        );
    }

    #[test]
    fn int_into_string() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringArgField(stringArg: 1)
              }
            }
        "#,
        );
    }

    #[test]
    fn float_into_string() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringArgField(stringArg: 1.0)
              }
            }
        "#,
        );
    }

    #[test]
    fn boolean_into_string() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringArgField(stringArg: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn unquoted_string_into_string() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringArgField(stringArg: BAR)
              }
            }
        "#,
        );
    }

    #[test]
    fn string_into_int() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: "3")
              }
            }
        "#,
        );
    }

    #[test]
    fn unquoted_string_into_int() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: FOO)
              }
            }
        "#,
        );
    }

    #[test]
    fn simple_float_into_int() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: 3.0)
              }
            }
        "#,
        );
    }

    #[test]
    fn float_into_int() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                intArgField(intArg: 3.333)
              }
            }
        "#,
        );
    }

    #[test]
    fn string_into_float() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                floatArgField(floatArg: "3.333")
              }
            }
        "#,
        );
    }

    #[test]
    fn boolean_into_float() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                floatArgField(floatArg: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn unquoted_into_float() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                floatArgField(floatArg: FOO)
              }
            }
        "#,
        );
    }

    #[test]
    fn int_into_boolean() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                booleanArgField(booleanArg: 2)
              }
            }
        "#,
        );
    }

    #[test]
    fn float_into_boolean() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                booleanArgField(booleanArg: 1.0)
              }
            }
        "#,
        );
    }

    #[test]
    fn string_into_boolean() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                booleanArgField(booleanArg: "true")
              }
            }
        "#,
        );
    }

    #[test]
    fn unquoted_into_boolean() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                booleanArgField(booleanArg: TRUE)
              }
            }
        "#,
        );
    }

    #[test]
    fn float_into_id() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                idArgField(idArg: 1.0)
              }
            }
        "#,
        );
    }

    #[test]
    fn boolean_into_id() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                idArgField(idArg: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn unquoted_into_id() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                idArgField(idArg: SOMETHING)
              }
            }
        "#,
        );
    }

    #[test]
    fn int_into_enum() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: 2)
              }
            }
        "#,
        );
    }

    #[test]
    fn float_into_enum() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: 1.0)
              }
            }
        "#,
        );
    }

    // #[test]
    // fn string_into_enum() {
    //     expect_fails_rule!(
    //         factory,
    //         r#"
    //         {
    //           dog {
    //             doesKnowCommand(dogCommand: "SIT")
    //           }
    //         }
    //     "#,
    //     );
    // }

    #[test]
    fn boolean_into_enum() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: true)
              }
            }
        "#,
        );
    }

    #[test]
    fn unknown_enum_value_into_enum() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: JUGGLE)
              }
            }
        "#,
        );
    }

    #[test]
    fn different_case_enum_value_into_enum() {
        expect_fails_rule!(
            factory,
            r#"
            {
              dog {
                doesKnowCommand(dogCommand: sit)
              }
            }
        "#,
        );
    }

    #[test]
    fn good_list_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringListArgField(stringListArg: ["one", "two"])
              }
            }
        "#,
        );
    }

    #[test]
    fn empty_list_value() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringListArgField(stringListArg: [])
              }
            }
        "#,
        );
    }

    #[test]
    fn single_value_into_list() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringListArgField(stringListArg: "one")
              }
            }
        "#,
        );
    }

    #[test]
    fn incorrect_item_type() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringListArgField(stringListArg: ["one", 2])
              }
            }
        "#,
        );
    }

    #[test]
    fn single_value_of_incorrect_type() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                stringListArgField(stringListArg: 1)
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
    fn multiple_reqs_on_mixed_list() {
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
    fn all_reqs_and_opts_on_mixed_list() {
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
    fn incorrect_value_type() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                multipleReqs(req2: "two", req1: "one")
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
    fn optional_arg_despite_required_field_in_type() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_only_required() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: { requiredField: true })
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_required_field_can_be_falsy() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: { requiredField: false })
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_including_required() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: { requiredField: true, intField: 4 })
              }
            }
        "#,
        );
    }

    #[test]
    fn full_object() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: {
                  requiredField: true,
                  intField: 4,
                  stringField: "foo",
                  booleanField: false,
                  stringListField: ["one", "two"]
                })
              }
            }
        "#,
        );
    }

    #[test]
    fn full_object_with_fields_in_different_order() {
        expect_passes_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: {
                  stringListField: ["one", "two"],
                  booleanField: false,
                  requiredField: true,
                  stringField: "foo",
                  intField: 4,
                })
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_missing_required() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: { intField: 4 })
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_invalid_field_type() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: {
                  stringListField: ["one", 2],
                  requiredField: true,
                })
              }
            }
        "#,
        );
    }

    #[test]
    fn partial_object_unknown_field_arg() {
        expect_fails_rule!(
            factory,
            r#"
            {
              complicatedArgs {
                complexArgField(complexArg: {
                  requiredField: true,
                  unknownField: "value"
                })
              }
            }
        "#,
        );
    }

    #[test]
    fn directive_with_valid_types() {
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
    fn directive_with_incorrect_types() {
        expect_fails_rule!(
            factory,
            r#"
        {
          dog @include(if: "yes") {
            name @skip(if: ENUM)
          }
        }
        "#,
        );
    }
}
