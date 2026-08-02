#![forbid(unsafe_code)]

//! Candidate-neutral semantic fixtures for Baochip encoding research.
//!
//! These values are typed semantic inputs, not protocol bytes. This crate does
//! not select an encoding, canonicalization algorithm, integrity suite, or
//! physical representation.

/// Protected semantic domains. Enum layout and Rust discriminants are not
/// protocol identifiers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectClass {
    PersistentState,
    AuthorityMetadata,
    ExecutionReceipt,
    LifecycleAudit,
    Endorsement,
    ReferenceValues,
}

/// Security-relevant lifecycle states. Enum layout is not a wire format.
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

/// Source of a staged provisioning generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvisioningOrigin {
    Initial,
    Recommission,
}

/// Exact semantic type of an extension value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExtensionValue {
    Unsigned(u64),
    Boolean(bool),
    Bytes(Vec<u8>),
    Text(String),
}

/// One retained extension entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Extension {
    pub identifier: String,
    pub critical: bool,
    pub value: ExtensionValue,
}

/// Subject scope that prevents cross-device or cross-lineage substitution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubjectScope {
    pub device_identifier: Vec<u8>,
    pub device_generation: u64,
    pub key_generation: Option<u64>,
}

/// Required receipt lineage mode. This is semantic context, not a key format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLineageContext {
    KeyGeneration(u64),
    ProvisioningGeneration(u64),
}

/// Context common to every protected semantic object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedContext {
    pub object_class: ObjectClass,
    pub profile_identifier: String,
    pub schema_version: u64,
    pub integrity_suite_identifier: String,
    pub subject: SubjectScope,
    /// Entries are kept in strictly increasing identifier order so the fixture
    /// representation denotes a set without silently dropping ordering
    /// differences. This is a fixture rule, not a selected byte encoding.
    pub extensions: Vec<Extension>,
}

/// Complete executable-model persistent-state projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistentStateProjection {
    pub context: ProtectedContext,
    pub slot_id: u8,
    pub commit_id: u64,
    pub lifecycle_state: LifecycleState,
    pub device_generation: u64,
    pub transition_counter: u64,
    pub measurement_epoch: u64,
    pub receipt_sequence: u64,
    pub active_version: u64,
    pub pending_version: Option<u64>,
    pub provisioning_generation: Option<u64>,
    pub provisioning_origin: Option<ProvisioningOrigin>,
    pub identity_active: bool,
}

/// Current lifecycle-model receipt fields embedded in a prepared execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentReceiptClaims {
    pub lifecycle_state: LifecycleState,
    pub device_generation: u64,
    pub transition_counter: u64,
    pub measurement_epoch: u64,
    pub receipt_sequence: u64,
    pub active_version: u64,
    pub challenge: Option<[u8; 16]>,
}

/// Current non-secret lifecycle audit fields embedded in a prepared execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleAuditProjection {
    pub previous_state: LifecycleState,
    pub resulting_state: LifecycleState,
    pub device_generation: u64,
    pub staged_device_generation: Option<u64>,
    pub transition_counter: u64,
}

/// Complete current lifecycle execution staged by the persistence model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionProjection {
    pub audit: LifecycleAuditProjection,
    pub receipt: Option<CurrentReceiptClaims>,
}

/// Stable lifecycle-model rejection classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RejectionClass {
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

/// Prepared command result withheld until selector commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedOutcomeProjection {
    Applied(ExecutionProjection),
    Rejected(RejectionClass),
}

/// Phase-specific authority fields. The raw selected slot remains outside the
/// phase because it exists in every phase.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthorityPhaseProjection {
    Clean,
    Prepared {
        candidate_slot: u8,
        commit_id: u64,
        prepared_outcome: PreparedOutcomeProjection,
    },
    Committed {
        previous_slot: u8,
        selected_next_slot: u8,
        commit_id: u64,
    },
}

/// Complete authority-metadata semantic projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorityMetadataProjection {
    pub context: ProtectedContext,
    pub raw_selected_slot: u8,
    /// Commit identifier for each present logical record. `None` denotes an
    /// empty slot, so presence and record identity cannot contradict.
    pub record_commit_ids: [Option<u64>; 2],
    pub phase: AuthorityPhaseProjection,
}

