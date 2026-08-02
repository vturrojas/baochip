use super::*;
use baochip_model::Authorizations;

fn begin_provisioning() -> Command {
    Command::BeginProvisioning {
        authorizations: Authorizations {
            root: true,
            physical_presence: true,
            ..Authorizations::none()
        },
    }
}

fn decommission() -> Command {
    Command::Decommission {
        authorizations: Authorizations {
            decommission: true,
            independent: true,
            ..Authorizations::none()
        },
    }
}

fn prepared_model() -> DurableModel {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
}

fn committed_model() -> DurableModel {
    let mut durable = prepared_model();
    durable.commit().expect("commit selector");
    durable
}

fn operational_model() -> DurableModel {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .prepare(begin_provisioning())
        .expect("prepare begin");
    durable.commit().expect("commit begin");
    durable.cleanup().expect("cleanup begin");
    durable
        .prepare(Command::CommitProvisioning {
            authorizations: Authorizations {
                root: true,
                owner: true,
                ..Authorizations::none()
            },
        })
        .expect("prepare provisioning commit");
    durable.commit().expect("commit provisioning");
    durable.cleanup().expect("cleanup provisioning");
    durable
}

#[test]
fn prepared_empty_slot_is_rejected_without_mutation() {
    let mut durable = prepared_model();
    let PersistencePhase::Prepared { slot, .. } = durable.phase else {
        panic!("expected prepared phase");
    };
    durable.slots[slot] = None;
    let before = durable.clone();

    assert_eq!(
        durable.commit(),
        Err(PersistenceError::MissingCandidateRecord)
    );
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingCandidateRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn prepared_recovery_validates_active_before_discarding_candidate() {
    let mut durable = prepared_model();
    durable.slots[durable.active_slot] = None;
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn prepared_slot_equal_to_selector_is_rejected_without_erasing_authority() {
    let mut durable = prepared_model();
    let commit_id = durable.active_commit_id().expect("active commit") + 1;
    durable.phase = PersistencePhase::Prepared {
        slot: durable.active_slot,
        commit_id,
    };
    let before = durable.clone();

    assert_eq!(durable.commit(), Err(PersistenceError::SlotConflict));
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::SlotConflict)
    );
    assert_eq!(durable, before);
}

