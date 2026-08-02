use super::*;

const POSITIVE_FIXTURE_IDS: [&str; 17] = [
    "persistent-blank-absent-key-generation",
    "persistent-blank-zero-key-generation",
    "persistent-operational-u64-boundaries",
    "persistent-provisioning-initial",
    "persistent-provisioning-recommission",
    "persistent-update-pending",
    "persistent-recovery",
    "persistent-operational-receipt-release",
    "persistent-revoked",
    "persistent-decommissioned",
    "persistent-fault",
    "authority-clean",
    "authority-prepared-applied",
    "authority-prepared-rejected",
    "authority-committed",
    "receipt-minimal-optionals-absent",
    "receipt-optionals-present",
];

const VALIDATION_ERRORS: [ValidationError; 17] = [
    ValidationError::EmptyIdentifier,
    ValidationError::EmptySubject,
    ValidationError::EmptyRequiredValue,
    ValidationError::WrongObjectClass,
    ValidationError::InvalidSlot,
    ValidationError::MissingRecord,
    ValidationError::UnexpectedRecord,
    ValidationError::SlotConflict,
    ValidationError::SelectorMismatch,
    ValidationError::DuplicateExtension,
    ValidationError::UnorderedExtensions,
    ValidationError::CommitIdMismatch,
    ValidationError::AuthorityPhaseMismatch,
    ValidationError::AuthorityContextMismatch,
    ValidationError::StateContextMismatch,
    ValidationError::InconsistentState,
    ValidationError::InconsistentExecution,
];

/// Successful corpus-conformance counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CorpusConformanceSummary {
    pub positive_fixtures: usize,
    pub negative_fixtures: usize,
    pub validation_error_classes: usize,
}

/// Stable corpus-level drift or conformance failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CorpusConformanceError {
    PositiveManifestMismatch,
    EmptyFixtureMetadata(&'static str),
    DuplicateFixtureIdentifier(&'static str),
    InvalidPositiveFixture {
        identifier: &'static str,
        error: ValidationError,
    },
    NegativeErrorCoverageMismatch,
    InvalidNegativeFixture(NegativeFixtureFailure),
}

/// Validate the frozen positive manifest, negative corpus, and exact error
/// coverage without selecting or emitting protocol bytes.
pub fn validate_corpus_conformance() -> Result<CorpusConformanceSummary, CorpusConformanceError> {
    let positives = positive_fixtures();
    let positive_ids: Vec<_> = positives.iter().map(|fixture| fixture.identifier).collect();
    if positive_ids != POSITIVE_FIXTURE_IDS {
        return Err(CorpusConformanceError::PositiveManifestMismatch);
    }

    for (index, fixture) in positives.iter().enumerate() {
        validate_metadata(fixture.identifier, fixture.purpose)?;
        if positives[..index]
            .iter()
            .any(|prior| prior.identifier == fixture.identifier)
        {
            return Err(CorpusConformanceError::DuplicateFixtureIdentifier(
                fixture.identifier,
            ));
        }
        if let Err(error) = fixture.object.validate() {
            return Err(CorpusConformanceError::InvalidPositiveFixture {
                identifier: fixture.identifier,
                error,
            });
        }
    }

    let negatives = negative_fixtures();
    for (index, fixture) in negatives.iter().enumerate() {
        validate_metadata(fixture.identifier, fixture.purpose)?;
        if negatives[..index]
            .iter()
            .any(|prior| prior.identifier == fixture.identifier)
        {
            return Err(CorpusConformanceError::DuplicateFixtureIdentifier(
                fixture.identifier,
            ));
        }
        fixture
            .validate_expected()
            .map_err(CorpusConformanceError::InvalidNegativeFixture)?;
    }

    let mut actual_errors: Vec<_> = negatives
        .iter()
        .map(|fixture| fixture.expected_error)
        .collect();
    actual_errors.sort_by_key(validation_error_index);
    if actual_errors != VALIDATION_ERRORS {
        return Err(CorpusConformanceError::NegativeErrorCoverageMismatch);
    }

    Ok(CorpusConformanceSummary {
        positive_fixtures: positives.len(),
        negative_fixtures: negatives.len(),
        validation_error_classes: VALIDATION_ERRORS.len(),
    })
}

fn validate_metadata(
    identifier: &'static str,
    purpose: &'static str,
) -> Result<(), CorpusConformanceError> {
    if identifier.is_empty() || purpose.is_empty() {
        return Err(CorpusConformanceError::EmptyFixtureMetadata(identifier));
    }
    Ok(())
}

const fn validation_error_index(error: &ValidationError) -> usize {
    match error {
        ValidationError::EmptyIdentifier => 0,
        ValidationError::EmptySubject => 1,
        ValidationError::EmptyRequiredValue => 2,
        ValidationError::WrongObjectClass => 3,
        ValidationError::InvalidSlot => 4,
        ValidationError::MissingRecord => 5,
        ValidationError::UnexpectedRecord => 6,
        ValidationError::SlotConflict => 7,
        ValidationError::SelectorMismatch => 8,
        ValidationError::DuplicateExtension => 9,
        ValidationError::UnorderedExtensions => 10,
        ValidationError::CommitIdMismatch => 11,
        ValidationError::AuthorityPhaseMismatch => 12,
        ValidationError::AuthorityContextMismatch => 13,
        ValidationError::StateContextMismatch => 14,
        ValidationError::InconsistentState => 15,
        ValidationError::InconsistentExecution => 16,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_corpus_is_conformant() {
        assert_eq!(
            validate_corpus_conformance(),
            Ok(CorpusConformanceSummary {
                positive_fixtures: 17,
                negative_fixtures: 17,
                validation_error_classes: 17,
            })
        );
    }
}
