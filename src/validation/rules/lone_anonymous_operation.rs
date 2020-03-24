use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{Definition, Document, OperationDefinition};

#[derive(Default)]
pub struct LoneAnonymousOperation {
    operation_count: Option<usize>,
}

impl<'a> Visitor<'a> for LoneAnonymousOperation {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, doc: &'a Document) {
        self.operation_count = Some(
            doc.definitions
                .iter()
                .filter(|d| match d {
                    Definition::Operation(_) => true,
                    Definition::Fragment(_) => false,
                })
                .count(),
        );
    }

    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        if let Some(operation_count) = self.operation_count {
            let (err, pos) = match operation_definition {
                OperationDefinition::SelectionSet(s) => (operation_count > 1, s.span.0),
                OperationDefinition::Query(query) if query.name.is_none() => {
                    (operation_count > 1, query.position)
                }
                OperationDefinition::Mutation(mutation) if mutation.name.is_none() => {
                    (operation_count > 1, mutation.position)
                }
                OperationDefinition::Subscription(subscription) if subscription.name.is_none() => {
                    (operation_count > 1, subscription.position)
                }
                _ => {
                    return;
                }
            };

            if err {
                ctx.report_error(
                    vec![pos],
                    "This anonymous operation must be the only defined operation",
                );
            }
        }
    }
}
