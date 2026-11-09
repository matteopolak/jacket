mod descriptors;
mod exact_cleanup;
mod native_releases;
mod releases;
mod stdlib_allocations;

use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn allocator_mode_guards_distinguish_allocate_resize_and_free() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = r#"
            #import "Basic";
            #import "Hash_Table";
            Item :: struct { storage: *void; }
            probe_release :: (p: *Item, a: Allocator) { a.proc(.FREE, 0, 0, p.storage, a.data); }
            probe_resize :: (p: *Item, a: Allocator) { a.proc(.RESIZE, 32, 16, p.storage, a.data); }
            probe_allocate :: (p: *Item, a: Allocator) { a.proc(.ALLOCATE, 32, 0, null, a.data); }
            probe_conditional :: (p: *Item, a: Allocator, flag: bool) { if flag probe_release(p, a); }
            main :: () {
                xs: [..] int;
                array_reserve(*xs, 16);
                table: Table(int, int);
                table_add(*table, 1, 2);
                builder: String_Builder;
                append(*builder, "hello");
            }
        "#;
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let get = |name: &str| {
        let (i, _) =
            c.pr.procs
                .iter()
                .enumerate()
                .find(|(_, p)| p.name.as_str() == name)
                .unwrap();
        &summaries.procs[&ProcId(i as u32)]
    };
    assert!(
        get("probe_release")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.guard.is_none() && e.target.steps.len() == 2)
    );
    assert!(
        get("probe_resize")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Resize && e.guard.is_none())
    );
    assert!(get("probe_allocate").may.is_empty());
    assert!(
        get("probe_conditional")
            .may
            .keys()
            .any(|e| e.kind == Kind::Free)
    );
    assert!(get("probe_conditional").guaranteed.is_empty());
    assert!(get("free").guaranteed.keys().any(|e| e.kind == Kind::Free
        && e.guard.is_none()
        && e.target.root == Root::Parameter(0)
        && e.target.steps.is_empty()));
    let table_effects: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.as_str() == "table_add" && !p.is_poly)
            .filter_map(|(i, _)| summaries.procs.get(&ProcId(i as u32)))
            .collect();
    assert!(
        table_effects
            .iter()
            .any(|s| s.may.keys().any(|e| e.kind == Kind::Free)),
        "{table_effects:?}"
    );
    assert!(
        table_effects.iter().any(|s| !s.returns.is_empty()),
        "{table_effects:?}"
    );
    let reserves: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                p.name.as_str() == "array_reserve"
                    && !p.is_poly
                    && !c.proc_has_note(ProcId(*i as u32), "GrowsArray")
            })
            .filter_map(|(i, _)| summaries.procs.get(&ProcId(i as u32)))
            .collect();
    assert!(!reserves.is_empty());
    assert!(
        reserves
            .iter()
            .all(|s| s.guaranteed.keys().any(|e| e.kind == Kind::Resize))
    );
}

#[test]
fn prefix_compaction_preserves_independent_allocation_effects() {
    for kind in [Kind::Resize, Kind::Free] {
        let mut effects = HashMap::default();
        let parent = Effect {
            target: Path {
                root: Root::Parameter(0),
                steps: vec![Step::Storage],
            },
            kind,
            guard: None,
        };
        let mut child = parent.clone();
        child.target.steps.extend([Step::Element(0), Step::Storage]);
        insert(&mut effects, child.clone(), Vec::new());
        insert(&mut effects, parent.clone(), Vec::new());
        assert!(effects.contains_key(&parent));
        assert!(
            effects.contains_key(&child),
            "an outer allocation does not subsume a nested allocation"
        );
    }
}

