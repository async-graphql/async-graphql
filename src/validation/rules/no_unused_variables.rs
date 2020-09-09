use crate::parser::types::{
    ExecutableDocument, FragmentDefinition, FragmentSpread, Name, OperationDefinition, Value,
    VariableDefinition,
};
use crate::validation::utils::{referenced_variables, Scope};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::{Pos, Positioned};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct NoUnusedVariables<'a> {
    defined_variables: HashMap<Option<&'a str>, HashSet<(&'a str, Pos)>>,
    used_variables: HashMap<Scope<'a>, Vec<&'a str>>,
    current_scope: Option<Scope<'a>>,
    spreads: HashMap<Scope<'a>, Vec<&'a str>>,
}

impl<'a> NoUnusedVariables<'a> {
    fn find_used_vars(
        &self,
        from: &Scope<'a>,
        defined: &HashSet<&'a str>,
        used: &mut HashSet<&'a str>,
        visited: &mut HashSet<Scope<'a>>,
    ) {
        if visited.contains(from) {
            return;
        }

        visited.insert(from.clone());

        if let Some(used_vars) = self.used_variables.get(from) {
            for var in used_vars {
                if defined.contains(var) {
                    used.insert(var);
                }
            }
        }

        if let Some(spreads) = self.spreads.get(from) {
            for spread in spreads {
                self.find_used_vars(&Scope::Fragment(spread), defined, used, visited);
            }
        }
    }
}

impl<'a> Visitor<'a> for NoUnusedVariables<'a> {
    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, _doc: &'a ExecutableDocument) {
        for (op_name, def_vars) in &self.defined_variables {
            let mut used = HashSet::new();
            let mut visited = HashSet::new();
            self.find_used_vars(
                &Scope::Operation(*op_name),
                &def_vars.iter().map(|(name, _)| *name).collect(),
                &mut used,
                &mut visited,
            );

            for (var, pos) in def_vars.iter().filter(|(var, _)| !used.contains(var)) {
                if let Some(op_name) = op_name {
                    ctx.report_error(
                        vec![*pos],
                        format!(
                            r#"Variable "${}" is not used by operation "{}""#,
                            var, op_name
                        ),
                    );
                } else {
                    ctx.report_error(vec![*pos], format!(r#"Variable "${}" is not used"#, var));
                }
            }
        }
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        let op_name = operation_definition
            .node
            .name
            .as_ref()
            .map(|name| &*name.node);
        self.current_scope = Some(Scope::Operation(op_name));
        self.defined_variables.insert(op_name, HashSet::new());
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.current_scope = Some(Scope::Fragment(&fragment_definition.node.name.node));
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        if let Some(Scope::Operation(ref name)) = self.current_scope {
            if let Some(vars) = self.defined_variables.get_mut(name) {
                vars.insert((&variable_definition.node.name.node, variable_definition.pos));
            }
        }
    }

    fn enter_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _name: &'a Positioned<Name>,
        value: &'a Positioned<Value>,
    ) {
        if let Some(ref scope) = self.current_scope {
            self.used_variables
                .entry(scope.clone())
                .or_insert_with(Vec::new)
                .append(&mut referenced_variables(&value.node));
        }
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
                .push(&fragment_spread.node.fragment_name.node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory<'a>() -> NoUnusedVariables<'a> {
        NoUnusedVariables::default()
    }

    #[test]
    fn uses_all_variables() {
        expect_passes_rule!(
            factory,
            r#"
          query ($a: String, $b: String, $c: String) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }

    #[test]
    fn uses_all_variables_deeply() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            field(a: $a) {
              field(b: $b) {
                field(c: $c)
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn uses_all_variables_deeply_in_inline_fragments() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            ... on Type {
              field(a: $a) {
                field(b: $b) {
                  ... on Type {
                    field(c: $c)
                  }
                }
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn uses_all_variables_in_fragments() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a) {
              ...FragB
            }
          }
          fragment FragB on Type {
            field(b: $b) {
              ...FragC
            }
          }
          fragment FragC on Type {
            field(c: $c)
          }
        "#,
        );
    }

    #[test]
    fn variable_used_by_fragment_in_multiple_operations() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String) {
            ...FragA
          }
          query Bar($b: String) {
            ...FragB
          }
          fragment FragA on Type {
            field(a: $a)
          }
          fragment FragB on Type {
            field(b: $b)
          }
        "#,
        );
    }

    #[test]
    fn variable_used_by_recursive_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String) {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a) {
              ...FragA
            }
          }
        "#,
        );
    }

    #[test]
    fn variable_not_used() {
        expect_fails_rule!(
            factory,
            r#"
          query ($a: String, $b: String, $c: String) {
            field(a: $a, b: $b)
          }
        "#,
        );
    }

    #[test]
    fn multiple_variables_not_used_1() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            field(b: $b)
          }
        "#,
        );
    }

    #[test]
    fn variable_not_used_in_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a) {
              ...FragB
            }
          }
          fragment FragB on Type {
            field(b: $b) {
              ...FragC
            }
          }
          fragment FragC on Type {
            field
          }
        "#,
        );
    }

    #[test]
    fn multiple_variables_not_used_2() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            ...FragA
          }
          fragment FragA on Type {
            field {
              ...FragB
            }
          }
          fragment FragB on Type {
            field(b: $b) {
              ...FragC
            }
          }
          fragment FragC on Type {
            field
          }
        "#,
        );
    }

    #[test]
    fn variable_not_used_by_unreferenced_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a)
          }
          fragment FragB on Type {
            field(b: $b)
          }
        "#,
        );
    }

    #[test]
    fn variable_not_used_by_fragment_used_by_other_operation() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
            ...FragA
          }
          query Bar($a: String) {
            ...FragB
          }
          fragment FragA on Type {
            field(a: $a)
          }
          fragment FragB on Type {
            field(b: $b)
          }
        "#,
        );
    }
}
