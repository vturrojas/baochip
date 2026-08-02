use super::*;
use baochip_model::{Authorizations, ModelTestFault};

fn begin_provisioning() -> Command {
    Command::BeginProvisioning {
        authorizations: Authorizations {
            root: true,
            physical_presence: true,
            ..Authorizations::none()
        },
    }
}

fn commit_provisioning() -> Command {
    Command::CommitProvisioning {
        authorizations: Authorizations {
            root: true,
            owner: true,
            ..Authorizations::none()
        },
    }
}

fn operational_state() -> StateMachine {
    let mut state = StateMachine::new();
    state
        .apply(begin_provisioning())
        .expect("begin provisioning");
    state
        .apply(commit_provisioning())
        .expect("commit provisioning");
    state
}

#[test]
fn corrupted_active_record_fails_closed_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::ActiveRecord)
        .expect("inject active corruption");
    let before = durable.clone();

    assert_eq!(
        durable.active_state(),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(
        durable.prepare(begin_provisioning()),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_prepared_candidate_is_discarded_without_promotion() {
    let initial = StateMachine::new();
    let mut durable = DurableModel::new(initial.clone());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt candidate");

    let audit = durable
        .crash_and_recover()
        .expect("recover authoritative previous record");

    assert_eq!(
        audit,
        PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityPrevious,
            prior_lifecycle: LifecycleState::Blank,
            resulting_lifecycle: LifecycleState::Blank,
            commit_id: 0,
        }
    );
    assert_eq!(durable.active_state(), Ok(&initial));
    assert_eq!(durable.active_commit_id(), Ok(0));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn corrupted_committed_previous_record_is_discarded() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt obsolete previous record");

    let audit = durable
        .crash_and_recover()
        .expect("recover selected next record");

    assert_eq!(
        audit,
        PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityNext,
            prior_lifecycle: LifecycleState::Provisioning,
            resulting_lifecycle: LifecycleState::Provisioning,
            commit_id: 1,
        }
    );
    assert_eq!(
        durable.active_state().expect("active").lifecycle(),
        LifecycleState::Provisioning
    );
    assert_eq!(durable.active_commit_id(), Ok(1));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn corrupted_selector_in_clean_phase_recovers_sole_valid_record() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");

    let audit = durable
        .crash_and_recover()
        .expect("recover sole valid record");

    assert_eq!(
        audit.operation,
        PersistenceOperation::RecoveredIntegritySoleValid
    );
    assert_eq!(durable.active_commit_id(), Ok(0));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn corrupted_selector_in_prepared_phase_recovers_previous() {
    let initial = StateMachine::new();
    let mut durable = DurableModel::new(initial.clone());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");

    let audit = durable
        .crash_and_recover()
        .expect("prepared phase identifies previous authority");

    assert_eq!(
        audit.operation,
        PersistenceOperation::RecoveredIntegrityPrevious
    );
    assert_eq!(durable.active_state(), Ok(&initial));
    assert_eq!(durable.active_commit_id(), Ok(0));
}

#[test]
fn corrupted_selector_in_committed_phase_recovers_selected_next() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");

    let audit = durable
        .crash_and_recover()
        .expect("committed metadata identifies selected next");

    assert_eq!(
        audit.operation,
        PersistenceOperation::RecoveredIntegrityNext
    );
    assert_eq!(
        durable.active_state().expect("active").lifecycle(),
        LifecycleState::Provisioning
    );
    assert_eq!(durable.active_commit_id(), Ok(1));
}

