use super::*;

#[derive(Default)]
struct ChangingFs {
    reads: Cell<usize>,
    bytes: RefCell<Vec<u8>>,
    canonical: RefCell<PathBuf>,
    listing: RefCell<Vec<(String, bool)>>,
}

impl FileSystem for ChangingFs {
    fn read(&self, _: &Path) -> Option<Vec<u8>> {
        self.reads.set(self.reads.get() + 1);
        Some(self.bytes.borrow().clone())
    }

    fn is_file(&self, _: &Path) -> bool {
        true
    }

    fn is_dir(&self, _: &Path) -> bool {
        false
    }

    fn canonical(&self, _: &Path) -> PathBuf {
        self.canonical.borrow().clone()
    }

    fn list_dir(&self, _: &Path) -> Vec<(String, bool)> {
        self.listing.borrow().clone()
    }
}

#[test]
fn optional_hint_pressure_cannot_poison_required_semantic_inputs() {
    let native = Rc::new(ChangingFs::default());
    *native.listing.borrow_mut() = (0..22_000)
        .map(|index| (format!("{index:05}{}", "x".repeat(180)), false))
        .collect();
    let parent = Rc::new(InputEpochFs::new(native));
    let _ = parent.list_dir(Path::new("semantic-first"));
    let hints = parent
        .hint_fork()
        .expect("the initial semantic+hint prefix fits");
    let _ = hints.list_dir(Path::new("hint-only"));
    assert!(parent.retained_bytes() <= MAX_BYTES);
    let _ = parent.list_dir(Path::new("semantic-later"));
    assert!(
        parent.seal_after_validation(),
        "required semantic inputs fit by themselves; optional authority must yield first"
    );
    assert!(
        !hints.accept_hint_inputs(),
        "discarded optional output is unavailable"
    );
    assert!(parent.retained_bytes() <= MAX_BYTES);
}

#[test]
fn optional_hint_pressure_yields_to_required_semantic_hydration() {
    let native = Rc::new(ChangingFs::default());
    *native.listing.borrow_mut() = (0..22_000)
        .map(|index| (format!("{index:05}{}", "x".repeat(180)), false))
        .collect();
    *native.bytes.borrow_mut() = vec![b'x'; 8 * 1024 * 1024];
    let parent = Rc::new(InputEpochFs::new(native));
    let _ = parent.list_dir(Path::new("semantic-first"));
    let hints = parent.hint_fork().unwrap();
    let _ = hints.list_dir(Path::new("hint-only"));
    assert!(parent.seal_after_validation());
    assert!(hints.accept_hint_inputs());
    assert!(parent.read(Path::new("semantic-later")).is_none());
    parent.reconcile_step_inner(1, 256 * 1024);
    assert!(
        parent.native_required.borrow().is_empty(),
        "semantic hydration fits after optional authority yields"
    );
    for _ in 0..16 {
        parent.reconcile_step_inner(32, 256 * 1024);
    }
    assert!(parent.ready());
    assert_eq!(
        parent.read(Path::new("semantic-later")).unwrap().len(),
        8 * 1024 * 1024
    );
    assert!(!hints.accept_hint_inputs());
    assert!(parent.retained_bytes() <= MAX_BYTES);
}

#[test]
fn hint_only_kind_change_preserves_parent_but_newly_shared_kind_revokes_it() {
    for shared in [false, true] {
        let native = Rc::new(ChangingFs::default());
        let parent = Rc::new(InputEpochFs::new(native.clone()));
        assert!(parent.is_file(Path::new("same-path")));
        let hints = parent.hint_fork().unwrap();
        assert!(hints.read(Path::new("same-path")).is_some());
        if shared {
            assert!(parent.read(Path::new("same-path")).is_some());
        }
        assert!(parent.seal_after_validation());
        assert!(hints.accept_hint_inputs());
        *native.bytes.borrow_mut() = b"changed".to_vec();
        let mut changed = None;
        for _ in 0..8 {
            changed = changed.or_else(|| parent.reconcile_step(32, 256 * 1024));
        }
        assert_eq!(changed.is_some(), shared);
        assert_eq!(parent.ready(), !shared);
        if !shared {
            assert!(hints.ready());
            assert_eq!(hints.read(Path::new("same-path")).unwrap(), b"changed");
        }
    }
}

