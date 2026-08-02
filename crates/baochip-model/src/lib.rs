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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProvisioningOrigin {
    Initial,
    Recommission,
}

impl ProvisioningOrigin {
    const fn abort_state(self) -> LifecycleState {
        match self {
            Self::Initial => LifecycleState::Blank,
            Self::Recommission => LifecycleState::Revoked,
        }
    }
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

/// Abstract, test-controlled result of candidate update validation.
///
/// This records semantic validation outcomes only. It does not perform
/// signature verification, compatibility analysis, or cryptography.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpdateValidation {
    pub candidate_authenticated: bool,
    pub compatible: bool,
    pub integrity_valid: bool,
}

impl UpdateValidation {
    #[must_use]
    pub const fn passed() -> Self {
        Self {
            candidate_authenticated: true,
            compatible: true,
            integrity_valid: true,
        }
    }

    const fn permits_activation(self) -> bool {
        self.candidate_authenticated && self.compatible && self.integrity_valid
    }
}

/// Canonical reason a staged update returns to the active image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateRejectionCause {
    /// A trusted, test-controlled validation result rejected the candidate.
    ValidationFailure(UpdateValidation),
    /// Update and owner authorities explicitly cancelled the candidate.
    AuthorizedCancellation(Authorizations),
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
    AbortProvisioning,
    StageUpdate {
        authorizations: Authorizations,
        candidate_version: u64,
    },
    AcceptUpdate {
        validation: UpdateValidation,
    },
    RejectUpdate {
        cause: UpdateRejectionCause,
    },
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
    pub staged_device_generation: Option<u64>,
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
    provisioning_origin: Option<ProvisioningOrigin>,
    identity_active: bool,
}

/// Typed fault conditions available only when the explicit `test-support`
/// feature is enabled.
///
/// This API exists to exercise otherwise unreachable fail-closed paths. It is
/// not part of the Baochip production model and the feature is disabled by
/// default.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelTestFault {
    ReceiptSequenceExhausted,
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
            provisioning_origin: None,
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

    /// Inject an explicitly typed model fault for cross-crate tests.
    ///
    /// This method is compiled only with the non-default `test-support`
    /// feature. It must never be used as a state-loading or recovery API.
    #[cfg(feature = "test-support")]
    pub fn inject_test_fault(&mut self, fault: ModelTestFault) {
        match fault {
            ModelTestFault::ReceiptSequenceExhausted => {
                self.receipt_sequence = u64::MAX;
            }
        }
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
            Command::AbortProvisioning => {
                self.abort_provisioning()?;
                None
            }
            Command::StageUpdate {
                authorizations,
                candidate_version,
            } => {
                self.stage_update(authorizations, candidate_version)?;
                None
            }
            Command::AcceptUpdate { validation } => {
                self.accept_update(validation)?;
                None
            }
            Command::RejectUpdate { cause } => {
                self.reject_update(cause)?;
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
                staged_device_generation: self.provisioning_generation,
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
        self.advance_transition_counter()?;
        self.provisioning_generation = Some(next_generation);
        self.provisioning_origin = Some(ProvisioningOrigin::Initial);
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

        let Some(generation) = self.provisioning_generation else {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        };
        if self.provisioning_origin.is_none() {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        }
        if generation <= self.device_generation {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        }
        self.provisioning_generation = None;
        self.provisioning_origin = None;
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

    fn abort_provisioning(&mut self) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::Provisioning {
            return Err(Rejection::InvalidState);
        }
        let Some(origin) = self.provisioning_origin else {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        };
        self.advance_transition_counter()?;
        self.pending_version = None;
        self.provisioning_generation = None;
        self.provisioning_origin = None;
        self.identity_active = false;
        self.lifecycle = origin.abort_state();
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

    fn accept_update(&mut self, validation: UpdateValidation) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::UpdatePending {
            return Err(Rejection::InvalidState);
        }
        let Some(candidate) = self.pending_version else {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        };
        if candidate <= self.active_version {
            return Err(Rejection::RollbackDetected);
        }
        if !validation.permits_activation() {
            return Err(Rejection::InvalidTransition);
        }

        self.advance_transition_counter()?;
        self.active_version = candidate;
        self.pending_version = None;
        self.lifecycle = LifecycleState::Operational;
        Ok(())
    }

    fn reject_update(&mut self, cause: UpdateRejectionCause) -> Result<(), Rejection> {
        if self.lifecycle != LifecycleState::UpdatePending {
            return Err(Rejection::InvalidState);
        }
        if self.pending_version.is_none() {
            self.enter_fault();
            return Err(Rejection::InternalInvariantViolation);
        }
        match cause {
            UpdateRejectionCause::ValidationFailure(validation) => {
                if validation.permits_activation() {
                    return Err(Rejection::InvalidTransition);
                }
            }
            UpdateRejectionCause::AuthorizedCancellation(auth) => {
                if !auth.update || !auth.owner {
                    return Err(Rejection::Unauthorized);
                }
            }
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
        if matches!(
            self.lifecycle,
            LifecycleState::Blank | LifecycleState::Revoked
        ) {
            return Err(Rejection::InvalidState);
        }
        if !auth.revocation {
            return Err(Rejection::Unauthorized);
        }
        self.advance_transition_counter()?;
        self.pending_version = None;
        self.provisioning_generation = None;
        self.provisioning_origin = None;
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
        self.advance_transition_counter()?;
        self.provisioning_generation = Some(next_generation);
        self.provisioning_origin = Some(ProvisioningOrigin::Recommission);
        self.pending_version = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Provisioning;
        Ok(())
    }

    fn decommission(&mut self, auth: Authorizations) -> Result<(), Rejection> {
        if !auth.decommission || !(auth.physical_presence || auth.independent) {
            return Err(Rejection::Unauthorized);
        }
        self.transition_counter = self.transition_counter.saturating_add(1);
        self.pending_version = None;
        self.provisioning_generation = None;
        self.provisioning_origin = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Decommissioned;
        Ok(())
    }

    fn start_measurement_epoch(&mut self) -> Result<(), Rejection> {
        self.require_operational()?;
        let Some(next_epoch) = self.measurement_epoch.checked_add(1) else {
            self.enter_fault();
            return Err(Rejection::CounterExhausted);
        };
        self.measurement_epoch = next_epoch;
        Ok(())
    }

    fn issue_receipt(&mut self, challenge: Option<[u8; 16]>) -> Result<ReceiptClaims, Rejection> {
        self.require_operational()?;
        if !self.identity_active {
            self.enter_fault();
            return Err(Rejection::IntegrityFailure);
        }
        let Some(next_sequence) = self.receipt_sequence.checked_add(1) else {
            self.enter_fault();
            return Err(Rejection::CounterExhausted);
        };
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
        let Some(next_counter) = self.transition_counter.checked_add(1) else {
            self.enter_fault();
            return Err(Rejection::CounterExhausted);
        };
        self.transition_counter = next_counter;
        Ok(())
    }

    fn enter_fault(&mut self) {
        self.pending_version = None;
        self.provisioning_generation = None;
        self.provisioning_origin = None;
        self.identity_active = false;
        self.lifecycle = LifecycleState::Fault;
    }
}

#[cfg(test)]
mod adversarial_tests;
#[cfg(test)]
mod baseline_tests;

#[cfg(test)]
mod invariant_tests;

#[cfg(test)]
mod update_tests;