#[test]
fn corrupted_clean_selector_ignores_out_of_range_raw_active_slot_and_recovers_sole_valid() {
    let initial = StateMachine::new();
    let mut durable = DurableModel::new(initial.clone());
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    durable.active_slot = usize::MAX;

    assert_eq!(
        durable.crash_and_recover(),
        Ok(PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegritySoleValid,
            prior_lifecycle: LifecycleState::Blank,
            resulting_lifecycle: LifecycleState::Blank,
            commit_id: 0,
        })
    );
    assert_eq!(durable.active_slot, 0);
    assert_eq!(durable.active_state(), Ok(&initial));
    assert_eq!(durable.active_commit_id(), Ok(0));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn corrupted_prepared_selector_ignores_out_of_range_raw_active_slot_and_recovers_previous() {
    let initial = StateMachine::new();
    let expected = DurableModel::new(initial.clone());
    let mut durable = DurableModel::new(initial);
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    durable.active_slot = usize::MAX;

    assert_eq!(
        durable.crash_and_recover(),
        Ok(PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityPrevious,
            prior_lifecycle: LifecycleState::Provisioning,
            resulting_lifecycle: LifecycleState::Blank,
            commit_id: 0,
        })
    );
    assert_eq!(durable.active_slot, 0);
    assert_eq!(durable.phase(), PersistencePhase::Clean);
    assert_eq!(durable, expected);
}

#[test]
fn corrupted_committed_selector_ignores_out_of_range_raw_active_slot_and_recovers_selected() {
    let mut expected = StateMachine::new();
    expected
        .apply(begin_provisioning())
        .expect("derive selected state");
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    let PersistencePhase::Committed { active_slot, .. } = durable.phase else {
        panic!("expected committed phase");
    };
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    durable.active_slot = usize::MAX;

    assert_eq!(
        durable.crash_and_recover(),
        Ok(PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityNext,
            prior_lifecycle: LifecycleState::Blank,
            resulting_lifecycle: LifecycleState::Provisioning,
            commit_id: 1,
        })
    );
    assert_eq!(durable.active_slot, active_slot);
    assert_eq!(durable.active_state(), Ok(&expected));
    assert_eq!(durable.active_commit_id(), Ok(1));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn corrupted_selected_committed_record_never_rolls_back() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    durable
        .inject_test_fault(IntegrityTestFault::ActiveRecord)
        .expect("corrupt selected next");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_selector_with_no_valid_record_is_unrecoverable() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::AllRecords)
        .expect("corrupt records");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::NoValidRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_selector_with_two_valid_records_is_ambiguous() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::DuplicateActiveRecord)
        .expect("duplicate valid record");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::AmbiguousIntegrityRecovery)
    );
    assert_eq!(durable, before);
}

#[test]
fn fault_producing_rejection_is_committed_durably() {
    let mut state = operational_state();
    state.inject_test_fault(ModelTestFault::ReceiptSequenceExhausted);
    let mut durable = DurableModel::new(state);

    let prepared = durable
        .prepare(Command::IssueReceipt { challenge: None })
        .expect("fault transition must stage");
    assert!(matches!(prepared, PrepareResult::Staged { .. }));
    assert_eq!(
        durable
            .active_state()
            .expect("previous remains active")
            .lifecycle(),
        LifecycleState::Operational
    );

    let committed = durable.commit().expect("commit fault state");
    assert_eq!(
        committed.outcome,
        CommandOutcome::Rejected(Rejection::CounterExhausted)
    );
    assert_eq!(
        durable
            .active_state()
            .expect("fault is authoritative")
            .lifecycle(),
        LifecycleState::Fault
    );
}

#[test]
fn corrupted_prepared_candidate_still_rejects_forged_phase_commit_id_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase else {
        panic!("expected prepared phase");
    };
    durable.phase = PersistencePhase::Prepared {
        slot,
        commit_id: commit_id + 1,
    };
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt candidate");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_prepared_candidate_still_requires_outcome_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.prepared_outcome = None;
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt candidate");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingPreparedOutcome)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_committed_previous_rejects_unexpected_outcome_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt previous");
    durable.prepared_outcome = Some(CommandOutcome::Rejected(Rejection::InvalidState));
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::UnexpectedPreparedOutcome)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_clean_selector_rejects_unexpected_outcome_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    durable.prepared_outcome = Some(CommandOutcome::Rejected(Rejection::InvalidState));
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::UnexpectedPreparedOutcome)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_committed_selector_still_requires_previous_record_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    let PersistencePhase::Committed { previous_slot, .. } = durable.phase else {
        panic!("expected committed phase");
    };
    durable.slots[previous_slot] = None;
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::MissingPreviousRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_prepared_selector_audit_binds_candidate_to_previous_lifecycle() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");

    let audit = durable.crash_and_recover().expect("recover previous");

    assert_eq!(
        audit,
        PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityPrevious,
            prior_lifecycle: LifecycleState::Provisioning,
            resulting_lifecycle: LifecycleState::Blank,
            commit_id: 0,
        }
    );
}

