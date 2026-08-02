#![forbid(unsafe_code)]

//! Atomic persistence and interruption model for Baochip research.

use baochip_model::{Command, Execution, LifecycleState, Rejection, StateMachine};

/// Abstract logical transaction phase.
///
/// These names describe selector authority in this executable model. They do
/// not assert completion by any physical storage technology.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistencePhase {
    Clean,
    Prepared {
        slot: usize,
        commit_id: u64,
    },
    Committed {
        previous_slot: usize,
        active_slot: usize,
        commit_id: u64,
    },
}

/// Outcome produced by the lifecycle model while preparing a record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandOutcome {
    Applied(Execution),
    Rejected(Rejection),
}

/// Persistence-level operation recorded for research inspection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceOperation {
    Prepared,
    RejectedWithoutChange,
    SelectorCommitted,
    PreviousRecordCleaned,
    RecoveredStable,
    RecoveredPrevious,
    RecoveredNext,
    RecoveredIntegrityPrevious,
    RecoveredIntegrityNext,
    RecoveredIntegritySoleValid,
}

/// Abstract integrity verdict supplied by a future record-authentication
/// layer.
///
/// This is deliberately not a checksum, digest, MAC, signature, or physical
/// media claim. Increment 3 models recovery policy after a verdict exists.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrityVerdict {
    Valid,
    Corrupted,
}

/// Non-secret persistence audit event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PersistenceAudit {
    pub operation: PersistenceOperation,
    pub prior_lifecycle: LifecycleState,
    pub resulting_lifecycle: LifecycleState,
    pub commit_id: u64,
}

/// Result of preparing a lifecycle command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrepareResult {
    Staged {
        audit: PersistenceAudit,
    },
    NotStaged {
        rejection: Rejection,
        audit: PersistenceAudit,
    },
}

/// Result released only after the abstract authoritative selector advances.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitResult {
    pub outcome: CommandOutcome,
    pub audit: PersistenceAudit,
}

/// Stable persistence-model failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceError {
    Busy,
    NoPreparedRecord,
    NoCommittedRecord,
    MissingActiveRecord,
    MissingCandidateRecord,
    MissingPreviousRecord,
    MissingPreparedOutcome,
    UnexpectedPreparedOutcome,
    UnexpectedRecord,
    InvalidSlotIndex,
    SlotConflict,
    SelectorMismatch,
    CommitIdMismatch,
    CommitIdExhausted,
    SuccessfulCommandWithoutDurableChange,
    CorruptedSelector,
    CorruptedActiveRecord,
    CorruptedCandidateRecord,
    CorruptedPreviousRecord,
    AmbiguousIntegrityRecovery,
    NoValidRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Record {
    commit_id: u64,
    state: StateMachine,
    integrity: IntegrityVerdict,
}

/// Two-slot durable state model with an abstract atomic selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableModel {
    slots: [Option<Record>; 2],
    active_slot: usize,
    phase: PersistencePhase,
    prepared_outcome: Option<CommandOutcome>,
    selector_integrity: IntegrityVerdict,
}

/// Typed integrity faults available only to this crate's tests.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IntegrityTestFault {
    ActiveRecord,
    InactiveRecord,
    Selector,
    AllRecords,
    DuplicateActiveRecord,
}

