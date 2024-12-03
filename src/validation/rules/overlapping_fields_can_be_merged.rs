use std::collections::{HashMap, HashSet};

use crate::{
    parser::types::{Field, Selection, SelectionSet},
    validation::visitor::{Visitor, VisitorContext},
    Positioned,
};

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
            visited: Default::default(),
            ctx,
        };
        find_conflicts.find(None, selection_set);
    }
}

struct FindConflicts<'a, 'ctx> {
    outputs: HashMap<(Option<&'a str>, &'a str), &'a Positioned<Field>>,
    visited: HashSet<&'a str>,
    ctx: &'a mut VisitorContext<'ctx>,
}

impl<'a> FindConflicts<'a, '_> {
    pub fn find(&mut self, on_type: Option<&'a str>, selection_set: &'a Positioned<SelectionSet>) {
        for selection in &selection_set.node.items {
            match &selection.node {
                Selection::Field(field) => {
                    let output_name = field
                        .node
                        .alias
                        .as_ref()
                        .map(|name| &name.node)
                        .unwrap_or_else(|| &field.node.name.node);
                    self.add_output(on_type, &output_name, field);
                }
                Selection::InlineFragment(inline_fragment) => {
                    let on_type = inline_fragment
                        .node
                        .type_condition
                        .as_ref()
                        .map(|cond| cond.node.on.node.as_str());
                    self.find(on_type, &inline_fragment.node.selection_set);
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) =
                        self.ctx.fragment(&fragment_spread.node.fragment_name.node)
                    {
                        let on_type = Some(fragment.node.type_condition.node.on.node.as_str());

                        if !self
                            .visited
                            .insert(fragment_spread.node.fragment_name.node.as_str())
                        {
                            // To avoid recursing itself, this error is detected by the
                            // `NoFragmentCycles` validator.
                            continue;
                        }

                        self.find(on_type, &fragment.node.selection_set);
                    }
                }
            }
        }
    }

    fn add_output(
        &mut self,
        on_type: Option<&'a str>,
        name: &'a str,
        field: &'a Positioned<Field>,
    ) {
        if let Some(prev_field) = self.outputs.get(&(on_type, name)) {
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
            self.outputs.insert((on_type, name), field);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> OverlappingFieldsCanBeMerged {
        OverlappingFieldsCanBeMerged
    }

    #[test]
    fn same_field_on_different_type() {
        expect_passes_rule!(
            factory,
            r#"
          {
           pet {
            ... on Dog {
                doesKnowCommand(dogCommand: SIT)
            }
            ... on Cat {
                doesKnowCommand(catCommand: JUMP)
            }
           }
          }
        "#,
        );
    }

    #[test]
    fn same_field_on_same_type() {
        expect_fails_rule!(
            factory,
            r#"
          {
           pet {
            ... on Dog {
                doesKnowCommand(dogCommand: SIT)
            }
            ... on Dog {
                doesKnowCommand(dogCommand: Heel)
            }
           }
          }
        "#,
        );
    }

    #[test]
    fn same_alias_on_different_type() {
        expect_passes_rule!(
            factory,
            r#"
          {
           pet {
            ... on Dog {
                volume: barkVolume
            }
            ... on Cat {
                volume: meowVolume
            }
           }
          }
        "#,
        );
    }
}
