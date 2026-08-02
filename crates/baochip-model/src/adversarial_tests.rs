use super::*;

pub(super) fn provisioning_start_auth() -> Authorizations {
    Authorizations {
        root: true,
        physical_presence: true,
        ..Authorizations::none()
    }
}

pub(super) fn provisioning_commit_auth() -> Authorizations {
    Authorizations {
        root: true,
        owner: true,
        ..Authorizations::none()
    }
}

pub(super) fn operational_machine() -> StateMachine {
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
fn aborted_recommission_returns_to_revoked() {
    let mut machine = operational_machine();
    machine
        .apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        })
        .expect("operational identity should revoke");
    machine
        .apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("recommission should begin");

    machine
        .apply(Command::AbortProvisioning)
        .expect("aborted recommission should terminate its staged provisioning");

    assert_eq!(machine.lifecycle(), LifecycleState::Revoked);
    assert_eq!(machine.provisioning_generation, None);
    assert_eq!(
        machine.apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        }),
        Err(Rejection::InvalidState)
    );
}

#[test]
fn blank_rejects_revocation_without_blocking_authorized_decommission() {
    let mut machine = StateMachine::new();
    let before = machine.clone();

    assert_eq!(
        machine.apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        }),
        Err(Rejection::InvalidState)
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
        .expect("authorized blank device retirement should remain available");
    assert_eq!(machine.lifecycle(), LifecycleState::Decommissioned);
}

#[test]
fn provisioning_audit_binds_the_staged_generation() {
    let mut initial = StateMachine::new();
    let initial_execution = initial
        .apply(Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        })
        .expect("initial provisioning should begin");
    assert_eq!(initial_execution.audit.staged_device_generation, Some(1));

    let mut recommission = machine_in(LifecycleState::Revoked);
    let next_generation = recommission.device_generation() + 1;
    let recommission_execution = recommission
        .apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("recommission should begin");
    assert_eq!(
        recommission_execution.audit.staged_device_generation,
        Some(next_generation)
    );
}

#[derive(Clone, Copy, Debug)]
enum CommandKind {
    BeginProvisioning,
    CommitProvisioning,
    AbortProvisioning,
    StageUpdate,
    AcceptUpdate,
    RejectUpdate,
    EnterRecovery,
    CompleteRecovery,
    Revoke,
    BeginRecommission,
    Decommission,
    StartMeasurementEpoch,
    IssueReceipt,
}

const COMMAND_KINDS: [CommandKind; 13] = [
    CommandKind::BeginProvisioning,
    CommandKind::CommitProvisioning,
    CommandKind::AbortProvisioning,
    CommandKind::StageUpdate,
    CommandKind::AcceptUpdate,
    CommandKind::RejectUpdate,
    CommandKind::EnterRecovery,
    CommandKind::CompleteRecovery,
    CommandKind::Revoke,
    CommandKind::BeginRecommission,
    CommandKind::Decommission,
    CommandKind::StartMeasurementEpoch,
    CommandKind::IssueReceipt,
];

pub(super) fn machine_in(state: LifecycleState) -> StateMachine {
    match state {
        LifecycleState::Blank => StateMachine::new(),
        LifecycleState::Provisioning => {
            let mut machine = StateMachine::new();
            machine
                .apply(Command::BeginProvisioning {
                    authorizations: provisioning_start_auth(),
                })
                .expect("provisioning setup should succeed");
            machine
        }
        LifecycleState::Operational => operational_machine(),
        LifecycleState::UpdatePending => {
            let mut machine = operational_machine();
            machine
                .apply(Command::StageUpdate {
                    authorizations: Authorizations {
                        update: true,
                        owner: true,
                        ..Authorizations::none()
                    },
                    candidate_version: 2,
                })
                .expect("update setup should succeed");
            machine
        }
        LifecycleState::Recovery => {
            let mut machine = operational_machine();
            machine
                .apply(Command::EnterRecovery {
                    authorizations: Authorizations {
                        recovery: true,
                        independent: true,
                        ..Authorizations::none()
                    },
                })
                .expect("recovery setup should succeed");
            machine
        }
        LifecycleState::Revoked => {
            let mut machine = operational_machine();
            machine
                .apply(Command::Revoke {
                    authorizations: Authorizations {
                        revocation: true,
                        ..Authorizations::none()
                    },
                })
                .expect("revocation setup should succeed");
            machine
        }
        LifecycleState::Decommissioned => {
            let mut machine = operational_machine();
            machine
                .apply(Command::Decommission {
                    authorizations: Authorizations {
                        decommission: true,
                        independent: true,
                        ..Authorizations::none()
                    },
                })
                .expect("decommission setup should succeed");
            machine
        }
        LifecycleState::Fault => {
            let mut machine = operational_machine();
            machine.enter_fault();
            machine
        }
    }
}

