use super::*;
use jai_ir::{IntExpr, ValueExpr};
use jai_source::{SourceMap, Span};
use jai_types::{
    Integer, IntegerType, RecordKind, RecordReflectionFlag, RecordReflectionPolicy, TypeView,
};
use jai_vm::{EffectOutcome, Limits, ProcedureAvailability, ProcedureProvider, Vm};
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

struct Provider<'a>(Ref<'a, TypeRegistry>);
impl ProcedureProvider for Provider<'_> {
    fn types(&self) -> &dyn TypeView {
        &*self.0
    }
    fn procedure(&self, _: jai_ir::ProcedureId) -> ProcedureAvailability<'_> {
        ProcedureAvailability::Missing
    }
}
struct Effects {
    events: Rc<RefCell<Vec<bool>>>,
    reject: bool,
}
impl CompilerEffects for Effects {
    fn begin(&mut self) {}
    fn request(&mut self, _: jai_vm::CompilerRequest) -> EffectOutcome {
        unreachable!()
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.events.borrow_mut().push(commit);
        if commit && self.reject {
            Err(jai_vm::Error::EffectRejected(
                "authored commit rejection".into(),
            ))
        } else {
            Ok(())
        }
    }
}
fn source() -> (SourceMap, SourceSpan, SourceOrigin) {
    let mut sources = SourceMap::default();
    let id = sources.insert("source-policy-publication.jai".into(), "#run {}".into());
    let location = SourceSpan {
        source: id,
        span: Span::new(0, 7),
    };
    let origin = SourceOrigin {
        workspace: jai_vm::WorkspaceId::from_raw(3).unwrap(),
        path: sources.get(id).unwrap().path().into(),
        start: 0,
        end: 7,
        body_hash: 0,
        body: b"#run {}".to_vec(),
        specialization: vec![1, 2],
    };
    (sources, location, origin)
}

#[test]
fn detached_source_policy_commit_updates_once_and_host_rejection_keeps_old_epoch() {
    for reject in [false, true] {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let types = RefCell::new(types);
        let mut meta = crate::reflection::MetaContext::default();
        let (sources, location, origin) = source();
        let source = sources.get(location.source).unwrap();
        let mut journal = ReflectionPolicyJournal::new(
            jai_ir::SourceProcedureIdentity::new(source, location).unwrap(),
        );
        journal
            .stage(
                &*types.borrow(),
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
            )
            .unwrap();
        let provider = Provider(types.borrow());
        let events = Rc::default();
        let mut vm = Vm::new(
            &provider,
            Effects {
                events: Rc::clone(&events),
                reject,
            },
            Limits::default(),
        )
        .unwrap();
        vm.pin_publication_source_origin(origin.clone()).unwrap();
        let expression = ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::S64, 42)));
        let publication = vm
            .prepare_expression_publication(None, &expression, |_, _| Ok(9))
            .unwrap_or_else(|_| panic!("prepare failed"));
        drop(provider);
        let mut types = types.borrow_mut();
        let result = commit_reflection_publication(
            publication,
            ReflectionPublicationInput {
                types: &mut types,
                meta: &mut meta,
                journal,
                owner: ReflectionPublicationSource { source, location },
                run: ReflectionPublicationSource { source, location },
                origin: &origin,
            },
            |payload, receipt| (payload, receipt.len()),
        )
        .unwrap_or_else(|_| panic!("source guard preparation failed"));
        assert_eq!(&*events.borrow(), &[true]);
        if reject {
            assert!(result.published.is_none());
            assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 0);
            assert_eq!(meta.reflection_policy_epoch(), 0);
        } else {
            assert_eq!(result.published, Some((9, 1)));
            assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 1);
            assert_eq!(meta.reflection_policy_epoch(), 1);
        }
    }
}