/// Future complete execution-receipt projection used only as a semantic
/// fixture. The current lifecycle model implements only a documented subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionReceiptProjection {
    pub context: ProtectedContext,
    /// Commit identifier of the authoritative persistent snapshot whose
    /// selector commit released this receipt.
    pub authority_commit_id: u64,
    pub lineage: ReceiptLineageContext,
    pub key_identifier: Vec<u8>,
    pub lifecycle_state: LifecycleState,
    pub device_generation: u64,
    pub transition_counter: u64,
    pub measurement_epoch: u64,
    pub receipt_sequence: Option<u64>,
    pub active_version: u64,
    pub challenge: Option<Vec<u8>>,
    pub measurement_root: Vec<u8>,
    pub measurement_context: String,
    pub policy_identifier: String,
    pub policy_version: u64,
    pub input_commitment: Option<Vec<u8>>,
    pub output_commitment: Option<Vec<u8>>,
}

/// One supported semantic object in the initial fixture corpus.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticObject {
    PersistentState(PersistentStateProjection),
    AuthorityMetadata(AuthorityMetadataProjection),
    ExecutionReceipt(ExecutionReceiptProjection),
}

/// Named fixture and the distinction it is intended to exercise.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fixture {
    pub identifier: &'static str,
    pub purpose: &'static str,
    pub object: SemanticObject,
}

/// Stable semantic-validation failures. They are not parser rejection codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    EmptyIdentifier,
    EmptySubject,
    EmptyRequiredValue,
    WrongObjectClass,
    InvalidSlot,
    MissingRecord,
    UnexpectedRecord,
    SlotConflict,
    SelectorMismatch,
    DuplicateExtension,
    UnorderedExtensions,
    CommitIdMismatch,
    AuthorityPhaseMismatch,
    AuthorityContextMismatch,
    StateContextMismatch,
    InconsistentState,
    InconsistentExecution,
}

impl ProtectedContext {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.profile_identifier.is_empty() || self.integrity_suite_identifier.is_empty() {
            return Err(ValidationError::EmptyIdentifier);
        }
        if self.subject.device_identifier.is_empty() {
            return Err(ValidationError::EmptySubject);
        }
        for extension in &self.extensions {
            if extension.identifier.is_empty() {
                return Err(ValidationError::EmptyIdentifier);
            }
        }
        for pair in self.extensions.windows(2) {
            if pair[0].identifier == pair[1].identifier {
                return Err(ValidationError::DuplicateExtension);
            }
            if pair[0].identifier > pair[1].identifier {
                return Err(ValidationError::UnorderedExtensions);
            }
        }
        Ok(())
    }
}

impl PersistentStateProjection {
    /// Validate semantic invariants without producing bytes.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.context.validate()?;
        if self.context.object_class != ObjectClass::PersistentState {
            return Err(ValidationError::WrongObjectClass);
        }
        validate_slot(self.slot_id)?;
        if self.context.subject.device_generation != self.device_generation {
            return Err(ValidationError::InconsistentState);
        }

        let identity_must_be_active = matches!(
            self.lifecycle_state,
            LifecycleState::Operational | LifecycleState::UpdatePending | LifecycleState::Recovery
        );
        if self.identity_active != identity_must_be_active {
            return Err(ValidationError::InconsistentState);
        }

        let provisioning = self.lifecycle_state == LifecycleState::Provisioning;
        let has_provisioning_generation = self.provisioning_generation.is_some();
        let has_provisioning_origin = self.provisioning_origin.is_some();
        if (provisioning && !(has_provisioning_generation && has_provisioning_origin))
            || (!provisioning && (has_provisioning_generation || has_provisioning_origin))
        {
            return Err(ValidationError::InconsistentState);
        }
        if let Some(generation) = self.provisioning_generation {
            if self.device_generation.checked_add(1) != Some(generation) {
                return Err(ValidationError::InconsistentState);
            }
        }
        match self.provisioning_origin {
            Some(ProvisioningOrigin::Initial) if self.device_generation != 0 => {
                return Err(ValidationError::InconsistentState);
            }
            Some(ProvisioningOrigin::Recommission) if self.device_generation == 0 => {
                return Err(ValidationError::InconsistentState);
            }
            _ => {}
        }
        let update_pending = self.lifecycle_state == LifecycleState::UpdatePending;
        if update_pending != self.pending_version.is_some() {
            return Err(ValidationError::InconsistentState);
        }
        if let Some(version) = self.pending_version {
            if version <= self.active_version {
                return Err(ValidationError::InconsistentState);
            }
        }
        Ok(())
    }
}