#[test]
fn active_hint_query_defers_optional_native_service() {
    let native = Rc::new(ChangingFs::default());
    let parent = Rc::new(InputEpochFs::new(native.clone()));
    assert!(parent.seal_after_validation());
    let hints = parent.hint_fork().unwrap();
    hints.enter_hint_query();
    assert!(hints.read(Path::new("optional")).is_none());
    for _ in 0..8 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert_eq!(native.reads.get(), 0);
    hints.leave_hint_query();
    for _ in 0..8 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert!(hints.ready());
    assert!(native.reads.get() > 0);
}

#[test]
fn expired_optional_proof_owner_recovers_through_bounded_hydration() {
    let native = Rc::new(ChangingFs::default());
    let parent = Rc::new(InputEpochFs::new(native));
    assert!(parent.seal_after_validation());
    let hints = parent.hint_fork().unwrap();
    // A real oversized render proof can own these bytes only until its query drops.
    let owner = Arc::new(ReadBuffer::new(Arc::from([])));
    hints.sealed.set(false);
    hints.record(
        Path::new("optional"),
        Response::Read(Some(ReadBytes::ProofOwned(Arc::downgrade(&owner)))),
    );
    assert!(hints.accept_hint_inputs());
    drop(owner);
    for _ in 0..16 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert!(parent.ready());
    assert!(
        hints.ready(),
        "expired optional bytes require a fresh bounded successor"
    );
    assert!(hints.read(Path::new("optional")).is_some());
}

#[test]
fn native_event_during_optional_service_cannot_promote_old_parent_output() {
    struct ResetFs {
        parent: RefCell<std::rc::Weak<InputEpochFs>>,
    }
    impl FileSystem for ResetFs {
        fn read(&self, _: &Path) -> Option<Vec<u8>> {
            self.parent.borrow().upgrade().unwrap().begin_capture();
            Some(b"fresh".to_vec())
        }
        fn is_file(&self, _: &Path) -> bool {
            false
        }
        fn is_dir(&self, _: &Path) -> bool {
            false
        }
        fn list_dir(&self, _: &Path) -> Vec<(String, bool)> {
            Vec::new()
        }
    }
    let native = Rc::new(ResetFs {
        parent: RefCell::default(),
    });
    let parent = Rc::new(InputEpochFs::new(native.clone()));
    *native.parent.borrow_mut() = Rc::downgrade(&parent);
    assert!(parent.seal_after_validation());
    let hints = parent.hint_fork().unwrap();
    assert!(hints.read(Path::new("optional")).is_none());
    for _ in 0..4 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert!(!hints.accept_hint_inputs());
    assert!(!parent.ready());
}

#[test]
fn oversized_optional_hydration_stays_explicitly_unavailable_without_retrying() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let parent = Rc::new(InputEpochFs::new(native.clone()));
    assert!(parent.seal_after_validation());
    let hints = parent.hint_fork().unwrap();
    assert!(hints.read(Path::new("optional")).is_none());
    for _ in 0..8 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert!(!hints.hint_query_available());
    assert!(!hints.native_required.borrow().is_empty());
    let reads = native.reads.get();
    for _ in 0..32 {
        parent.reconcile_step(32, 256 * 1024);
    }
    assert_eq!(
        native.reads.get(),
        reads,
        "budget refusal waits for a parent reset; it never loops native reads"
    );
    assert!(parent.ready());
}

