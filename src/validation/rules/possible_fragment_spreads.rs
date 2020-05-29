use crate::parser::query::{Definition, Document, FragmentSpread, InlineFragment, TypeCondition};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;
use std::collections::HashMap;

#[derive(Default)]
pub struct PossibleFragmentSpreads<'a> {
    fragment_types: HashMap<&'a str, &'a str>,
}

impl<'a> Visitor<'a> for PossibleFragmentSpreads<'a> {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, doc: &'a Document) {
        for d in doc.definitions() {
            if let Definition::Fragment(fragment) = &d.node {
                let TypeCondition::On(type_name) = &fragment.type_condition.node;
                self.fragment_types
                    .insert(fragment.name.as_str(), type_name);
            }
        }
    }

    fn enter_fragment_spread(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a Positioned<FragmentSpread>,
    ) {
        if let Some(fragment_type) = self
            .fragment_types
            .get(fragment_spread.fragment_name.as_str())
        {
            if let Some(current_type) = ctx.current_type() {
                if let Some(on_type) = ctx.registry.types.get(*fragment_type) {
                    if !current_type.type_overlap(on_type) {
                        ctx.report_error(
                            vec![fragment_spread.position()],
                            format!(
                                "Fragment \"{}\" cannot be spread here as objects of type \"{}\" can never be of type \"{}\"",
                                &fragment_spread.fragment_name, current_type.name(), fragment_type
                            ),
                        );
                    }
                }
            }
        }
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        inline_fragment: &'a Positioned<InlineFragment>,
    ) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(TypeCondition::On(fragment_type)) =
                &inline_fragment.type_condition.as_ref().map(|c| &c.node)
            {
                if let Some(on_type) = ctx.registry.types.get(fragment_type.as_str()) {
                    if !parent_type.type_overlap(&on_type) {
                        ctx.report_error(
                            vec![inline_fragment.position()],
                            format!(
                                "Fragment cannot be spread here as objects of type \"{}\" \
             can never be of type \"{}\"",
                                parent_type.name(),
                                fragment_type
                            ),
                        )
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory<'a>() -> PossibleFragmentSpreads<'a> {
        PossibleFragmentSpreads::default()
    }

    #[test]
    fn of_the_same_object() {
        expect_passes_rule!(
            factory,
            r#"
          fragment objectWithinObject on Dog { ...dogFragment }
          fragment dogFragment on Dog { barkVolume }
        "#,
        );
    }

    #[test]
    fn of_the_same_object_with_inline_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          fragment objectWithinObjectAnon on Dog { ... on Dog { barkVolume } }
        "#,
        );
    }

    #[test]
    fn object_into_an_implemented_interface() {
        expect_passes_rule!(
            factory,
            r#"
          fragment objectWithinInterface on Pet { ...dogFragment }
          fragment dogFragment on Dog { barkVolume }
        "#,
        );
    }

    #[test]
    fn object_into_containing_union() {
        expect_passes_rule!(
            factory,
            r#"
          fragment objectWithinUnion on CatOrDog { ...dogFragment }
          fragment dogFragment on Dog { barkVolume }
        "#,
        );
    }

    #[test]
    fn union_into_contained_object() {
        expect_passes_rule!(
            factory,
            r#"
          fragment unionWithinObject on Dog { ...catOrDogFragment }
          fragment catOrDogFragment on CatOrDog { __typename }
        "#,
        );
    }

    #[test]
    fn union_into_overlapping_interface() {
        expect_passes_rule!(
            factory,
            r#"
          fragment unionWithinInterface on Pet { ...catOrDogFragment }
          fragment catOrDogFragment on CatOrDog { __typename }
        "#,
        );
    }

    #[test]
    fn union_into_overlapping_union() {
        expect_passes_rule!(
            factory,
            r#"
          fragment unionWithinUnion on DogOrHuman { ...catOrDogFragment }
          fragment catOrDogFragment on CatOrDog { __typename }
        "#,
        );
    }

    #[test]
    fn interface_into_implemented_object() {
        expect_passes_rule!(
            factory,
            r#"
          fragment interfaceWithinObject on Dog { ...petFragment }
          fragment petFragment on Pet { name }
        "#,
        );
    }

    #[test]
    fn interface_into_overlapping_interface() {
        expect_passes_rule!(
            factory,
            r#"
          fragment interfaceWithinInterface on Pet { ...beingFragment }
          fragment beingFragment on Being { name }
        "#,
        );
    }

    #[test]
    fn interface_into_overlapping_interface_in_inline_fragment() {
        expect_passes_rule!(
            factory,
            r#"
          fragment interfaceWithinInterface on Pet { ... on Being { name } }
        "#,
        );
    }

    #[test]
    fn interface_into_overlapping_union() {
        expect_passes_rule!(
            factory,
            r#"
          fragment interfaceWithinUnion on CatOrDog { ...petFragment }
          fragment petFragment on Pet { name }
        "#,
        );
    }

    #[test]
    fn different_object_into_object() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidObjectWithinObject on Cat { ...dogFragment }
          fragment dogFragment on Dog { barkVolume }
        "#,
        );
    }

    #[test]
    fn different_object_into_object_in_inline_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidObjectWithinObjectAnon on Cat {
            ... on Dog { barkVolume }
          }
        "#,
        );
    }

    #[test]
    fn object_into_not_implementing_interface() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidObjectWithinInterface on Pet { ...humanFragment }
          fragment humanFragment on Human { pets { name } }
        "#,
        );
    }

    #[test]
    fn object_into_not_containing_union() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidObjectWithinUnion on CatOrDog { ...humanFragment }
          fragment humanFragment on Human { pets { name } }
        "#,
        );
    }

    #[test]
    fn union_into_not_contained_object() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidUnionWithinObject on Human { ...catOrDogFragment }
          fragment catOrDogFragment on CatOrDog { __typename }
        "#,
        );
    }

    #[test]
    fn union_into_non_overlapping_interface() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidUnionWithinInterface on Pet { ...humanOrAlienFragment }
          fragment humanOrAlienFragment on HumanOrAlien { __typename }
        "#,
        );
    }

    #[test]
    fn union_into_non_overlapping_union() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidUnionWithinUnion on CatOrDog { ...humanOrAlienFragment }
          fragment humanOrAlienFragment on HumanOrAlien { __typename }
        "#,
        );
    }

    #[test]
    fn interface_into_non_implementing_object() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidInterfaceWithinObject on Cat { ...intelligentFragment }
          fragment intelligentFragment on Intelligent { iq }
        "#,
        );
    }

    #[test]
    fn interface_into_non_overlapping_interface() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidInterfaceWithinInterface on Pet {
            ...intelligentFragment
          }
          fragment intelligentFragment on Intelligent { iq }
        "#,
        );
    }

    #[test]
    fn interface_into_non_overlapping_interface_in_inline_fragment() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidInterfaceWithinInterfaceAnon on Pet {
            ...on Intelligent { iq }
          }
        "#,
        );
    }

    #[test]
    fn interface_into_non_overlapping_union() {
        expect_fails_rule!(
            factory,
            r#"
          fragment invalidInterfaceWithinUnion on HumanOrAlien { ...petFragment }
          fragment petFragment on Pet { name }
        "#,
        );
    }
}