#[test]
fn intrinsic_memory_and_vector_search_have_no_release_effects() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String("#import \"Basic\"; #import \"String\"; probe_search :: (s: string) -> int { return find_index_from_left(s, #char \"x\"); }".into())]).unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let (id, _) =
        c.pr.procs
            .iter()
            .enumerate()
            .find(|(_, p)| p.name.as_str() == "probe_search")
            .unwrap();
    let bad: Vec<_> = summaries.procs[&ProcId(id as u32)]
        .may
        .iter()
        .filter(|(e, _)| matches!(e.kind, Kind::Free | Kind::Unknown))
        .map(|(e, chain)| {
            (
                e,
                chain
                    .iter()
                    .map(|s| {
                        if *s == Span::NONE {
                            "...".to_owned()
                        } else {
                            c.src.sources.snippet(*s).to_owned()
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn pointee_resources_preserve_distinct_header_ownership() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String("release_handle :: (value: s32) {} @FreesArgument\nrelease_field :: (value: *s32) { release_handle(value.*); }".into())]).unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let id =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "release_field")
            .unwrap();
    let summary = &summaries.procs[&ProcId(id as u32)];
    assert!(
        summary
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.steps == [Step::Deref]),
        "{summary:?}"
    );
}

#[test]
fn allocator_epoch_requires_returned_context_storage_after_ir_release() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let stdlib = root.join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let mut source =
        std::fs::read_to_string(root.join("tests/lint/pointer_lifetime/strict/return-origins.jai"))
            .unwrap();
    source.push_str(r"
independent :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    return null;
}

incidental :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    unused := context.temporary_storage;
    return context.default_allocator.proc(mode, size, old_size, old_memory, data);
}

context_owned :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    return context.temporary_storage.data;
}
");
    c.begin_sources(&[ProgramSource::String(source)]).unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let before = Summaries::build(&c);
    assert!(!c.ed.facts().unwrap().lifetime_globals.is_empty());
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let names: HashSet<_> = after
        .allocators
        .temporary
        .iter()
        .map(|p| c.pr.procs[p.0 as usize].name.as_str())
        .collect();
    assert!(names.contains("temporary_allocator_proc"));
    assert!(names.contains("context_owned"), "{names:?}");
    assert!(!names.contains("incidental"));
    assert!(!names.contains("independent"));
    assert!(!after.allocators.heap_functions.is_empty());
}

#[test]
fn empty_error_results_preserve_body_inferred_storage_without_notes() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = r#"
        #import "Basic";
        empty_or_temp :: (fail: bool) -> string {
            if fail return "";
            return tprint("%", 42);
        }
        empty_or_copy :: (fail: bool) -> string {
            if fail return "";
            return copy_string(tprint("%", 42));
        }
        literal_or_temp :: (permanent: bool) -> string {
            if permanent return "permanent";
            return tprint("%", 42);
        }
        replace :: (destination: *string, value: string) { destination.* = value; }
        changed_empty :: (value: string) -> string {
            text := "";
            replace(*text, value);
            return text;
        }
        scratch_only :: () { text := tprint("%", 7); }
        scoped_scratch :: () { push_allocator(temp); memory := alloc(32); }
        release_only :: (memory: *void) { free(memory); }
        main :: () {}
    "#;
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    let original_context = c.ty.context_type.take().unwrap();
    let rebuilt_context = c.context_type(Span::NONE).unwrap();
    assert_ne!(original_context, rebuilt_context);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        &after.procs[&ProcId(id as u32)]
    };
    for (name, expected) in [
        ("empty_or_temp", Root::Temporary),
        ("tprint", Root::Temporary),
        ("empty_or_copy", Root::CallerAllocator),
        ("builder_to_string", Root::CallerAllocator),
    ] {
        let summary = get(name);
        assert_eq!(
            summary.returns.get(&0).map(|(p, _)| &p.root),
            Some(&expected),
            "{name}: {summary:?}"
        );
    }
    assert!(get("literal_or_temp").returns.is_empty());
    assert!(get("literal_or_temp").may_returns.contains(&0));
    assert!(get("changed_empty").returns.is_empty());
    for name in ["scratch_only", "scoped_scratch", "empty_or_copy"] {
        let summary = get(name);
        assert!(
            summary
                .allocation_effects
                .contains(&(Root::Temporary, None)),
            "{name}: {summary:?}"
        );
    }
    assert!(
        get("empty_or_copy")
            .allocation_effects
            .contains(&(Root::CallerAllocator, None))
    );
    assert!(get("release_only").allocation_effects.is_empty());
    let macros: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.as_str() == "push_allocator")
            .collect();
    assert!(!macros.is_empty());
    for (id, _) in macros {
        assert!(after.procs[&ProcId(id as u32)].pushes_context);
    }
}

