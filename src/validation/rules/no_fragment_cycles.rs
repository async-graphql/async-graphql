use crate::error::RuleError;
use crate::parser::query::{Document, FragmentDefinition, FragmentSpread};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::{Pos, Positioned};
use std::collections::{HashMap, HashSet};

struct CycleDetector<'a> {
    visited: HashSet<&'a str>,
    spreads: &'a HashMap<&'a str, Vec<(&'a str, Pos)>>,
    path_indices: HashMap<&'a str, usize>,
    errors: Vec<RuleError>,
}

impl<'a> CycleDetector<'a> {
    fn detect_from(&mut self, from: &'a str, path: &mut Vec<(&'a str, Pos)>) {
        self.visited.insert(from);

        if !self.spreads.contains_key(from) {
            return;
        }

        self.path_indices.insert(from, path.len());

        for (name, pos) in &self.spreads[from] {
            let index = self.path_indices.get(name).cloned();

            if let Some(index) = index {
                let err_pos = if index < path.len() {
                    path[index].1
                } else {
                    *pos
                };

                self.errors.push(RuleError {
                    locations: vec![err_pos],
                    message: format!("Cannot spread fragment \"{}\"", name),
                });
            } else if !self.visited.contains(name) {
                path.push((name, *pos));
                self.detect_from(name, path);
                path.pop();
            }
        }

        self.path_indices.remove(from);
    }
}

#[derive(Default)]
pub struct NoFragmentCycles<'a> {
    current_fragment: Option<&'a str>,
    spreads: HashMap<&'a str, Vec<(&'a str, Pos)>>,
    fragment_order: Vec<&'a str>,
}

impl<'a> Visitor<'a> for NoFragmentCycles<'a> {
    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, _doc: &'a Document) {
        let mut detector = CycleDetector {
            visited: HashSet::new(),
            spreads: &self.spreads,
            path_indices: HashMap::new(),
            errors: Vec::new(),
        };

        for frag in &self.fragment_order {
            if !detector.visited.contains(frag) {
                let mut path = Vec::new();
                detector.detect_from(frag, &mut path);
            }
        }

        ctx.append_errors(detector.errors);
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.current_fragment = Some(&fragment_definition.name);
        self.fragment_order.push(&fragment_definition.name);
    }

    fn exit_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.current_fragment = None;
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        if let Some(current_fragment) = self.current_fragment {
            self.spreads
                .entry(current_fragment)
                .or_insert_with(Vec::new)
                .push((&fragment_spread.fragment_name, fragment_spread.position()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> NoFragmentCycles<'a> {
        NoFragmentCycles::default()
    }

    #[test]
    fn single_reference_is_valid() {
        expect_passes_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB }
          fragment fragB on Dog { name }
        "#,
        );
    }

    #[test]
    fn spreading_twice_is_not_circular() {
        expect_passes_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB, ...fragB }
          fragment fragB on Dog { name }
        "#,
        );
    }

    #[test]
    fn spreading_twice_indirectly_is_not_circular() {
        expect_passes_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB, ...fragC }
          fragment fragB on Dog { ...fragC }
          fragment fragC on Dog { name }
        "#,
        );
    }

    #[test]
    fn double_spread_within_abstract_types() {
        expect_passes_rule!(
            factory,
            r#"
          fragment nameFragment on Pet {
            ... on Dog { name }
            ... on Cat { name }
          }
          fragment spreadsInAnon on Pet {
            ... on Dog { ...nameFragment }
            ... on Cat { ...nameFragment }
          }
        "#,
        );
    }

    #[test]
    fn does_not_false_positive_on_unknown_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          fragment nameFragment on Pet {
            ...UnknownFragment
          }
        "#,
        );
    }

    #[test]
    fn spreading_recursively_within_field_fails() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Human { relatives { ...fragA } },
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_directly() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragA }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_directly_within_inline_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Pet {
            ... on Dog {
              ...fragA
            }
          }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_indirectly() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB }
          fragment fragB on Dog { ...fragA }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_indirectly_reports_opposite_order() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragB on Dog { ...fragA }
          fragment fragA on Dog { ...fragB }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_indirectly_within_inline_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Pet {
            ... on Dog {
              ...fragB
            }
          }
          fragment fragB on Pet {
            ... on Dog {
              ...fragA
            }
          }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_deeply() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB }
          fragment fragB on Dog { ...fragC }
          fragment fragC on Dog { ...fragO }
          fragment fragX on Dog { ...fragY }
          fragment fragY on Dog { ...fragZ }
          fragment fragZ on Dog { ...fragO }
          fragment fragO on Dog { ...fragP }
          fragment fragP on Dog { ...fragA, ...fragX }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_deeply_two_paths() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB, ...fragC }
          fragment fragB on Dog { ...fragA }
          fragment fragC on Dog { ...fragA }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_deeply_two_paths_alt_traversal_order() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragC }
          fragment fragB on Dog { ...fragC }
          fragment fragC on Dog { ...fragA, ...fragB }
        "#,
        );
    }

    #[test]
    fn no_spreading_itself_deeply_and_immediately() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog { ...fragB }
          fragment fragB on Dog { ...fragB, ...fragC }
          fragment fragC on Dog { ...fragA, ...fragB }
        "#,
        );
    }
}
