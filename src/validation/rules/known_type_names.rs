use crate::registry::TypeName;
use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{
    FragmentDefinition, InlineFragment, TypeCondition, VariableDefinition,
};
use graphql_parser::Pos;

#[derive(Default)]
pub struct KnownTypeNames;

impl<'a> Visitor<'a> for KnownTypeNames {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a FragmentDefinition,
    ) {
        let TypeCondition::On(name) = &fragment_definition.type_condition;
        validate_type(ctx, &name, fragment_definition.position);
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        validate_type(
            ctx,
            TypeName::get_basic_typename(&variable_definition.var_type.to_string()),
            variable_definition.position,
        );
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        inline_fragment: &'a InlineFragment,
    ) {
        if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
            validate_type(ctx, &name, inline_fragment.position);
        }
    }
}

fn validate_type(ctx: &mut VisitorContext<'_>, type_name: &str, pos: Pos) {
    if ctx.registry.types.get(type_name).is_none() {
        ctx.report_error(vec![pos], format!(r#"Unknown type "{}""#, type_name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::test_harness::{expect_fails_rule, expect_passes_rule};

    pub fn factory() -> KnownTypeNames {
        KnownTypeNames::default()
    }

    #[test]
    fn known_type_names_are_valid() {
        expect_passes_rule(
            factory,
            r#"
          query Foo($var: String, $required: [String!]!) {
            user(id: 4) {
              pets { ... on Pet { name }, ...PetFields, ... { name } }
            }
          }
          fragment PetFields on Pet {
            name
          }
        "#,
        );
    }

    #[test]
    fn unknown_type_names_are_invalid() {
        expect_fails_rule(
            factory,
            r#"
          query Foo($var: JumbledUpLetters) {
            user(id: 4) {
              name
              pets { ... on Badger { name }, ...PetFields }
            }
          }
          fragment PetFields on Peettt {
            name
          }
        "#,
        );
    }
}
