#![forbid(unsafe_code)]

//! Dependency-free executable lifecycle model for Baochip research.
//!
//! This crate models security semantics. It does not implement cryptography,
//! persistent storage, a wire format, or hardware behavior.

/// Security-relevant device lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleState {
    Blank,
    Provisioning,
    Operational,
    UpdatePending,
    Recovery,
    Revoked,
    Decommissioned,
    Fault,
}

/// Authorities and independent conditions supplied with a command.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Authorizations {
    pub root: bool,
    pub owner: bool,
    pub update: bool,
    pub recovery: bool,
    pub revocation: bool,
    pub decommission: bool,
    pub physical_presence: bool,
    pub independent: bool,
}

impl Authorizations {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            root: false,
            owner: false,
            update: false,
            recovery: false,
            revocation: false,
            decommission: false,
            physical_presence: false,
            independent: false,
        }
    }
}

/// Commands accepted by the first model increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Command {
    BeginProvisioning {
        authorizations: Authorizations,
    },
    CommitProvisioning {
        authorizations: Authorizations,
    },
    StageUpdate {
        authorizations: Authorizations,
        candidate_version: u64,
    },
    AcceptUpdate,
    RejectUpdate,
    EnterRecovery {
        authorizations: Authorizations,
    },
    CompleteRecovery {
        authorizations: Authorizations,
        recovered_version: u64,
    },
    Revoke {
        authorizations: Authorizations,
    },
    BeginRecommission {
        authorizations: Authorizations,
    },
    Decommission {
        authorizations: Authorizations,
    },
    StartMeasurementEpoch,
    IssueReceipt {
        challenge: Option<[u8; 16]>,
    },
}

/// Stable failure classes exposed by the model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rejection {
    InvalidState,
    Unauthorized,
    InvalidTransition,
    InvalidVersion,
    RollbackDetected,
    ReplayDetected,
    MalformedInput,
    UnsupportedProfile,
    UnsupportedCriticalExtension,
    PersistenceFailure,
    CounterExhausted,
    IntegrityFailure,
    Decommissioned,
    InternalInvariantViolation,
}

/// Abstract receipt claims emitted by the lifecycle model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptClaims {
    pub lifecycle_state: LifecycleState,
    pub device_generation: u64,
    pub transition_counter: u64,
    pub measurement_epoch: u64,
    pub receipt_sequence: u64,
    pub active_version: u64,
    pub challenge: Option<[u8; 16]>,
}

/// Non-secret audit record for a modeled command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub previous_state: LifecycleState,
    pub resulting_state: LifecycleState,
    pub device_generation: u64,
    pub transition_counter: u64,
}

/// Successful command output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Execution {
    pub audit: AuditEvent,
    pub receipt: Option<ReceiptClaims>,
}

