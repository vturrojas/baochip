use super::*;
use baochip_model::{
    Authorizations, Command, Execution as ModelExecution, LifecycleState as ModelLifecycleState,
    ModelTestFault, ReceiptClaims as ModelReceiptClaims, Rejection as ModelRejection, StateMachine,
    UpdateValidation,
};
use baochip_persistence_model::{
    CommandOutcome, DurableModel, PersistenceOperation, PersistencePhase, PrepareResult,
};

fn fixture(identifier: &str) -> Fixture {
    positive_fixtures()
        .into_iter()
        .find(|fixture| fixture.identifier == identifier)
        .expect("fixture must exist")
}

fn root_physical() -> Authorizations {
    Authorizations {
        root: true,
        physical_presence: true,
        ..Authorizations::none()
    }
}

fn root_owner() -> Authorizations {
    Authorizations {
        root: true,
        owner: true,
        ..Authorizations::none()
    }
}

fn update_owner() -> Authorizations {
    Authorizations {
        owner: true,
        update: true,
        ..Authorizations::none()
    }
}

fn provisioned_machine() -> StateMachine {
    let mut machine = StateMachine::new();
    machine
        .apply(Command::BeginProvisioning {
            authorizations: root_physical(),
        })
        .expect("initial provisioning must begin");
    machine
        .apply(Command::CommitProvisioning {
            authorizations: root_owner(),
        })
        .expect("initial provisioning must commit");
    machine
}

fn receipt_release_machine() -> (StateMachine, ModelReceiptClaims) {
    let mut machine = provisioned_machine();
    machine
        .apply(Command::StageUpdate {
            authorizations: update_owner(),
            candidate_version: 4,
        })
        .expect("update must stage");
    machine
        .apply(Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        })
        .expect("update must activate");
    for _ in 0..3 {
        machine
            .apply(Command::StartMeasurementEpoch)
            .expect("measurement epoch must advance");
    }
    let execution = machine
        .apply(Command::IssueReceipt { challenge: None })
        .expect("operational identity must issue a receipt");
    let receipt = execution.receipt.expect("receipt command must emit claims");
    (machine, receipt)
}

fn map_lifecycle(state: ModelLifecycleState) -> LifecycleState {
    match state {
        ModelLifecycleState::Blank => LifecycleState::Blank,
        ModelLifecycleState::Provisioning => LifecycleState::Provisioning,
        ModelLifecycleState::Operational => LifecycleState::Operational,
        ModelLifecycleState::UpdatePending => LifecycleState::UpdatePending,
        ModelLifecycleState::Recovery => LifecycleState::Recovery,
        ModelLifecycleState::Revoked => LifecycleState::Revoked,
        ModelLifecycleState::Decommissioned => LifecycleState::Decommissioned,
        ModelLifecycleState::Fault => LifecycleState::Fault,
    }
}

fn map_rejection(rejection: ModelRejection) -> RejectionClass {
    match rejection {
        ModelRejection::InvalidState => RejectionClass::InvalidState,
        ModelRejection::Unauthorized => RejectionClass::Unauthorized,
        ModelRejection::InvalidTransition => RejectionClass::InvalidTransition,
        ModelRejection::InvalidVersion => RejectionClass::InvalidVersion,
        ModelRejection::RollbackDetected => RejectionClass::RollbackDetected,
        ModelRejection::ReplayDetected => RejectionClass::ReplayDetected,
        ModelRejection::MalformedInput => RejectionClass::MalformedInput,
        ModelRejection::UnsupportedProfile => RejectionClass::UnsupportedProfile,
        ModelRejection::UnsupportedCriticalExtension => {
            RejectionClass::UnsupportedCriticalExtension
        }
        ModelRejection::PersistenceFailure => RejectionClass::PersistenceFailure,
        ModelRejection::CounterExhausted => RejectionClass::CounterExhausted,
        ModelRejection::IntegrityFailure => RejectionClass::IntegrityFailure,
        ModelRejection::Decommissioned => RejectionClass::Decommissioned,
        ModelRejection::InternalInvariantViolation => RejectionClass::InternalInvariantViolation,
    }
}

