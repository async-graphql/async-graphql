use std::collections::{HashMap, HashSet};

use async_graphql_value::Value;

use crate::{
    parser::types::{
        ExecutableDocument, FragmentDefinition, FragmentSpread, OperationDefinition,
        VariableDefinition,
    },
    validation::{
        utils::{referenced_variables, Scope},
        visitor::{Visitor, VisitorContext},
    },
    Name, Pos, Positioned,
};

#[derive(Default)]
pub struct NoUndefinedVariables<'a> {
    defined_variables: HashMap<Option<&'a str>, (Pos, HashSet<&'a str>)>,
    used_variables: HashMap<Scope<'a>, HashMap<&'a str, Pos>>,
    current_scope: Option<Scope<'a>>,
    spreads: HashMap<Scope<'a>, Vec<&'a str>>,
}

impl<'a> NoUndefinedVariables<'a> {
    fn find_undef_vars(
        &'a self,
        scope: &Scope<'a>,
        defined: &HashSet<&'a str>,
        undef: &mut Vec<(&'a str, Pos)>,
        visited: &mut HashSet<Scope<'a>>,
    ) {
        if visited.contains(scope) {
            return;
        }

        visited.insert(*scope);

        if let Some(used_vars) = self.used_variables.get(scope) {
            for (var, pos) in used_vars {
                if !defined.contains(var) {
                    undef.push((*var, *pos));
                }
            }
        }

        if let Some(spreads) = self.spreads.get(scope) {
            for spread in spreads {
                self.find_undef_vars(&Scope::Fragment(spread), defined, undef, visited);
            }
        }
    }
}

impl<'a> Visitor<'a> for NoUndefinedVariables<'a> {
    fn exit_document(&mut self, ctx: &mut VisitorContext<'a>, _doc: &'a ExecutableDocument) {
        for (op_name, (def_pos, def_vars)) in &self.defined_variables {
            let mut undef = Vec::new();
            let mut visited = HashSet::new();
            self.find_undef_vars(
                &Scope::Operation(*op_name),
                def_vars,
                &mut undef,
                &mut visited,
            );

            for (var, pos) in undef {
                if let Some(op_name) = op_name {
                    ctx.report_error(
                        vec![*def_pos, pos],
                        format!(
                            r#"Variable "${}" is not defined by operation "{}""#,
                            var, op_name
                        ),
                    );
                } else {
                    ctx.report_error(vec![pos], format!(r#"Variable "${}" is not defined"#, var));
                }
            }
        }
    }

    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        name: Option<&'a Name>,
        operation_definition: &'a Positioned<OperationDefinition>,
    ) {
        let name = name.map(Name::as_str);
        self.current_scope = Some(Scope::Operation(name));
        self.defined_variables
            .insert(name, (operation_definition.pos, HashSet::new()));
    }

    fn enter_fragment_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        name: &'a Name,
        _fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        self.current_scope = Some(Scope::Fragment(name));
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        if let Some(Scope::Operation(ref name)) = self.current_scope {
            if let Some(&mut (_, ref mut vars)) = self.defined_variables.get_mut(name) {
                vars.insert(&variable_definition.node.name.node);
            }
        }
    }

    fn enter_argument(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        name: &'a Positioned<Name>,
        value: &'a Positioned<Value>,
    ) {
        if let Some(ref scope) = self.current_scope {
            self.used_variables
                .entry(*scope)
                .or_insert_with(HashMap::new)
                .extend(
                    referenced_variables(&value.node)
                        .into_iter()
                        .map(|n| (n, name.pos)),
                );
        }
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        if let Some(ref scope) = self.current_scope {
            self.spreads
                .entry(*scope)
                .or_insert_with(Vec::new)
                .push(&fragment_spread.node.fragment_name.node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory<'a>() -> NoUndefinedVariables<'a> {
        NoUndefinedVariables::default()
    }

    #[test]
    fn all_variables_defined() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }

    #[test]
    fn all_variables_deeply_defined() {
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
    fn all_variables_deeply_defined_in_inline_fragments_defined() {
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
    fn all_variables_in_fragments_deeply_defined() {
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
    fn variable_within_single_fragment_defined_in_multiple_operations() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String) {
            ...FragA
          }
          query Bar($a: String) {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a)
          }
        "#,
        );
    }

    #[test]
    fn variable_within_fragments_defined_in_operations() {
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
    fn variable_within_recursive_fragment_defined() {
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
    fn variable_not_defined() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String, $c: String) {
            field(a: $a, b: $b, c: $c, d: $d)
          }
        "#,
        );
    }

    #[test]
    fn variable_not_defined_by_unnamed_query() {
        expect_fails_rule!(
            factory,
            r#"
          {
            field(a: $a)
          }
        "#,
        );
    }

    #[test]
    fn multiple_variables_not_defined() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }

    #[test]
    fn variable_in_fragment_not_defined_by_unnamed_query() {
        expect_fails_rule!(
            factory,
            r#"
          {
            ...FragA
          }
          fragment FragA on Type {
            field(a: $a)
          }
        "#,
        );
    }

    #[test]
    fn variable_in_fragment_not_defined_by_operation() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String, $b: String) {
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
    fn multiple_variables_in_fragments_not_defined() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
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
    fn single_variable_in_fragment_not_defined_by_multiple_operations() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: String) {
            ...FragAB
          }
          query Bar($a: String) {
            ...FragAB
          }
          fragment FragAB on Type {
            field(a: $a, b: $b)
          }
        "#,
        );
    }

    #[test]
    fn variables_in_fragment_not_defined_by_multiple_operations() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
            ...FragAB
          }
          query Bar($a: String) {
            ...FragAB
          }
          fragment FragAB on Type {
            field(a: $a, b: $b)
          }
        "#,
        );
    }

    #[test]
    fn variable_in_fragment_used_by_other_operation() {
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

    #[test]
    fn multiple_undefined_variables_produce_multiple_errors() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($b: String) {
            ...FragAB
          }
          query Bar($a: String) {
            ...FragAB
          }
          fragment FragAB on Type {
            field1(a: $a, b: $b)
            ...FragC
            field3(a: $a, b: $b)
          }
          fragment FragC on Type {
            field2(c: $c)
          }
        "#,
        );
    }
}
