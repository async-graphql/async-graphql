use crate::parser::query::FragmentDefinition;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueFragmentNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueFragmentNames<'a> {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a Positioned<FragmentDefinition>,
    ) {
        if !self.names.insert(&fragment_definition.name) {
            ctx.report_error(
                vec![fragment_definition.position()],
                format!(
                    "There can only be one fragment named \"{}\"",
                    fragment_definition.name
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> UniqueFragmentNames<'a> {
        UniqueFragmentNames::default()
    }

    #[test]
    fn no_fragments() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              name
            }
          }
        "#,
        );
    }

    #[test]
    fn one_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              ...fragA
            }
          }
          fragment fragA on Dog {
            name
          }
        "#,
        );
    }

    #[test]
    fn many_fragments() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dog {
              ...fragA
              ...fragB
              ...fragC
            }
          }
          fragment fragA on Dog {
            name
          }
          fragment fragB on Dog {
            nickname
          }
          fragment fragC on Dog {
            barkVolume
          }
        "#,
        );
    }

    #[test]
    fn inline_fragments_always_unique() {
        expect_passes_rule!(
            factory,
            r#"
          {
            dorOrHuman {
              ...on Dog {
                name
              }
              ...on Dog {
                barkVolume
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn fragment_and_operation_named_the_same() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo {
            dog {
              ...Foo
            }
          }
          fragment Foo on Dog {
            name
          }
        "#,
        );
    }

    #[test]
    fn fragments_named_the_same() {
        expect_fails_rule!(
            factory,
            r#"
          {
            dog {
              ...fragA
            }
          }
          fragment fragA on Dog {
            name
          }
          fragment fragA on Dog {
            barkVolume
          }
        "#,
        );
    }

    #[test]
    fn fragments_named_the_same_no_reference() {
        expect_fails_rule!(
            factory,
            r#"
          fragment fragA on Dog {
            name
          }
          fragment fragA on Dog {
            barkVolume
          }
        "#,
        );
    }
}
