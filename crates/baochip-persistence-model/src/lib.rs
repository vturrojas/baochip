#![forbid(unsafe_code)]

//! Atomic persistence and interruption model for Baochip research.

use baochip_model::{Command, Execution, LifecycleState, Rejection, StateMachine};

/// Durable transaction phase.
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
        outcome: CommandOutcome,
        audit: PersistenceAudit,
    },
    NotStaged {
        rejection: Rejection,
        audit: PersistenceAudit,
    },
}

/// Stable persistence-model failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistenceError {
    Busy,
    NoPreparedRecord,
    NoCommittedRecord,
    MissingActiveRecord,
    MissingCandidateRecord,
    CommitIdExhausted,
    SuccessfulCommandWithoutDurableChange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Record {
    commit_id: u64,
    state: StateMachine,
}

/// Two-slot durable state model with an abstract atomic selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableModel {
    slots: [Option<Record>; 2],
    active_slot: usize,
    phase: PersistencePhase,
}

impl DurableModel {
    #[must_use]
    pub fn new(initial_state: StateMachine) -> Self {
        Self {
            slots: [
                Some(Record {
                    commit_id: 0,
                    state: initial_state,
                }),
                None,
            ],
            active_slot: 0,
            phase: PersistencePhase::Clean,
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
        Ok(&self.active_record()?.state)
    }

    /// Return the currently authoritative logical commit identifier.
    ///
    /// # Errors
    ///
    /// Returns [`PersistenceError::MissingActiveRecord`] if the selected slot
    /// has no complete record.
    pub fn active_commit_id(&self) -> Result<u64, PersistenceError> {
        Ok(self.active_record()?.commit_id)
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
        if self.phase != PersistencePhase::Clean {
            return Err(PersistenceError::Busy);
        }

        let active = self.active_record()?.clone();
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
        self.slots[candidate_slot] = Some(Record {
            commit_id,
            state: candidate,
        });
        self.phase = PersistencePhase::Prepared {
            slot: candidate_slot,
            commit_id,
        };

        let outcome = match command_result {
            Ok(execution) => CommandOutcome::Applied(execution),
            Err(rejection) => CommandOutcome::Rejected(rejection),
        };
        Ok(PrepareResult::Staged {
            outcome,
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
    pub fn commit(&mut self) -> Result<PersistenceAudit, PersistenceError> {
        let PersistencePhase::Prepared { slot, commit_id } = self.phase else {
            return Err(PersistenceError::NoPreparedRecord);
        };
        let previous = self.active_record()?.state.lifecycle();
        let next = self.slots[slot]
            .as_ref()
            .ok_or(PersistenceError::MissingCandidateRecord)?
            .state
            .lifecycle();
        let previous_slot = self.active_slot;
        self.active_slot = slot;
        self.phase = PersistencePhase::Committed {
            previous_slot,
            active_slot: slot,
            commit_id,
        };

        Ok(PersistenceAudit {
            operation: PersistenceOperation::SelectorCommitted,
            prior_lifecycle: previous,
            resulting_lifecycle: next,
            commit_id,
        })
    }

    /// Erase the obsolete previous record and return to a clean state.
    ///
    /// # Errors
    ///
    /// Returns an error unless selector commit has completed.
    pub fn cleanup(&mut self) -> Result<PersistenceAudit, PersistenceError> {
        let PersistencePhase::Committed {
            previous_slot,
            commit_id,
            ..
        } = self.phase
        else {
            return Err(PersistenceError::NoCommittedRecord);
        };
        let lifecycle = self.active_record()?.state.lifecycle();
        self.slots[previous_slot] = None;
        self.phase = PersistencePhase::Clean;

        Ok(PersistenceAudit {
            operation: PersistenceOperation::PreviousRecordCleaned,
            prior_lifecycle: lifecycle,
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
        let phase = self.phase;
        match phase {
            PersistencePhase::Clean => {
                let record = self.active_record()?;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredStable,
                    prior_lifecycle: record.state.lifecycle(),
                    resulting_lifecycle: record.state.lifecycle(),
                    commit_id: record.commit_id,
                })
            }
            PersistencePhase::Prepared { slot, .. } => {
                let candidate_lifecycle = self.slots[slot]
                    .as_ref()
                    .ok_or(PersistenceError::MissingCandidateRecord)?
                    .state
                    .lifecycle();
                self.slots[slot] = None;
                self.phase = PersistencePhase::Clean;
                let active = self.active_record()?;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredPrevious,
                    prior_lifecycle: candidate_lifecycle,
                    resulting_lifecycle: active.state.lifecycle(),
                    commit_id: active.commit_id,
                })
            }
            PersistencePhase::Committed {
                previous_slot,
                commit_id,
                ..
            } => {
                let previous_lifecycle = self.slots[previous_slot]
                    .as_ref()
                    .ok_or(PersistenceError::MissingActiveRecord)?
                    .state
                    .lifecycle();
                self.slots[previous_slot] = None;
                self.phase = PersistencePhase::Clean;
                let active = self.active_record()?;
                Ok(PersistenceAudit {
                    operation: PersistenceOperation::RecoveredNext,
                    prior_lifecycle: previous_lifecycle,
                    resulting_lifecycle: active.state.lifecycle(),
                    commit_id,
                })
            }
        }
    }

    fn active_record(&self) -> Result<&Record, PersistenceError> {
        self.slots[self.active_slot]
            .as_ref()
            .ok_or(PersistenceError::MissingActiveRecord)
    }

    const fn inactive_slot(&self) -> usize {
        1 - self.active_slot
    }
}

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
