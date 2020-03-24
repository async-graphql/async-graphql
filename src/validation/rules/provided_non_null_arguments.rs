use crate::registry::TypeName;
use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::Field;
use graphql_parser::schema::Directive;

#[derive(Default)]
pub struct ProvidedNonNullArguments;

impl<'a> Visitor<'a> for ProvidedNonNullArguments {
    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Directive) {
        if let Some(schema_directive) = ctx.registry.directives.get(&directive.name) {
            for arg in schema_directive.args.values() {
                if TypeName::create(&arg.ty).is_non_null()
                    && arg.default_value.is_none()
                    && directive
                        .arguments
                        .iter()
                        .find(|(name, _)| name == arg.name)
                        .is_none()
                {
                    ctx.report_error(vec![directive.position],
                            format!(
                                "Directive \"@{}\" argument \"{}\" of type \"{}\" is required but not provided",
                                directive.name, arg.name, arg.ty
                            ));
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(schema_field) = parent_type.field_by_name(&field.name) {
                for arg in schema_field.args.values() {
                    if TypeName::create(&arg.ty).is_non_null()
                        && arg.default_value.is_none()
                        && field
                            .arguments
                            .iter()
                            .find(|(name, _)| name == arg.name)
                            .is_none()
                    {
                        ctx.report_error(vec![field.position],
                             format!(
                                 r#"Field "{}" argument "{}" of type "{}" is required but not provided"#,
                                 field.name, arg.name, parent_type.name()
                             ));
                    }
                }
            }
        }
    }
}
