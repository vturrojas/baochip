use super::*;

fn provisioning_start_auth() -> Authorizations {
    Authorizations {
        root: true,
        physical_presence: true,
        ..Authorizations::none()
    }
}

fn provisioning_commit_auth() -> Authorizations {
    Authorizations {
        root: true,
        owner: true,
        ..Authorizations::none()
    }
}

fn operational_machine() -> StateMachine {
    let mut machine = StateMachine::new();
    machine
        .apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        })
        .expect("authorized provisioning should begin");
    machine
        .apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        })
        .expect("authorized provisioning should commit");
    machine
}

#[test]
fn initial_provisioning_creates_first_generation() {
    let machine = operational_machine();
    assert_eq!(machine.lifecycle(), LifecycleState::Operational);
    assert_eq!(machine.device_generation(), 1);
    assert_eq!(machine.transition_counter(), 1);
    assert_eq!(machine.active_version(), 1);
}

#[test]
fn accepted_staging_transitions_advance_rollback_state() {
    let mut machine = StateMachine::new();
    machine
        .apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        })
        .expect("authorized provisioning should begin");
    assert_eq!(machine.transition_counter(), 1);

    machine
        .apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        })
        .expect("authorized provisioning should commit");
    machine
        .apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        })
        .expect("revocation should succeed");
    let counter_before_recommission = machine.transition_counter();

    machine
        .apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        })
        .expect("authorized recommission should begin");
    assert_eq!(
        machine.transition_counter(),
        counter_before_recommission + 1
    );
}

#[test]
fn decommission_advances_transition_counter_when_available() {
    let mut machine = operational_machine();
    let counter_before_decommission = machine.transition_counter();

    machine
        .apply(Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("authorized decommission should succeed");

    assert_eq!(
        machine.transition_counter(),
        counter_before_decommission + 1
    );
}

#[test]
fn provisioning_requires_root_and_physical_presence() {
    let mut machine = StateMachine::new();
    let result = machine.apply(Command::BeginProvisioning {
        authorizations: Authorizations {
            root: true,
            ..Authorizations::none()
        },
    });
    assert_eq!(result, Err(Rejection::Unauthorized));
    assert_eq!(machine.lifecycle(), LifecycleState::Blank);
}

#[test]
fn abort_provisioning_discards_staged_generation_and_records_transition() {
    let mut machine = StateMachine::new();
    machine
        .apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        })
        .expect("provisioning should begin");
    let counter_before_abort = machine.transition_counter();

    machine
        .apply(Command::AbortProvisioning)
        .expect("provisioning abort should return to blank");

    assert_eq!(machine.lifecycle(), LifecycleState::Blank);
    assert_eq!(machine.provisioning_generation, None);
    assert_eq!(machine.transition_counter(), counter_before_abort + 1);
    assert!(!machine.identity_active);
}

#[test]
fn update_rejects_rollback_and_accepts_newer_version() {
    let mut machine = operational_machine();
    let auth = Authorizations {
        owner: true,
        update: true,
        ..Authorizations::none()
    };

    assert_eq!(
        machine.apply(Command::StageUpdate {
            authorizations: auth,
            candidate_version: 1,
        }),
        Err(Rejection::RollbackDetected)
    );

    machine
        .apply(Command::StageUpdate {
            authorizations: auth,
            candidate_version: 2,
        })
        .expect("newer authorized update should stage");
    machine
        .apply(Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        })
        .expect("staged update should commit");
    assert_eq!(machine.active_version(), 2);
    assert_eq!(machine.lifecycle(), LifecycleState::Operational);
}

#[test]
fn update_rejection_requires_validation_failure_or_authorized_cancellation() {
    let update_auth = Authorizations {
        owner: true,
        update: true,
        ..Authorizations::none()
    };
    let mut unauthorized = operational_machine();
    unauthorized
        .apply(Command::StageUpdate {
            authorizations: update_auth,
            candidate_version: 2,
        })
        .expect("update should stage");
    let before = unauthorized.clone();
    assert_eq!(
        unauthorized.apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations::none()),
        }),
        Err(Rejection::Unauthorized)
    );
    assert_eq!(unauthorized, before);

    let mut failed_validation = before.clone();
    failed_validation
        .apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::ValidationFailure(UpdateValidation {
                candidate_authenticated: false,
                compatible: true,
                integrity_valid: true,
            }),
        })
        .expect("a modeled validation failure should reject the candidate");
    assert_eq!(failed_validation.lifecycle(), LifecycleState::Operational);
    assert_eq!(failed_validation.active_version(), 1);

    let mut cancelled = before;
    cancelled
        .apply(Command::RejectUpdate {
            cause: UpdateRejectionCause::AuthorizedCancellation(update_auth),
        })
        .expect("update and owner authorities may cancel a candidate");
    assert_eq!(cancelled.lifecycle(), LifecycleState::Operational);
    assert_eq!(cancelled.active_version(), 1);
}