/// Committed state for the first executable model increment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateMachine {
    lifecycle: LifecycleState,
    device_generation: u64,
    transition_counter: u64,
    measurement_epoch: u64,
    receipt_sequence: u64,
    active_version: u64,
    pending_version: Option<u64>,
    provisioning_generation: Option<u64>,
    identity_active: bool,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            lifecycle: LifecycleState::Blank,
            device_generation: 0,
            transition_counter: 0,
            measurement_epoch: 0,
            receipt_sequence: 0,
            active_version: 0,
            pending_version: None,
            provisioning_generation: None,
            identity_active: false,
        }
    }

    #[must_use]
    pub const fn lifecycle(&self) -> LifecycleState {
        self.lifecycle
    }

    #[must_use]
    pub const fn device_generation(&self) -> u64 {
        self.device_generation
    }

    #[must_use]
    pub const fn transition_counter(&self) -> u64 {
        self.transition_counter
    }

    #[must_use]
    pub const fn measurement_epoch(&self) -> u64 {
        self.measurement_epoch
    }

    #[must_use]
    pub const fn receipt_sequence(&self) -> u64 {
        self.receipt_sequence
    }

    #[must_use]
    pub const fn active_version(&self) -> u64 {
        self.active_version
    }

    /// Apply one deterministic command to the committed model state.
    ///
    /// # Errors
    ///
    /// Returns a stable [`Rejection`] when the command is unauthorized,
    /// invalid in the current lifecycle state, would roll state backward, or
    /// would exhaust a protected counter.
    pub fn apply(&mut self, command: Command) -> Result<Execution, Rejection> {
        if self.lifecycle == LifecycleState::Decommissioned {
            return Err(Rejection::Decommissioned);
        }

        let previous_state = self.lifecycle;
        let receipt = match command {
            Command::BeginProvisioning { authorizations } => {
                self.begin_provisioning(authorizations)?;
                None
            }
            Command::CommitProvisioning { authorizations } => {
                self.commit_provisioning(authorizations)?;
                None
            }
            Command::StageUpdate {
                authorizations,
                candidate_version,
            } => {
                self.stage_update(authorizations, candidate_version)?;
                None
            }
            Command::AcceptUpdate => {
                self.accept_update()?;
                None
            }
            Command::RejectUpdate => {
                self.reject_update()?;
                None
            }
            Command::EnterRecovery { authorizations } => {
                self.enter_recovery(authorizations)?;
                None
            }
            Command::CompleteRecovery {
                authorizations,
                recovered_version,
            } => {
                self.complete_recovery(authorizations, recovered_version)?;
                None
            }
            Command::Revoke { authorizations } => {
                self.revoke(authorizations)?;
                None
            }
            Command::BeginRecommission { authorizations } => {
                self.begin_recommission(authorizations)?;
                None
            }
            Command::Decommission { authorizations } => {
                self.decommission(authorizations)?;
                None
            }
            Command::StartMeasurementEpoch => {
                self.start_measurement_epoch()?;
                None
            }
            Command::IssueReceipt { challenge } => Some(self.issue_receipt(challenge)?),
        };

        Ok(Execution {
            audit: AuditEvent {
                previous_state,
                resulting_state: self.lifecycle,
                device_generation: self.device_generation,
                transition_counter: self.transition_counter,
            },
            receipt,
        })
    }

    fn begin_provisioning(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::Blank {
            return Err(Rejection::InvalidState);
        }
        if !auth.root || !auth.physical_presence {
            return Err(Rejection::Unauthorized);
        }

        let next_generation = self
            .device_generation
            .checked_add(1)
            .ok_or(Rejection::CounterExhausted)?;
        self.provisioning_generation = Some(next_generation);
        self.lifecycle = LifecycleState::Provisioning;
        Ok(())
    }

    fn commit_provisioning(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::Provisioning {
            return Err(Rejection::InvalidState);
        }
        if !auth.root || !auth.owner {
            return Err(Rejection::Unauthorized);
        }

        let generation = self
            .provisioning_generation
            .take()
            .ok_or(Rejection::InternalInvariantViolation)?;
        self.device_generation = generation;
        self.transition_counter = 1;
        self.measurement_epoch = 0;
        self.receipt_sequence = 0;
        self.active_version = 1;
        self.pending_version = None;
        self.identity_active = true;
        self.lifecycle = LifecycleState::Operational;
        Ok(())
    }

    fn stage_update(
        &mut self,
        auth: Authorizations,
        candidate_version: u64,
    ) -> Result<(), Rejection> {
        self.require_operational()?;
        if !auth.update || !auth.owner {
            return Err(Rejection::Unauthorized);
        }
        if candidate_version <= self.active_version {
            return Err(Rejection::RollbackDetected);
        }

        self.advance_transition_counter()?;
        self.pending_version = Some(candidate_version);
        self.lifecycle = LifecycleState::UpdatePending;
        Ok(())
    }

    fn accept_update(&mut self) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::UpdatePending {
            return Err(Rejection::InvalidState);
        }
        let candidate = self
            .pending_version
            .ok_or(Rejection::InternalInvariantViolation)?;
        if candidate <= self.active_version {
            return Err(Rejection::RollbackDetected);
        }

        self.advance_transition_counter()?;
        self.active_version = candidate;
        self.pending_version = None;
        self.lifecycle = LifecycleState::Operational;
        Ok(())
    }

    fn reject_update(&mut self) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::UpdatePending {
            return Err(Rejection::InvalidState);
        }
        self.advance_transition_counter()?;
        self.pending_version = None;
        self.lifecycle = LifecycleState::Operational;
        Ok(())
    }

    fn enter_recovery(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        self.require_operational()?;
        if !auth.recovery || !(auth.physical_presence || auth.independent) {
            return Err(Rejection::Unauthorized);
        }
        self.advance_transition_counter()?;
        self.lifecycle = LifecycleState::Recovery;
        Ok(())
    }

    fn complete_recovery(
        &mut self,
        auth: Authorizations,
        recovered_version: u64,
    ) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::Recovery {
            return Err(Rejection::InvalidState);
        }
        if !auth.recovery || !auth.owner {
            return Err(Rejection::Unauthorized);
        }
        if recovered_version < self.active_version {
            return Err(Rejection::RollbackDetected);
        }
        self.advance_transition_counter()?;
        self.active_version = recovered_version;
        self.lifecycle = LifecycleState::Operational;
        Ok(())
    }

    fn revoke(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if !auth.revocation {
            return Err(Rejection::Unauthorized);
        }
        if matches!(
            self.lifecycle,
            LifecycleState::Blank | LifecycleState::Revoked
        ) {
            return Err(Rejection::InvalidState);
        }
        self.advance_transition_counter()?;
        self.pending_version = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Revoked;
        Ok(())
    }

    fn begin_recommission(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::Revoked {
            return Err(Rejection::InvalidState);
        }
        if !auth.root || !auth.owner || !(auth.physical_presence || auth.independent) {
            return Err(Rejection::Unauthorized);
        }
        let next_generation = self
            .device_generation
            .checked_add(1)
            .ok_or(Rejection::CounterExhausted)?;
        self.provisioning_generation = Some(next_generation);
        self.pending_version = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Provisioning;
        Ok(())
    }

    fn decommission(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if !auth.decommission || !(auth.physical_presence || auth.independent) {
            return Err(Rejection::Unauthorized);
        }
        self.pending_version = None;
        self.provisioning_generation = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Decommissioned;
        Ok(())
    }

    fn start_measurement_epoch(&mut self) -> Result<(), Rejection> {
        self.require_operational()?;
        self.measurement_epoch = self
            .measurement_epoch
            .checked_add(1)
            .ok_or(Rejection::CounterExhausted)?;
        Ok(())
    }

    fn issue_receipt(&mut self, challenge: Option<[u8; 16]>) -> Result<ReceiptClaims, Rejection> {
        self.require_operational()?;
        if !self.identity_active {
            return Err(Rejection::IntegrityFailure);
        }
        let next_sequence = self
            .receipt_sequence
            .checked_add(1)
            .ok_or(Rejection::CounterExhausted)?;
        self.receipt_sequence = next_sequence;

        Ok(ReceiptClaims {
            lifecycle_state: self.lifecycle,
            device_generation: self.device_generation,
            transition_counter: self.transition_counter,
            measurement_epoch: self.measurement_epoch,
            receipt_sequence: next_sequence,
            active_version: self.active_version,
            challenge,
        })
    }

    fn require_operational(&self) -> Result<(), Rejection> {
        if self.lifecycle == LifecycleState::Operational {
            Ok(())
        } else {
            Err(Rejection::InvalidState)
        }
    }

    fn advance_transition_counter(&mut self) -> Result<(), Rejection> {
        self.transition_counter = self
            .transition_counter
            .checked_add(1)
            .ok_or(Rejection::CounterExhausted)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
            .apply(Command::AcceptUpdate)
            .expect("staged update should commit");
        assert_eq!(machine.active_version(), 2);
        assert_eq!(machine.lifecycle(), LifecycleState::Operational);
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
            machine.apply(Command::AcceptUpdate),
            Err(Rejection::InvalidState)
        );
        assert_eq!(
            machine.apply(Command::StartMeasurementEpoch),
            Err(Rejection::InvalidState)
        );
    }

    #[test]
    fn receipt_counter_exhaustion_fails_closed() {
        let mut machine = operational_machine();
        machine.receipt_sequence = u64::MAX;
        assert_eq!(
            machine.apply(Command::IssueReceipt { challenge: None }),
            Err(Rejection::CounterExhausted)
        );
        assert_eq!(machine.receipt_sequence(), u64::MAX);
    }
}
