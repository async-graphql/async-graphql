use crate::parser::query::{
    Definition, Document, FragmentDefinition, FragmentSpread, OperationDefinition,
};
use crate::validation::utils::{operation_name, Scope};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::{Pos, Positioned};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct NoUnusedFragments<'a> {
    spreads: HashMap<Scope<'a>, Vec<&'a str>>,
    defined_fragments: HashSet<(&'a str, Pos)>,
    current_scope: Option<Scope<'a>>,
}

impl<'a> NoUnusedFragments<'a> {
    fn find_reachable_fragments(&self, from: &Scope<'a>, result: &mut HashSet<&'a str>) {
        if let Scope::Fragment(name) = *from {
            if result.contains(name) {
                return;
            } else {
                result.insert(name);
            }
        }

        if let Some(spreads) = self.spreads.get(from) {
            for spread in spreads {
                self.find_reachable_fragments(&Scope::Fragment(spread), result)
            }
        }
    }
}

impl<'a> Visitor<'a> for NoUnusedFragments<'a> {
    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, doc: &'a Document) {
        let mut reachable = HashSet::new();

        for def in doc.definitions() {
            if let Definition::Operation(operation_definition) = &def.node {
                let (name, _) = operation_name(operation_definition);
                self.find_reachable_fragments(&Scope::Operation(name), &mut reachable);
            }
        }

        for (fragment_name, pos) in &self.defined_fragments {
            if !reachable.contains(fragment_name) {
                ctx.report_error(
                    vec![*pos],
                    format!(r#"Fragment "{}" is never used"#, fragment_name),
                );
            }
        }
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        let (op_name, _) = operation_name(operation_definition);
        self.current_scope = Some(Scope::Operation(op_name));
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.current_scope = Some(Scope::Fragment(fragment_definition.name.as_str()));
        self.defined_fragments.insert((
            fragment_definition.name.as_str(),
            fragment_definition.position(),
        ));
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        if let Some(ref scope) = self.current_scope {
            self.spreads
                .entry(scope.clone())
                .or_insert_with(Vec::new)
                .push(fragment_spread.fragment_name.as_str());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> NoUnusedFragments<'a> {
        NoUnusedFragments::default()
    }

    #[test]
    fn all_fragment_names_are_used() {
        expect_passes_rule!(
            factory,
            r#"
          {
            human(id: 4) {
              ...HumanFields1
              ... on Human {
                ...HumanFields2
              }
            }
          }
          fragment HumanFields1 on Human {
            name
            ...HumanFields3
          }
          fragment HumanFields2 on Human {
            name
          }
          fragment HumanFields3 on Human {
            name
          }
        "#,
        );
    }

    #[test]
    fn all_fragment_names_are_used_by_multiple_operations() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            human(id: 4) {
              ...HumanFields1
            }
          }
          query Bar {
            human(id: 4) {
              ...HumanFields2
            }
          }
          fragment HumanFields1 on Human {
            name
            ...HumanFields3
          }
          fragment HumanFields2 on Human {
            name
          }
          fragment HumanFields3 on Human {
            name
          }
        "#,
        );
    }

    #[test]
    fn contains_unknown_fragments() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            human(id: 4) {
              ...HumanFields1
            }
          }
          query Bar {
            human(id: 4) {
              ...HumanFields2
            }
          }
          fragment HumanFields1 on Human {
            name
            ...HumanFields3
          }
          fragment HumanFields2 on Human {
            name
          }
          fragment HumanFields3 on Human {
            name
          }
          fragment Unused1 on Human {
            name
          }
          fragment Unused2 on Human {
            name
          }
        "#,
        );
    }

    #[test]
    fn contains_unknown_fragments_with_ref_cycle() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            human(id: 4) {
              ...HumanFields1
            }
          }
          query Bar {
            human(id: 4) {
              ...HumanFields2
            }
          }
          fragment HumanFields1 on Human {
            name
            ...HumanFields3
          }
          fragment HumanFields2 on Human {
            name
          }
          fragment HumanFields3 on Human {
            name
          }
          fragment Unused1 on Human {
            name
            ...Unused2
          }
          fragment Unused2 on Human {
            name
            ...Unused1
          }
        "#,
        );
    }

    #[test]
    fn contains_unknown_and_undef_fragments() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo {
            human(id: 4) {
              ...bar
            }
          }
          fragment foo on Human {
            name
          }
        "#,
        );
    }
}
