use crate::parser::query::{Field, Selection, SelectionSet};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;
use std::collections::HashMap;

#[derive(Default)]
pub struct OverlappingFieldsCanBeMerged;

impl<'a> Visitor<'a> for OverlappingFieldsCanBeMerged {
    fn enter_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        selection_set: &'a Positioned<SelectionSet>,
    ) {
        let mut find_conflicts = FindConflicts {
            outputs: Default::default(),
            ctx,
        };
        find_conflicts.find(selection_set);
    }
}

struct FindConflicts<'a, 'ctx> {
    outputs: HashMap<&'a str, &'a Positioned<Field>>,
    ctx: &'a mut VisitorContext<'ctx>,
}

impl<'a, 'ctx> FindConflicts<'a, 'ctx> {
    pub fn find(&mut self, selection_set: &'a Positioned<SelectionSet>) {
        for selection in &selection_set.items {
            match &selection.node {
                Selection::Field(field) => {
                    let output_name = field
                        .alias
                        .as_ref()
                        .map(|name| name.node)
                        .unwrap_or_else(|| field.name.node);
                    self.add_output(output_name, field);
                }
                Selection::InlineFragment(inline_fragment) => {
                    self.find(&inline_fragment.selection_set);
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) = self.ctx.fragment(&fragment_spread.fragment_name) {
                        self.find(&fragment.selection_set);
                    }
                }
            }
        }
    }

    fn add_output(&mut self, name: &'a str, field: &'a Positioned<Field>) {
        if let Some(prev_field) = self.outputs.get(name) {
            if prev_field.name != field.name {
                self.ctx.report_error(
                    vec![prev_field.position(), field.position()],
                    format!("Fields \"{}\" conflict because \"{}\" and \"{}\" are different fields. Use different aliases on the fields to fetch both if this was intentional.",
                            name, prev_field.name, field.name));
            }

            // check arguments
            if prev_field.arguments.len() != field.arguments.len() {
                self.ctx.report_error(
                    vec![prev_field.position(), field.position()],
                    format!("Fields \"{}\" conflict because they have differing arguments. Use different aliases on the fields to fetch both if this was intentional.", name));
            }

            for (name, value) in &prev_field.arguments {
                match field.get_argument(name.node)
                {
                    Some(other_value) if value == other_value => {}
                    _=> self.ctx.report_error(
                        vec![prev_field.position(), field.position()],
                        format!("Fields \"{}\" conflict because they have differing arguments. Use different aliases on the fields to fetch both if this was intentional.", name)),
                }
            }
        } else {
            self.outputs.insert(name, field);
        }
    }
}