impl ExecutionProjection {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(receipt) = &self.receipt {
            if self.audit.previous_state != LifecycleState::Operational
                || self.audit.resulting_state != LifecycleState::Operational
                || self.audit.staged_device_generation.is_some()
                || receipt.lifecycle_state != self.audit.resulting_state
                || receipt.device_generation != self.audit.device_generation
                || receipt.transition_counter != self.audit.transition_counter
                || receipt.device_generation == 0
                || receipt.transition_counter == 0
                || receipt.receipt_sequence == 0
                || receipt.active_version == 0
            {
                return Err(ValidationError::InconsistentExecution);
            }
        }
        Ok(())
    }
}

impl PreparedOutcomeProjection {
    fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::Applied(execution) => execution.validate(),
            Self::Rejected(_) => Ok(()),
        }
    }
}

impl AuthorityMetadataProjection {
    /// Validate phase, slot, presence, and prepared-outcome invariants.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.context.validate()?;
        if self.context.object_class != ObjectClass::AuthorityMetadata {
            return Err(ValidationError::WrongObjectClass);
        }
        validate_slot(self.raw_selected_slot)?;

        match &self.phase {
            AuthorityPhaseProjection::Clean => {
                if self.record_commit_ids[usize::from(self.raw_selected_slot)].is_none() {
                    return Err(ValidationError::MissingRecord);
                }
                if self.record_commit_ids[usize::from(other_slot(self.raw_selected_slot))].is_some()
                {
                    return Err(ValidationError::UnexpectedRecord);
                }
            }
            AuthorityPhaseProjection::Prepared {
                candidate_slot,
                commit_id,
                prepared_outcome,
            } => {
                validate_slot(*candidate_slot)?;
                if *candidate_slot == self.raw_selected_slot {
                    return Err(ValidationError::SlotConflict);
                }
                let previous_commit_id = self.record_commit_id(self.raw_selected_slot)?;
                let candidate_commit_id = self.record_commit_id(*candidate_slot)?;
                if previous_commit_id.checked_add(1) != Some(*commit_id)
                    || candidate_commit_id != *commit_id
                {
                    return Err(ValidationError::CommitIdMismatch);
                }
                prepared_outcome.validate()?;
            }
            AuthorityPhaseProjection::Committed {
                previous_slot,
                selected_next_slot,
                commit_id,
            } => {
                validate_slot(*previous_slot)?;
                validate_slot(*selected_next_slot)?;
                if previous_slot == selected_next_slot {
                    return Err(ValidationError::SlotConflict);
                }
                if *selected_next_slot != self.raw_selected_slot {
                    return Err(ValidationError::SelectorMismatch);
                }
                let previous_commit_id = self.record_commit_id(*previous_slot)?;
                let selected_commit_id = self.record_commit_id(*selected_next_slot)?;
                if previous_commit_id.checked_add(1) != Some(*commit_id)
                    || selected_commit_id != *commit_id
                {
                    return Err(ValidationError::CommitIdMismatch);
                }
            }
        }
        Ok(())
    }

    fn record_commit_id(&self, slot: u8) -> Result<u64, ValidationError> {
        self.record_commit_ids[usize::from(slot)].ok_or(ValidationError::MissingRecord)
    }
}

