use super::*;

fn fixture(identifier: &str) -> Fixture {
    positive_fixtures()
        .into_iter()
        .find(|fixture| fixture.identifier == identifier)
        .expect("fixture must exist")
}

#[test]
fn receipt_requires_consistent_lineage_context() {
    let mut fixture = fixture("receipt-minimal-optionals-absent");
    let SemanticObject::ExecutionReceipt(receipt) = &mut fixture.object else {
        panic!("expected receipt fixture");
    };
    receipt.lineage = ReceiptLineageContext::ProvisioningGeneration(2);

    assert_eq!(receipt.validate(), Err(ValidationError::InconsistentState));
}

#[test]
fn persistent_state_rejects_impossible_identity_eligibility() {
    let mut fixture = fixture("persistent-blank-absent-key-generation");
    let SemanticObject::PersistentState(state) = &mut fixture.object else {
        panic!("expected persistent-state fixture");
    };
    state.identity_active = true;

    assert_eq!(state.validate(), Err(ValidationError::InconsistentState));
}

#[test]
fn prepared_non_operational_execution_cannot_carry_a_receipt() {
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
        lifecycle_state: LifecycleState::UpdatePending,
        device_generation: 1,
        transition_counter: 2,
        measurement_epoch: 0,
        receipt_sequence: 1,
        active_version: 1,
        challenge: None,
    });

    assert_eq!(
        authority.validate(),
        Err(ValidationError::InconsistentExecution)
    );
}

#[test]
fn authority_phase_rejects_zero_commit_identifier() {
    let mut fixture = fixture("authority-prepared-applied");
    let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
        panic!("expected authority fixture");
    };
    let AuthorityPhaseProjection::Prepared { commit_id, .. } = &mut authority.phase else {
        panic!("expected prepared phase");
    };
    *commit_id = 0;

    assert_eq!(authority.validate(), Err(ValidationError::CommitIdMismatch));
}

#[test]
fn authority_commit_identifiers_bind_both_records_without_wrap() {
    let mut fixture = fixture("authority-prepared-applied");
    let SemanticObject::AuthorityMetadata(authority) = &mut fixture.object else {
        panic!("expected authority fixture");
    };
    authority.record_commit_ids[1] = Some(3);
    assert_eq!(authority.validate(), Err(ValidationError::CommitIdMismatch));

    authority.record_commit_ids = [Some(u64::MAX), Some(0)];
    let AuthorityPhaseProjection::Prepared { commit_id, .. } = &mut authority.phase else {
        panic!("expected prepared phase");
    };
    *commit_id = 0;
    assert_eq!(authority.validate(), Err(ValidationError::CommitIdMismatch));
}

#[test]
fn receipt_release_binds_committed_authority_and_subject() {
    let receipt = fixture("receipt-minimal-optionals-absent");
    let SemanticObject::ExecutionReceipt(receipt) = receipt.object else {
        panic!("expected receipt fixture");
    };
    let authority = fixture("authority-committed");
    let SemanticObject::AuthorityMetadata(mut authority) = authority.object else {
        panic!("expected authority fixture");
    };
    assert_eq!(receipt.validate_release_authority(&authority), Ok(()));

    let mut state = fixture("persistent-operational-receipt-release");
    let SemanticObject::PersistentState(state) = &mut state.object else {
        panic!("expected persistent-state fixture");
    };
    assert_eq!(receipt.validate_release(&authority, state), Ok(()));

    state.measurement_epoch += 1;
    assert_eq!(
        receipt.validate_release(&authority, state),
        Err(ValidationError::StateContextMismatch)
    );
    state.measurement_epoch = receipt.measurement_epoch;
    state.slot_id = 0;
    assert_eq!(
        receipt.validate_release(&authority, state),
        Err(ValidationError::StateContextMismatch)
    );

    authority.context.subject.device_generation = 2;
    assert_eq!(
        receipt.validate_release_authority(&authority),
        Err(ValidationError::AuthorityContextMismatch)
    );

    authority.context.subject = receipt.context.subject.clone();
    authority.phase = AuthorityPhaseProjection::Clean;
    authority.record_commit_ids = [None, Some(4)];
    assert_eq!(
        receipt.validate_release_authority(&authority),
        Err(ValidationError::AuthorityPhaseMismatch)
    );
}