impl DurableModel {
    #[must_use]
    pub fn new(initial_state: StateMachine) -> Self {
        Self {
            slots: [
                Some(Record {
                    commit_id: 0,
                    state: initial_state,
                    integrity: IntegrityVerdict::Valid,
                }),
                None,
            ],
            active_slot: 0,
            phase: PersistencePhase::Clean,
            prepared_outcome: None,
            selector_integrity: IntegrityVerdict::Valid,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> PersistencePhase {
        self.phase
    }

    /// Return the currently authoritative lifecycle state.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::MissingActiveRecord`] if the selected slot
    /// has no complete record.
    pub fn active_state(&self) -> Result<&StateMachine, PersistenceError> {
        self.validate_internal_state()?;
        Ok(&self
            .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
            .state)
    }

    /// Return the currently authoritative logical commit identifier.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::MissingActiveRecord`] if the selected slot
    /// has no complete record.
    pub fn active_commit_id(&self) -> Result<u64, PersistenceError> {
        self.validate_internal_state()?;
        Ok(self
            .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
            .commit_id)
    }

    /// Execute a command against a clone and prepare any resulting durable
    /// state in the inactive slot.
    ///
    /// # Errors
    ///
    /// Returns an error if another transaction is active, the active record is
    /// missing, the commit identifier is exhausted, or a successful command
    /// unexpectedly produces no durable state change.
    pub fn prepare(&mut self, command: Command) -> Result<PrepareResult, PersistenceError> {
        self.validate_internal_state()?;
        if self.phase != PersistencePhase::Clean {
            return Err(PersistenceError::Busy);
        }

        let active = self
            .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
            .clone();
        let prior_lifecycle = active.state.lifecycle();
        let mut candidate = active.state.clone();
        let command_result = candidate.apply(command);
        let resulting_lifecycle = candidate.lifecycle();

        if candidate == active.state {
            return match command_result {
                Err(rejection) => Ok(PrepareResult::NotStaged {
                    rejection,
                    audit: PersistenceAudit {
                        operation: PersistenceOperation::RejectedWithoutChange,
                        prior_lifecycle,
                        resulting_lifecycle,
                        commit_id: active.commit_id,
                    },
                }),
                Ok(_) => Err(PersistenceError::SuccessfulCommandWithoutDurableChange),
            };
        }

        let commit_id = active
            .commit_id
            .checked_add(1)
            .ok_or(PersistenceError::CommitIdExhausted)?;
        let candidate_slot = self.inactive_slot();
        let outcome = match command_result {
            Ok(execution) => CommandOutcome::Applied(execution),
            Err(rejection) => CommandOutcome::Rejected(rejection),
        };
        self.slots[candidate_slot] = Some(Record {
            commit_id,
            state: candidate,
            integrity: IntegrityVerdict::Valid,
        });
        self.prepared_outcome = Some(outcome);
        self.phase = PersistencePhase::Prepared {
            slot: candidate_slot,
            commit_id,
        };

        Ok(PrepareResult::Staged {
            audit: PersistenceAudit {
                operation: PersistenceOperation::Prepared,
                prior_lifecycle,
                resulting_lifecycle,
                commit_id,
            },
        })
    }

    /// Atomically select the prepared record as authoritative.
    ///
    /// # Errors
    ///
    /// Returns an error unless a complete prepared record exists.
    pub fn commit(&mut self) -> Result<CommitResult, PersistenceError> {
        self.validate_internal_state()?;
        let PersistencePhase::Prepared { slot, commit_id } = self.phase else {
            return Err(PersistenceError::NoPreparedRecord);
        };
        let previous = self
            .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
            .state
            .lifecycle();
        let next = self
            .record_at(slot, PersistenceError::MissingCandidateRecord)?
            .state
            .lifecycle();
        let outcome = self
            .prepared_outcome
            .ok_or(PersistenceError::MissingPreparedOutcome)?;
        let previous_slot = self.active_slot;
        self.active_slot = slot;
        self.prepared_outcome = None;
        self.phase = PersistencePhase::Committed {
            previous_slot,
            active_slot: slot,
            commit_id,
        };

        Ok(CommitResult {
            outcome,
            audit: PersistenceAudit {
                operation: PersistenceOperation::SelectorCommitted,
                prior_lifecycle: previous,
                resulting_lifecycle: next,
                commit_id,
            },
        })
    }

    /// Erase the obsolete previous record and return to a clean state.
    ///
    /// # Errors
    ///
    /// Returns an error unless selector commit has completed.
    pub fn cleanup(&mut self) -> Result<PersistenceAudit, PersistenceError> {
        self.validate_internal_state()?;
        let PersistencePhase::Committed {
            previous_slot,
            commit_id,
            ..
        } = self.phase
        else {
            return Err(PersistenceError::NoCommittedRecord);
        };
        let previous_lifecycle = self
            .record_at(previous_slot, PersistenceError::MissingPreviousRecord)?
            .state
            .lifecycle();
        let lifecycle = self
            .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
            .state
            .lifecycle();
        self.slots[previous_slot] = None;
        self.phase = PersistencePhase::Clean;

        Ok(PersistenceAudit {
            operation: PersistenceOperation::PreviousRecordCleaned,
            prior_lifecycle: previous_lifecycle,
            resulting_lifecycle: lifecycle,
            commit_id,
        })
    }

    /// Simulate power loss and recover according to the authoritative selector.
    ///
    /// # Errors
    ///
    /// Returns an error if the selector identifies no complete active record.
    pub fn crash_and_recover(&mut self) -> Result<PersistenceAudit, PersistenceError> {
        if self.selector_integrity == IntegrityVerdict::Corrupted
            || self
                .slots
                .iter()
                .flatten()
                .any(|record| record.integrity == IntegrityVerdict::Corrupted)
        {
            return self.recover_integrity_failure();
        }
        self.validate_internal_state()?;
        let phase = self.phase;
        match phase {
            PersistencePhase::Clean => {
                let record =
                    self.record_at(self.active_slot, PersistenceError::MissingActiveRecord)?;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredStable,
                    prior_lifecycle: record.state.lifecycle(),
                    resulting_lifecycle: record.state.lifecycle(),
                    commit_id: record.commit_id,
                })
            }
            PersistencePhase::Prepared { slot, .. } => {
                let candidate_lifecycle = self
                    .record_at(slot, PersistenceError::MissingCandidateRecord)?
                    .state
                    .lifecycle();
                let active =
                    self.record_at(self.active_slot, PersistenceError::MissingActiveRecord)?;
                let active_lifecycle = active.state.lifecycle();
                let active_commit_id = active.commit_id;
                self.slots[slot] = None;
                self.prepared_outcome = None;
                self.phase = PersistencePhase::Clean;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredPrevious,
                    prior_lifecycle: candidate_lifecycle,
                    resulting_lifecycle: active_lifecycle,
                    commit_id: active_commit_id,
                })
            }
            PersistencePhase::Committed {
                previous_slot,
                commit_id,
                ..
            } => {
                let previous_lifecycle = self
                    .record_at(previous_slot, PersistenceError::MissingPreviousRecord)?
                    .state
                    .lifecycle();
                let active_lifecycle = self
                    .record_at(self.active_slot, PersistenceError::MissingActiveRecord)?
                    .state
                    .lifecycle();
                self.slots[previous_slot] = None;
                self.phase = PersistencePhase::Clean;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredNext,
                    prior_lifecycle: previous_lifecycle,
                    resulting_lifecycle: active_lifecycle,
                    commit_id,
                })
            }
        }
    }

    /// Recover after an abstract integrity verdict reports corrupted selector
    /// or record material.
    ///
    /// The plan is fully validated before mutation. Prepared candidates are
    /// never promoted, and committed selected records are never rolled back to
    /// an obsolete previous record.
    fn recover_integrity_failure(&mut self) -> Result<PersistenceAudit, PersistenceError> {
        match self.phase {
            PersistencePhase::Clean => {
                if self.prepared_outcome.is_some() {
                    return Err(PersistenceError::UnexpectedPreparedOutcome);
                }
                if self.selector_integrity == IntegrityVerdict::Valid {
                    self.validate_slot_index(self.active_slot)?;
                    let active =
                        self.record_at(self.active_slot, PersistenceError::MissingActiveRecord)?;
                    if active.integrity == IntegrityVerdict::Corrupted {
                        return Err(PersistenceError::CorruptedActiveRecord);
                    }
                    let inactive_slot = self.inactive_slot();
                    let Some(inactive) = self.slots[inactive_slot].as_ref() else {
                        return Err(PersistenceError::NoValidRecord);
                    };
                    if inactive.integrity == IntegrityVerdict::Valid {
                        return Err(PersistenceError::AmbiguousIntegrityRecovery);
                    }
                    let lifecycle = active.state.lifecycle();
                    let commit_id = active.commit_id;
                    self.slots[inactive_slot] = None;
                    return Ok(PersistenceAudit {
                        operation: PersistenceOperation::RecoveredIntegritySoleValid,
                        prior_lifecycle: lifecycle,
                        resulting_lifecycle: lifecycle,
                        commit_id,
                    });
                }

                let valid_slots: Vec<usize> = self
                    .slots
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, record)| {
                        record
                            .as_ref()
                            .filter(|record| record.integrity == IntegrityVerdict::Valid)
                            .map(|_| slot)
                    })
                    .collect();
                let [selected] = valid_slots.as_slice() else {
                    return if valid_slots.is_empty() {
                        Err(PersistenceError::NoValidRecord)
                    } else {
                        Err(PersistenceError::AmbiguousIntegrityRecovery)
                    };
                };
                let selected = *selected;
                let record = self.record_at(selected, PersistenceError::MissingActiveRecord)?;
                let lifecycle = record.state.lifecycle();
                let commit_id = record.commit_id;
                let obsolete = 1 - selected;
                self.slots[obsolete] = None;
                self.active_slot = selected;
                self.selector_integrity = IntegrityVerdict::Valid;
                self.prepared_outcome = None;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredIntegritySoleValid,
                    prior_lifecycle: lifecycle,
                    resulting_lifecycle: lifecycle,
                    commit_id,
                })
            }
            PersistencePhase::Prepared { slot, commit_id } => {
                self.validate_slot_index(slot)?;
                if self.prepared_outcome.is_none() {
                    return Err(PersistenceError::MissingPreparedOutcome);
                }
                let previous_slot = 1 - slot;
                if self.selector_integrity == IntegrityVerdict::Valid
                    && self.active_slot != previous_slot
                {
                    return Err(PersistenceError::SelectorMismatch);
                }
                let previous =
                    self.record_at(previous_slot, PersistenceError::MissingActiveRecord)?;
                if previous.integrity == IntegrityVerdict::Corrupted {
                    return Err(PersistenceError::CorruptedActiveRecord);
                }
                let expected = previous
                    .commit_id
                    .checked_add(1)
                    .ok_or(PersistenceError::CommitIdMismatch)?;
                if commit_id != expected {
                    return Err(PersistenceError::CommitIdMismatch);
                }
                let previous_lifecycle = previous.state.lifecycle();
                let previous_commit_id = previous.commit_id;
                let mut prior_lifecycle = previous_lifecycle;
                if let Some(candidate) = self.slots[slot].as_ref() {
                    if candidate.integrity == IntegrityVerdict::Valid {
                        if candidate.commit_id != commit_id {
                            return Err(PersistenceError::CommitIdMismatch);
                        }
                        prior_lifecycle = candidate.state.lifecycle();
                    }
                }
                self.slots[slot] = None;
                self.active_slot = previous_slot;
                self.selector_integrity = IntegrityVerdict::Valid;
                self.prepared_outcome = None;
                self.phase = PersistencePhase::Clean;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredIntegrityPrevious,
                    prior_lifecycle,
                    resulting_lifecycle: previous_lifecycle,
                    commit_id: previous_commit_id,
                })
            }
            PersistencePhase::Committed {
                previous_slot,
                active_slot,
                commit_id,
            } => {
                self.validate_slot_index(previous_slot)?;
                self.validate_slot_index(active_slot)?;
                if previous_slot == active_slot {
                    return Err(PersistenceError::SlotConflict);
                }
                if self.prepared_outcome.is_some() {
                    return Err(PersistenceError::UnexpectedPreparedOutcome);
                }
                if self.selector_integrity == IntegrityVerdict::Valid
                    && self.active_slot != active_slot
                {
                    return Err(PersistenceError::SelectorMismatch);
                }
                let previous =
                    self.record_at(previous_slot, PersistenceError::MissingPreviousRecord)?;
                let selected =
                    self.record_at(active_slot, PersistenceError::MissingActiveRecord)?;
                if selected.integrity == IntegrityVerdict::Corrupted {
                    return Err(PersistenceError::CorruptedActiveRecord);
                }
                if selected.commit_id != commit_id {
                    return Err(PersistenceError::CommitIdMismatch);
                }
                let selected_lifecycle = selected.state.lifecycle();
                let selected_commit_id = selected.commit_id;
                let prior_lifecycle = if previous.integrity == IntegrityVerdict::Valid {
                    let expected = previous
                        .commit_id
                        .checked_add(1)
                        .ok_or(PersistenceError::CommitIdMismatch)?;
                    if selected.commit_id != expected {
                        return Err(PersistenceError::CommitIdMismatch);
                    }
                    previous.state.lifecycle()
                } else {
                    selected_lifecycle
                };
                self.slots[previous_slot] = None;
                self.active_slot = active_slot;
                self.selector_integrity = IntegrityVerdict::Valid;
                self.prepared_outcome = None;
                self.phase = PersistencePhase::Clean;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredIntegrityNext,
                    prior_lifecycle,
                    resulting_lifecycle: selected_lifecycle,
                    commit_id: selected_commit_id,
                })
            }
        }
    }

    fn validate_internal_state(&self) -> Result<(), PersistenceError> {
        self.validate_slot_index(self.active_slot)?;
        if self.selector_integrity == IntegrityVerdict::Corrupted {
            return Err(PersistenceError::CorruptedSelector);
        }
        let active = self.record_at(self.active_slot, PersistenceError::MissingActiveRecord)?;
        if active.integrity == IntegrityVerdict::Corrupted {
            return Err(PersistenceError::CorruptedActiveRecord);
        }

        match self.phase {
            PersistencePhase::Clean => {
                if self.prepared_outcome.is_some() {
                    return Err(PersistenceError::UnexpectedPreparedOutcome);
                }
                if self.slots[self.inactive_slot()].is_some() {
                    return Err(PersistenceError::UnexpectedRecord);
                }
            }
            PersistencePhase::Prepared { slot, commit_id } => {
                self.validate_slot_index(slot)?;
                if slot == self.active_slot {
                    return Err(PersistenceError::SlotConflict);
                }
                let candidate = self.record_at(slot, PersistenceError::MissingCandidateRecord)?;
                if candidate.integrity == IntegrityVerdict::Corrupted {
                    return Err(PersistenceError::CorruptedCandidateRecord);
                }
                if self.prepared_outcome.is_none() {
                    return Err(PersistenceError::MissingPreparedOutcome);
                }
                let expected_commit_id = active
                    .commit_id
                    .checked_add(1)
                    .ok_or(PersistenceError::CommitIdMismatch)?;
                if commit_id != candidate.commit_id || commit_id != expected_commit_id {
                    return Err(PersistenceError::CommitIdMismatch);
                }
            }
            PersistencePhase::Committed {
                previous_slot,
                active_slot,
                commit_id,
            } => {
                self.validate_slot_index(previous_slot)?;
                self.validate_slot_index(active_slot)?;
                if active_slot != self.active_slot {
                    return Err(PersistenceError::SelectorMismatch);
                }
                if previous_slot == active_slot {
                    return Err(PersistenceError::SlotConflict);
                }
                if self.prepared_outcome.is_some() {
                    return Err(PersistenceError::UnexpectedPreparedOutcome);
                }
                let previous =
                    self.record_at(previous_slot, PersistenceError::MissingPreviousRecord)?;
                if previous.integrity == IntegrityVerdict::Corrupted {
                    return Err(PersistenceError::CorruptedPreviousRecord);
                }
                let expected_commit_id = previous
                    .commit_id
                    .checked_add(1)
                    .ok_or(PersistenceError::CommitIdMismatch)?;
                if commit_id != active.commit_id || commit_id != expected_commit_id {
                    return Err(PersistenceError::CommitIdMismatch);
                }
            }
        }
        Ok(())
    }

    fn record_at(
        &self,
        slot: usize,
        missing: PersistenceError,
    ) -> Result<&Record, PersistenceError> {
        self.validate_slot_index(slot)?;
        self.slots[slot].as_ref().ok_or(missing)
    }

    const fn validate_slot_index(&self, slot: usize) -> Result<(), PersistenceError> {
        if slot < self.slots.len() {
            Ok(())
        } else {
            Err(PersistenceError::InvalidSlotIndex)
        }
    }

    const fn inactive_slot(&self) -> usize {
        1 - self.active_slot
    }

    #[cfg(test)]
    pub(crate) fn inject_test_fault(
        &mut self,
        fault: IntegrityTestFault,
    ) -> Result<(), PersistenceError> {
        match fault {
            IntegrityTestFault::ActiveRecord => {
                self.validate_slot_index(self.active_slot)?;
                self.slots[self.active_slot]
                    .as_mut()
                    .ok_or(PersistenceError::MissingActiveRecord)?
                    .integrity = IntegrityVerdict::Corrupted;
            }
            IntegrityTestFault::InactiveRecord => {
                self.validate_slot_index(self.active_slot)?;
                let inactive = self.inactive_slot();
                self.slots[inactive]
                    .as_mut()
                    .ok_or(PersistenceError::MissingCandidateRecord)?
                    .integrity = IntegrityVerdict::Corrupted;
            }
            IntegrityTestFault::Selector => {
                self.selector_integrity = IntegrityVerdict::Corrupted;
            }
            IntegrityTestFault::AllRecords => {
                for record in self.slots.iter_mut().flatten() {
                    record.integrity = IntegrityVerdict::Corrupted;
                }
            }
            IntegrityTestFault::DuplicateActiveRecord => {
                self.validate_slot_index(self.active_slot)?;
                let duplicate = self.slots[self.active_slot]
                    .clone()
                    .ok_or(PersistenceError::MissingActiveRecord)?;
                let inactive = self.inactive_slot();
                self.slots[inactive] = Some(duplicate);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod adversarial_tests;

#[cfg(test)]
mod integrity_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use baochip_model::Authorizations;

    fn begin_provisioning() -> Command {
        Command::BeginProvisioning {
            authorizations: Authorizations {
                root: true,
                physical_presence: true,
                ..Authorizations::none()
            },
        }
    }

    fn decommission() -> Command {
        Command::Decommission {
            authorizations: Authorizations {
                decommission: true,
                independent: true,
                ..Authorizations::none()
            },
        }
    }

    #[test]
    fn crash_after_prepare_recovers_exact_previous_state() {
        let initial = StateMachine::new();
        let mut durable = DurableModel::new(initial.clone());
        durable.prepare(begin_provisioning()).expect("prepare");

        let audit = durable.crash_and_recover().expect("recover previous");
        assert_eq!(audit.operation, PersistenceOperation::RecoveredPrevious);
        assert_eq!(durable.active_state(), Ok(&initial));
        assert_eq!(durable.phase(), PersistencePhase::Clean);
    }

    #[test]
    fn crash_after_commit_recovers_exact_next_state() {
        let mut expected = StateMachine::new();
        expected
            .apply(begin_provisioning())
            .expect("expected state");
        let mut durable = DurableModel::new(StateMachine::new());
        durable.prepare(begin_provisioning()).expect("prepare");
        durable.commit().expect("commit selector");

        let audit = durable.crash_and_recover().expect("recover next");
        assert_eq!(audit.operation, PersistenceOperation::RecoveredNext);
        assert_eq!(durable.active_state(), Ok(&expected));
        assert_eq!(durable.phase(), PersistencePhase::Clean);
    }

    #[test]
    fn cleanup_preserves_selected_next_state() {
        let mut durable = DurableModel::new(StateMachine::new());
        durable.prepare(begin_provisioning()).expect("prepare");
        durable.commit().expect("commit selector");
        durable.cleanup().expect("clean previous");

        assert_eq!(
            durable.active_state().expect("active").lifecycle(),
            LifecycleState::Provisioning
        );
        assert_eq!(durable.active_commit_id(), Ok(1));
    }

    #[test]
    fn unchanged_rejection_creates_no_candidate() {
        let mut durable = DurableModel::new(StateMachine::new());
        let result = durable
            .prepare(Command::IssueReceipt { challenge: None })
            .expect("rejection is modeled");

        assert!(matches!(
            result,
            PrepareResult::NotStaged {
                rejection: Rejection::InvalidState,
                ..
            }
        ));
        assert_eq!(durable.phase(), PersistencePhase::Clean);
        assert_eq!(durable.active_commit_id(), Ok(0));
    }

    #[test]
    fn phase_ordering_fails_closed() {
        let mut durable = DurableModel::new(StateMachine::new());
        assert_eq!(durable.commit(), Err(PersistenceError::NoPreparedRecord));
        assert_eq!(durable.cleanup(), Err(PersistenceError::NoCommittedRecord));

        durable.prepare(begin_provisioning()).expect("prepare");
        assert_eq!(durable.prepare(decommission()), Err(PersistenceError::Busy));
        assert_eq!(durable.cleanup(), Err(PersistenceError::NoCommittedRecord));
    }

    #[test]
    fn commit_identifiers_advance_across_transactions() {
        let mut durable = DurableModel::new(StateMachine::new());
        durable.prepare(begin_provisioning()).expect("prepare one");
        durable.commit().expect("commit one");
        durable.cleanup().expect("clean one");
        assert_eq!(durable.active_commit_id(), Ok(1));

        durable.prepare(decommission()).expect("prepare two");
        durable.commit().expect("commit two");
        durable.cleanup().expect("clean two");
        assert_eq!(durable.active_commit_id(), Ok(2));
        assert_eq!(
            durable.active_state().expect("active").lifecycle(),
            LifecycleState::Decommissioned
        );
    }

    #[test]
    fn stable_crash_preserves_the_authoritative_record() {
        let initial = StateMachine::new();
        let mut durable = DurableModel::new(initial.clone());

        let audit = durable.crash_and_recover().expect("stable recovery");

        assert_eq!(audit.operation, PersistenceOperation::RecoveredStable);
        assert_eq!(durable.active_state(), Ok(&initial));
        assert_eq!(durable.active_commit_id(), Ok(0));
        assert_eq!(durable.phase(), PersistencePhase::Clean);
    }

    #[test]
    fn commit_id_exhaustion_does_not_stage_or_mutate_state() {
        let mut durable = DurableModel::new(StateMachine::new());
        durable.slots[0].as_mut().expect("initial record").commit_id = u64::MAX;
        let before = durable.clone();

        assert_eq!(
            durable.prepare(begin_provisioning()),
            Err(PersistenceError::CommitIdExhausted)
        );
        assert_eq!(durable, before);
    }
}