impl ExecutionReceiptProjection {
    /// Validate required receipt semantics without serializing the object.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.context.validate()?;
        if self.context.object_class != ObjectClass::ExecutionReceipt {
            return Err(ValidationError::WrongObjectClass);
        }
        if self.context.subject.device_generation != self.device_generation {
            return Err(ValidationError::InconsistentState);
        }
        match self.lineage {
            ReceiptLineageContext::KeyGeneration(generation)
                if self.context.subject.key_generation != Some(generation) =>
            {
                return Err(ValidationError::InconsistentState);
            }
            ReceiptLineageContext::ProvisioningGeneration(generation)
                if self.context.subject.key_generation.is_some()
                    || generation != self.device_generation =>
            {
                return Err(ValidationError::InconsistentState);
            }
            _ => {}
        }
        if self.key_identifier.is_empty()
            || self.measurement_root.is_empty()
            || self.measurement_context.is_empty()
            || self.policy_identifier.is_empty()
        {
            return Err(ValidationError::EmptyRequiredValue);
        }
        if self.input_commitment.as_ref().is_some_and(Vec::is_empty)
            || self.output_commitment.as_ref().is_some_and(Vec::is_empty)
        {
            return Err(ValidationError::EmptyRequiredValue);
        }
        Ok(())
    }

    /// Validate that selector commit in a matching authority object released
    /// this receipt.
    pub fn validate_release_authority(
        &self,
        authority: &AuthorityMetadataProjection,
    ) -> Result<(), ValidationError> {
        self.validate()?;
        authority.validate()?;
        let AuthorityPhaseProjection::Committed {
            selected_next_slot,
            commit_id,
            ..
        } = authority.phase
        else {
            return Err(ValidationError::AuthorityPhaseMismatch);
        };
        let selected_commit_id = authority.record_commit_id(selected_next_slot)?;
        if authority.context.subject != self.context.subject
            || commit_id != self.authority_commit_id
            || selected_commit_id != self.authority_commit_id
        {
            return Err(ValidationError::AuthorityContextMismatch);
        }
        Ok(())
    }

    /// Validate every receipt claim represented in the authoritative
    /// persistent snapshot that released it.
    pub fn validate_authoritative_state(
        &self,
        state: &PersistentStateProjection,
    ) -> Result<(), ValidationError> {
        self.validate()?;
        state.validate()?;
        if state.context.subject != self.context.subject
            || state.commit_id != self.authority_commit_id
            || state.lifecycle_state != self.lifecycle_state
            || state.device_generation != self.device_generation
            || state.transition_counter != self.transition_counter
            || state.measurement_epoch != self.measurement_epoch
            || state.active_version != self.active_version
            || self
                .receipt_sequence
                .is_some_and(|sequence| sequence != state.receipt_sequence)
        {
            return Err(ValidationError::StateContextMismatch);
        }
        Ok(())
    }

    /// Validate the complete semantic release relationship among receipt,
    /// committed authority metadata, and selected persistent state.
    pub fn validate_release(
        &self,
        authority: &AuthorityMetadataProjection,
        state: &PersistentStateProjection,
    ) -> Result<(), ValidationError> {
        self.validate_release_authority(authority)?;
        self.validate_authoritative_state(state)?;
        let AuthorityPhaseProjection::Committed {
            selected_next_slot, ..
        } = authority.phase
        else {
            return Err(ValidationError::AuthorityPhaseMismatch);
        };
        if selected_next_slot != state.slot_id {
            return Err(ValidationError::StateContextMismatch);
        }
        Ok(())
    }
}

impl SemanticObject {
    /// Validate the object's semantic invariants without selecting bytes.
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::PersistentState(state) => state.validate(),
            Self::AuthorityMetadata(authority) => authority.validate(),
            Self::ExecutionReceipt(receipt) => receipt.validate(),
        }
    }
}

fn validate_slot(slot: u8) -> Result<(), ValidationError> {
    if slot > 1 {
        return Err(ValidationError::InvalidSlot);
    }
    Ok(())
}

const fn other_slot(slot: u8) -> u8 {
    1 - slot
}