fn without_body_notes(c: &mut Compiler) {
    let valid: HashSet<_> =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .iter()
            .filter(|(id, b)| {
                c.pr.procs[id.0 as usize].body_state == jaic::sema::procs::BodyState::Done
                    && std::sync::Arc::ptr_eq(&c.pr.procs[id.0 as usize].lit, &b.syntax)
            })
            .map(|(id, _)| *id)
            .collect();
    // Remove every body annotation after checking, including annotations on
    // imported polymorphic declarations. Bodyless primitive contracts survive.
    for i in 0..c.pr.procs.len() {
        if c.pr.procs[i].lit.body.is_none() {
            continue;
        }
        let mut lit = (*c.pr.procs[i].lit).clone();
        let mut header = (*lit.header).clone();
        header.notes.clear();
        lit.header = std::sync::Arc::new(header);
        c.pr.procs[i].lit = std::sync::Arc::new(lit);
        c.pr.proc_decl_notes.remove(&ProcId(i as u32));
    }
    let syntax: Vec<_> = c.pr.procs.iter().map(|p| p.lit.clone()).collect();
    for (id, body) in &mut c.ed.facts_mut().unwrap().lifetime_bodies {
        if valid.contains(id) {
            body.syntax = syntax[id.0 as usize].clone();
        }
    }
}

#[test]
fn output_and_exit_effects_follow_bodies_and_standard_handle_values() {
    for windows in [false, true] {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        if windows {
            options.os = jaic::target::TargetOs::Windows;
            options.cpu = jaic::target::TargetCpu::X64;
        }
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
        facts.lint = true;
        c.ed.record(facts);
        let source = r#"
            #import "Basic";
            lib :: #system_library "c";
            raw_write :: (fd: s32, data: *u8, count: u64) -> s64 #foreign lib "write"; @WritesOutput(0)
            raw_exit :: (code: s32) #foreign lib "exit"; @Exits
            select_output :: (error: bool) {
                fd: s32 = 1;
                if error fd = 2;
                raw_write(fd, "x".data, 1);
            }
            file_output :: (fd: s32) { raw_write(fd, "x".data, 1); }
            not_output :: () { raw_write(9, "x".data, 1); }
            change :: (p: *s32) { p.* = 9; }
            changed_output :: () { fd: s32 = 1; change(*fd); raw_write(fd, "x".data, 1); }
            wrapped_print :: () { print("%", 7); }
            wrapped_exit :: () { raw_exit(1); }
            conditional_exit :: (flag: bool) { if flag raw_exit(1); }
            all_exit :: (flag: bool) { if flag raw_exit(1); else raw_exit(2); }
            never :: () { while true {} }
            may_diverge :: (flag: bool) { while flag {} raw_exit(1); }
            unreachable_output :: () { raw_exit(1); print("unreachable"); }
            main :: () {}
        "#;
        c.begin_sources(&[ProgramSource::String(source.into())])
            .unwrap();
        c.finish_program().unwrap();
        let diagnostics = c.ide_check_all();
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        without_body_notes(&mut c);
        let before = Summaries::build(&c);
        c.release_lowered_code();
        let after = Summaries::build(&c);
        assert_eq!(before.procs, after.procs);
        let get = |name: &str| {
            let id =
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap();
            &after.procs[&ProcId(id as u32)]
        };
        for name in ["select_output", "wrapped_print"] {
            assert!(
                get(name).observable.output,
                "windows={windows}, {name}: {:?}",
                get(name)
            );
        }
        for name in ["file_output", "not_output", "changed_output"] {
            assert!(
                !get(name).observable.output,
                "windows={windows}, {name}: {:?}",
                get(name)
            );
        }
        for name in ["wrapped_exit", "all_exit", "unreachable_output"] {
            assert!(
                get(name).observable.exit,
                "windows={windows}, {name}: {:?}",
                get(name)
            );
        }
        for name in ["wrapped_print", "conditional_exit", "never", "may_diverge"] {
            assert!(
                !get(name).observable.exit,
                "windows={windows}, {name}: {:?}",
                get(name)
            );
        }
        assert!(!get("unreachable_output").observable.output);
    }
}

