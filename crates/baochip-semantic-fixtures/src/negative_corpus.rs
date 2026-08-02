use super::*;

/// Validation operation exercised by one candidate-neutral negative fixture.
///
/// Cross-object variants preserve the distinction between an invalid object
/// and an individually valid object used under the wrong release context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NegativeCase {
    Object(SemanticObject),
    ReceiptAuthority {
        receipt: ExecutionReceiptProjection,
        authority: AuthorityMetadataProjection,
    },
    ReceiptState {
        receipt: ExecutionReceiptProjection,
        state: PersistentStateProjection,
    },
    ReceiptRelease {
        receipt: Box<ExecutionReceiptProjection>,
        authority: AuthorityMetadataProjection,
        state: PersistentStateProjection,
    },
}

impl NegativeCase {
    /// Evaluate the semantic operation represented by this negative case.
    pub fn evaluate(&self) -> Result<(), ValidationError> {
        match self {
            Self::Object(object) => object.validate(),
            Self::ReceiptAuthority { receipt, authority } => {
                receipt.validate_release_authority(authority)
            }
            Self::ReceiptState { receipt, state } => receipt.validate_authoritative_state(state),
            Self::ReceiptRelease {
                receipt,
                authority,
                state,
            } => receipt.validate_release(authority, state),
        }
    }
}

/// One stable negative semantic fixture and its exact expected error class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegativeFixture {
    pub identifier: &'static str,
    pub purpose: &'static str,
    pub case: NegativeCase,
    pub expected_error: ValidationError,
}

/// A negative fixture did not fail with its pinned semantic error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NegativeFixtureFailure {
    pub identifier: &'static str,
    pub expected_error: ValidationError,
    pub actual: Result<(), ValidationError>,
}

