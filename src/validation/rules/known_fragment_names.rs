use crate::parser::types::FragmentSpread;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct KnownFragmentNames;

impl<'a> Visitor<'a> for KnownFragmentNames {
    fn enter_fragment_spread(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        if !ctx.is_known_fragment(&fragment_spread.node.fragment_name.node) {
            ctx.report_error(
                vec![fragment_spread.pos],
                format!(
                    r#"Unknown fragment: "{}""#,
                    fragment_spread.node.fragment_name.node
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn factory() -> KnownFragmentNames {
        KnownFragmentNames::default()
    }

    #[test]
    fn known() {
        expect_passes_rule!(
            factory,
            r#"
          {
            human(id: 4) {
              ...HumanFields1
              ... on Human {
                ...HumanFields2
              }
              ... {
                name
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
    fn unknown() {
        expect_fails_rule!(
            factory,
            r#"
          {
            human(id: 4) {
              ...UnknownFragment1
              ... on Human {
                ...UnknownFragment2
              }
            }
          }
          fragment HumanFields on Human {
            name
            ...UnknownFragment3
          }
        "#,
        );
    }
}