#[test]
fn known_events_match_raw_history_and_returned_identities_without_native_io() {
    let make = |raw: &str, kind, value| {
        ActualInputs(Rc::new(BTreeMap::from([(
            (std::ffi::OsString::from(raw), kind),
            value,
        )])))
    };
    let raw = make("/workspace/a/data", 0, Response::Read(None));
    assert!(raw.concerns_tree(Path::new("/workspace/a")));
    assert!(!raw.concerns_tree(Path::new("/workspace/b")));
    let listing = make("/workspace/a", 3, Response::Listing(Vec::new()));
    assert!(listing.concerns_tree(Path::new("/workspace/a/new")));
    assert!(!listing.concerns_tree(Path::new("/workspace/a/nested/new")));
    let target = make(
        "/alias/file",
        4,
        Response::Canonical(PathBuf::from("/physical/file")),
    );
    assert!(target.concerns_tree(Path::new("/physical")));
    let ancestor = make("/", 4, Response::Canonical(PathBuf::from("/")));
    assert!(
        !ancestor.concerns_tree(Path::new("/unrelated/file")),
        "canonical ancestors are not blanket dependencies"
    );
    let stamp = make("/workspace/a", 6, Response::Stamp(Some(1)));
    assert!(stamp.concerns_tree(Path::new("/workspace/a/nested/file")));
    let windows = make(r"\\?\c:\workspace\data", 0, Response::Read(None));
    assert!(
        windows.concerns_tree_with(Path::new("C:/workspace"), None, |path| {
            PathBuf::from(jaic::paths::native_spelling(path.to_str().unwrap(), true).unwrap())
        })
    );
}

#[test]
fn event_retirement_only_skips_exact_available_fresh_responses() {
    let path = Path::new("/outside/marker");
    let key = (path.as_os_str().to_owned(), 4);
    let old = ActualInputs(Rc::new(BTreeMap::from([(
        key.clone(),
        Response::Canonical(path.into()),
    )])));
    assert!(
        !old.concerns_tree_since(path, Some(&old.clone())),
        "fresh identical marker preserves owner"
    );
    let empty = ActualInputs(Rc::default());
    assert!(old.concerns_tree_since(path, Some(&empty)));
    let different = ActualInputs(Rc::new(BTreeMap::from([(
        key.clone(),
        Response::Canonical("/changed".into()),
    )])));
    assert!(old.concerns_tree_since(path, Some(&different)));
    let mut mixed = (*old.0).clone();
    mixed.insert((path.as_os_str().to_owned(), 0), Response::Read(None));
    let mixed = ActualInputs(Rc::new(mixed));
    assert!(
        mixed.concerns_tree_since(path, Some(&old)),
        "one matching row cannot hide another affected row"
    );
    let owner = Arc::new(ReadBuffer::new(Arc::from(&b"bytes"[..])));
    let expired = ActualInputs(Rc::new(BTreeMap::from([(
        (path.as_os_str().to_owned(), 0),
        Response::Read(Some(ReadBytes::ProofOwned(Arc::downgrade(&owner)))),
    )])));
    drop(owner);
    assert!(
        expired.concerns_tree_since(path, Some(&expired)),
        "identical expired weak owners remain unavailable"
    );
}