#[test]
fn receipt_release_binds_complete_shared_protected_context() {
    let SemanticObject::ExecutionReceipt(receipt) =
        fixture("receipt-minimal-optionals-absent").object
    else {
        panic!("expected receipt fixture");
    };
    let SemanticObject::AuthorityMetadata(authority) = fixture("authority-committed").object else {
        panic!("expected authority fixture");
    };
    let SemanticObject::PersistentState(state) =
        fixture("persistent-operational-receipt-release").object
    else {
        panic!("expected persistent-state fixture");
    };

    let mut mutations = Vec::new();
    let mut profile = receipt.clone();
    profile.context.profile_identifier.push_str(".other");
    mutations.push(profile);
    let mut schema = receipt.clone();
    schema.context.schema_version += 1;
    mutations.push(schema);
    let mut suite = receipt.clone();
    suite.context.integrity_suite_identifier.push_str(".other");
    mutations.push(suite);
    let mut extensions = receipt;
    extensions.context.extensions[0].critical = true;
    mutations.push(extensions);

    for mutation in mutations {
        assert_eq!(mutation.validate(), Ok(()));
        assert_eq!(authority.validate(), Ok(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(
            mutation.validate_release_authority(&authority),
            Err(ValidationError::AuthorityContextMismatch)
        );
        assert_eq!(
            mutation.validate_authoritative_state(&state),
            Err(ValidationError::StateContextMismatch)
        );
        assert_eq!(
            mutation.validate_release(&authority, &state),
            Err(ValidationError::AuthorityContextMismatch)
        );
    }
}

#[test]
fn non_operational_snapshot_cannot_release_a_receipt() {
    let SemanticObject::ExecutionReceipt(mut receipt) =
        fixture("receipt-minimal-optionals-absent").object
    else {
        panic!("expected receipt fixture");
    };
    let SemanticObject::AuthorityMetadata(authority) = fixture("authority-committed").object else {
        panic!("expected authority fixture");
    };
    let SemanticObject::PersistentState(mut state) =
        fixture("persistent-operational-receipt-release").object
    else {
        panic!("expected persistent-state fixture");
    };
    receipt.lifecycle_state = LifecycleState::Recovery;
    state.lifecycle_state = LifecycleState::Recovery;

    assert_eq!(receipt.validate(), Ok(()));
    assert_eq!(authority.validate(), Ok(()));
    assert_eq!(state.validate(), Ok(()));
    assert_eq!(
        receipt.validate_release(&authority, &state),
        Err(ValidationError::StateContextMismatch)
    );
}

#[test]
fn persistent_single_field_mutations_fail_closed() {
    let mut update = fixture("persistent-update-pending");
    let SemanticObject::PersistentState(update) = &mut update.object else {
        panic!("expected persistent-state fixture");
    };
    update.pending_version = None;
    assert_eq!(update.validate(), Err(ValidationError::InconsistentState));

    let mut update = fixture("persistent-update-pending");
    let SemanticObject::PersistentState(update) = &mut update.object else {
        panic!("expected persistent-state fixture");
    };
    update.pending_version = Some(update.active_version);
    assert_eq!(update.validate(), Err(ValidationError::InconsistentState));

    let mut provisioning = fixture("persistent-provisioning-recommission");
    let SemanticObject::PersistentState(provisioning) = &mut provisioning.object else {
        panic!("expected persistent-state fixture");
    };
    provisioning.provisioning_generation = Some(provisioning.device_generation);
    assert_eq!(
        provisioning.validate(),
        Err(ValidationError::InconsistentState)
    );

    let mut operational = fixture("persistent-operational-u64-boundaries");
    let SemanticObject::PersistentState(operational) = &mut operational.object else {
        panic!("expected persistent-state fixture");
    };
    operational.slot_id = 2;
    assert_eq!(operational.validate(), Err(ValidationError::InvalidSlot));
}

#[test]
fn required_receipt_values_fail_closed_one_field_at_a_time() {
    let receipt = fixture("receipt-minimal-optionals-absent");
    let SemanticObject::ExecutionReceipt(receipt) = receipt.object else {
        panic!("expected receipt fixture");
    };

    let mut mutation = receipt.clone();
    mutation.key_identifier.clear();
    assert_eq!(
        mutation.validate(),
        Err(ValidationError::EmptyRequiredValue)
    );

    let mut mutation = receipt.clone();
    mutation.measurement_root.clear();
    assert_eq!(
        mutation.validate(),
        Err(ValidationError::EmptyRequiredValue)
    );

    let mut mutation = receipt.clone();
    mutation.measurement_context.clear();
    assert_eq!(
        mutation.validate(),
        Err(ValidationError::EmptyRequiredValue)
    );

    let mut mutation = receipt.clone();
    mutation.policy_identifier.clear();
    assert_eq!(
        mutation.validate(),
        Err(ValidationError::EmptyRequiredValue)
    );

    let mut mutation = receipt;
    mutation.output_commitment = Some(Vec::new());
    assert_eq!(
        mutation.validate(),
        Err(ValidationError::EmptyRequiredValue)
    );
}

#[test]
fn security_context_mutations_produce_distinct_valid_receipts() {
    let baseline = fixture("receipt-minimal-optionals-absent");
    let SemanticObject::ExecutionReceipt(baseline) = baseline.object else {
        panic!("expected receipt fixture");
    };

    let mut mutations = Vec::new();
    let mut profile = baseline.clone();
    profile.context.profile_identifier.push_str(".other");
    mutations.push(profile);
    let mut schema = baseline.clone();
    schema.context.schema_version = 2;
    mutations.push(schema);
    let mut suite = baseline.clone();
    suite.context.integrity_suite_identifier.push_str(".other");
    mutations.push(suite);
    let mut subject = baseline.clone();
    subject.context.subject.device_identifier.push(0x01);
    mutations.push(subject);
    let mut generation = baseline.clone();
    generation.context.subject.device_generation = 2;
    generation.device_generation = 2;
    generation.lineage = ReceiptLineageContext::ProvisioningGeneration(2);
    mutations.push(generation);
    let mut key_generation = baseline.clone();
    key_generation.context.subject.key_generation = Some(2);
    key_generation.lineage = ReceiptLineageContext::KeyGeneration(2);
    mutations.push(key_generation);
    let mut lifecycle = baseline.clone();
    lifecycle.lifecycle_state = LifecycleState::Recovery;
    mutations.push(lifecycle);
    let mut extension_type = baseline.clone();
    extension_type.context.extensions[0].value = ExtensionValue::Text(String::from("00ff"));
    mutations.push(extension_type);

    for mutation in mutations {
        assert_eq!(mutation.validate(), Ok(()));
        assert_ne!(mutation, baseline);
    }
}

#[test]
fn positive_corpus_objects_and_labels_are_unique() {
    let fixtures = positive_fixtures();
    for (index, fixture) in fixtures.iter().enumerate() {
        for prior in &fixtures[..index] {
            assert_ne!(fixture.identifier, prior.identifier);
            assert_ne!(fixture.object, prior.object);
        }
    }
}

#[test]
fn positive_corpus_pins_documented_authority_and_receipt_coverage() {
    let fixtures = positive_fixtures();
    let mut clean = false;
    let mut prepared_applied = false;
    let mut prepared_rejected = false;
    let mut committed = false;
    let mut receipt_absent = false;
    let mut receipt_present = false;
    let mut persistent_slots = [false; 2];
    let mut identity_values = [false; 2];
    let mut extension_types = [false; 4];
    let mut extension_criticality = [false; 2];
    let mut lineage_modes = [false; 2];

    for fixture in fixtures {
        for extension in match &fixture.object {
            SemanticObject::PersistentState(state) => &state.context.extensions,
            SemanticObject::AuthorityMetadata(authority) => &authority.context.extensions,
            SemanticObject::ExecutionReceipt(receipt) => &receipt.context.extensions,
        } {
            extension_criticality[usize::from(extension.critical)] = true;
            extension_types[match extension.value {
                ExtensionValue::Unsigned(_) => 0,
                ExtensionValue::Boolean(_) => 1,
                ExtensionValue::Bytes(_) => 2,
                ExtensionValue::Text(_) => 3,
            }] = true;
        }
        match fixture.object {
            SemanticObject::AuthorityMetadata(authority) => match authority.phase {
                AuthorityPhaseProjection::Clean => clean = true,
                AuthorityPhaseProjection::Prepared {
                    prepared_outcome: PreparedOutcomeProjection::Applied(_),
                    ..
                } => prepared_applied = true,
                AuthorityPhaseProjection::Prepared {
                    prepared_outcome: PreparedOutcomeProjection::Rejected(_),
                    ..
                } => prepared_rejected = true,
                AuthorityPhaseProjection::Committed { .. } => committed = true,
            },
            SemanticObject::ExecutionReceipt(receipt) => {
                lineage_modes[match receipt.lineage {
                    ReceiptLineageContext::KeyGeneration(_) => 0,
                    ReceiptLineageContext::ProvisioningGeneration(_) => 1,
                }] = true;
                let optionals_present = receipt.receipt_sequence.is_some()
                    && receipt.challenge.is_some()
                    && receipt.input_commitment.is_some()
                    && receipt.output_commitment.is_some();
                let optionals_absent = receipt.receipt_sequence.is_none()
                    && receipt.challenge.is_none()
                    && receipt.input_commitment.is_none()
                    && receipt.output_commitment.is_none();
                receipt_present |= optionals_present;
                receipt_absent |= optionals_absent;
            }
            SemanticObject::PersistentState(state) => {
                persistent_slots[usize::from(state.slot_id)] = true;
                identity_values[usize::from(state.identity_active)] = true;
            }
        }
    }

    assert!(clean && prepared_applied && prepared_rejected && committed);
    assert!(receipt_absent && receipt_present);
    assert!(persistent_slots.into_iter().all(|covered| covered));
    assert!(identity_values.into_iter().all(|covered| covered));
    assert!(extension_types.into_iter().all(|covered| covered));
    assert!(extension_criticality.into_iter().all(|covered| covered));
    assert!(lineage_modes.into_iter().all(|covered| covered));
}

#[test]
fn advertised_error_classes_fail_closed() {
    let mut persistent = fixture("persistent-blank-absent-key-generation");
    let SemanticObject::PersistentState(state) = &mut persistent.object else {
        panic!("expected persistent-state fixture");
    };
    state.context.profile_identifier.clear();
    assert_eq!(state.validate(), Err(ValidationError::EmptyIdentifier));

    let mut persistent = fixture("persistent-blank-absent-key-generation");
    let SemanticObject::PersistentState(state) = &mut persistent.object else {
        panic!("expected persistent-state fixture");
    };
    state.context.subject.device_identifier.clear();
    assert_eq!(state.validate(), Err(ValidationError::EmptySubject));

    let mut prepared = fixture("authority-prepared-applied");
    let SemanticObject::AuthorityMetadata(authority) = &mut prepared.object else {
        panic!("expected authority fixture");
    };
    authority.record_commit_ids[1] = None;
    assert_eq!(authority.validate(), Err(ValidationError::MissingRecord));

    let mut committed = fixture("authority-committed");
    let SemanticObject::AuthorityMetadata(authority) = &mut committed.object else {
        panic!("expected authority fixture");
    };
    let AuthorityPhaseProjection::Committed {
        selected_next_slot,
        previous_slot,
        ..
    } = &mut authority.phase
    else {
        panic!("expected committed phase");
    };
    *previous_slot = *selected_next_slot;
    assert_eq!(authority.validate(), Err(ValidationError::SlotConflict));
}