fn map_execution(execution: ModelExecution) -> ExecutionProjection {
    ExecutionProjection {
        audit: LifecycleAuditProjection {
            previous_state: map_lifecycle(execution.audit.previous_state),
            resulting_state: map_lifecycle(execution.audit.resulting_state),
            device_generation: execution.audit.device_generation,
            staged_device_generation: execution.audit.staged_device_generation,
            transition_counter: execution.audit.transition_counter,
        },
        receipt: execution.receipt.map(|receipt| CurrentReceiptClaims {
            lifecycle_state: map_lifecycle(receipt.lifecycle_state),
            device_generation: receipt.device_generation,
            transition_counter: receipt.transition_counter,
            measurement_epoch: receipt.measurement_epoch,
            receipt_sequence: receipt.receipt_sequence,
            active_version: receipt.active_version,
            challenge: receipt.challenge,
        }),
    }
}

fn assert_machine_matches_projection(
    machine: &StateMachine,
    projection: &PersistentStateProjection,
) {
    assert_eq!(
        map_lifecycle(machine.lifecycle()),
        projection.lifecycle_state
    );
    assert_eq!(machine.device_generation(), projection.device_generation);
    assert_eq!(machine.transition_counter(), projection.transition_counter);
    assert_eq!(machine.measurement_epoch(), projection.measurement_epoch);
    assert_eq!(machine.receipt_sequence(), projection.receipt_sequence);
    assert_eq!(machine.active_version(), projection.active_version);
}

#[test]
fn lifecycle_and_rejection_domains_are_exhaustively_mapped() {
    let lifecycle_pairs = [
        (ModelLifecycleState::Blank, LifecycleState::Blank),
        (
            ModelLifecycleState::Provisioning,
            LifecycleState::Provisioning,
        ),
        (
            ModelLifecycleState::Operational,
            LifecycleState::Operational,
        ),
        (
            ModelLifecycleState::UpdatePending,
            LifecycleState::UpdatePending,
        ),
        (ModelLifecycleState::Recovery, LifecycleState::Recovery),
        (ModelLifecycleState::Revoked, LifecycleState::Revoked),
        (
            ModelLifecycleState::Decommissioned,
            LifecycleState::Decommissioned,
        ),
        (ModelLifecycleState::Fault, LifecycleState::Fault),
    ];
    for (model, fixture) in lifecycle_pairs {
        assert_eq!(map_lifecycle(model), fixture);
    }

    let rejection_pairs = [
        (ModelRejection::InvalidState, RejectionClass::InvalidState),
        (ModelRejection::Unauthorized, RejectionClass::Unauthorized),
        (
            ModelRejection::InvalidTransition,
            RejectionClass::InvalidTransition,
        ),
        (
            ModelRejection::InvalidVersion,
            RejectionClass::InvalidVersion,
        ),
        (
            ModelRejection::RollbackDetected,
            RejectionClass::RollbackDetected,
        ),
        (
            ModelRejection::ReplayDetected,
            RejectionClass::ReplayDetected,
        ),
        (
            ModelRejection::MalformedInput,
            RejectionClass::MalformedInput,
        ),
        (
            ModelRejection::UnsupportedProfile,
            RejectionClass::UnsupportedProfile,
        ),
        (
            ModelRejection::UnsupportedCriticalExtension,
            RejectionClass::UnsupportedCriticalExtension,
        ),
        (
            ModelRejection::PersistenceFailure,
            RejectionClass::PersistenceFailure,
        ),
        (
            ModelRejection::CounterExhausted,
            RejectionClass::CounterExhausted,
        ),
        (
            ModelRejection::IntegrityFailure,
            RejectionClass::IntegrityFailure,
        ),
        (
            ModelRejection::Decommissioned,
            RejectionClass::Decommissioned,
        ),
        (
            ModelRejection::InternalInvariantViolation,
            RejectionClass::InternalInvariantViolation,
        ),
    ];
    for (model, fixture) in rejection_pairs {
        assert_eq!(map_rejection(model), fixture);
    }
}