#[test]
fn release_may_provenance_follows_cursor_and_ambiguous_copies_without_guarantees() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = r#"
        #import "Basic";
        Node :: struct { next: *Node; }
        Holder :: struct { head: *Node; }
        release_list :: (holder: *Holder) {
            node := holder.head;
            while node {
                next := node.next;
                free(node);
                node = next;
            }
            holder.head = null;
        }
        release_selected :: (a: *void, b: *void, choose: bool) {
            selected := a;
            if choose selected = b;
            free(selected);
        }
        main :: () {}
    "#;
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let summaries = Summaries::build(&c);
    for (name, formals) in [("release_list", vec![0]), ("release_selected", vec![0, 1])] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let summary = &summaries.procs[&ProcId(id as u32)];
        for formal in formals {
            assert!(
                summary
                    .may
                    .keys()
                    .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(formal)),
                "{name}: {summary:?}"
            );
        }
        assert!(
            !summary.guaranteed.keys().any(|e| e.kind == Kind::Free),
            "{name}: {summary:?}"
        );
    }
}

#[test]
fn foreign_release_protocols_propagate_without_wrapper_notes() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = r#"
        #import "Basic";
        lib :: #system_library "c";
        raw_free :: (memory: *void) #foreign lib "free"; @FreesArgument
        raw_close :: (handle: s32) -> s32 #foreign lib "close"; @MaybeFreesArgument
        Handle :: struct { value: s32; }
        release_memory :: (memory: *void) { raw_free(memory); }
        release_handle :: (handle: *Handle) {
            if handle.value >= 0 raw_close(handle.value);
            handle.value = -1;
        }
        free :: (handle: *Handle) { handle.value = 0; }
        main :: () {}
    "#;
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let source_file =
        c.pr.procs
            .iter()
            .find(|p| p.name.as_str() == "release_memory")
            .unwrap()
            .span
            .file;
    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name && p.lit.header.span.file == source_file)
                .unwrap();
        &after.procs[&ProcId(id as u32)]
    };
    assert!(
        get("release_memory")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0))
    );
    assert!(
        get("release_handle")
            .may
            .keys()
            .any(|e| e.kind == Kind::Free
                && e.target.root == Root::Parameter(0)
                && !e.target.steps.is_empty())
    );
    assert!(
        !get("release_handle")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free)
    );
    assert!(!get("free").may.keys().any(|e| e.kind == Kind::Free));
}

#[test]
fn bodyless_pointer_contracts_follow_the_lexical_interface_not_abi_identity() {
    for known_body in [false, true] {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
        facts.lint = true;
        c.ed.record(facts);
        let mut source = r"
            Release :: #type (memory: *void) #c_call; @FreesArgument
            Plain :: #type (memory: *void) #c_call;
            #assert Release == Plain;
        "
        .to_string();
        if known_body {
            source.push_str(
                r"
                noop :: (memory: *void) #c_call {}
                invoke_known :: (memory: *void) {
                    callback: Release = noop;
                    callback(memory);
                }
            ",
            );
        } else {
            source.push_str(
                r"
                Table :: struct { release: Release; plain: Plain; }
                invoke_release :: (table: *Table, memory: *void) { table.release(memory); }
                invoke_plain :: (table: *Table, memory: *void) { table.plain(memory); }
            ",
            );
        }
        source.push_str("main :: () {}\n");
        c.begin_sources(&[ProgramSource::String(source)]).unwrap();
        c.finish_program().unwrap();
        let diagnostics = c.ide_check_all();
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        without_body_notes(&mut c);
        let before = Summaries::build(&c);
        c.release_lowered_code();
        let after = Summaries::build(&c);
        assert_eq!(before.procs, after.procs);
        let get = |name: &str| {
            let id =
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap();
            &after.procs[&ProcId(id as u32)]
        };
        if known_body {
            assert!(!get("invoke_known").may.keys().any(|e| e.kind == Kind::Free));
        } else {
            assert!(
                get("invoke_release")
                    .guaranteed
                    .keys()
                    .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(1)),
                "{:?}",
                get("invoke_release")
            );
            assert!(!get("invoke_plain").may.keys().any(|e| e.kind == Kind::Free));
        }
    }
}

