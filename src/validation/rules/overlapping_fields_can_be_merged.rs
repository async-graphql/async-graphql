use crate::parser::types::{Field, Selection, SelectionSet};
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
        for selection in &selection_set.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    let output_name = field
                        .node
                        .alias
                        .as_ref()
                        .map(|name| &name.node)
                        .unwrap_or_else(|| &field.node.name.node);
                    self.add_output(&output_name, field);
                }
                Selection::InlineFragment(inline_fragment) => {
                    self.find(&inline_fragment.node.selection_set);
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) =
                        self.ctx.fragment(&fragment_spread.node.fragment_name.node)
                    {
                        self.find(&fragment.node.selection_set);
                    }
                }
            }
        }
    }

    fn add_output(&mut self, name: &'a str, field: &'a Positioned<Field>) {
        if let Some(prev_field) = self.outputs.get(name) {
            if prev_field.node.name.node != field.node.name.node {
                self.ctx.report_error(
                    vec![prev_field.pos, field.pos],
                    format!("Fields \"{}\" conflict because \"{}\" and \"{}\" are different fields. Use different aliases on the fields to fetch both if this was intentional.",
                            name, prev_field.node.name.node, field.node.name.node));
            }

            // check arguments
            if prev_field.node.arguments.len() != field.node.arguments.len() {
                self.ctx.report_error(
                    vec![prev_field.pos, field.pos],
                    format!("Fields \"{}\" conflict because they have differing arguments. Use different aliases on the fields to fetch both if this was intentional.", name));
            }

            for (name, value) in &prev_field.node.arguments {
                match field.node.get_argument(&name.node) {
                    Some(other_value) if value == other_value => {}
                    _=> self.ctx.report_error(
                        vec![prev_field.pos, field.pos],
                        format!("Fields \"{}\" conflict because they have differing arguments. Use different aliases on the fields to fetch both if this was intentional.", name)),
                }
            }
        } else {
            self.outputs.insert(name, field);
        }
    }
}
