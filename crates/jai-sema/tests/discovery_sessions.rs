//! Real source guards retain their checked VM checkpoint until graph publication.
use jai_modules::{GraphDiscovery, GraphOptions, SourceOverlay};
use jai_sema::{
    DiscoveryReadiness, PreparedDiscoveryOutcome, PreparedDiscoveryRequests,
    PreparedDiscoverySession,
};
use jai_vm::{
    CompilerEffects, CompilerRequest, CompilerResponse, Dependency, EffectKey, EffectOutcome,
};
use std::path::Path;

#[derive(Default)]
struct Effects {
    ready: bool,
    begins: usize,
    requests: usize,
    resumes: usize,
    finishes: Vec<bool>,
    origins: Vec<jai_vm::SourceOrigin>,
    host_scope: Option<jai_vm::host_effects::FilePathScope>,
    host_key: Option<jai_vm::host_effects::HostRequestKey>,
    host_requests: usize,
}
impl CompilerEffects for Effects {
    fn host_file_scope(&self) -> Option<jai_vm::host_effects::FilePathScope> {
        self.host_scope.clone()
    }
    fn host_request(
        &mut self,
        request: jai_vm::host_effects::HostRequest,
    ) -> jai_vm::host_effects::HostOutcome {
        let jai_vm::host_effects::HostRequest::ReadFileForOpen(path) = request else {
            panic!("unexpected host request");
        };
        assert_eq!(path.relative(), Path::new("input.txt"));
        self.host_requests += 1;
        let key = *self
            .host_key
            .get_or_insert_with(jai_vm::host_effects::HostRequestKey::allocate);
        jai_vm::host_effects::HostOutcome::Pending(key)
    }
    fn poll_host_request(
        &mut self,
        key: jai_vm::host_effects::HostRequestKey,
    ) -> jai_vm::host_effects::HostOutcome {
        assert_eq!(Some(key), self.host_key);
        if self.ready {
            jai_vm::host_effects::HostOutcome::Ready(jai_vm::host_effects::HostResponse::FileOpen(
                jai_vm::host_effects::FileOpenObservation::Bytes(b"abc".to_vec()),
            ))
        } else {
            jai_vm::host_effects::HostOutcome::Pending(key)
        }
    }
    fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
        self.origins.push(origin);
    }
    fn begin(&mut self) {
        self.begins += 1;
    }
    fn request(&mut self, request: CompilerRequest) -> EffectOutcome {
        assert!(matches!(request, CompilerRequest::CreateWorkspace { .. }));
        self.requests += 1;
        EffectOutcome::Pending(EffectKey(42))
    }
    fn poll_request(&mut self, _: &CompilerRequest, key: EffectKey) -> EffectOutcome {
        assert_eq!(key, EffectKey(42));
        if self.ready {
            EffectOutcome::Ready(CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        } else {
            EffectOutcome::Pending(key)
        }
    }
    fn suspend(&mut self) -> Result<(), jai_vm::Error> {
        Ok(())
    }
    fn resume(&mut self) -> Result<(), jai_vm::Error> {
        self.resumes += 1;
        Ok(())
    }
    fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
        self.finishes.push(commit);
        Ok(())
    }
}

fn overlay() -> SourceOverlay {
    let mut overlay = SourceOverlay::new();
    overlay.insert(Path::new("/guard-session/main.jai"), br#"
        compiler_create_workspace::(name:string)->s64 #compiler;
        calls:int;
        recipe::()->bool { calls+=1; receipt:=compiler_create_workspace("child"); return calls==1 && receipt==42; }
        #if #run,stallable recipe() { #load "chosen.jai"; } else { #load "missing.jai"; }
        main::()->int{return ANSWER;}
    "#.to_vec()).unwrap();
    overlay
        .insert(
            Path::new("/guard-session/chosen.jai"),
            b"ANSWER::42;".to_vec(),
        )
        .unwrap();
    overlay
}

fn options(graph: &jai_modules::ModuleGraph) -> jai_sema::ResolveOptions {
    jai_sema::ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..Default::default()
    }
}