fn context(
    object_class: ObjectClass,
    device_generation: u64,
    key_generation: Option<u64>,
) -> ProtectedContext {
    ProtectedContext {
        object_class,
        profile_identifier: String::from("baochip.fixture.profile"),
        schema_version: 1,
        integrity_suite_identifier: String::from("baochip.fixture.unselected-suite"),
        subject: SubjectScope {
            device_identifier: vec![0x42, 0x43],
            device_generation,
            key_generation,
        },
        extensions: vec![
            Extension {
                identifier: String::from("example.bytes"),
                critical: false,
                value: ExtensionValue::Bytes(vec![0x00, 0xff]),
            },
            Extension {
                identifier: String::from("example.flag"),
                critical: true,
                value: ExtensionValue::Boolean(false),
            },
            Extension {
                identifier: String::from("example.text"),
                critical: false,
                value: ExtensionValue::Text(String::from("fixture")),
            },
            Extension {
                identifier: String::from("example.unsigned"),
                critical: false,
                value: ExtensionValue::Unsigned(0),
            },
        ],
    }
}

fn audit(resulting_state: LifecycleState) -> LifecycleAuditProjection {
    LifecycleAuditProjection {
        previous_state: LifecycleState::Operational,
        resulting_state,
        device_generation: 1,
        staged_device_generation: None,
        transition_counter: 2,
    }
}