/// Stable failure to construct the frozen negative corpus from its positive
/// operands. This reports internal corpus drift instead of panicking.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NegativeCorpusError {
    MissingPositiveFixture(&'static str),
    UnexpectedPositiveObject(&'static str),
    UnexpectedAuthorityShape(&'static str),
}

impl NegativeFixture {
    /// Confirm that this negative fixture fails closed with its exact error.
    pub fn validate_expected(&self) -> Result<(), NegativeFixtureFailure> {
        let actual = self.case.evaluate();
        if actual == Err(self.expected_error) {
            Ok(())
        } else {
            Err(NegativeFixtureFailure {
                identifier: self.identifier,
                expected_error: self.expected_error,
                actual,
            })
        }
    }
}

fn fixture(identifier: &'static str) -> Result<Fixture, NegativeCorpusError> {
    positive_fixtures()
        .into_iter()
        .find(|fixture| fixture.identifier == identifier)
        .ok_or(NegativeCorpusError::MissingPositiveFixture(identifier))
}

fn persistent(identifier: &'static str) -> Result<PersistentStateProjection, NegativeCorpusError> {
    let SemanticObject::PersistentState(state) = fixture(identifier)?.object else {
        return Err(NegativeCorpusError::UnexpectedPositiveObject(identifier));
    };
    Ok(state)
}

fn authority(identifier: &'static str) -> Result<AuthorityMetadataProjection, NegativeCorpusError> {
    let SemanticObject::AuthorityMetadata(authority) = fixture(identifier)?.object else {
        return Err(NegativeCorpusError::UnexpectedPositiveObject(identifier));
    };
    Ok(authority)
}

fn receipt(identifier: &'static str) -> Result<ExecutionReceiptProjection, NegativeCorpusError> {
    let SemanticObject::ExecutionReceipt(receipt) = fixture(identifier)?.object else {
        return Err(NegativeCorpusError::UnexpectedPositiveObject(identifier));
    };
    Ok(receipt)
}

/// Candidate-neutral negative corpus.
///
/// The initial corpus pins exactly one deterministic case for every stable
/// semantic [`ValidationError`]. These are invalid typed meanings, not
/// malformed protocol bytes or parser vectors.
///
/// # Errors
///
/// Returns [`NegativeCorpusError`] if the positive operands drift from the
/// object or authority shapes required by the frozen negative corpus.
pub fn negative_fixtures() -> Result<Vec<NegativeFixture>, NegativeCorpusError> {
    let mut empty_identifier = persistent("persistent-blank-absent-key-generation")?;
    empty_identifier.context.profile_identifier.clear();

    let mut empty_subject = persistent("persistent-blank-absent-key-generation")?;
    empty_subject.context.subject.device_identifier.clear();

    let mut empty_required_value = receipt("receipt-minimal-optionals-absent")?;
    empty_required_value.key_identifier.clear();

    let mut wrong_object_class = persistent("persistent-blank-absent-key-generation")?;
    wrong_object_class.context.object_class = ObjectClass::ExecutionReceipt;

    let mut invalid_slot = persistent("persistent-blank-absent-key-generation")?;
    invalid_slot.slot_id = 2;

    let mut missing_record = authority("authority-prepared-applied")?;
    missing_record.record_commit_ids[1] = None;

    let mut unexpected_record = authority("authority-clean")?;
    unexpected_record.record_commit_ids[1] = Some(2);

    let mut slot_conflict = authority("authority-prepared-applied")?;
    let AuthorityPhaseProjection::Prepared { candidate_slot, .. } = &mut slot_conflict.phase else {
        return Err(NegativeCorpusError::UnexpectedAuthorityShape(
            "authority-prepared-applied",
        ));
    };
    *candidate_slot = slot_conflict.raw_selected_slot;

    let mut selector_mismatch = authority("authority-committed")?;
    selector_mismatch.raw_selected_slot = 0;

    let mut duplicate_extension = persistent("persistent-blank-absent-key-generation")?;
    duplicate_extension.context.extensions[1].identifier =
        duplicate_extension.context.extensions[0].identifier.clone();

    let mut unordered_extensions = persistent("persistent-blank-absent-key-generation")?;
    unordered_extensions.context.extensions.reverse();

    let mut commit_id_mismatch = authority("authority-prepared-applied")?;
    let AuthorityPhaseProjection::Prepared { commit_id, .. } = &mut commit_id_mismatch.phase else {
        return Err(NegativeCorpusError::UnexpectedAuthorityShape(
            "authority-prepared-applied",
        ));
    };
    *commit_id = 3;

    let receipt_for_phase = receipt("receipt-minimal-optionals-absent")?;
    let clean_authority = authority("authority-clean")?;

    let receipt_for_authority = receipt("receipt-minimal-optionals-absent")?;
    let mut mismatched_authority = authority("authority-committed")?;
    mismatched_authority
        .context
        .subject
        .device_identifier
        .push(0x01);

    let receipt_for_state = receipt("receipt-minimal-optionals-absent")?;
    let authority_for_state = authority("authority-committed")?;
    let mut mismatched_state = persistent("persistent-operational-receipt-release")?;
    mismatched_state.measurement_epoch += 1;

    let mut inconsistent_state = persistent("persistent-blank-absent-key-generation")?;
    inconsistent_state.context.subject.device_generation = 1;

    let mut inconsistent_execution = authority("authority-prepared-applied")?;
    let AuthorityPhaseProjection::Prepared {
        prepared_outcome: PreparedOutcomeProjection::Applied(execution),
        ..
    } = &mut inconsistent_execution.phase
    else {
        return Err(NegativeCorpusError::UnexpectedAuthorityShape(
            "authority-prepared-applied",
        ));
    };
    execution.receipt = Some(CurrentReceiptClaims {
        lifecycle_state: LifecycleState::Operational,
        device_generation: 1,
        transition_counter: 2,
        measurement_epoch: 0,
        receipt_sequence: 0,
        active_version: 1,
        challenge: None,
    });

    Ok(vec![
        NegativeFixture {
            identifier: "negative-empty-identifier",
            purpose: "required protected-context identifier is empty",
            case: NegativeCase::Object(SemanticObject::PersistentState(empty_identifier)),
            expected_error: ValidationError::EmptyIdentifier,
        },
        NegativeFixture {
            identifier: "negative-empty-subject",
            purpose: "protected subject has no device identifier",
            case: NegativeCase::Object(SemanticObject::PersistentState(empty_subject)),
            expected_error: ValidationError::EmptySubject,
        },
        NegativeFixture {
            identifier: "negative-empty-required-value",
            purpose: "receipt key identifier is present but empty",
            case: NegativeCase::Object(SemanticObject::ExecutionReceipt(empty_required_value)),
            expected_error: ValidationError::EmptyRequiredValue,
        },
        NegativeFixture {
            identifier: "negative-wrong-object-class",
            purpose: "persistent-state payload is domain-substituted as a receipt",
            case: NegativeCase::Object(SemanticObject::PersistentState(wrong_object_class)),
            expected_error: ValidationError::WrongObjectClass,
        },
        NegativeFixture {
            identifier: "negative-invalid-slot",
            purpose: "persistent record names a slot outside the two-slot model",
            case: NegativeCase::Object(SemanticObject::PersistentState(invalid_slot)),
            expected_error: ValidationError::InvalidSlot,
        },
        NegativeFixture {
            identifier: "negative-missing-record",
            purpose: "prepared authority omits its candidate record",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(missing_record)),
            expected_error: ValidationError::MissingRecord,
        },
        NegativeFixture {
            identifier: "negative-unexpected-record",
            purpose: "clean authority retains an untracked second record",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(unexpected_record)),
            expected_error: ValidationError::UnexpectedRecord,
        },
        NegativeFixture {
            identifier: "negative-slot-conflict",
            purpose: "prepared candidate aliases the selected previous slot",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(slot_conflict)),
            expected_error: ValidationError::SlotConflict,
        },
        NegativeFixture {
            identifier: "negative-selector-mismatch",
            purpose: "committed selector does not select the declared next record",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(selector_mismatch)),
            expected_error: ValidationError::SelectorMismatch,
        },
        NegativeFixture {
            identifier: "negative-duplicate-extension",
            purpose: "two retained extensions share one semantic identifier",
            case: NegativeCase::Object(SemanticObject::PersistentState(duplicate_extension)),
            expected_error: ValidationError::DuplicateExtension,
        },
        NegativeFixture {
            identifier: "negative-unordered-extensions",
            purpose: "extension set is supplied in non-increasing identifier order",
            case: NegativeCase::Object(SemanticObject::PersistentState(unordered_extensions)),
            expected_error: ValidationError::UnorderedExtensions,
        },
        NegativeFixture {
            identifier: "negative-commit-id-mismatch",
            purpose: "authority phase commit does not bind the candidate record",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(commit_id_mismatch)),
            expected_error: ValidationError::CommitIdMismatch,
        },
        NegativeFixture {
            identifier: "negative-authority-phase-mismatch",
            purpose: "valid receipt is presented under clean rather than committed authority",
            case: NegativeCase::ReceiptAuthority {
                receipt: receipt_for_phase,
                authority: clean_authority,
            },
            expected_error: ValidationError::AuthorityPhaseMismatch,
        },
        NegativeFixture {
            identifier: "negative-authority-context-mismatch",
            purpose: "valid receipt and committed authority name different subjects",
            case: NegativeCase::ReceiptAuthority {
                receipt: receipt_for_authority,
                authority: mismatched_authority,
            },
            expected_error: ValidationError::AuthorityContextMismatch,
        },
        NegativeFixture {
            identifier: "negative-state-context-mismatch",
            purpose: "complete release disagrees with authoritative measurement epoch",
            case: NegativeCase::ReceiptRelease {
                receipt: Box::new(receipt_for_state),
                authority: authority_for_state,
                state: mismatched_state,
            },
            expected_error: ValidationError::StateContextMismatch,
        },
        NegativeFixture {
            identifier: "negative-inconsistent-state",
            purpose: "protected subject generation contradicts persistent state",
            case: NegativeCase::Object(SemanticObject::PersistentState(inconsistent_state)),
            expected_error: ValidationError::InconsistentState,
        },
        NegativeFixture {
            identifier: "negative-inconsistent-execution",
            purpose: "prepared execution carries a zero receipt sequence",
            case: NegativeCase::Object(SemanticObject::AuthorityMetadata(inconsistent_execution)),
            expected_error: ValidationError::InconsistentExecution,
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_corpus_exercises_the_complete_receipt_release_operation() {
        assert_eq!(
            negative_fixtures()
                .expect("frozen negative corpus must construct")
                .into_iter()
                .filter(|fixture| matches!(fixture.case, NegativeCase::ReceiptRelease { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn every_negative_fixture_is_unique_and_fails_with_its_pinned_error() {
        let fixtures = negative_fixtures().expect("frozen negative corpus must construct");
        for (index, fixture) in fixtures.iter().enumerate() {
            assert!(!fixture.identifier.is_empty());
            assert!(!fixture.purpose.is_empty());
            assert_eq!(
                fixture.validate_expected(),
                Ok(()),
                "{}",
                fixture.identifier
            );
            for prior in &fixtures[..index] {
                assert_ne!(fixture.identifier, prior.identifier);
                assert_ne!(fixture.case, prior.case);
            }
        }
    }

    #[test]
    fn cross_object_negative_operands_are_individually_valid() {
        for fixture in negative_fixtures().expect("frozen negative corpus must construct") {
            match fixture.case {
                NegativeCase::Object(_) => {}
                NegativeCase::ReceiptAuthority { receipt, authority } => {
                    assert_eq!(receipt.validate(), Ok(()), "{}", fixture.identifier);
                    assert_eq!(authority.validate(), Ok(()), "{}", fixture.identifier);
                }
                NegativeCase::ReceiptState { receipt, state } => {
                    assert_eq!(receipt.validate(), Ok(()), "{}", fixture.identifier);
                    assert_eq!(state.validate(), Ok(()), "{}", fixture.identifier);
                }
                NegativeCase::ReceiptRelease {
                    receipt,
                    authority,
                    state,
                } => {
                    assert_eq!(receipt.validate(), Ok(()), "{}", fixture.identifier);
                    assert_eq!(authority.validate(), Ok(()), "{}", fixture.identifier);
                    assert_eq!(state.validate(), Ok(()), "{}", fixture.identifier);
                }
            }
        }
    }
}
