use crate::error::RuleError;
use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{Document, FragmentDefinition, FragmentSpread};
use graphql_parser::Pos;
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
        fragment_definition: &'a FragmentDefinition,
    ) {
        self.current_fragment = Some(&fragment_definition.name);
        self.fragment_order.push(&fragment_definition.name);
    }

    fn exit_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _fragment_definition: &'a FragmentDefinition,
    ) {
        self.current_fragment = None;
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a FragmentSpread,
    ) {
        if let Some(current_fragment) = self.current_fragment {
            self.spreads
                .entry(current_fragment)
                .or_insert_with(Vec::new)
                .push((&fragment_spread.fragment_name, fragment_spread.position));
        }
    }
}
