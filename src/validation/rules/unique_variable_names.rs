use crate::parser::types::{OperationDefinition, VariableDefinition};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueVariableNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueVariableNames<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        self.names.clear();
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        if !self.names.insert(&variable_definition.node.name.node) {
            ctx.report_error(
                vec![variable_definition.pos],
                format!(
                    "There can only be one variable named \"${}\"",
                    variable_definition.node.name.node
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory<'a>() -> UniqueVariableNames<'a> {
        UniqueVariableNames::default()
    }

    #[test]
    fn unique_variable_names() {
        expect_passes_rule!(
            factory,
            r#"
          query A($x: Int, $y: String) { __typename }
          query B($x: String, $y: Int) { __typename }
        "#,
        );
    }

    #[test]
    fn duplicate_variable_names() {
        expect_fails_rule!(
            factory,
            r#"
          query A($x: Int, $x: Int, $x: String) { __typename }
          query B($x: String, $x: Int) { __typename }
          query C($x: Int, $x: Int) { __typename }
        "#,
        );
    }
}
