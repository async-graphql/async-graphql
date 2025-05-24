use crate::{
    Name, Positioned,
    model::__DirectiveLocation,
    parser::types::{
        Directive, Field, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
        OperationType,
    },
    validation::visitor::{Visitor, VisitorContext},
};

#[derive(Default)]
pub struct KnownDirectives {
    location_stack: Vec<__DirectiveLocation>,
}

impl<'a> Visitor<'a> for KnownDirectives {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: Option<&'a Name>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        self.location_stack
            .push(match &operation_definition.node.ty {
                OperationType::Query => __DirectiveLocation::QUERY,
                OperationType::Mutation => __DirectiveLocation::MUTATION,
                OperationType::Subscription => __DirectiveLocation::SUBSCRIPTION,
            });
    }

    fn exit_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: Option<&'a Name>,
        _operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        self.location_stack.pop();
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Name,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.location_stack
            .push(__DirectiveLocation::FRAGMENT_DEFINITION);
    }

    fn exit_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Name,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.location_stack.pop();
    }

    fn enter_directive(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        directive: &'a Positioned<Directive>,
    ) {
        if let Some(schema_directive) = ctx
            .registry
            .directives
            .get(directive.node.name.node.as_str())
        {
            if let Some(current_location) = self.location_stack.last() {
                if !schema_directive.locations.contains(current_location) {
                    ctx.report_error(
                        vec![directive.pos],
                        format!(
                            "Directive \"{}\" may not be used on \"{:?}\"",
                            directive.node.name.node, current_location
                        ),
                    )
                }
            }
        } else {
            ctx.report_error(
                vec![directive.pos],
                format!("Unknown directive \"{}\"", directive.node.name.node),
            );
        }
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Positioned<Field>) {
        self.location_stack.push(__DirectiveLocation::FIELD);
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Positioned<Field>) {
        self.location_stack.pop();
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        self.location_stack
            .push(__DirectiveLocation::FRAGMENT_SPREAD);
    }

    fn exit_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        self.location_stack.pop();
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a Positioned<InlineFragment>,
    ) {
        self.location_stack
            .push(__DirectiveLocation::INLINE_FRAGMENT);
    }

    fn exit_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a Positioned<InlineFragment>,
    ) {
        self.location_stack.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> KnownDirectives {
        KnownDirectives::default()
    }

    #[test]
    fn with_no_directives() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            name
            ...Frag
          }
          fragment Frag on Dog {
            name
          }
        "#,
        );
    }

    #[test]
    fn with_known_directives() {
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
    fn with_unknown_directive() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog @unknown(directive: "value") {
              name
            }
          }
        "#,
        );
    }

    #[test]
    fn with_many_unknown_directives() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog @unknown(directive: "value") {
              name
            }
            human @unknown(directive: "value") {
              name
              pets @unknown(directive: "value") {
                name
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn with_well_placed_directives() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            name @include(if: true)
            ...Frag @include(if: true)
            skippedField @skip(if: true)
            ...SkippedFrag @skip(if: true)
          }
          mutation Bar {
            someField
          }
        "#,
        );
    }

    #[test]
    fn with_misplaced_directives() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo @include(if: true) {
            name
            ...Frag
          }
          mutation Bar {
            someField
          }
        "#,
        );
    }
}