#[test]
fn lifecycle_checkpoints_match_frozen_semantic_projections() {
    let machine = StateMachine::new();
    let SemanticObject::PersistentState(blank) =
        fixture("persistent-blank-absent-key-generation").object
    else {
        panic!("expected blank persistent fixture");
    };
    assert_machine_matches_projection(&machine, &blank);

    let mut machine = StateMachine::new();
    machine
        .apply(Command::BeginProvisioning {
            authorizations: root_physical(),
        })
        .expect("initial provisioning must begin");
    let SemanticObject::PersistentState(provisioning) =
        fixture("persistent-provisioning-initial").object
    else {
        panic!("expected provisioning persistent fixture");
    };
    assert_machine_matches_projection(&machine, &provisioning);

    let (machine, claims) = receipt_release_machine();
    let SemanticObject::PersistentState(release_state) =
        fixture("persistent-operational-receipt-release").object
    else {
        panic!("expected release persistent fixture");
    };
    assert_machine_matches_projection(&machine, &release_state);
    assert_eq!(claims.lifecycle_state, ModelLifecycleState::Operational);
    assert_eq!(claims.device_generation, release_state.device_generation);
    assert_eq!(claims.transition_counter, release_state.transition_counter);
    assert_eq!(claims.measurement_epoch, release_state.measurement_epoch);
    assert_eq!(claims.receipt_sequence, release_state.receipt_sequence);
    assert_eq!(claims.active_version, release_state.active_version);
}

#[test]
fn model_receipt_claims_satisfy_complete_semantic_release_binding() {
    let (_, claims) = receipt_release_machine();
    let receipt = ExecutionReceiptProjection {
        context: context(
            ObjectClass::ExecutionReceipt,
            claims.device_generation,
            None,
        ),
        authority_commit_id: 4,
        lineage: ReceiptLineageContext::ProvisioningGeneration(claims.device_generation),
        key_identifier: vec![0x01],
        lifecycle_state: map_lifecycle(claims.lifecycle_state),
        device_generation: claims.device_generation,
        transition_counter: claims.transition_counter,
        measurement_epoch: claims.measurement_epoch,
        receipt_sequence: Some(claims.receipt_sequence),
        active_version: claims.active_version,
        challenge: claims.challenge.map(Vec::from),
        measurement_root: vec![0xaa],
        measurement_context: String::from("fixture.model-conformance"),
        policy_identifier: String::from("fixture.policy"),
        policy_version: 1,
        input_commitment: None,
        output_commitment: None,
    };
    let SemanticObject::AuthorityMetadata(authority) = fixture("authority-committed").object else {
        panic!("expected committed authority fixture");
    };
    let SemanticObject::PersistentState(state) =
        fixture("persistent-operational-receipt-release").object
    else {
        panic!("expected release persistent fixture");
    };

    assert_eq!(receipt.validate_release(&authority, &state), Ok(()));
}

#[test]
fn persistence_phases_project_to_valid_authority_metadata() {
    let (machine, _) = receipt_release_machine();
    let command = Command::StageUpdate {
        authorizations: update_owner(),
        candidate_version: 5,
    };
    let mut candidate = machine.clone();
    let execution = candidate
        .apply(command)
        .expect("candidate update must stage in lifecycle model");

    let mut durable = DurableModel::new(machine);
    assert!(matches!(durable.phase(), PersistencePhase::Clean));
    assert_eq!(durable.active_commit_id(), Ok(0));

    assert!(matches!(
        durable.prepare(command),
        Ok(PrepareResult::Staged { .. })
    ));
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase() else {
        panic!("expected prepared persistence phase");
    };
    let prepared = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: 0,
        record_commit_ids: [Some(0), Some(commit_id)],
        phase: AuthorityPhaseProjection::Prepared {
            candidate_slot: u8::try_from(slot).expect("two-slot index must fit u8"),
            commit_id,
            prepared_outcome: PreparedOutcomeProjection::Applied(map_execution(execution)),
        },
    };
    assert_eq!(prepared.validate(), Ok(()));

    durable.commit().expect("prepared record must commit");
    let PersistencePhase::Committed {
        previous_slot,
        active_slot,
        commit_id,
    } = durable.phase()
    else {
        panic!("expected committed persistence phase");
    };
    let committed = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: u8::try_from(active_slot).expect("two-slot index must fit u8"),
        record_commit_ids: [Some(0), Some(commit_id)],
        phase: AuthorityPhaseProjection::Committed {
            previous_slot: u8::try_from(previous_slot).expect("two-slot index must fit u8"),
            selected_next_slot: u8::try_from(active_slot).expect("two-slot index must fit u8"),
            commit_id,
        },
    };
    assert_eq!(committed.validate(), Ok(()));

    durable
        .cleanup()
        .expect("committed transaction must clean up");
    assert!(matches!(durable.phase(), PersistencePhase::Clean));
    assert_eq!(durable.active_commit_id(), Ok(commit_id));
    let clean = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: u8::try_from(active_slot).expect("two-slot index must fit u8"),
        record_commit_ids: [None, Some(commit_id)],
        phase: AuthorityPhaseProjection::Clean,
    };
    assert_eq!(clean.validate(), Ok(()));
}