/// Initial positive corpus shared by future candidate encoders.
///
/// Fixture values such as profile and suite identifiers are evaluation-only
/// examples. They do not assign production identifiers or select a suite.
#[must_use]
pub fn positive_fixtures() -> Vec<Fixture> {
    vec![
        Fixture {
            identifier: "persistent-blank-absent-key-generation",
            purpose: "zero counters and absent optional lineage",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 0, None),
                slot_id: 0,
                commit_id: 0,
                lifecycle_state: LifecycleState::Blank,
                device_generation: 0,
                transition_counter: 0,
                measurement_epoch: 0,
                receipt_sequence: 0,
                active_version: 0,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-blank-zero-key-generation",
            purpose: "present zero differs from absent lineage",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 0, Some(0)),
                slot_id: 0,
                commit_id: 0,
                lifecycle_state: LifecycleState::Blank,
                device_generation: 0,
                transition_counter: 0,
                measurement_epoch: 0,
                receipt_sequence: 0,
                active_version: 0,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-operational-u64-boundaries",
            purpose: "maximum-width unsigned semantic values",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, u64::MAX, Some(u64::MAX)),
                slot_id: 1,
                commit_id: u64::MAX,
                lifecycle_state: LifecycleState::Operational,
                device_generation: u64::MAX,
                transition_counter: u64::MAX,
                measurement_epoch: u64::MAX,
                receipt_sequence: u64::MAX,
                active_version: u64::MAX,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: true,
            }),
        },
        Fixture {
            identifier: "persistent-provisioning-initial",
            purpose: "present initial provisioning generation and origin",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 0, None),
                slot_id: 1,
                commit_id: 1,
                lifecycle_state: LifecycleState::Provisioning,
                device_generation: 0,
                transition_counter: 1,
                measurement_epoch: 0,
                receipt_sequence: 0,
                active_version: 0,
                pending_version: None,
                provisioning_generation: Some(1),
                provisioning_origin: Some(ProvisioningOrigin::Initial),
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-provisioning-recommission",
            purpose: "present recommission generation and origin",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 0,
                commit_id: 2,
                lifecycle_state: LifecycleState::Provisioning,
                device_generation: 1,
                transition_counter: 2,
                measurement_epoch: 0,
                receipt_sequence: 0,
                active_version: 1,
                pending_version: None,
                provisioning_generation: Some(2),
                provisioning_origin: Some(ProvisioningOrigin::Recommission),
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-update-pending",
            purpose: "present staged update version",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 1,
                commit_id: 3,
                lifecycle_state: LifecycleState::UpdatePending,
                device_generation: 1,
                transition_counter: 3,
                measurement_epoch: 1,
                receipt_sequence: 2,
                active_version: 1,
                pending_version: Some(2),
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: true,
            }),
        },
        Fixture {
            identifier: "persistent-recovery",
            purpose: "recovery lifecycle without staged update or provisioning",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 0,
                commit_id: 4,
                lifecycle_state: LifecycleState::Recovery,
                device_generation: 1,
                transition_counter: 4,
                measurement_epoch: 1,
                receipt_sequence: 2,
                active_version: 1,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: true,
            }),
        },
        Fixture {
            identifier: "persistent-operational-receipt-release",
            purpose: "authoritative snapshot matching the positive receipt release",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, None),
                slot_id: 1,
                commit_id: 4,
                lifecycle_state: LifecycleState::Operational,
                device_generation: 1,
                transition_counter: 3,
                measurement_epoch: 3,
                receipt_sequence: 1,
                active_version: 4,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: true,
            }),
        },
        Fixture {
            identifier: "persistent-revoked",
            purpose: "revoked lifecycle with inactive identity",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 1,
                commit_id: 5,
                lifecycle_state: LifecycleState::Revoked,
                device_generation: 1,
                transition_counter: 5,
                measurement_epoch: 1,
                receipt_sequence: 2,
                active_version: 1,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-decommissioned",
            purpose: "terminal lifecycle with inactive identity",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 0,
                commit_id: 6,
                lifecycle_state: LifecycleState::Decommissioned,
                device_generation: 1,
                transition_counter: 6,
                measurement_epoch: 1,
                receipt_sequence: 2,
                active_version: 1,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "persistent-fault",
            purpose: "fault lifecycle disables identity eligibility",
            object: SemanticObject::PersistentState(PersistentStateProjection {
                context: context(ObjectClass::PersistentState, 1, Some(1)),
                slot_id: 1,
                commit_id: 7,
                lifecycle_state: LifecycleState::Fault,
                device_generation: 1,
                transition_counter: u64::MAX,
                measurement_epoch: 1,
                receipt_sequence: 2,
                active_version: 1,
                pending_version: None,
                provisioning_generation: None,
                provisioning_origin: None,
                identity_active: false,
            }),
        },
        Fixture {
            identifier: "authority-clean",
            purpose: "stable selected record with no candidate",
            object: SemanticObject::AuthorityMetadata(AuthorityMetadataProjection {
                context: context(ObjectClass::AuthorityMetadata, 1, Some(1)),
                raw_selected_slot: 0,
                record_commit_ids: [Some(1), None],
                phase: AuthorityPhaseProjection::Clean,
            }),
        },
        Fixture {
            identifier: "authority-prepared-applied",
            purpose: "candidate plus complete withheld execution",
            object: SemanticObject::AuthorityMetadata(AuthorityMetadataProjection {
                context: context(ObjectClass::AuthorityMetadata, 1, Some(1)),
                raw_selected_slot: 0,
                record_commit_ids: [Some(1), Some(2)],
                phase: AuthorityPhaseProjection::Prepared {
                    candidate_slot: 1,
                    commit_id: 2,
                    prepared_outcome: PreparedOutcomeProjection::Applied(ExecutionProjection {
                        audit: audit(LifecycleState::UpdatePending),
                        receipt: None,
                    }),
                },
            }),
        },
        Fixture {
            identifier: "authority-prepared-rejected",
            purpose: "candidate plus stable withheld rejection",
            object: SemanticObject::AuthorityMetadata(AuthorityMetadataProjection {
                context: context(ObjectClass::AuthorityMetadata, 1, Some(1)),
                raw_selected_slot: 1,
                record_commit_ids: [Some(3), Some(2)],
                phase: AuthorityPhaseProjection::Prepared {
                    candidate_slot: 0,
                    commit_id: 3,
                    prepared_outcome: PreparedOutcomeProjection::Rejected(
                        RejectionClass::IntegrityFailure,
                    ),
                },
            }),
        },
        Fixture {
            identifier: "authority-committed",
            purpose: "selected next record with retained previous record",
            object: SemanticObject::AuthorityMetadata(AuthorityMetadataProjection {
                context: context(ObjectClass::AuthorityMetadata, 1, None),
                raw_selected_slot: 1,
                record_commit_ids: [Some(3), Some(4)],
                phase: AuthorityPhaseProjection::Committed {
                    previous_slot: 0,
                    selected_next_slot: 1,
                    commit_id: 4,
                },
            }),
        },
        Fixture {
            identifier: "receipt-minimal-optionals-absent",
            purpose: "complete required future receipt with optionals absent",
            object: SemanticObject::ExecutionReceipt(ExecutionReceiptProjection {
                context: context(ObjectClass::ExecutionReceipt, 1, None),
                authority_commit_id: 4,
                lineage: ReceiptLineageContext::ProvisioningGeneration(1),
                key_identifier: vec![0x01],
                lifecycle_state: LifecycleState::Operational,
                device_generation: 1,
                transition_counter: 3,
                measurement_epoch: 3,
                receipt_sequence: None,
                active_version: 4,
                challenge: None,
                measurement_root: vec![0xaa],
                measurement_context: String::from("fixture.measurement"),
                policy_identifier: String::from("fixture.policy"),
                policy_version: 1,
                input_commitment: None,
                output_commitment: None,
            }),
        },
        Fixture {
            identifier: "receipt-optionals-present",
            purpose: "present zero and byte-valued optionals remain distinct",
            object: SemanticObject::ExecutionReceipt(ExecutionReceiptProjection {
                context: context(ObjectClass::ExecutionReceipt, 1, Some(0)),
                authority_commit_id: 1,
                lineage: ReceiptLineageContext::KeyGeneration(0),
                key_identifier: vec![0x00],
                lifecycle_state: LifecycleState::Operational,
                device_generation: 1,
                transition_counter: 0,
                measurement_epoch: 0,
                receipt_sequence: Some(0),
                active_version: 0,
                challenge: Some(vec![0; 16]),
                measurement_root: vec![0x00],
                measurement_context: String::from("fixture.measurement.zero"),
                policy_identifier: String::from("fixture.policy.zero"),
                policy_version: 0,
                input_commitment: Some(vec![0x00]),
                output_commitment: Some(vec![0x00]),
            }),
        },
    ]
}