#[test]
fn retained_actual_inputs_only_deduct_exact_proof_owned_payload() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; 4096];
    let epoch = InputEpochFs::new(native.clone());
    let proof = RefCell::new(super::super::Observations::default());
    let observed = super::super::ObservedFs {
        base: &epoch,
        proof: Some(&proof),
        secondary: None,
    };
    let path = Path::new("data");
    let owner = observed.read_shared(path).unwrap();
    let actual = epoch.actual_inputs();
    let full = actual.retained_bytes();
    assert_eq!(
        actual.retained_bytes_with_proof(&proof.borrow()),
        full - owner.len()
    );
    assert_eq!(
        actual.retained_bytes_with_proof(&super::super::Observations::default()),
        full
    );
    let independent = RefCell::new(super::super::Observations::default());
    let other = super::super::ObservedFs {
        base: &*native,
        proof: Some(&independent),
        secondary: None,
    };
    assert_eq!(
        other.read_shared(path).unwrap().as_slice(),
        owner.as_slice()
    );
    assert_eq!(
        actual.retained_bytes_with_proof(&independent.borrow()),
        full,
        "equal independent bytes still retain a separate allocation"
    );
    let different_raw = ActualInputs(Rc::new(BTreeMap::from([(
        (std::ffi::OsString::from("./data"), 0),
        Response::Read(Some(ReadBytes::Owned(owner.clone()))),
    )])));
    assert_eq!(
        different_raw.retained_bytes_with_proof(&proof.borrow()),
        different_raw.retained_bytes(),
        "a normalized-equivalent raw spelling does not prove exact response ownership"
    );
    let weak = ActualInputs(Rc::new(BTreeMap::from([(
        (path.as_os_str().to_owned(), 0),
        Response::Read(Some(ReadBytes::ProofOwned(Arc::downgrade(&owner)))),
    )])));
    assert_eq!(
        weak.retained_bytes_with_proof(&proof.borrow()),
        weak.retained_bytes(),
        "weak metadata stays charged even when its payload is proof-owned"
    );
    assert!(
        proof.borrow().retained_bytes() + actual.retained_bytes_with_proof(&proof.borrow())
            < proof.borrow().retained_bytes() + full
    );
}

#[test]
fn oversized_reads_borrow_the_exact_proof_owner_without_epoch_megaretention() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let epoch = InputEpochFs::new(native.clone());
    let proof = RefCell::new(super::super::Observations::default());
    let observed = super::super::ObservedFs {
        base: &epoch,
        proof: Some(&proof),
        secondary: None,
    };
    let returned = observed.read_shared(Path::new("large")).unwrap();
    let weak = Arc::downgrade(&returned);
    drop(returned);
    assert!(
        epoch.seal_after_validation(),
        "the existing exact proof owns supported oversized bytes"
    );
    let original = epoch.actual_inputs();
    assert!(
        original.retained_bytes() < MAX_BYTES,
        "carrier metadata cannot charge a second retained payload"
    );
    let reads = native.reads.get();
    assert_eq!(
        epoch.read_shared(Path::new("large")).unwrap().len(),
        MAX_BYTES + 1
    );
    assert_eq!(native.reads.get(), reads);
    let mut retained = super::super::Observations::default();
    retained.merge(&proof.borrow());
    *proof.borrow_mut() = super::super::Observations::default();
    assert!(
        weak.upgrade().is_some(),
        "merged retained proof must keep the exact read owner"
    );
    drop(retained);
    assert!(
        weak.upgrade().is_none(),
        "epoch must not keep oversized payload alive after its proof owner"
    );
    assert!(
        !epoch.reuse_actual(&original),
        "identical expired weak handles are unknown, not equal responses"
    );
    assert!(!epoch.ready());
    assert!(epoch.read_shared(Path::new("large")).is_none());
    assert_eq!(
        native.reads.get(),
        reads,
        "expiry cannot read native bytes inline"
    );
}

#[test]
fn each_equal_capture_read_proof_owns_the_actual_epoch_response() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let epoch = InputEpochFs::new(native);
    let first = RefCell::new(super::super::Observations::default());
    let second = RefCell::new(super::super::Observations::default());
    for proof in [&first, &second] {
        let observed = super::super::ObservedFs {
            base: &epoch,
            proof: Some(proof),
            secondary: None,
        };
        assert_eq!(
            observed.read_shared(Path::new("large")).unwrap().len(),
            MAX_BYTES + 1
        );
    }
    *first.borrow_mut() = super::super::Observations::default();
    assert!(
        epoch.seal_after_validation(),
        "the second exact proof must own the epoch response independently of the first proof"
    );
    let original = epoch.actual_inputs();
    *second.borrow_mut() = super::super::Observations::default();
    epoch.begin_request_capture();
    assert!(
        !epoch.reuse_actual(&original),
        "expired original response cannot enter a fresh capture as authority"
    );
}

