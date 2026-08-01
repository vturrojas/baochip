use super::adversarial_tests::{machine_in, operational_machine, provisioning_commit_auth};
use super::*;

#[test]
fn receipt_integrity_failure_enters_fault() {
    let mut machine = operational_machine();
    machine.identity_active = false;

    assert_eq!(
        machine.apply(Command::IssueReceipt { challenge: None }),
        Err(Rejection::IntegrityFailure)
    );
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn missing_provisioning_stage_enters_fault() {
    let mut machine = StateMachine::new();
    machine.lifecycle = LifecycleState::Provisioning;
    machine.provisioning_generation = None;

    assert_eq!(
        machine.apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        }),
        Err(Rejection::InternalInvariantViolation)
    );
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn missing_update_stage_enters_fault() {
    let mut machine = operational_machine();
    machine.lifecycle = LifecycleState::UpdatePending;
    machine.pending_version = None;

    assert_eq!(
        machine.apply(Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        }),
        Err(Rejection::InternalInvariantViolation)
    );
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn update_rejection_with_missing_stage_enters_fault() {
    let mut machine = machine_in(LifecycleState::UpdatePending);
    machine.pending_version = None;

    assert_eq!(
        machine.apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations {
                update: true,
                owner: true,
                ..Authorizations::none()
            }),
        }),
        Err(Rejection::InternalInvariantViolation)
    );
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn provisioning_commit_rejects_nonadvancing_staged_generation() {
    let mut initial = machine_in(LifecycleState::Provisioning);
    initial.provisioning_generation = Some(initial.device_generation());
    assert_eq!(
        initial.apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        }),
        Err(Rejection::InternalInvariantViolation)
    );
    assert_eq!(initial.lifecycle(), LifecycleState::Fault);

    for staged_generation in [0, 1] {
        let mut recommission = machine_in(LifecycleState::Revoked);
        recommission
            .apply(Command::BeginRecommission {
                authorizations: Authorizations {
                    root: true,
                    owner: true,
                    independent: true,
                    ..Authorizations::none()
                },
            })
            .expect("recommission should stage generation two");
        recommission.provisioning_generation = Some(staged_generation);

        assert_eq!(
            recommission.apply(Command::CommitProvisioning {
                authorizations: provisioning_commit_auth(),
            }),
            Err(Rejection::InternalInvariantViolation)
        );
        assert_eq!(recommission.lifecycle(), LifecycleState::Fault);
    }
}

#[test]
fn provisioning_abort_erases_all_staged_fields() {
    let mut machine = machine_in(LifecycleState::Provisioning);
    machine.pending_version = Some(99);

    machine
        .apply(Command::AbortProvisioning)
        .expect("valid provisioning abort should succeed");

    assert_eq!(machine.lifecycle(), LifecycleState::Blank);
    assert_eq!(machine.pending_version, None);
    assert_eq!(machine.provisioning_generation, None);
    assert_eq!(machine.provisioning_origin, None);
}
