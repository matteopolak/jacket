//! Foreign release protocols come from ABI symbols, never ordinary wrapper names.
use super::*;

#[test]
fn fallible_windows_release_symbols_export_only_possible_frees() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        lib :: #system_library "kernel32";
        local_release :: (p:*void)->*void #foreign lib "LocalFree";
        global_release :: (p:*void)->*void #foreign lib "GlobalFree";
        key_release :: (p:*void)->u32 #foreign lib "RegCloseKey";
        release_local :: (p:*void) {local_release(p);}
        release_global :: (p:*void) {global_release(p);}
        release_key :: (p:*void) {key_release(p);}
        LocalFree :: (p:*void) {}
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    for name in ["release_local", "release_global", "release_key"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let s = &after.procs[&id];
        assert!(
            s.may
                .keys()
                .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0)),
            "{name}: {s:?}"
        );
        assert!(!s.guaranteed.keys().any(|e| e.kind == Kind::Free));
    }
    let named = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "LocalFree" && p.lit.body.is_some())
            .unwrap() as u32,
    );
    assert!(!after.procs[&named].may.keys().any(|e| e.kind == Kind::Free));
}

#[test]
fn native_allocation_contracts_authenticate_selected_target_library_and_fixed_abi() {
    use jaic::target::{TargetCpu, TargetOs};
    let source = r#"
        C::#system_library "libc";
        S::#system_library "System";
        O::#system_library "not-a-c-runtime";
        range_malloc_native::(bytes:u64)->*void #foreign C "malloc";
        range_free_native::(memory:*void) #foreign C "free";
        range_malloc_system::(bytes:u64)->*void #foreign S "malloc";
        range_malloc_other::(bytes:u64)->*void #foreign O "malloc";
        range_malloc_signed::(bytes:s64)->*void #foreign C "malloc";
        range_malloc_narrow::(bytes:u32)->*void #foreign C "malloc";
        range_malloc_variadic::(bytes:..u64)->*void #foreign C "malloc";
        range_malloc_return::(bytes:u64)->u64 #foreign C "malloc";
        range_malloc_body::(bytes:u64)->*void {return null;}
        main::(){range_free_native(null);range_malloc_native(64);range_malloc_system(64);range_malloc_other(64);range_malloc_signed(64);range_malloc_narrow(64);range_malloc_variadic(64);range_malloc_return(64);range_malloc_body(64);}
    "#;
    for (os, cpu, supported) in [
        (TargetOs::MacOS, TargetCpu::Arm64, true),
        (TargetOs::MacOS, TargetCpu::X64, true),
        (TargetOs::Linux, TargetCpu::X64, true),
        (TargetOs::Linux, TargetCpu::Arm64, true),
        (TargetOs::Windows, TargetCpu::X64, false),
        (TargetOs::Wasm, TargetCpu::Wasm, false),
    ] {
        let mut options = Options::host();
        options.os = os;
        options.cpu = cpu;
        let mut c = super::stdlib_allocations::checked_source_options(source, options);
        let all = Summaries::build(&c);
        let id = |name: &str| {
            ProcId(
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap() as u32,
            )
        };
        for (name, expected) in [
            ("range_malloc_native", supported),
            ("range_malloc_system", os == TargetOs::MacOS),
            ("range_malloc_other", false),
            ("range_malloc_signed", false),
            ("range_malloc_narrow", false),
            ("range_malloc_variadic", false),
            ("range_malloc_return", false),
            ("range_malloc_body", false),
        ] {
            assert_eq!(
                all.procs[&id(name)].native_allocation.is_some(),
                expected,
                "{os:?}/{cpu:?}: {name}"
            );
        }
        let free_id = id("range_free_native");
        let free_supported = cpu != TargetCpu::Wasm && os != TargetOs::Wasm;
        assert_eq!(
            all.procs[&free_id]
                .guaranteed
                .keys()
                .any(|e| e.kind == Kind::Free),
            free_supported
        );
        let selected = id("range_malloc_native");
        let before = all.procs[&selected].native_allocation.clone();
        c.release_lowered_code();
        assert_eq!(
            Summaries::build(&c).procs[&selected].native_allocation,
            before
        );
        for primitive in [selected, free_id] {
            let jaic::sema::procs::ProcTarget::Foreign(fid) =
                c.pr.procs[primitive.0 as usize].target.unwrap()
            else {
                unreachable!()
            };
            let original = c.out.program.foreigns[fid.0 as usize].sig.clone();
            for corruption in [
                "params",
                "returns",
                "convention",
                "varargs",
                "fixed",
                "aggregate",
            ] {
                let mut changed = original.clone();
                match corruption {
                    "params" => changed.params.push(jaic::ir::Ty::Ptr),
                    "returns" => changed.returns.push(jaic::ir::Ty::I64),
                    "convention" => changed.conv = jaic::ir::Conv::Jai,
                    "varargs" => changed.c_varargs = true,
                    "fixed" => changed.c_fixed += 1,
                    "aggregate" => changed.c_abi = Some(Box::default()),
                    _ => unreachable!(),
                }
                c.out.program.foreigns[fid.0 as usize].sig = changed;
                let rejected = Summaries::build(&c);
                assert!(
                    rejected.procs[&primitive].native_allocation.is_none(),
                    "{corruption}"
                );
                assert!(
                    !rejected.procs[&primitive]
                        .guaranteed
                        .keys()
                        .any(|e| e.kind == Kind::Free),
                    "{corruption}"
                );
            }
            c.out.program.foreigns[fid.0 as usize].sig = original;
        }
        if let Some(contract) = before {
            assert!(contract.foreign().unwrap().current(&c, selected));
            let jaic::sema::procs::ProcTarget::Foreign(fid) =
                c.pr.procs[selected.0 as usize].target.unwrap()
            else {
                unreachable!()
            };
            c.out.program.foreigns[fid.0 as usize].is_data = true;
            assert!(
                !contract.foreign().unwrap().current(&c, selected),
                "changed selected primitive cannot reuse range contract"
            );
        }
    }
}