#[test]
fn raw_spelling_read_owners_survive_normalized_proof_merge_and_drop() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let epoch = InputEpochFs::new(native.clone());
    let proof = RefCell::new(super::super::Observations::default());
    let observed = super::super::ObservedFs {
        base: &epoch,
        proof: Some(&proof),
        secondary: None,
    };
    let a = Path::new("root/./data");
    let b = Path::new("root/data");
    assert_eq!(a, b);
    assert_ne!(a.as_os_str(), b.as_os_str());
    let returned_a = observed.read_shared(a).unwrap();
    let returned_b = observed.read_shared(b).unwrap();
    let owner_a = Arc::downgrade(&returned_a);
    let owner_b = Arc::downgrade(&returned_b);
    drop((returned_a, returned_b));
    assert!(
        epoch.seal_after_validation(),
        "both raw first responses need their own retained owners"
    );
    let original = epoch.actual_inputs();
    let mut retained = super::super::Observations::default();
    retained.merge(&proof.borrow());
    *proof.borrow_mut() = super::super::Observations::default();
    assert!(owner_a.upgrade().is_some());
    assert!(
        owner_b.upgrade().is_some(),
        "merge must retain both raw spellings behind one normalized row"
    );
    assert!(
        retained.retained_bytes() >= 2 * (MAX_BYTES + 1),
        "distinct payload allocations must both be charged"
    );
    assert!(epoch.reuse_actual(&original));
    *native.bytes.borrow_mut() = vec![b'y'; MAX_BYTES + 1];
    assert_eq!(retained.remove_invalid(&*native).len(), 1);
    assert!(owner_a.upgrade().is_none());
    assert!(owner_b.upgrade().is_none());
    assert!(!epoch.reuse_actual(&original));
}

#[test]
fn weak_payload_replay_uses_full_read_bytes_for_the_reconciliation_budget() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let epoch = InputEpochFs::new(native.clone());
    let proof = RefCell::new(super::super::Observations::default());
    let observed = super::super::ObservedFs {
        base: &epoch,
        proof: Some(&proof),
        secondary: None,
    };
    observed.read_shared(Path::new("large-a")).unwrap();
    observed.read_shared(Path::new("large-b")).unwrap();
    assert!(epoch.seal_after_validation());
    assert!(epoch.actual_inputs().retained_bytes() < MAX_BYTES);
    native.reads.set(0);
    epoch.reconcile_step(32, 256 * 1024);
    assert_eq!(
        native.reads.get(),
        1,
        "one oversized native read exhausts the expected-byte slice"
    );
    assert!(epoch.ready());
}

#[test]
fn oversized_unknown_read_uses_a_later_native_capture_without_hydration_loop() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = vec![b'x'; MAX_BYTES + 1];
    let epoch = InputEpochFs::new(native.clone());
    assert!(epoch.seal_after_validation());
    assert!(epoch.read_shared(Path::new("large")).is_none());
    assert_eq!(native.reads.get(), 0);
    for _ in 0..128 {
        epoch.reconcile_step(1, 1024);
    }
    assert_eq!(
        native.reads.get(),
        1,
        "oversized hydration is not retried forever"
    );
    assert!(!epoch.ready());
    assert!(!epoch.has_pending_coverage());
    assert!(
        epoch.take_updates_pending(),
        "native fallback must schedule a later client transaction"
    );
    epoch.begin_request_capture();
    assert!(
        epoch.seal_after_validation(),
        "strict native answer validation remains available"
    );
    assert!(
        !epoch.ready(),
        "an unrelated answer cannot seal over a required oversized operation"
    );
    let proof = RefCell::new(super::super::Observations::default());
    let observed = super::super::ObservedFs {
        base: &epoch,
        proof: Some(&proof),
        secondary: None,
    };
    assert_eq!(
        observed.read_shared(Path::new("large")).unwrap().len(),
        MAX_BYTES + 1
    );
    assert!(
        epoch.seal_after_validation(),
        "a later explicit native capture has an exact proof owner"
    );
}

