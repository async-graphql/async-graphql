#[cfg(test)]
#[macro_use]
mod test_harness;

mod rules;
mod suggestion;
mod utils;
mod visitor;
mod visitors;

pub use visitor::VisitorContext;
use visitor::{visit, VisitorNil};

use crate::{
    parser::types::ExecutableDocument, registry::Registry, CacheControl, ServerError, Variables,
};

/// Validation results.
#[derive(Debug, Copy, Clone)]
pub struct ValidationResult {
    /// Cache control
    pub cache_control: CacheControl,

    /// Query complexity
    pub complexity: usize,

    /// Query depth
    pub depth: usize,
}

/// Validation mode
#[derive(Copy, Clone, Debug)]
pub enum ValidationMode {
    /// Execute all validation rules.
    Strict,

    /// The executor itself also has error handling, so it can improve
    /// performance, but it can lose some error messages.
    Fast,
}

pub(crate) fn check_rules(
    registry: &Registry,
    doc: &ExecutableDocument,
    variables: Option<&Variables>,
    mode: ValidationMode,
    limit_complexity: Option<usize>,
    limit_depth: Option<usize>,
) -> Result<ValidationResult, Vec<ServerError>> {
    let mut cache_control = CacheControl::default();
    let mut complexity = 0;
    let mut depth = 0;

    let errors = match mode {
        ValidationMode::Strict => {
            let mut ctx = VisitorContext::new(registry, doc, variables);
            let mut visitor = VisitorNil
                .with(rules::ArgumentsOfCorrectType::default())
                .with(rules::DefaultValuesOfCorrectType)
                .with(rules::FieldsOnCorrectType)
                .with(rules::FragmentsOnCompositeTypes)
                .with(rules::KnownArgumentNames::default())
                .with(rules::NoFragmentCycles::default())
                .with(rules::KnownFragmentNames)
                .with(rules::KnownTypeNames)
                .with(rules::NoUndefinedVariables::default())
                .with(rules::NoUnusedFragments::default())
                .with(rules::NoUnusedVariables::default())
                .with(rules::UniqueArgumentNames::default())
                .with(rules::UniqueVariableNames::default())
                .with(rules::VariablesAreInputTypes)
                .with(rules::VariableInAllowedPosition::default())
                .with(rules::ScalarLeafs)
                .with(rules::PossibleFragmentSpreads::default())
                .with(rules::ProvidedNonNullArguments)
                .with(rules::KnownDirectives::default())
                .with(rules::DirectivesUnique)
                .with(rules::OverlappingFieldsCanBeMerged)
                .with(rules::UploadFile);
            visit(&mut visitor, &mut ctx, doc);

            let mut visitor = VisitorNil
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                })
                .with(visitors::ComplexityCalculate::new(&mut complexity))
                .with(visitors::DepthCalculate::new(&mut depth));
            visit(&mut visitor, &mut ctx, doc);
            ctx.errors
        }
        ValidationMode::Fast => {
            let mut ctx = VisitorContext::new(registry, doc, variables);
            let mut visitor = VisitorNil
                .with(rules::NoFragmentCycles::default())
                .with(rules::UploadFile)
                .with(visitors::CacheControlCalculate {
                    cache_control: &mut cache_control,
                })
                .with(visitors::ComplexityCalculate::new(&mut complexity))
                .with(visitors::DepthCalculate::new(&mut depth));
            visit(&mut visitor, &mut ctx, doc);
            ctx.errors
        }
    };

    // check limit
    if let Some(limit_complexity) = limit_complexity {
        if complexity > limit_complexity {
            return Err(vec![ServerError::new("Query is too complex.", None)]);
        }
    }

    if let Some(limit_depth) = limit_depth {
        if depth > limit_depth {
            return Err(vec![ServerError::new("Query is nested too deep.", None)]);
        }
    }

    if !errors.is_empty() {
        return Err(errors.into_iter().map(Into::into).collect());
    }

    Ok(ValidationResult {
        cache_control,
        complexity,
        depth,
    })
}