#[cfg(test)]
mod adversarial_tests;
#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(identifier: &str) -> Fixture {
        positive_fixtures()
            .into_iter()
            .find(|fixture| fixture.identifier == identifier)
            .expect("fixture must exist")
    }

    #[test]
    fn every_positive_fixture_is_unique_and_valid() {
        let fixtures = positive_fixtures();
        for (index, fixture) in fixtures.iter().enumerate() {
            assert!(!fixture.identifier.is_empty());
            assert!(!fixture.purpose.is_empty());
            assert_eq!(fixture.object.validate(), Ok(()), "{}", fixture.identifier);
            assert!(
                fixtures[..index]
                    .iter()
                    .all(|prior| prior.identifier != fixture.identifier)
            );
        }
    }

    #[test]
    fn absent_and_present_zero_lineage_are_distinct() {
        assert_ne!(
            fixture("persistent-blank-absent-key-generation").object,
            fixture("persistent-blank-zero-key-generation").object
        );
    }

    #[test]
    fn positive_corpus_covers_every_lifecycle_state_identity() {
        let mut covered = [false; 8];
        for fixture in positive_fixtures() {
            if let SemanticObject::PersistentState(state) = fixture.object {
                let index = match state.lifecycle_state {
                    LifecycleState::Blank => 0,
                    LifecycleState::Provisioning => 1,
                    LifecycleState::Operational => 2,
                    LifecycleState::UpdatePending => 3,
                    LifecycleState::Recovery => 4,
                    LifecycleState::Revoked => 5,
                    LifecycleState::Decommissioned => 6,
                    LifecycleState::Fault => 7,
                };
                covered[index] = true;
            }
        }
        assert!(covered.into_iter().all(|present| present));
    }

    #[test]
    fn object_class_substitution_fails_validation() {
        let mut fixture = fixture("persistent-blank-absent-key-generation");
        let SemanticObject::PersistentState(state) = &mut fixture.object else {
            panic!("expected persistent-state fixture");
        };
        state.context.object_class = ObjectClass::ExecutionReceipt;
        assert_eq!(state.validate(), Err(ValidationError::WrongObjectClass));
    }

    #[test]
    fn persistent_state_rejects_partial_provisioning_fields() {
        let mut fixture = fixture("persistent-provisioning-initial");
        let SemanticObject::PersistentState(state) = &mut fixture.object else {
            panic!("expected persistent-state fixture");
        };
        state.provisioning_origin = None;
        assert_eq!(state.validate(), Err(ValidationError::InconsistentState));
    }

    #[test]
    fn persistent_state_rejects_subject_generation_mismatch() {
        let mut fixture = fixture("persistent-operational-u64-boundaries");
        let SemanticObject::PersistentState(state) = &mut fixture.object else {
            panic!("expected persistent-state fixture");
        };
        state.context.subject.device_generation = 0;
        assert_eq!(state.validate(), Err(ValidationError::InconsistentState));
    }

    #[test]
    fn duplicate_extension_identifier_fails_validation() {
        let mut fixture = fixture("persistent-blank-absent-key-generation");
        let SemanticObject::PersistentState(state) = &mut fixture.object else {
            panic!("expected persistent-state fixture");
        };
        state.context.extensions[1].identifier = state.context.extensions[0].identifier.clone();
        assert_eq!(state.validate(), Err(ValidationError::DuplicateExtension));
    }

    #[test]
    fn extension_order_is_not_silently_ignored() {
        let mut fixture = fixture("persistent-blank-absent-key-generation");
        let SemanticObject::PersistentState(state) = &mut fixture.object else {
            panic!("expected persistent-state fixture");
        };
        state.context.extensions.reverse();
        assert_eq!(state.validate(), Err(ValidationError::UnorderedExtensions));
    }

    #[test]
    fn clean_phase_rejects_an_unexpected_second_record() {
        let mut fixture = fixture("authority-clean");
        let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
            panic!("expected authority fixture");
        };
        authority.record_commit_ids = [Some(1), Some(2)];
        assert_eq!(authority.validate(), Err(ValidationError::UnexpectedRecord));
    }

    #[test]
    fn authority_rejects_an_out_of_range_raw_slot() {
        let mut fixture = fixture("authority-clean");
        let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
            panic!("expected authority fixture");
        };
        authority.raw_selected_slot = 2;
        assert_eq!(authority.validate(), Err(ValidationError::InvalidSlot));
    }

    #[test]
    fn prepared_phase_rejects_candidate_selector_conflict() {
        let mut fixture = fixture("authority-prepared-applied");
        let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
            panic!("expected authority fixture");
        };
        let AuthorityPhaseProjection::Prepared { candidate_slot, .. } = &mut authority.phase else {
            panic!("expected prepared phase");
        };
        *candidate_slot = authority.raw_selected_slot;
        assert_eq!(authority.validate(), Err(ValidationError::SlotConflict));
    }

    #[test]
    fn committed_phase_requires_selector_to_match_next() {
        let mut fixture = fixture("authority-committed");
        let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
            panic!("expected authority fixture");
        };
        authority.raw_selected_slot = 0;
        assert_eq!(authority.validate(), Err(ValidationError::SelectorMismatch));
    }

    #[test]
    fn prepared_execution_must_bind_consistent_receipt_state() {
        let mut fixture = fixture("authority-prepared-applied");
        let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
            panic!("expected authority fixture");
        };
        let AuthorityPhaseProjection::Prepared {
            prepared_outcome: PreparedOutcomeProjection::Applied(execution),
            ..
        } = &mut authority.phase
        else {
            panic!("expected applied prepared outcome");
        };
        execution.receipt = Some(CurrentReceiptClaims {
            lifecycle_state: LifecycleState::Operational,
            device_generation: 1,
            transition_counter: 2,
            measurement_epoch: 0,
            receipt_sequence: 0,
            active_version: 1,
            challenge: None,
        });
        assert_eq!(
            authority.validate(),
            Err(ValidationError::InconsistentExecution)
        );
    }

    #[test]
    fn receipt_rejects_present_but_empty_commitment() {
        let mut fixture = fixture("receipt-optionals-present");
        let SemanticObject::ExecutionReceipt(receipt) = &mut fixture.object else {
            panic!("expected receipt fixture");
        };
        receipt.input_commitment = Some(Vec::new());
        assert_eq!(receipt.validate(), Err(ValidationError::EmptyRequiredValue));
    }
}
