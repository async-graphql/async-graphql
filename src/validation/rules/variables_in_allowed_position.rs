use crate::registry::TypeName;
use crate::visitor::{Visitor, VisitorContext};
use crate::Value;
use graphql_parser::query::{Field, OperationDefinition, VariableDefinition};
use graphql_parser::schema::Directive;
use graphql_parser::Pos;
use std::collections::HashMap;

#[derive(Default)]
pub struct VariableInAllowedPosition<'a> {
    var_types: HashMap<&'a str, String>,
}

impl<'a> VariableInAllowedPosition<'a> {
    fn check_type(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        pos: Pos,
        except_type: &str,
        value: &Value,
    ) {
        let ty = TypeName::create(except_type);
        match (ty, value) {
            (_, Value::Variable(name)) => {
                if let Some(var_type) = self.var_types.get(name.as_str()) {
                    if except_type != var_type {
                        ctx.report_error(
                            vec![pos],
                            format!(
                                "Variable \"{}\" of type \"{}\" used in position expecting type \"{}\"",
                                name, var_type, except_type
                            ),
                        );
                    }
                }
            }
            (TypeName::List(elem_type), Value::List(values)) => {
                for value in values {
                    self.check_type(ctx, pos, elem_type, value);
                }
            }
            (TypeName::NonNull(elem_type), value) => {
                self.check_type(ctx, pos, elem_type, value);
            }
            _ => {}
        }
    }
}

impl<'a> Visitor<'a> for VariableInAllowedPosition<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.var_types.clear();
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        self.var_types.insert(
            variable_definition.name.as_str(),
            variable_definition.var_type.to_string(),
        );
    }

    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Directive) {
        if let Some(schema_directive) = ctx.registry.directives.get(directive.name.as_str()) {
            for (name, value) in &directive.arguments {
                if let Some(input) = schema_directive.args.get(name.as_str()) {
                    self.check_type(ctx, directive.position, &input.ty, value);
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(schema_field) = parent_type.field_by_name(&field.name) {
                for (name, value) in &field.arguments {
                    if let Some(arg) = schema_field.args.get(name.as_str()) {
                        self.check_type(ctx, field.position, &arg.ty, value);
                    }
                }
            }
        }
    }
}