#[test]
fn corrupted_committed_selector_audit_binds_previous_to_selected_lifecycle() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");

    let audit = durable.crash_and_recover().expect("recover selected");

    assert_eq!(
        audit,
        PersistenceAudit {
            operation: PersistenceOperation::RecoveredIntegrityNext,
            prior_lifecycle: LifecycleState::Blank,
            resulting_lifecycle: LifecycleState::Provisioning,
            commit_id: 1,
        }
    );
}

#[test]
fn corrupted_selector_with_out_of_range_phase_indices_fails_without_mutation_or_panic() {
    let mut prepared = DurableModel::new(StateMachine::new());
    prepared.prepare(begin_provisioning()).expect("prepare");
    prepared.phase = PersistencePhase::Prepared {
        slot: 2,
        commit_id: 1,
    };
    prepared
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt prepared selector");
    let prepared_before = prepared.clone();

    assert_eq!(
        prepared.crash_and_recover(),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(prepared, prepared_before);

    let mut committed = DurableModel::new(StateMachine::new());
    committed.prepare(begin_provisioning()).expect("prepare");
    committed.commit().expect("commit selector");
    let PersistencePhase::Committed {
        active_slot,
        commit_id,
        ..
    } = committed.phase
    else {
        panic!("expected committed phase");
    };
    committed.phase = PersistencePhase::Committed {
        previous_slot: 2,
        active_slot,
        commit_id,
    };
    committed
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt committed selector");
    let committed_before = committed.clone();

    assert_eq!(
        committed.crash_and_recover(),
        Err(PersistenceError::InvalidSlotIndex)
    );
    assert_eq!(committed, committed_before);
}

#[test]
fn rejected_fault_transition_crash_before_commit_restores_exact_prior_and_withholds_outcome() {
    let mut state = operational_state();
    state.inject_test_fault(ModelTestFault::ReceiptSequenceExhausted);
    let mut durable = DurableModel::new(state);
    let exact_prior = durable.clone();

    assert_eq!(
        durable
            .prepare(Command::IssueReceipt { challenge: None })
            .expect("fault transition stages"),
        PrepareResult::Staged {
            audit: PersistenceAudit {
                operation: PersistenceOperation::Prepared,
                prior_lifecycle: LifecycleState::Operational,
                resulting_lifecycle: LifecycleState::Fault,
                commit_id: 1,
            },
        }
    );

    assert_eq!(
        durable.crash_and_recover(),
        Ok(PersistenceAudit {
            operation: PersistenceOperation::RecoveredPrevious,
            prior_lifecycle: LifecycleState::Fault,
            resulting_lifecycle: LifecycleState::Operational,
            commit_id: 0,
        })
    );
    assert_eq!(durable, exact_prior);
}

#[test]
fn rejected_fault_transition_crash_after_commit_preserves_exact_fault_snapshot_and_audit() {
    let mut prior = operational_state();
    prior.inject_test_fault(ModelTestFault::ReceiptSequenceExhausted);
    let mut expected_fault = prior.clone();
    assert_eq!(
        expected_fault.apply(Command::IssueReceipt { challenge: None }),
        Err(Rejection::CounterExhausted)
    );
    let mut durable = DurableModel::new(prior);
    durable
        .prepare(Command::IssueReceipt { challenge: None })
        .expect("fault transition stages");

    assert_eq!(
        durable.commit().expect("commit fault state"),
        CommitResult {
            outcome: CommandOutcome::Rejected(Rejection::CounterExhausted),
            audit: PersistenceAudit {
                operation: PersistenceOperation::SelectorCommitted,
                prior_lifecycle: LifecycleState::Operational,
                resulting_lifecycle: LifecycleState::Fault,
                commit_id: 1,
            },
        }
    );
    assert_eq!(durable.active_state(), Ok(&expected_fault));

    assert_eq!(
        durable.crash_and_recover(),
        Ok(PersistenceAudit {
            operation: PersistenceOperation::RecoveredNext,
            prior_lifecycle: LifecycleState::Operational,
            resulting_lifecycle: LifecycleState::Fault,
            commit_id: 1,
        })
    );
    assert_eq!(durable.active_state(), Ok(&expected_fault));
    assert_eq!(durable.active_commit_id(), Ok(1));
    assert_eq!(durable.phase(), PersistencePhase::Clean);
}

#[test]
fn valid_prepared_candidate_never_replaces_corrupted_previous_authority() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable
        .inject_test_fault(IntegrityTestFault::ActiveRecord)
        .expect("corrupt previous authority");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn clean_valid_selector_never_promotes_valid_duplicate_over_corrupted_selected_record() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable
        .inject_test_fault(IntegrityTestFault::DuplicateActiveRecord)
        .expect("duplicate selected record");
    durable
        .inject_test_fault(IntegrityTestFault::ActiveRecord)
        .expect("corrupt selected record");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CorruptedActiveRecord)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_prepared_selector_rejects_malformed_phase_commit_id_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase else {
        panic!("expected prepared phase");
    };
    durable.phase = PersistencePhase::Prepared {
        slot,
        commit_id: commit_id + 1,
    };
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_committed_selector_rejects_malformed_phase_commit_id_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
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
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_previous_still_rejects_phase_id_mismatched_with_selected_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
    let PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id,
    } = durable.phase
    else {
        panic!("expected committed phase");
    };
    durable
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt previous record");
    durable.phase = PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id: commit_id + 1,
    };
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::CommitIdMismatch)
    );
    assert_eq!(durable, before);
}