fn authorized_command(kind: CommandKind, machine: &StateMachine) -> Command {
    match kind {
        CommandKind::BeginProvisioning => Command::BeginProvisioning {
            authorizations: provisioning_start_auth(),
        },
        CommandKind::CommitProvisioning => Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        },
        CommandKind::AbortProvisioning => Command::AbortProvisioning,
        CommandKind::StageUpdate => Command::StageUpdate {
            authorizations: Authorizations {
                update: true,
                owner: true,
                ..Authorizations::none()
            },
            candidate_version: machine.active_version.saturating_add(1),
        },
        CommandKind::AcceptUpdate => Command::AcceptUpdate {
            validation: UpdateValidation::passed(),
        },
        CommandKind::RejectUpdate => Command::RejectUpdate {
            cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations {
                update: true,
                owner: true,
                ..Authorizations::none()
            }),
        },
        CommandKind::EnterRecovery => Command::EnterRecovery {
            authorizations: Authorizations {
                recovery: true,
                independent: true,
                ..Authorizations::none()
            },
        },
        CommandKind::CompleteRecovery => Command::CompleteRecovery {
            authorizations: Authorizations {
                recovery: true,
                owner: true,
                ..Authorizations::none()
            },
            recovered_version: machine.active_version,
        },
        CommandKind::Revoke => Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        },
        CommandKind::BeginRecommission => Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                independent: true,
                ..Authorizations::none()
            },
        },
        CommandKind::Decommission => Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                independent: true,
                ..Authorizations::none()
            },
        },
        CommandKind::StartMeasurementEpoch => Command::StartMeasurementEpoch,
        CommandKind::IssueReceipt => Command::IssueReceipt { challenge: None },
    }
}

fn expected_state(state: LifecycleState, kind: CommandKind) -> Option<LifecycleState> {
    match (state, kind) {
        (LifecycleState::Blank, CommandKind::BeginProvisioning) => {
            Some(LifecycleState::Provisioning)
        }
        (LifecycleState::Provisioning, CommandKind::CommitProvisioning) => {
            Some(LifecycleState::Operational)
        }
        (LifecycleState::Provisioning, CommandKind::AbortProvisioning) => {
            Some(LifecycleState::Blank)
        }
        (LifecycleState::Provisioning, CommandKind::Revoke)
        | (LifecycleState::Operational, CommandKind::Revoke)
        | (LifecycleState::UpdatePending, CommandKind::Revoke)
        | (LifecycleState::Recovery, CommandKind::Revoke)
        | (LifecycleState::Fault, CommandKind::Revoke) => Some(LifecycleState::Revoked),
        (LifecycleState::Operational, CommandKind::StageUpdate) => {
            Some(LifecycleState::UpdatePending)
        }
        (LifecycleState::Operational, CommandKind::EnterRecovery) => Some(LifecycleState::Recovery),
        (LifecycleState::Operational, CommandKind::StartMeasurementEpoch)
        | (LifecycleState::Operational, CommandKind::IssueReceipt) => {
            Some(LifecycleState::Operational)
        }
        (LifecycleState::UpdatePending, CommandKind::AcceptUpdate)
        | (LifecycleState::UpdatePending, CommandKind::RejectUpdate)
        | (LifecycleState::Recovery, CommandKind::CompleteRecovery) => {
            Some(LifecycleState::Operational)
        }
        (LifecycleState::Revoked, CommandKind::BeginRecommission) => {
            Some(LifecycleState::Provisioning)
        }
        (state, CommandKind::Decommission) if state != LifecycleState::Decommissioned => {
            Some(LifecycleState::Decommissioned)
        }
        _ => None,
    }
}

#[test]
fn every_state_command_pair_has_an_explicit_outcome() {
    let states = [
        LifecycleState::Blank,
        LifecycleState::Provisioning,
        LifecycleState::Operational,
        LifecycleState::UpdatePending,
        LifecycleState::Recovery,
        LifecycleState::Revoked,
        LifecycleState::Decommissioned,
        LifecycleState::Fault,
    ];

    for state in states {
        for kind in COMMAND_KINDS {
            let mut machine = machine_in(state);
            let before = machine.clone();
            let command = authorized_command(kind, &machine);
            let outcome = machine.apply(command);

            if let Some(resulting_state) = expected_state(state, kind) {
                let execution = outcome.unwrap_or_else(|rejection| {
                    panic!("{state:?} + {kind:?} unexpectedly rejected: {rejection:?}")
                });
                assert_eq!(machine.lifecycle(), resulting_state, "{state:?} + {kind:?}");
                assert_eq!(
                    execution.audit.previous_state, state,
                    "{state:?} + {kind:?}"
                );
                assert_eq!(
                    execution.audit.resulting_state, resulting_state,
                    "{state:?} + {kind:?}"
                );
                assert_eq!(execution.audit.device_generation, machine.device_generation);
                assert_eq!(
                    execution.audit.transition_counter,
                    machine.transition_counter
                );
            } else {
                let expected_rejection = if state == LifecycleState::Decommissioned {
                    Rejection::Decommissioned
                } else {
                    Rejection::InvalidState
                };
                assert_eq!(outcome, Err(expected_rejection), "{state:?} + {kind:?}");
                assert_eq!(
                    machine, before,
                    "rejected {state:?} + {kind:?} mutated state"
                );
            }
        }
    }
}