#[test]
fn recovered_previous_and_next_project_to_clean_authority_metadata() {
    let (machine, _) = receipt_release_machine();
    let command = Command::StageUpdate {
        authorizations: update_owner(),
        candidate_version: 5,
    };
    let mut durable = DurableModel::new(machine);

    durable.prepare(command).expect("candidate must prepare");
    let PersistencePhase::Prepared { slot, .. } = durable.phase() else {
        panic!("expected prepared phase");
    };
    let previous_slot = 1 - slot;
    let recovered_previous = durable
        .crash_and_recover()
        .expect("prepared crash must recover previous");
    assert_eq!(
        recovered_previous.operation,
        PersistenceOperation::RecoveredPrevious
    );
    assert!(matches!(durable.phase(), PersistencePhase::Clean));
    let mut record_commit_ids = [None; 2];
    record_commit_ids[previous_slot] = Some(0);
    let previous_authority = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: u8::try_from(previous_slot).expect("two-slot index must fit u8"),
        record_commit_ids,
        phase: AuthorityPhaseProjection::Clean,
    };
    assert_eq!(previous_authority.validate(), Ok(()));
    assert_eq!(durable.active_commit_id(), Ok(0));
    assert_eq!(
        durable.active_state().map(StateMachine::lifecycle),
        Ok(ModelLifecycleState::Operational)
    );

    durable
        .prepare(command)
        .expect("candidate must prepare again");
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase() else {
        panic!("expected prepared phase");
    };
    durable.commit().expect("candidate must commit");
    let recovered_next = durable
        .crash_and_recover()
        .expect("committed crash must recover next");
    assert_eq!(
        recovered_next.operation,
        PersistenceOperation::RecoveredNext
    );
    assert!(matches!(durable.phase(), PersistencePhase::Clean));
    let mut record_commit_ids = [None; 2];
    record_commit_ids[slot] = Some(commit_id);
    let next_authority = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: u8::try_from(slot).expect("two-slot index must fit u8"),
        record_commit_ids,
        phase: AuthorityPhaseProjection::Clean,
    };
    assert_eq!(next_authority.validate(), Ok(()));
    assert_eq!(durable.active_commit_id(), Ok(commit_id));
    assert_eq!(
        durable.active_state().map(StateMachine::lifecycle),
        Ok(ModelLifecycleState::UpdatePending)
    );
}

#[test]
fn model_rejected_fault_prepare_projects_to_withheld_authority_outcome() {
    let mut machine = provisioned_machine();
    machine.inject_test_fault(ModelTestFault::ReceiptSequenceExhausted);
    let mut durable = DurableModel::new(machine);

    durable
        .prepare(Command::IssueReceipt { challenge: None })
        .expect("fault-producing rejection must stage");
    let PersistencePhase::Prepared { slot, commit_id } = durable.phase() else {
        panic!("expected prepared phase");
    };
    let previous_slot = 1 - slot;
    let mut record_commit_ids = [None; 2];
    record_commit_ids[previous_slot] = Some(0);
    record_commit_ids[slot] = Some(commit_id);
    let prepared = AuthorityMetadataProjection {
        context: context(ObjectClass::AuthorityMetadata, 1, None),
        raw_selected_slot: u8::try_from(previous_slot).expect("two-slot index must fit u8"),
        record_commit_ids,
        phase: AuthorityPhaseProjection::Prepared {
            candidate_slot: u8::try_from(slot).expect("two-slot index must fit u8"),
            commit_id,
            prepared_outcome: PreparedOutcomeProjection::Rejected(map_rejection(
                ModelRejection::CounterExhausted,
            )),
        },
    };
    assert_eq!(prepared.validate(), Ok(()));
    assert_eq!(
        durable.active_state().map(StateMachine::lifecycle),
        Ok(ModelLifecycleState::Operational)
    );

    let committed = durable.commit().expect("fault state must commit");
    assert_eq!(
        committed.outcome,
        CommandOutcome::Rejected(ModelRejection::CounterExhausted)
    );
    assert_eq!(
        durable.active_state().map(StateMachine::lifecycle),
        Ok(ModelLifecycleState::Fault)
    );
}