#[test]
fn recovery_requires_recovery_authority_and_second_condition() {
    let mut machine = operational_machine();
    assert_eq!(
        machine.apply(Command::EnterRecovery {
            authorizations: Authorizations {
                recovery: true,
                ..Authorizations::none()
            },
        }),
        Err(Rejection::Unauthorized)
    );

    machine
        .apply(Command::EnterRecovery {
            authorizations: Authorizations {
                recovery: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("independently authorized recovery should begin");
    assert_eq!(machine.lifecycle(), LifecycleState::Recovery);
}

#[test]
fn revoked_identity_cannot_issue_and_recommission_advances_generation() {
    let mut machine = operational_machine();
    machine
        .apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        })
        .expect("revocation authority should revoke");

    assert_eq!(
        machine.apply(Command::IssueReceipt { challenge: None }),
        Err(Rejection::InvalidState)
    );
    assert_eq!(
        machine.apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        }),
        Err(Rejection::InvalidState)
    );

    machine
        .apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        })
        .expect("authorized recommission should begin");
    machine
        .apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        })
        .expect("reprovisioning should commit a new identity generation");

    assert_eq!(machine.device_generation(), 2);
    assert_eq!(machine.lifecycle(), LifecycleState::Operational);
    assert_eq!(machine.receipt_sequence(), 0);
}

#[test]
fn revocation_is_available_from_blank_and_clears_all_staged_state() {
    let revocation_auth = Authorizations {
        revocation: true,
        ..Authorizations::none()
    };
    let mut blank = StateMachine::new();
    blank
        .apply(Command::Revoke {
            authorizations: revocation_auth,
        })
        .expect("blank is nonterminal and may be revoked");
    assert_eq!(blank.lifecycle(), LifecycleState::Revoked);

    let mut provisioning = StateMachine::new();
    provisioning
        .apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        })
        .expect("provisioning should begin");
    assert!(provisioning.provisioning_generation.is_some());
    provisioning
        .apply(Command::Revoke {
            authorizations: revocation_auth,
        })
        .expect("provisioning identity should be revocable");
    assert_eq!(provisioning.lifecycle(), LifecycleState::Revoked);
    assert_eq!(provisioning.provisioning_generation, None);
    assert_eq!(provisioning.pending_version, None);
    assert!(!provisioning.identity_active);
}

#[test]
fn receipt_sequence_advances_and_binds_state() {
    let mut machine = operational_machine();
    machine
        .apply(Command::StartMeasurementEpoch)
        .expect("operational device should start an epoch");
    let challenge = [0xA5; 16];
    let execution = machine
        .apply(Command::IssueReceipt {
            challenge: Some(challenge),
        })
        .expect("operational device should issue a receipt");
    let receipt = execution.receipt.expect("receipt command returns claims");

    assert_eq!(receipt.device_generation, 1);
    assert_eq!(receipt.measurement_epoch, 1);
    assert_eq!(receipt.receipt_sequence, 1);
    assert_eq!(receipt.challenge, Some(challenge));
}

#[test]
fn decommission_is_terminal() {
    let mut machine = operational_machine();
    machine
        .apply(Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        })
        .expect("authorized decommission should succeed");

    assert_eq!(machine.lifecycle(), LifecycleState::Decommissioned);
    assert_eq!(
        machine.apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        }),
        Err(Rejection::Decommissioned)
    );
}

#[test]
fn undocumented_state_command_pairs_fail_closed() {
    let mut machine = StateMachine::new();
    assert_eq!(
        machine.apply(Command::IssueReceipt { challenge: None }),
        Err(Rejection::InvalidState)
    );
    assert_eq!(
        machine.apply(Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        }),
        Err(Rejection::InvalidState)
    );
    assert_eq!(
        machine.apply(Command::StartMeasurementEpoch),
        Err(Rejection::InvalidState)
    );
}

#[test]
fn receipt_counter_exhaustion_enters_fault_without_wrapping() {
    let mut machine = operational_machine();
    machine.receipt_sequence = u64::MAX;
    assert_eq!(
        machine.apply(Command::IssueReceipt { challenge: None }),
        Err(Rejection::CounterExhausted)
    );
    assert_eq!(machine.receipt_sequence(), u64::MAX);
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn measurement_epoch_exhaustion_enters_fault_without_wrapping() {
    let mut machine = operational_machine();
    machine.measurement_epoch = u64::MAX;

    assert_eq!(
        machine.apply(Command::StartMeasurementEpoch),
        Err(Rejection::CounterExhausted)
    );
    assert_eq!(machine.measurement_epoch(), u64::MAX);
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);
}

#[test]
fn transition_counter_exhaustion_enters_fault_but_allows_decommission() {
    let mut machine = operational_machine();
    machine.transition_counter = u64::MAX;

    assert_eq!(
        machine.apply(Command::EnterRecovery {
            authorizations: Authorizations {
                recovery: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        }),
        Err(Rejection::CounterExhausted)
    );
    assert_eq!(machine.transition_counter(), u64::MAX);
    assert_eq!(machine.lifecycle(), LifecycleState::Fault);

    machine
        .apply(Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        })
        .expect("decommission must remain available after counter exhaustion");
    assert_eq!(machine.lifecycle(), LifecycleState::Decommissioned);
    assert_eq!(machine.transition_counter(), u64::MAX);
}

#[test]
fn device_generation_exhaustion_blocks_recommission_but_allows_decommission() {
    let mut machine = operational_machine();
    machine
        .apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        })
        .expect("revocation should succeed");
    machine.device_generation = u64::MAX;
    let before = machine.clone();

    assert_eq!(
        machine.apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                independent: true,
                ..Authorizations::none()
            },
        }),
        Err(Rejection::CounterExhausted)
    );
    assert_eq!(machine, before);

    machine
        .apply(Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("decommission is the remaining terminal path");
    assert_eq!(machine.lifecycle(), LifecycleState::Decommissioned);
}