#[test]
fn missing_authority_never_mutates_complete_state() {
    let cases = vec![
        (
            "begin provisioning without root",
            StateMachine::new(),
            Command::BeginProvisioning {
                authorizations: Authorizations {
                    physical_presence: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "begin provisioning without presence",
            StateMachine::new(),
            Command::BeginProvisioning {
                authorizations: Authorizations {
                    root: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "commit provisioning without owner",
            machine_in(LifecycleState::Provisioning),
            Command::CommitProvisioning {
                authorizations: Authorizations {
                    root: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "commit provisioning without root",
            machine_in(LifecycleState::Provisioning),
            Command::CommitProvisioning {
                authorizations: Authorizations {
                    owner: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "stage update without update authority",
            operational_machine(),
            Command::StageUpdate {
                authorizations: Authorizations {
                    owner: true,
                    ..Authorizations::none()
                },
                candidate_version: 2,
            },
        ),
        (
            "stage update without owner policy",
            operational_machine(),
            Command::StageUpdate {
                authorizations: Authorizations {
                    update: true,
                    ..Authorizations::none()
                },
                candidate_version: 2,
            },
        ),
        (
            "cancel update without owner",
            machine_in(LifecycleState::UpdatePending),
            Command::RejectUpdate {
                cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations {
                    update: true,
                    ..Authorizations::none()
                }),
            },
        ),
        (
            "cancel update without update authority",
            machine_in(LifecycleState::UpdatePending),
            Command::RejectUpdate {
                cause: UpdateRejectionCause::AuthorizedCancellation(Authorizations {
                    owner: true,
                    ..Authorizations::none()
                }),
            },
        ),
        (
            "enter recovery without second condition",
            operational_machine(),
            Command::EnterRecovery {
                authorizations: Authorizations {
                    recovery: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "enter recovery without recovery authority",
            operational_machine(),
            Command::EnterRecovery {
                authorizations: Authorizations {
                    independent: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "complete recovery without owner",
            machine_in(LifecycleState::Recovery),
            Command::CompleteRecovery {
                authorizations: Authorizations {
                    recovery: true,
                    ..Authorizations::none()
                },
                recovered_version: 1,
            },
        ),
        (
            "complete recovery without recovery authority",
            machine_in(LifecycleState::Recovery),
            Command::CompleteRecovery {
                authorizations: Authorizations {
                    owner: true,
                    ..Authorizations::none()
                },
                recovered_version: 1,
            },
        ),
        (
            "revoke without revocation authority",
            operational_machine(),
            Command::Revoke {
                authorizations: Authorizations::none(),
            },
        ),
        (
            "recommission without root",
            machine_in(LifecycleState::Revoked),
            Command::BeginRecommission {
                authorizations: Authorizations {
                    owner: true,
                    independent: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "recommission without second condition",
            machine_in(LifecycleState::Revoked),
            Command::BeginRecommission {
                authorizations: Authorizations {
                    root: true,
                    owner: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "recommission without owner",
            machine_in(LifecycleState::Revoked),
            Command::BeginRecommission {
                authorizations: Authorizations {
                    root: true,
                    independent: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "decommission without decommission authority",
            operational_machine(),
            Command::Decommission {
                authorizations: Authorizations {
                    independent: true,
                    ..Authorizations::none()
                },
            },
        ),
        (
            "decommission without second condition",
            operational_machine(),
            Command::Decommission {
                authorizations: Authorizations {
                    decommission: true,
                    ..Authorizations::none()
                },
            },
        ),
    ];

    for (name, mut machine, command) in cases {
        let before = machine.clone();
        assert_eq!(
            machine.apply(command),
            Err(Rejection::Unauthorized),
            "{name}"
        );
        assert_eq!(machine, before, "{name} mutated state");
    }
}

#[test]
fn recovery_rejects_lower_and_records_equal_and_higher_versions() {
    let recovery_auth = Authorizations {
        recovery: true,
        owner: true,
        ..Authorizations::none()
    };

    let mut lower = machine_in(LifecycleState::Recovery);
    let before_lower = lower.clone();
    assert_eq!(
        lower.apply(Command::CompleteRecovery {
            authorizations: recovery_auth,
            recovered_version: 0,
        }),
        Err(Rejection::RollbackDetected)
    );
    assert_eq!(lower, before_lower);

    for recovered_version in [1, 2] {
        let mut machine = machine_in(LifecycleState::Recovery);
        let counter_before = machine.transition_counter;
        let execution = machine
            .apply(Command::CompleteRecovery {
                authorizations: recovery_auth,
                recovered_version,
            })
            .expect("equal and higher authorized recovery versions should commit");
        assert_eq!(machine.lifecycle(), LifecycleState::Operational);
        assert_eq!(machine.active_version(), recovered_version);
        assert_eq!(machine.transition_counter(), counter_before + 1);
        assert_eq!(execution.audit.previous_state, LifecycleState::Recovery);
    }
}

#[test]
fn receipt_sequence_is_unique_within_generation_and_scoped_across_recommission() {
    let mut machine = operational_machine();
    let first = machine
        .apply(Command::IssueReceipt { challenge: None })
        .expect("first receipt should issue")
        .receipt
        .expect("receipt claims should be present");
    let second = machine
        .apply(Command::IssueReceipt { challenge: None })
        .expect("second receipt should issue")
        .receipt
        .expect("receipt claims should be present");
    assert_ne!(first.receipt_sequence, second.receipt_sequence);
    assert_eq!(first.device_generation, second.device_generation);

    machine
        .apply(Command::Revoke {
            authorizations: Authorizations {
                revocation: true,
                ..Authorizations::none()
            },
        })
        .expect("identity should revoke");
    machine
        .apply(Command::BeginRecommission {
            authorizations: Authorizations {
                root: true,
                owner: true,
                independent: true,
                ..Authorizations::none()
            },
        })
        .expect("recommission should begin");
    machine
        .apply(Command::CommitProvisioning {
            authorizations: provisioning_commit_auth(),
        })
        .expect("new generation should commit");
    let next_generation = machine
        .apply(Command::IssueReceipt { challenge: None })
        .expect("new identity should issue")
        .receipt
        .expect("receipt claims should be present");

    assert_eq!(next_generation.receipt_sequence, 1);
    assert_ne!(first.device_generation, next_generation.device_generation);
    assert_ne!(
        (first.device_generation, first.receipt_sequence),
        (
            next_generation.device_generation,
            next_generation.receipt_sequence
        )
    );
}

#[test]
fn every_transition_counter_exhaustion_path_enters_fault_without_wrap() {
    let cases = [
        (LifecycleState::Blank, CommandKind::BeginProvisioning),
        (LifecycleState::Provisioning, CommandKind::AbortProvisioning),
        (LifecycleState::Operational, CommandKind::StageUpdate),
        (LifecycleState::Operational, CommandKind::EnterRecovery),
        (LifecycleState::Operational, CommandKind::Revoke),
        (LifecycleState::UpdatePending, CommandKind::AcceptUpdate),
        (LifecycleState::UpdatePending, CommandKind::RejectUpdate),
        (LifecycleState::Recovery, CommandKind::CompleteRecovery),
        (LifecycleState::Revoked, CommandKind::BeginRecommission),
    ];

    for (state, kind) in cases {
        let mut machine = machine_in(state);
        machine.transition_counter = u64::MAX;
        let command = authorized_command(kind, &machine);

        assert_eq!(
            machine.apply(command),
            Err(Rejection::CounterExhausted),
            "{state:?} + {kind:?}"
        );
        assert_eq!(machine.transition_counter(), u64::MAX);
        assert_eq!(machine.lifecycle(), LifecycleState::Fault);
        assert_eq!(machine.pending_version, None);
        assert_eq!(machine.provisioning_generation, None);
        assert_eq!(machine.provisioning_origin, None);
        assert!(!machine.identity_active);
    }
}

#[test]
fn revocation_erases_update_and_recommission_staging() {
    for state in [LifecycleState::UpdatePending, LifecycleState::Provisioning] {
        let mut machine = if state == LifecycleState::Provisioning {
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
                .expect("recommission staging should succeed");
            recommission
        } else {
            machine_in(state)
        };

        machine
            .apply(Command::Revoke {
                authorizations: Authorizations {
                    revocation: true,
                    ..Authorizations::none()
                },
            })
            .expect("revocation should erase staged state");
        assert_eq!(machine.lifecycle(), LifecycleState::Revoked);
        assert_eq!(machine.pending_version, None);
        assert_eq!(machine.provisioning_generation, None);
        assert_eq!(machine.provisioning_origin, None);
        assert!(!machine.identity_active);
    }
}