#[test]
fn sealed_read_keeps_accepted_bytes_without_live_read_through() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = b"accepted".to_vec();
    let epoch = InputEpochFs::new(native.clone());
    let path = Path::new("source");
    assert_eq!(epoch.read(path).as_deref(), Some(b"accepted".as_slice()));
    assert!(epoch.seal_after_validation());
    let accepted_reads = native.reads.get();
    *native.bytes.borrow_mut() = b"silent change".to_vec();
    assert_eq!(epoch.read(path).as_deref(), Some(b"accepted".as_slice()));
    assert_eq!(native.reads.get(), accepted_reads);
    assert!(epoch.ready());
}

#[test]
fn unknown_operation_is_unavailable_without_live_read_through() {
    let native = Rc::new(ChangingFs::default());
    let epoch = InputEpochFs::new(native.clone());
    assert!(epoch.seal_after_validation());
    assert_eq!(epoch.read(Path::new("unknown")), None);
    assert_eq!(native.reads.get(), 0);
    assert!(epoch.coverage_missed());
    assert!(!epoch.ready());
    assert!(
        !epoch.seal_after_validation(),
        "a missed request cannot be resealed as ready"
    );
    assert!(!epoch.is_native());
    assert!(epoch.tracks_native_inputs());
    assert_eq!(epoch.input_epoch_valid(), Some(false));
}

#[test]
fn missing_coverage_hydrates_only_between_requests_and_validates_the_successor() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = b"accepted".to_vec();
    let epoch = InputEpochFs::new(native.clone());
    assert_eq!(
        epoch.read(Path::new("known")).as_deref(),
        Some(b"accepted".as_slice())
    );
    assert!(epoch.seal_after_validation());
    let reads = native.reads.get();
    assert_eq!(epoch.read(Path::new("new")), None);
    assert_eq!(
        native.reads.get(),
        reads,
        "miss cannot read native input inline"
    );
    assert!(!epoch.ready());
    for _ in 0..128 {
        epoch.reconcile_step(1, 1024);
        if epoch.ready() {
            break;
        }
    }
    assert!(
        epoch.ready(),
        "bounded service must accept a fully validated successor"
    );
    let reads = native.reads.get();
    assert_eq!(
        epoch.read(Path::new("new")).as_deref(),
        Some(b"accepted".as_slice())
    );
    assert_eq!(
        epoch.read(Path::new("known")).as_deref(),
        Some(b"accepted".as_slice())
    );
    assert_eq!(native.reads.get(), reads, "both warm contexts stay sealed");
}

#[test]
fn successor_validation_rejects_changes_to_original_inputs() {
    let native = Rc::new(ChangingFs::default());
    *native.bytes.borrow_mut() = b"old".to_vec();
    let epoch = InputEpochFs::new(native.clone());
    assert_eq!(
        epoch.read(Path::new("known")).as_deref(),
        Some(b"old".as_slice())
    );
    assert!(epoch.seal_after_validation());
    assert_eq!(epoch.read(Path::new("new")), None);
    *native.bytes.borrow_mut() = b"changed".to_vec();
    let changed = (0..128).find_map(|_| epoch.reconcile_step(1, 1024));
    assert_eq!(changed.as_deref(), Some(Path::new("known")));
    assert!(!epoch.ready());
    assert!(!epoch.seal_after_validation());
}