#[test]
fn null_cleanup_paths_are_vacuous_but_unrelated_conditions_remain_possible() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        nullable :: (memory: *void) { if memory != null free(memory); }
        inverse :: (memory: *void) { if memory == null return; free(memory); }
        conditional :: (memory: *void, release: bool) { if release free(memory); }
        Other :: struct { a: *void; b: *void; }
        distinct :: (p: *Other) { if p.a != null free(p.b); }
        lib :: #system_library "c";
        raw_close :: (handle: s32) -> s32 #foreign lib "close"; @MaybeFreesArgument
        zero_descriptor :: (handle: s32) { if handle != 0 raw_close(handle); }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let summaries = Summaries::build(&c);
    for (name, definite) in [
        ("nullable", true),
        ("inverse", true),
        ("conditional", false),
        ("distinct", false),
        ("zero_descriptor", false),
    ] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let s = &summaries.procs[&ProcId(id as u32)];
        assert_eq!(
            s.guaranteed.keys().any(|e| e.kind == Kind::Free),
            definite,
            "{name}: {s:?}"
        );
    }
}

#[test]
fn minimum_and_maximum_effects_come_from_comparisons_and_component_choices() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(r#"
        #import "Basic";
        lesser :: (a: int,b: int)->int {if a < b return a; return b;}
        greater :: (a: int,b: int)->int {if a < b return b; return a;}
        Pair :: struct {x,y:int; #overlay(x) components:[2]int;}
        component_min :: (a:Pair,b:Pair)->Pair {return .{lesser(a.x,b.x),lesser(a.y,b.y)};}
        wrong_components :: (a:Pair,b:Pair)->Pair {return .{lesser(a.y,b.y),lesser(a.x,b.x)};}
        mixed_components :: (a:Pair,b:Pair)->Pair {return .{lesser(a.x,b.x),greater(a.y,b.y)};}
        reduction :: (a:int, values:..int)->int {result:=a; for value:values if value < result result=value; return result;}
        unrelated :: (a:int,b:int)->int {return a;}
        main :: () {}
    "#.into())]).unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let before = Summaries::build(&c);
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    for (name, expected) in [
        ("lesser", Some(Ordering::Minimum)),
        ("greater", Some(Ordering::Maximum)),
        ("component_min", Some(Ordering::Minimum)),
        ("reduction", Some(Ordering::Minimum)),
        ("wrong_components", None),
        ("mixed_components", None),
        ("unrelated", None),
    ] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        assert_eq!(after.procs[&ProcId(id as u32)].ordering, expected, "{name}");
    }
}

#[test]
fn destroy_heap_releases_allocator_data_without_treating_it_as_an_allocation() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        dispose :: (allocator:Allocator) {allocator.proc(.DESTROY_HEAP,0,0,null,allocator.data);}
        query :: (allocator:Allocator) {allocator.proc(.CAPS,0,0,null,allocator.data);}
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    without_body_notes(&mut c);
    let summaries = Summaries::build(&c);
    for (name, releases) in [("dispose", true), ("query", false)] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let s = &summaries.procs[&ProcId(id as u32)];
        assert_eq!(
            s.guaranteed
                .keys()
                .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0)),
            releases,
            "{s:?}"
        );
        assert!(s.allocation_effects.is_empty(), "{s:?}");
    }
}

#[test]
fn descriptor_fields_are_defined_before_their_uses_not_unrelated_error_returns() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        #import "File";
        direct :: (size:s64, fail:bool)->string,bool {
            if fail return "",false;
            memory:=cast(*u8) alloc(size);
            result:=string.{size,memory};
            if size>0 result.count=size-1;
            return result,true;
        }
        mixed :: (size:s64, choose:bool)->string {
            result:="static";
            if choose result=string.{size,cast(*u8) alloc(size)};
            return result;
        }
        forward :: (file:File)->string,bool {return read_entire_file(file);}
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
    c.release_lowered_code();
    let after = Summaries::build(&c);
    for (id, summary) in &before.procs {
        assert_eq!(
            Some(summary),
            after.procs.get(id),
            "{}",
            c.pr.procs[id.0 as usize].name.as_str()
        );
    }
    for (name, owned) in [("direct", true), ("mixed", false), ("forward", true)] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let s = &after.procs[&ProcId(id as u32)];
        assert_eq!(
            s.returns
                .get(&0)
                .is_some_and(|(p, _)| matches!(p.root, Root::CallerAllocator | Root::Heap)),
            owned,
            "{name}: {s:?}"
        );
    }
}