#[test]
fn stale_policy_or_changed_source_cancels_before_host_commit() {
    for changed_source in [false, true] {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let types = RefCell::new(types);
        let mut meta = crate::reflection::MetaContext::default();
        let (sources, location, origin) = source();
        let source = sources.get(location.source).unwrap();
        let mut journal = ReflectionPolicyJournal::new(
            jai_ir::SourceProcedureIdentity::new(source, location).unwrap(),
        );
        let hidden = RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]);
        journal.stage(&*types.borrow(), record, hidden).unwrap();
        if !changed_source {
            types
                .borrow_mut()
                .add_record_reflection_flags(
                    record,
                    RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoSizeComplaint]),
                )
                .unwrap();
        }
        let provider = Provider(types.borrow());
        let events = Rc::default();
        let mut vm = Vm::new(
            &provider,
            Effects {
                events: Rc::clone(&events),
                reject: false,
            },
            Limits::default(),
        )
        .unwrap();
        vm.pin_publication_source_origin(origin.clone()).unwrap();
        let expression = ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::S64, 42)));
        let publication = vm
            .prepare_expression_publication(None, &expression, |_, _| Ok(()))
            .unwrap_or_else(|_| panic!("prepare failed"));
        drop(provider);
        let mut types = types.borrow_mut();
        let mut expected_origin = origin;
        if changed_source {
            expected_origin.specialization.push(3);
        }
        let error = commit_reflection_publication(
            publication,
            ReflectionPublicationInput {
                types: &mut types,
                meta: &mut meta,
                journal,
                owner: ReflectionPublicationSource { source, location },
                run: ReflectionPublicationSource { source, location },
                origin: &expected_origin,
            },
            |_, _| panic!("rejected source cannot publish"),
        )
        .err()
        .expect("source publication rejected");
        assert!(error.cancellation_error.is_none());
        assert_eq!(&*events.borrow(), &[false]);
        assert_eq!(meta.reflection_policy_epoch(), 0);
        assert_eq!(
            types.record_reflection_policy(record).unwrap().bits(),
            if changed_source { 0 } else { 4 }
        );
    }
}

#[test]
fn same_callee_cannot_substitute_another_run_and_leaf_span_is_independent() {
    for wrong_run in [false, true] {
        let mut types = TypeRegistry::new();
        let record = types.reserve_record(RecordKind::Struct);
        types.define_record(record, []).unwrap();
        let types = RefCell::new(types);
        let mut meta = crate::reflection::MetaContext::default();
        let mut sources = SourceMap::default();
        let text = "callee::() {}\n#run callee();\n#run callee();";
        let source_id = sources.insert("two-source-runs.jai".into(), text.into());
        let source = sources.get(source_id).unwrap();
        let owner_location = SourceSpan {
            source: source_id,
            span: Span::new(0, text.find('\n').unwrap()),
        };
        let first = text.find("#run").unwrap();
        let second = text.rfind("#run").unwrap();
        let first_location = SourceSpan {
            source: source_id,
            span: Span::new(first, text[first..].find('\n').unwrap() + first),
        };
        let second_location = SourceSpan {
            source: source_id,
            span: Span::new(second, text.len()),
        };
        let run_origin = |location: SourceSpan| SourceOrigin {
            workspace: jai_vm::WorkspaceId::from_raw(4).unwrap(),
            path: source.path().into(),
            start: location.span.start,
            end: location.span.end,
            body_hash: 0,
            body: location.span.text(text).as_bytes().to_vec(),
            specialization: vec![7],
        };
        let first_origin = run_origin(first_location);
        let mut journal = ReflectionPolicyJournal::new(
            jai_ir::SourceProcedureIdentity::new(source, owner_location).unwrap(),
        );
        journal
            .stage_at(
                &*types.borrow(),
                record,
                RecordReflectionPolicy::from_flags([RecordReflectionFlag::NoTypeInfo]),
                jai_ir::SourceProcedureIdentity::new(source, owner_location).unwrap(),
            )
            .unwrap();
        assert_eq!(journal.calls()[0].source.location(), owner_location);
        let provider = Provider(types.borrow());
        let events = Rc::default();
        let mut vm = Vm::new(
            &provider,
            Effects {
                events: Rc::clone(&events),
                reject: false,
            },
            Limits::default(),
        )
        .unwrap();
        vm.pin_publication_source_origin(first_origin.clone())
            .unwrap();
        let expression = ValueExpr::Int(IntExpr::constant(Integer::wrapping(IntegerType::S64, 42)));
        let publication = vm
            .prepare_expression_publication(None, &expression, |_, _| Ok(()))
            .unwrap_or_else(|_| panic!("prepare failed"));
        drop(provider);
        let mut types = types.borrow_mut();
        let run_location = if wrong_run {
            second_location
        } else {
            first_location
        };
        let expected_origin = run_origin(run_location);
        let result = commit_reflection_publication(
            publication,
            ReflectionPublicationInput {
                types: &mut types,
                meta: &mut meta,
                journal,
                owner: ReflectionPublicationSource {
                    source,
                    location: owner_location,
                },
                run: ReflectionPublicationSource {
                    source,
                    location: run_location,
                },
                origin: &expected_origin,
            },
            |_, receipt| receipt.len(),
        );
        if wrong_run {
            assert!(result.is_err());
            assert_eq!(&*events.borrow(), &[false]);
            assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 0);
            assert_eq!(meta.reflection_policy_epoch(), 0);
        } else {
            let result = result.unwrap_or_else(|_| panic!("real run should publish"));
            assert_eq!(result.published, Some(1));
            assert_eq!(&*events.borrow(), &[true]);
            assert_eq!(types.record_reflection_policy(record).unwrap().bits(), 1);
            assert_eq!(meta.reflection_policy_epoch(), 1);
        }
    }
}
