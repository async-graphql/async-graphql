use crate::{
    Positioned,
    parser::types::Field,
    validation::visitor::{Visitor, VisitorContext},
};

#[derive(Default)]
pub struct ScalarLeafs;

impl<'a> Visitor<'a> for ScalarLeafs {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        if let Some(ty) = ctx.parent_type() {
            if let Some(schema_field) = ty.field_by_name(&field.node.name.node) {
                if let Some(ty) = ctx.registry.concrete_type_by_name(&schema_field.ty) {
                    if ty.is_leaf() && !field.node.selection_set.node.items.is_empty() {
                        ctx.report_error(vec![field.pos], format!(
                            "Field \"{}\" must not have a selection since type \"{}\" has no subfields",
                            field.node.name, ty.name()
                        ))
                    } else if !ty.is_leaf() && field.node.selection_set.node.items.is_empty() {
                        ctx.report_error(
                            vec![field.pos],
                            format!(
                                "Field \"{}\" of type \"{}\" must have a selection of subfields",
                                field.node.name,
                                ty.name()
                            ),
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> ScalarLeafs {
        ScalarLeafs
    }

    #[test]
    fn valid_scalar_selection() {
        expect_passes_rule!(
            factory,
            r#"
          fragment scalarSelection on Dog {
            barks
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn object_type_missing_selection() {
        expect_fails_rule!(
            factory,
            r#"
          query directQueryOnObjectWithoutSubFields {
            human
          }
        "#,
        );
    }

    #[test]
    fn interface_type_missing_selection() {
        expect_fails_rule!(
            factory,
            r#"
          {
            human { pets }
          }
        "#,
        );
    }

    #[test]
    fn valid_scalar_selection_with_args() {
        expect_passes_rule!(
            factory,
            r#"
          fragment scalarSelectionWithArgs on Dog {
            doesKnowCommand(dogCommand: SIT)
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn scalar_selection_not_allowed_on_boolean() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarSelectionsNotAllowedOnBoolean on Dog {
            barks { sinceWhen }
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn scalar_selection_not_allowed_on_enum() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarSelectionsNotAllowedOnEnum on Cat {
            furColor { inHexdec }
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn scalar_selection_not_allowed_with_args() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarSelectionsNotAllowedWithArgs on Dog {
            doesKnowCommand(dogCommand: SIT) { sinceWhen }
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn scalar_selection_not_allowed_with_directives() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarSelectionsNotAllowedWithDirectives on Dog {
            name @include(if: true) { isAlsoHumanName }
          }
          { __typename }
        "#,
        );
    }

    #[test]
    fn scalar_selection_not_allowed_with_directives_and_args() {
        expect_fails_rule!(
            factory,
            r#"
          fragment scalarSelectionsNotAllowedWithDirectivesAndArgs on Dog {
            doesKnowCommand(dogCommand: SIT) @include(if: true) { sinceWhen }
          }
          { __typename }
        "#,
        );
    }
}
