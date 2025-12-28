use std::collections::HashSet;

use crate::{
    Name, Positioned, VisitorContext,
    parser::types::{
        Directive, Field, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
        VariableDefinition,
    },
    validation::visitor::Visitor,
};

#[derive(Default)]
pub struct DirectivesUnique;

impl<'a> Visitor<'a> for DirectivesUnique {
    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        _name: Option<&'a Name>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        check_duplicate_directive(ctx, &operation_definition.node.directives);
    }

    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        _name: &'a Name,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        check_duplicate_directive(ctx, &fragment_definition.node.directives);
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        check_duplicate_directive(ctx, &variable_definition.node.directives);
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Positioned<Field>) {
        check_duplicate_directive(ctx, &field.node.directives);
    }

    fn enter_fragment_spread(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        check_duplicate_directive(ctx, &fragment_spread.node.directives);
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        inline_fragment: &'a Positioned<InlineFragment>,
    ) {
        check_duplicate_directive(ctx, &inline_fragment.node.directives);
    }
}

fn check_duplicate_directive(ctx: &mut VisitorContext<'_>, directives: &[Positioned<Directive>]) {
    let mut exists = HashSet::new();

    for directive in directives {
        let name = &directive.node.name.node;
        if let Some(meta_directive) = ctx.registry.directives.get(name.as_str())
            && !meta_directive.is_repeatable
        {
            if exists.contains(name) {
                ctx.report_error(
                    vec![directive.pos],
                    format!("Duplicate directive \"{}\"", name),
                );
                continue;
            }
            exists.insert(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> DirectivesUnique {
        DirectivesUnique
    }

    #[test]
    fn skip_on_field() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              name @skip(if: true)
            }
          }
        "#,
        );
    }

    #[test]
    fn duplicate_skip_on_field() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog {
              name @skip(if: true) @skip(if: false)
            }
          }
        "#,
        );
    }

    #[test]
    fn skip_on_fragment_spread() {
        expect_passes_rule!(
            factory,
            r#"
          fragment A on Dog {
            name
          }
          
          query {
            dog ... A @skip(if: true)
          }
        "#,
        );
    }

    #[test]
    fn duplicate_skip_on_fragment_spread() {
        expect_fails_rule!(
            factory,
            r#"
          fragment A on Dog {
            name
          }
          
          query {
            dog ... A @skip(if: true) @skip(if: false)
          }
        "#,
        );
    }

    #[test]
    fn skip_on_inline_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          query {
            dog ... @skip(if: true) {
                name
            }
          }
        "#,
        );
    }

    #[test]
    fn duplicate_skip_on_inline_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          query {
            dog ... @skip(if: true) @skip(if: false) {
                name
            }
          }
        "#,
        );
    }
}