#[test]
fn native_abort_nonreturn_requires_the_selected_c_runtime_abi() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        C::#system_library "libc";
        Other::#system_library "not-a-c-runtime";
        selected_abort::() #foreign C "abort";
        wrong_library::() #foreign Other "abort";
        wrong_parameter::(n:s64) #foreign C "abort";
        wrong_return::()->s64 #foreign C "abort";
        variadic_abort::(args:..s64) #foreign C "abort";
        ordinary_abort::(){}
        resolve_boundaries::(){selected_abort();wrong_library();wrong_parameter(1);value:=wrong_return();variadic_abort(1);ordinary_abort();}
        main::(){}
    "#,
    );
    let ids = [
        "selected_abort",
        "wrong_library",
        "wrong_parameter",
        "wrong_return",
        "variadic_abort",
        "ordinary_abort",
    ]
    .map(|name| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    });
    let check = |c: &Compiler| {
        let all = Summaries::build(c);
        for (index, id) in ids.iter().enumerate() {
            assert_eq!(
                all.procs[id].observable.exit,
                index == 0,
                "{}",
                c.proc(*id).name
            );
            if index == 0 {
                assert_eq!(all.procs[id].normal_return, Some(false));
            }
        }
    };
    check(&c);
    c.release_lowered_code();
    check(&c);
    let jaic::sema::procs::ProcTarget::Foreign(foreign) =
        c.pr.procs[ids[0].0 as usize].target.unwrap()
    else {
        unreachable!()
    };
    c.out.program.foreigns[foreign.0 as usize].sig.c_varargs = true;
    assert!(
        !super::super::observable::primitive(&c, ids[0]).exit,
        "stale lowered ABI rejects nonreturn"
    );
}
