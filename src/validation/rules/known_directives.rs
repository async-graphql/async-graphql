use crate::model::__DirectiveLocation;
use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{
    Field, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
};
use graphql_parser::schema::Directive;

#[derive(Default)]
pub struct KnownDirectives {
    location_stack: Vec<__DirectiveLocation>,
}

impl<'a> Visitor<'a> for KnownDirectives {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        self.location_stack.push(match operation_definition {
            OperationDefinition::SelectionSet(_) | OperationDefinition::Query(_) => {
                __DirectiveLocation::QUERY
            }
            OperationDefinition::Mutation(_) => __DirectiveLocation::MUTATION,
            OperationDefinition::Subscription(_) => __DirectiveLocation::SUBSCRIPTION,
        });
    }

    fn exit_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.location_stack.pop();
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_definition: &'a FragmentDefinition,
    ) {
        self.location_stack
            .push(__DirectiveLocation::FRAGMENT_DEFINITION);
    }

    fn exit_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_definition: &'a FragmentDefinition,
    ) {
        self.location_stack.pop();
    }

    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Directive) {
        if let Some(schema_directive) = ctx.registry.directives.get(directive.name.as_str()) {
            if let Some(current_location) = self.location_stack.last() {
                if !schema_directive.locations.contains(current_location) {
                    ctx.report_error(
                        vec![directive.position],
                        format!(
                            "Directive \"{}\" may not be used on \"{:?}\"",
                            directive.name, current_location
                        ),
                    )
                }
            }
        } else {
            ctx.report_error(
                vec![directive.position],
                format!("Unknown directive \"{}\"", directive.name),
            );
        }
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Field) {
        self.location_stack.push(__DirectiveLocation::FIELD);
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Field) {
        self.location_stack.pop();
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a FragmentSpread,
    ) {
        self.location_stack
            .push(__DirectiveLocation::FRAGMENT_SPREAD);
    }

    fn exit_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_spread: &'a FragmentSpread,
    ) {
        self.location_stack.pop();
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a InlineFragment,
    ) {
        self.location_stack
            .push(__DirectiveLocation::INLINE_FRAGMENT);
    }

    fn exit_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _inline_fragment: &'a InlineFragment,
    ) {
        self.location_stack.pop();
    }
}