#[test]
fn interrupted_successor_cannot_become_accepted() {
    struct ResetInterrupt;
    impl Drop for ResetInterrupt {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
        }
    }
    let _reset = ResetInterrupt;
    let epoch = InputEpochFs::new(Rc::new(ChangingFs::default()));
    epoch.read(Path::new("known"));
    assert!(epoch.seal_after_validation());
    assert_eq!(epoch.read(Path::new("new")), None);
    jaic::interrupt::arm();
    jaic::interrupt::request(true);
    assert!(jaic::interrupt::should_stop());
    for _ in 0..16 {
        epoch.reconcile_step(1, 1024);
        assert!(
            !epoch.ready(),
            "interruption must prevent successor publication"
        );
    }
    jaic::interrupt::disarm();
    jaic::interrupt::request(false);
    for _ in 0..128 {
        epoch.reconcile_step(1, 1024);
        if epoch.ready() {
            break;
        }
    }
    assert!(epoch.ready());
}

#[test]
fn an_unknown_placeholder_cannot_validate_a_matching_comparison_proof() {
    let native = Rc::new(ChangingFs::default());
    let epoch = InputEpochFs::new(native.clone());
    assert!(epoch.seal_after_validation());
    let mut proof = super::super::observations::Observations::default();
    proof.record(
        PathBuf::from("unknown"),
        super::super::observations::Observation::Read(None),
    );
    assert!(!proof.matches(&epoch));
    assert_eq!(native.reads.get(), 0);
}

#[test]
fn observed_sealed_authority_never_grants_direct_native_io() {
    let epoch = InputEpochFs::new(Rc::new(ChangingFs::default()));
    assert!(epoch.seal_after_validation());
    let observed = super::super::observations::ObservedFs {
        base: &epoch,
        proof: None,
        secondary: None,
    };
    assert!(!observed.is_native());
    assert!(observed.tracks_native_inputs());
    assert_eq!(observed.input_epoch_valid(), Some(true));
}

#[test]
fn canonical_spelling_and_listing_order_replay_exactly() {
    let native = Rc::new(ChangingFs::default());
    let raw = PathBuf::from("root/./source");
    *native.canonical.borrow_mut() = raw.clone();
    let rows = vec![("b".to_owned(), false), ("a".to_owned(), true)];
    *native.listing.borrow_mut() = rows.clone();
    let epoch = InputEpochFs::new(native);
    let path = Path::new("source");
    assert_eq!(epoch.canonical(path).as_os_str(), raw.as_os_str());
    assert_eq!(epoch.list_dir(path), rows);
    assert!(epoch.seal_after_validation());
    assert_eq!(epoch.canonical(path).as_os_str(), raw.as_os_str());
    assert_eq!(epoch.list_dir(path), rows);
}

#[test]
fn normalized_equal_canonical_spellings_are_not_equal_actual_responses() {
    let native = Rc::new(ChangingFs::default());
    *native.canonical.borrow_mut() = PathBuf::from("root/./source");
    let epoch = InputEpochFs::new(native.clone());
    let path = Path::new("source");
    let first = epoch.canonical(path);
    *native.canonical.borrow_mut() = PathBuf::from("root/source");
    let second = epoch.canonical(path);
    assert_ne!(first.as_os_str(), second.as_os_str());
    assert_eq!(
        jaic::paths::identity_spelling(&first),
        jaic::paths::identity_spelling(&second)
    );
    assert!(!epoch.seal_after_validation());
}

#[test]
fn normalized_equal_actual_response_churn_is_unsealable() {
    let native = Rc::new(ChangingFs::default());
    *native.listing.borrow_mut() = vec![("b".to_owned(), false), ("a".to_owned(), true)];
    let epoch = InputEpochFs::new(native.clone());
    let path = Path::new("source");
    let mut first = epoch.list_dir(path);
    native.listing.borrow_mut().reverse();
    let mut second = epoch.list_dir(path);
    first.sort();
    second.sort();
    assert_eq!(first, second, "comparison proofs agree");
    assert!(
        !epoch.seal_after_validation(),
        "different actual returns must not seal"
    );
}


