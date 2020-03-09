use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{Definition, Document, OperationDefinition};

#[derive(Default)]
pub struct LoneAnonymousOperation {
    operation_count: Option<usize>,
}

impl<'a> Visitor<'a> for LoneAnonymousOperation {
    fn enter_document(&mut self, _ctx: &mut ValidatorContext<'a>, doc: &'a Document) {
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
        ctx: &mut ValidatorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        if let Some(operation_count) = self.operation_count {
            if let OperationDefinition::SelectionSet(s) = operation_definition {
                if operation_count > 1 {
                    ctx.report_error(
                        vec![s.span.0, s.span.1],
                        "This anonymous operation must be the only defined operation",
                    );
                }
            }
        }
    }
}