#[test]
fn prepared_commit_id_must_match_candidate_record() {
    let mut durable = prepared_model();
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase else {
        panic!("expected prepared phase");
    };
    durable.phase = PersistencePhase::Prepared {
        slot,
        commit_id: commit_id + 1,
    };
    let before = durable.clone();

    assert_eq!(durable.commit(), Err(PersistenceError::CommitIdMismatch));
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn prepared_identifier_must_be_the_active_successor() {
    let mut durable = prepared_model();
    let PersistencePhase::Prepared { slot, .. } = durable.phase else {
        panic!("expected prepared phase");
    };
    durable.slots[slot]
        .as_mut()
        .expect("candidate record")
        .commit_id = 7;
    durable.phase = PersistencePhase::Prepared { slot, commit_id: 7 };
    let before = durable.clone();

    assert_eq!(durable.commit(), Err(PersistenceError::CommitIdMismatch));
    assert_eq!(durable, before);
}

#[test]
fn prepared_phase_requires_its_candidate_outcome() {
    let mut durable = prepared_model();
    durable.prepared_outcome = None;
    let before = durable.clone();

    assert_eq!(
        durable.commit(),
        Err(PersistenceError::MissingPreparedOutcome)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_phase_selector_must_match_actual_selector() {
    let mut durable = committed_model();
    let PersistencePhase::Committed {
        previous_slot,
        commit_id,
        ..
    } = durable.phase
    else {
        panic!("expected committed phase");
    };
    durable.phase = PersistencePhase::Committed {
        previous_slot,
        active_slot: previous_slot,
        commit_id,
    };
    let before = durable.clone();

    assert_eq!(durable.cleanup(), Err(PersistenceError::SelectorMismatch));
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::SelectorMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_previous_slot_cannot_equal_authoritative_slot() {
    let mut durable = committed_model();
    let PersistencePhase::Committed {
        active_slot,
        commit_id,
        ..
    } = durable.phase
    else {
        panic!("expected committed phase");
    };
    durable.phase = PersistencePhase::Committed {
        previous_slot: active_slot,
        active_slot,
        commit_id,
    };
    let before = durable.clone();

    assert_eq!(durable.cleanup(), Err(PersistenceError::SlotConflict));
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::SlotConflict)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_commit_id_must_match_selected_record() {
    let mut durable = committed_model();
    let PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id,
    } = durable.phase
    else {
        panic!("expected committed phase");
    };
    durable.phase = PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id: commit_id + 1,
    };
    let before = durable.clone();

    assert_eq!(durable.cleanup(), Err(PersistenceError::CommitIdMismatch));
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_phase_requires_the_previous_record() {
    let mut durable = committed_model();
    let PersistencePhase::Committed { previous_slot, .. } = durable.phase else {
        panic!("expected committed phase");
    };
    durable.slots[previous_slot] = None;
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingPreviousRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_recovery_validates_selected_before_clearing_previous() {
    let mut durable = committed_model();
    durable.slots[durable.active_slot] = None;
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_phase_rejects_a_leftover_prepared_outcome() {
    let mut durable = committed_model();
    durable.prepared_outcome = Some(CommandOutcome::Rejected(Rejection::InvalidState));
    let before = durable.clone();

    assert_eq!(
        durable.cleanup(),
        Err(PersistenceError::UnexpectedPreparedOutcome)
    );
    assert_eq!(durable, before);
}

#[test]
fn committed_identifier_cannot_regress_behind_previous_record() {
    let mut durable = committed_model();
    let PersistencePhase::Committed {
        previous_slot,
        active_slot,
        ..
    } = durable.phase
    else {
        panic!("expected committed phase");
    };
    durable.slots[active_slot]
        .as_mut()
        .expect("selected record")
        .commit_id = 0;
    durable.phase = PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id: 0,
    };
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn empty_authoritative_slot_is_a_stable_error() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.slots[durable.active_slot] = None;
    let before = durable.clone();

    assert_eq!(
        durable.active_state(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(
        durable.active_commit_id(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(
        durable.prepare(begin_provisioning()),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(durable, before);
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn clean_phase_rejects_an_untracked_record() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.slots[1] = Some(Record {
        commit_id: 1,
        state: StateMachine::new(),
        integrity: IntegrityVerdict::Valid,
    });
    let before = durable.clone();

    assert_eq!(
        durable.prepare(begin_provisioning()),
        Err(PersistenceError::UnexpectedRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn malformed_slot_indices_return_stable_errors_without_panicking() {
    let mut invalid_selector = DurableModel::new(StateMachine::new());
    invalid_selector.active_slot = 2;
    let selector_before = invalid_selector.clone();
    assert_eq!(
        invalid_selector.active_state(),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(
        invalid_selector.prepare(begin_provisioning()),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(invalid_selector, selector_before);

    let mut invalid_candidate = prepared_model();
    invalid_candidate.phase = PersistencePhase::Prepared {
        slot: 2,
        commit_id: 1,
    };
    let candidate_before = invalid_candidate.clone();
    assert_eq!(
        invalid_candidate.commit(),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(invalid_candidate, candidate_before);

    let mut invalid_previous = committed_model();
    let active_slot = invalid_previous.active_slot;
    invalid_previous.phase = PersistencePhase::Committed {
        previous_slot: 2,
        active_slot,
        commit_id: 1,
    };
    let previous_before = invalid_previous.clone();
    assert_eq!(
        invalid_previous.cleanup(),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(invalid_previous, previous_before);
}

#[test]
fn prepared_crash_allows_a_new_prepare_without_authoritative_id_regression() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare one");
    assert_eq!(durable.active_commit_id(), Ok(0));
    durable.crash_and_recover().expect("recover previous");
    assert_eq!(durable.active_commit_id(), Ok(0));

    let result = durable
        .prepare(begin_provisioning())
        .expect("prepare again");
    assert!(matches!(
        result,
        PrepareResult::Staged {
            audit: PersistenceAudit { commit_id: 1, .. },
            ..
        }
    ));
    assert_eq!(durable.active_commit_id(), Ok(0));
}

#[test]
fn committed_crash_supports_the_next_transaction() {
    let mut durable = committed_model();
    durable.crash_and_recover().expect("recover next");
    assert_eq!(durable.active_commit_id(), Ok(1));

    durable.prepare(decommission()).expect("prepare next");
    durable.commit().expect("commit next");
    durable.crash_and_recover().expect("recover second next");
    assert_eq!(durable.active_commit_id(), Ok(2));
    assert_eq!(
        durable.active_state().expect("active").lifecycle(),
        LifecycleState::Decommissioned
    );
}

#[test]
fn cleanup_supports_multiple_additional_transactions() {
    let mut durable = DurableModel::new(StateMachine::new());
    for expected_commit in 1..=3 {
        let command = if expected_commit == 1 {
            begin_provisioning()
        } else if expected_commit == 2 {
            Command::AbortProvisioning
        } else {
            decommission()
        };
        durable.prepare(command).expect("prepare transaction");
        durable.commit().expect("commit transaction");
        durable.cleanup().expect("cleanup transaction");
        assert_eq!(durable.active_commit_id(), Ok(expected_commit));
    }
}

#[test]
fn invalid_ordering_preserves_storage_for_later_valid_recovery() {
    let mut durable = DurableModel::new(StateMachine::new());
    let initial = durable.clone();
    assert_eq!(durable.commit(), Err(PersistenceError::NoPreparedRecord));
    assert_eq!(durable.cleanup(), Err(PersistenceError::NoCommittedRecord));
    assert_eq!(durable, initial);

    durable.prepare(begin_provisioning()).expect("prepare");
    assert_eq!(durable.cleanup(), Err(PersistenceError::NoCommittedRecord));
    durable
        .crash_and_recover()
        .expect("recover valid prepared state");
    assert_eq!(durable, initial);
}

#[test]
fn unchanged_rejection_audit_binds_both_lifecycles_and_active_commit() {
    let mut durable = DurableModel::new(StateMachine::new());
    let result = durable
        .prepare(Command::IssueReceipt { challenge: None })
        .expect("modeled rejection");

    assert_eq!(
        result,
        PrepareResult::NotStaged {
            rejection: Rejection::InvalidState,
            audit: PersistenceAudit {
                operation: PersistenceOperation::RejectedWithoutChange,
                prior_lifecycle: LifecycleState::Blank,
                resulting_lifecycle: LifecycleState::Blank,
                commit_id: 0,
            },
        }
    );
    assert_eq!(durable.phase(), PersistencePhase::Clean);
    assert_eq!(durable.active_commit_id(), Ok(0));
}

#[test]
fn selector_commit_is_the_only_authority_change() {
    let initial = StateMachine::new();
    let mut next = initial.clone();
    next.apply(begin_provisioning()).expect("expected next");
    let mut durable = DurableModel::new(initial.clone());

    durable.prepare(begin_provisioning()).expect("prepare");
    assert_eq!(durable.active_state(), Ok(&initial));
    assert_eq!(durable.active_commit_id(), Ok(0));

    durable.commit().expect("selector commit");
    assert_eq!(durable.active_state(), Ok(&next));
    assert_eq!(durable.active_commit_id(), Ok(1));
}

#[test]
fn cleanup_audit_distinguishes_obsolete_and_authoritative_lifecycles() {
    let mut durable = committed_model();

    let audit = durable.cleanup().expect("cleanup");

    assert_eq!(audit.prior_lifecycle, LifecycleState::Blank);
    assert_eq!(audit.resulting_lifecycle, LifecycleState::Provisioning);
    assert_eq!(audit.commit_id, 1);
}

#[test]
fn command_outcome_is_withheld_until_selector_commit() {
    let mut durable = operational_model();
    let staged = durable
        .prepare(Command::IssueReceipt {
            challenge: Some([0xA5; 16]),
        })
        .expect("prepare receipt");

    let PrepareResult::Staged { audit } = staged else {
        panic!("receipt must stage");
    };
    assert_eq!(audit.operation, PersistenceOperation::Prepared);
    assert_eq!(
        durable.active_state().expect("active").receipt_sequence(),
        0
    );

    let committed = durable.commit().expect("commit selector");
    let CommandOutcome::Applied(execution) = committed.outcome else {
        panic!("receipt command must apply");
    };
    assert_eq!(
        committed.audit.operation,
        PersistenceOperation::SelectorCommitted
    );
    assert_eq!(
        execution.receipt.expect("receipt claims").receipt_sequence,
        1
    );
    assert_eq!(
        durable.active_state().expect("active").receipt_sequence(),
        1
    );
}
