#[cfg(test)]
#[macro_use]
mod test_harness;

mod rules;
mod suggestion;
mod utils;
mod visitor;
mod visitors;

use crate::parser::types::ExecutableDocument;
use crate::registry::Registry;
use crate::{CacheControl, Error, Result, Variables};
use visitor::{visit, VisitorContext, VisitorNil};

pub struct CheckResult {
    pub cache_control: CacheControl,
    pub complexity: usize,
    pub depth: usize,
}

/// Validation mode
#[derive(Copy, Clone, Debug)]
pub enum ValidationMode {
    /// Execute all validation rules.
    Strict,

    /// The executor itself also has error handling, so it can improve performance, but it can lose some error messages.
    Fast,
}

pub fn check_rules(
    registry: &Registry,
    doc: &ExecutableDocument,
    variables: Option<&Variables>,
    mode: ValidationMode,
) -> Result<CheckResult> {
    let mut ctx = VisitorContext::new(registry, doc, variables);
    let mut cache_control = CacheControl::default();
    let mut complexity = 0;
    let mut depth = 0;

    match mode {
        ValidationMode::Strict => {
            let mut visitor = VisitorNil
                .with(rules::ArgumentsOfCorrectType::default())
                .with(rules::DefaultValuesOfCorrectType)
                .with(rules::FieldsOnCorrectType)
                .with(rules::FragmentsOnCompositeTypes)
                .with(rules::KnownArgumentNames::default())
                .with(rules::NoFragmentCycles::default())
                .with(rules::KnownFragmentNames)
                .with(rules::KnownTypeNames)
                .with(rules::LoneAnonymousOperation::default())
                .with(rules::NoUndefinedVariables::default())
                .with(rules::NoUnusedFragments::default())
                .with(rules::NoUnusedVariables::default())
                .with(rules::UniqueArgumentNames::default())
                .with(rules::UniqueFragmentNames::default())
                .with(rules::UniqueOperationNames::default())
                .with(rules::UniqueVariableNames::default())
                .with(rules::VariablesAreInputTypes)
                .with(rules::VariableInAllowedPosition::default())
                .with(rules::ScalarLeafs)
                .with(rules::PossibleFragmentSpreads::default())
                .with(rules::ProvidedNonNullArguments)
                .with(rules::KnownDirectives::default())
                .with(rules::OverlappingFieldsCanBeMerged)
                .with(rules::UploadFile)
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                })
                .with(visitors::ComplexityCalculate {
                    complexity: &mut complexity,
                })
                .with(visitors::DepthCalculate::new(&mut depth));
            visit(&mut visitor, &mut ctx, doc);
        }
        ValidationMode::Fast => {
            let mut visitor = VisitorNil
                .with(rules::NoFragmentCycles::default())
                .with(rules::UploadFile)
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                })
                .with(visitors::ComplexityCalculate {
                    complexity: &mut complexity,
                })
                .with(visitors::DepthCalculate::new(&mut depth));
            visit(&mut visitor, &mut ctx, doc);
        }
    }

    if !ctx.errors.is_empty() {
        return Err(Error::Rule { errors: ctx.errors });
    }
    Ok(CheckResult {
        cache_control,
        complexity,
        depth: depth as usize,
    })
}
