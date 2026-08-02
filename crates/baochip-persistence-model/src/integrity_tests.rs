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
        audit.operation,
        PersistenceOperation::RecoveredIntegrityPrevious
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
        audit.operation,
        PersistenceOperation::RecoveredIntegrityNext
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