#[test]
fn epoch_pressure_borrows_only_exact_accounted_read_owners() {
    struct PressureFs;
    impl FileSystem for PressureFs {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            Some(if path == Path::new("tiny") {
                vec![1]
            } else {
                vec![7; 6 * 1024 * 1024]
            })
        }
        fn is_file(&self, _: &Path) -> bool {
            true
        }
        fn is_dir(&self, _: &Path) -> bool {
            false
        }
        fn list_dir(&self, _: &Path) -> Vec<(String, bool)> {
            (0..9_000)
                .map(|i| (format!("{i:05}{}", "x".repeat(500)), false))
                .collect()
        }
    }
    for mode in ["matching", "independent", "raw", "borrowed"] {
        let proof = Rc::new(RefCell::new(super::super::Observations::default()));
        let mut epoch = InputEpochFs::new(Rc::new(PressureFs));
        epoch.read_owner = Rc::downgrade(&proof);
        let mut weak = Vec::new();
        for path in ["one/data", "two/data", "tiny"] {
            let owner = epoch.read_shared(Path::new(path)).unwrap();
            weak.push(Arc::downgrade(&owner));
            let retained = if mode == "independent" {
                Arc::new(ReadBuffer::new(owner.bytes().clone()))
            } else {
                owner.clone()
            };
            let key = if mode == "raw" {
                PathBuf::from(path)
                    .parent()
                    .unwrap_or(Path::new(""))
                    .join(".")
                    .join(Path::new(path).file_name().unwrap())
            } else {
                PathBuf::from(path)
            };
            proof.borrow_mut().record(
                key.clone(),
                super::super::observations::Observation::Read(Some(retained.bytes().clone())),
            );
            proof.borrow_mut().record_read_owner(key, retained);
        }
        let original = epoch.actual_inputs();
        if mode == "borrowed" {
            let _borrow = proof.borrow_mut();
            let _ = epoch.list_dir(Path::new("large-listing"));
        } else {
            let _ = epoch.list_dir(Path::new("large-listing"));
        }
        if mode != "matching" {
            assert!(
                !epoch.seal_after_validation(),
                "{mode} cannot certify a different or currently borrowed owner"
            );
            continue;
        }
        assert!(
            epoch.seal_after_validation(),
            "proof-owned payloads must leave room for required metadata without increasing the cap"
        );
        assert!(epoch.retained_bytes() < MAX_BYTES);
        assert!(
            proof.borrow().retained_bytes() >= 12 * 1024 * 1024,
            "surviving payload is charged to its real proof owner"
        );
        let current = epoch.actual_inputs();
        assert!(matches!(
            current.0.get(&("one/data".into(), 0)),
            Some(Response::Read(Some(ReadBytes::ProofOwned(_))))
        ));
        assert!(
            matches!(
                current.0.get(&("tiny".into(), 0)),
                Some(Response::Read(Some(ReadBytes::Owned(_))))
            ),
            "weak metadata must not grow a tiny read"
        );
        assert_eq!(
            current.0[&("one/data".into(), 0)].work_bytes(),
            6 * 1024 * 1024
        );
        assert!(
            matches!(
                original.0.get(&("one/data".into(), 0)),
                Some(Response::Read(Some(ReadBytes::Owned(_))))
            ),
            "old actual snapshots keep their original owner and charge"
        );
        assert!(original.retained_bytes() >= 12 * 1024 * 1024);
        assert!(epoch.seal_after_validation());
        *proof.borrow_mut() = super::super::Observations::default();
        assert!(
            weak[0].upgrade().is_some(),
            "the old immutable snapshot still owns its payload"
        );
        drop(original);
        assert!(weak[0].upgrade().is_none());
        assert!(!epoch.seal_after_validation());
        assert!(
            !epoch.validate_actual(),
            "expired proof ownership must reject authority"
        );
    }
}