#[test]
fn two_drives_resume_the_actual_guard_then_discover_only_its_chosen_file() {
    let overlay = overlay();
    let mut graph = GraphDiscovery::new(
        Path::new("/guard-session/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap();
    assert!(!graph.advance().unwrap().is_complete());
    let requests: Vec<_> = graph.pending_conditions().cloned().collect();
    assert_eq!(requests.len(), 1);
    let mut effects = Effects::default();
    let outcome = {
        let mut session = PreparedDiscoverySession::new(
            graph.graph(),
            &options(graph.graph()),
            PreparedDiscoveryRequests::Conditions(requests),
        )
        .unwrap();
        match session.drive(&mut effects) {
            DiscoveryReadiness::Pending(pending) => assert!(
                pending
                    .dependencies
                    .contains(&Dependency::Effect(EffectKey(42))),
                "{pending:?}"
            ),
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            DiscoveryReadiness::Complete(_) => panic!("a pending effect cannot select the branch"),
        }
        assert_eq!(effects.requests, 1);
        assert!(effects.finishes.is_empty());
        effects.ready = true;
        match session.drive(&mut effects) {
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Conditions(outcome)) => outcome,
            DiscoveryReadiness::Pending(pending) => panic!("{pending:?}"),
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            DiscoveryReadiness::Complete(_) => panic!("wrong request kind"),
        }
    };
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.begins, 1);
    assert_eq!(effects.finishes, [true]);
    assert!(effects.resumes > 0);
    assert_eq!(outcome.decisions.len(), 1);
    assert!(
        outcome.decisions[0].1,
        "the prefix increment must not execute twice"
    );
    for (id, selected) in outcome.decisions {
        graph.select_condition(id, selected).unwrap();
    }
    assert!(graph.advance().unwrap().is_complete());
    let graph = graph.into_graph().unwrap();
    assert!(
        graph
            .sources()
            .records()
            .iter()
            .all(|source| !source.path().ends_with("missing.jai"))
    );
    let program =
        jai_sema::resolve_graph_with_options(&graph, &options(&graph), &mut effects).unwrap();
    assert_eq!(
        effects.requests, 1,
        "semantic choices do not replay a completed source run"
    );
    let jai_vm::Outcome::Complete(values) = jai_vm::execute(&program, Default::default()).outcome
    else {
        panic!("VM failed");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn cancellation_rolls_back_the_exact_guard_and_publishes_no_selection() {
    let overlay = overlay();
    let mut graph = GraphDiscovery::new(
        Path::new("/guard-session/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap();
    graph.advance().unwrap();
    let requests: Vec<_> = graph.pending_conditions().cloned().collect();
    let mut effects = Effects::default();
    {
        let mut session = PreparedDiscoverySession::new(
            graph.graph(),
            &options(graph.graph()),
            PreparedDiscoveryRequests::Conditions(requests),
        )
        .unwrap();
        assert!(matches!(
            session.drive(&mut effects),
            DiscoveryReadiness::Pending(_)
        ));
        let origin = effects.origins.last().unwrap().clone();
        session.cancel(&mut effects).unwrap();
        session.cancel(&mut effects).unwrap();
        assert_eq!(effects.origins.last(), Some(&origin));
        assert!(matches!(
            session.drive(&mut effects),
            DiscoveryReadiness::Failed(_)
        ));
    }
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes, [false]);
    assert!(graph.graph().source_condition_selections().is_empty());
    assert_eq!(graph.pending_conditions().count(), 1);
    assert!(!graph.advance().unwrap().is_complete());
}

#[test]
fn a_ready_independent_guard_retires_an_unrelated_checkpoint_before_graph_advance() {
    let mut overlay = overlay();
    overlay
        .insert(
            Path::new("/guard-session/main.jai"),
            br#"
                compiler_create_workspace::(name:string)->s64 #compiler;
                recipe::()->bool{return compiler_create_workspace("child")==42;}
                #if #run,stallable recipe() { #load "chosen.jai"; }
                TEXT::"ready";
                #if TEXT=="ready" { #load "independent.jai"; }
                main::()->int{return ANSWER;}
            "#
            .to_vec(),
        )
        .unwrap();
    overlay
        .insert(
            Path::new("/guard-session/independent.jai"),
            b"INDEPENDENT::true;".to_vec(),
        )
        .unwrap();
    let mut graph = GraphDiscovery::new(
        Path::new("/guard-session/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap();
    assert!(!graph.advance().unwrap().is_complete());
    let requests: Vec<_> = graph.pending_conditions().cloned().collect();
    assert_eq!(requests.len(), 2);
    let parked = requests[0].id;
    let mut effects = Effects::default();
    let outcome = {
        let mut session = PreparedDiscoverySession::new(
            graph.graph(),
            &options(graph.graph()),
            PreparedDiscoveryRequests::Conditions(requests),
        )
        .unwrap();
        match session.drive(&mut effects) {
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Conditions(outcome)) => outcome,
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("the independent typed guard must allow source discovery to advance"),
        }
    };
    assert_eq!(effects.requests, 1);
    assert_eq!(effects.finishes, [false]);
    assert_eq!(outcome.decisions.len(), 1);
    assert_ne!(outcome.decisions[0].0, parked);
    assert!(outcome.decisions[0].1);
    for (id, selected) in outcome.decisions {
        graph.select_condition(id, selected).unwrap();
    }
    assert!(!graph.advance().unwrap().is_complete());
    assert_eq!(graph.pending_conditions().count(), 1);
    assert!(
        graph
            .graph()
            .sources()
            .records()
            .iter()
            .any(|source| source.path().ends_with("independent.jai"))
    );
    assert!(
        graph
            .graph()
            .sources()
            .records()
            .iter()
            .all(|source| !source.path().ends_with("chosen.jai"))
    );
}

#[test]
fn a_host_guard_keeps_live_local_buffers_and_closes_its_file_after_resume() {
    use jai_types::{Architecture, BuildTarget, ByteOrder, LayoutPolicy, OperatingSystem};
    let root = std::env::temp_dir().join(format!(
        "jai-guard-host-{}-{:?}",
        std::process::id(),
        jai_vm::host_effects::HostRequestKey::allocate()
    ));
    std::fs::create_dir(&root).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let scratch = Scratch(root);
    let modules = scratch.0.join("modules");
    let posix = modules.join("POSIX");
    std::fs::create_dir_all(posix.join("bindings/macos/arm64")).unwrap();
    std::fs::write(
        posix.join("module.jai"),
        "#load \"bindings/macos/arm64/stdio.jai\";",
    )
    .unwrap();
    std::fs::write(posix.join("bindings/macos/arm64/stdio.jai"), "FILE::struct{}; fopen::(path:*u8,mode:*u8)->*FILE #foreign libc; fclose::(file:*FILE)->s32 #foreign libc; libc::#system_library \"libc\";").unwrap();
    std::fs::write(scratch.0.join("chosen.jai"), "ANSWER::42;").unwrap();
    std::fs::write(
        scratch.0.join("main.jai"),
        r#"
        #import "POSIX";
        calls:int;
        recipe::()->bool {
            calls+=1;
            path:[10]u8=.[105,110,112,117,116,46,116,120,116,0];
            mode:[3]u8=.[114,98,0];
            file:=fopen(*path[0],*mode[0]);
            if file==null return false;
            fclose(file);
            return calls==1;
        }
        #if #run,stallable recipe() { #load "chosen.jai"; } else { #load "missing.jai"; }
        main::()->int{return ANSWER;}
    "#,
    )
    .unwrap();
    let target = BuildTarget {
        operating_system: OperatingSystem::MacOS,
        architecture: Architecture::Arm64,
        layout: LayoutPolicy::lp64(),
        byte_order: ByteOrder::Little,
    };
    let dirs = vec![modules];
    let mut graph = GraphDiscovery::with_target(
        &scratch.0.join("main.jai"),
        GraphOptions {
            import_dirs: dirs.clone(),
        },
        &jai_modules::Filesystem,
        target.clone(),
    )
    .unwrap();
    assert!(!graph.advance().unwrap().is_complete());
    let requests = graph.pending_conditions().cloned().collect();
    let mut effects = Effects {
        host_scope: Some(
            jai_vm::host_effects::FilePathScope::new(
                jai_vm::host_effects::FileRootId::allocate(),
                &scratch.0,
                Path::new(""),
            )
            .unwrap(),
        ),
        ..Default::default()
    };
    let outcome = {
        let mut options = options(graph.graph());
        options.target = Some(target.clone());
        options.file_abi =
            jai_sema::FileAbiBindingContext::from_graph(graph.graph(), &dirs, target);
        assert!(options.file_abi.is_some());
        let mut session = PreparedDiscoverySession::new(
            graph.graph(),
            &options,
            PreparedDiscoveryRequests::Conditions(requests),
        )
        .unwrap();
        match session.drive(&mut effects) {
            DiscoveryReadiness::Pending(pending) => assert!(
                pending
                    .dependencies
                    .iter()
                    .any(|dependency| matches!(dependency, Dependency::Host(_))),
                "{pending:?}"
            ),
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("host observation must stay pending"),
        }
        effects.ready = true;
        match session.drive(&mut effects) {
            DiscoveryReadiness::Complete(PreparedDiscoveryOutcome::Conditions(outcome)) => outcome,
            DiscoveryReadiness::Failed(error) => panic!("{error}"),
            _ => panic!("ready host observation must complete"),
        }
    };
    assert_eq!(effects.host_requests, 1);
    assert_eq!(effects.begins, 1);
    assert_eq!(effects.finishes, [true]);
    assert_eq!(outcome.decisions.len(), 1);
    assert!(outcome.decisions[0].1);
    for (id, selected) in outcome.decisions {
        graph.select_condition(id, selected).unwrap();
    }
    assert!(graph.advance().unwrap().is_complete());
    assert!(
        graph
            .graph()
            .sources()
            .records()
            .iter()
            .all(|source| !source.path().ends_with("missing.jai"))
    );
}