#[test]
fn direct_temporary_provider_calls_preserve_allocate_mode_without_tainting_caps_or_free() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        direct :: (n:s64)->*void {return temporary_allocator_proc(.ALLOCATE,n,0,null,null);}
        wrapper :: (n:s64)->*void {return talloc(n);}
        alias_wrapper :: (n:s64)->*void {return temporary_alloc(n);}
        caps :: ()->*void {return temporary_allocator_proc(.CAPS,0,0,null,null);}
        release :: ()->*void {return temporary_allocator_proc(.FREE,0,0,null,null);}
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
    for (id, summary) in &before.procs {
        assert_eq!(
            Some(summary),
            after.procs.get(id),
            "{}",
            c.pr.procs[id.0 as usize].name.as_str()
        );
    }
    for (name, temporary) in [
        ("direct", true),
        ("wrapper", true),
        ("alias_wrapper", true),
        ("caps", false),
        ("release", false),
    ] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let s = &after.procs[&ProcId(id as u32)];
        assert_eq!(
            s.returns
                .get(&0)
                .is_some_and(|(p, _)| p.root == Root::Temporary),
            temporary,
            "{name}: {s:?}"
        );
        assert_eq!(
            s.allocation_effects.contains(&(Root::Temporary, None)),
            temporary,
            "{name}: {s:?}"
        );
    }
}

#[test]
fn aligned_allocations_keep_their_origin_after_integer_round_up_and_ir_release() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        round_boundary :: (address:s64, alignment:s64)->s64 {
            assert(alignment>0);
            return ((address+alignment-1)/alignment)*alignment;
        }
        unrelated :: (address:s64, alignment:s64)->s64 {
            assert(alignment>0);
            return alignment;
        }
        aligned :: (n:s64)->*void {
            base:=alloc(n+63);
            return cast(*void) round_boundary(cast(s64) base,64);
        }
        temporary_aligned :: (n:s64)->*void {
            base:=talloc(n+63);
            return cast(*void) round_boundary(cast(s64) base,64);
        }
        not_an_address :: (n:s64)->*void {
            base:=alloc(n+63);
            return cast(*void) unrelated(cast(s64) base,64);
        }
        array :: (n:s64)->[]u8 {view,base:=NewArray(n,u8,alignment=64);return view;}
        mixed :: (n:s64, static_result:bool)->[]u8 {
            if static_result return u8.[1,2];
            view,base:=NewArray(n,u8,alignment=64);
            return view;
        }
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
    for (id, s) in &before.procs {
        assert_eq!(
            Some(s),
            after.procs.get(id),
            "{}",
            c.pr.procs[id.0 as usize].name.as_str()
        );
    }
    let temporary =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "temporary_aligned")
            .unwrap();
    assert!(
        after.procs[&ProcId(temporary as u32)]
            .returns
            .get(&0)
            .is_some_and(|(p, _)| p.root == Root::Temporary)
    );
    for (name, owned) in [
        ("aligned", true),
        ("not_an_address", false),
        ("array", true),
        ("mixed", false),
    ] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let s = &after.procs[&ProcId(id as u32)];
        assert_eq!(
            s.returns
                .get(&0)
                .is_some_and(|(p, _)| matches!(p.root, Root::Heap | Root::CallerAllocator)),
            owned,
            "{name}: {s:?}"
        );
    }
}

#[test]
fn temporary_descriptor_contents_do_not_replace_their_storage_origin() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        modified :: (text:string)->string {
            result:=copy_string(text,,temp);
            for *result {if it.*==0 it.*=1;}
            return result;
        }
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
    let summaries = Summaries::build(&c);
    assert_eq!(before.procs, summaries.procs);
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "modified")
            .unwrap() as u32,
    );
    let summary = &summaries.procs[&id];
    assert!(
        summary
            .returns
            .get(&0)
            .is_some_and(|(p, _)| p.root == Root::Temporary),
        "{summary:?}"
    );
}