#[test]
fn corrupted_committed_selector_rejects_slot_conflict_without_mutation() {
    let mut durable = DurableModel::new(StateMachine::new());
    durable.prepare(begin_provisioning()).expect("prepare");
    durable.commit().expect("commit selector");
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
    durable
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let before = durable.clone();

    assert_eq!(
        durable.crash_and_recover(),
        Err(PersistenceError::SlotConflict)
    );
    assert_eq!(durable, before);
}

#[test]
fn missing_selected_record_is_stable_with_corrupted_previous_or_selector() {
    let mut corrupted_previous = DurableModel::new(StateMachine::new());
    corrupted_previous
        .prepare(begin_provisioning())
        .expect("prepare");
    corrupted_previous.commit().expect("commit selector");
    corrupted_previous
        .inject_test_fault(IntegrityTestFault::InactiveRecord)
        .expect("corrupt previous record");
    let selected_slot = corrupted_previous.active_slot;
    corrupted_previous.slots[selected_slot] = None;
    let corrupted_previous_before = corrupted_previous.clone();

    assert_eq!(
        corrupted_previous.crash_and_recover(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(corrupted_previous, corrupted_previous_before);

    let mut corrupted_selector = DurableModel::new(StateMachine::new());
    corrupted_selector
        .prepare(begin_provisioning())
        .expect("prepare");
    corrupted_selector.commit().expect("commit selector");
    corrupted_selector
        .inject_test_fault(IntegrityTestFault::Selector)
        .expect("corrupt selector");
    let selected_slot = corrupted_selector.active_slot;
    corrupted_selector.slots[selected_slot] = None;
    let corrupted_selector_before = corrupted_selector.clone();

    assert_eq!(
        corrupted_selector.crash_and_recover(),
        Err(PersistenceError::MissingActiveRecord)
    );
    assert_eq!(corrupted_selector, corrupted_selector_before);
}
