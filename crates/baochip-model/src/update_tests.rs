use super::adversarial_tests::machine_in;
use super::*;

#[test]
fn update_acceptance_requires_complete_abstract_validation() {
    let failed_validation = UpdateValidation {
        candidate_authenticated: true,
        compatible: false,
        integrity_valid: true,
    };
    let mut rejected_acceptance = machine_in(LifecycleState::UpdatePending);
    let before = rejected_acceptance.clone();
    assert_eq!(
        rejected_acceptance.apply(Command::AcceptUpdate {
            validation: failed_validation,
        }),
        Err(Rejection::InvalidTransition)
    );
    assert_eq!(rejected_acceptance, before);

    let mut accepted = machine_in(LifecycleState::UpdatePending);
    accepted
        .apply(Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        })
        .expect("fully validated candidate should activate");
    assert_eq!(accepted.active_version(), 2);
    assert_eq!(accepted.lifecycle(), LifecycleState::Operational);
}

#[test]
fn update_rejection_distinguishes_validation_failure_from_cancellation() {
    let failed_validation = UpdateValidation {
        candidate_authenticated: false,
        compatible: true,
        integrity_valid: true,
    };
    let mut validation_rejection = machine_in(LifecycleState::UpdatePending);
    validation_rejection
        .apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::ValidationFailure(failed_validation),
        })
        .expect("trusted abstract validation failure should reject the candidate");
    assert_eq!(
        validation_rejection.lifecycle(),
        LifecycleState::Operational
    );

    let mut unauthorized_cancel = machine_in(LifecycleState::UpdatePending);
    let before = unauthorized_cancel.clone();
    assert_eq!(
        unauthorized_cancel.apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations::none()),
        }),
        Err(Rejection::Unauthorized)
    );
    assert_eq!(unauthorized_cancel, before);
}
