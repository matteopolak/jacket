//! The sidecar records checked instances and lowering choices, never sketches or failures.
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compile(source: &str, lint: bool, broken: bool) -> Compiler {
    let stdlib = test_support::repo::root().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = lint;
    c.ed.record(facts);
    let start = c.begin_sources(&[ProgramSource::String(source.to_owned())]);
    assert!(start.is_ok(), "{start:?}");
    let result = c.finish_program();
    assert_eq!(result.is_err(), broken, "{result:?}");
    let _ = c.ide_check_all();
    c
}

fn bodies(c: &Compiler) -> impl Iterator<Item = &LifetimeBody> {
    c.ed.facts().unwrap().lifetime_bodies.values()
}

fn events(body: &LifetimeBody) -> impl Iterator<Item = &Event> {
    body.blocks.iter().flat_map(|b| &b.events)
}

const POLY: &str = r"
grow :: (xs: *[..] int) {} @GrowsArray
poly :: ($G: bool) {
    xs: [..] int;
    p := *xs[0];
    #if G grow(*xs);
    p.* = 1;
}
unused :: (value: $T) { local := value; }
main :: () { poly(true); poly(false); }
";

#[test]
fn real_instantiations_keep_independent_choices_and_identities() {
    let mut c = compile(POLY, true, false);
    c.ide_sketch_polymorphs();
    let selected: Vec<_> = bodies(&c)
        .filter(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "poly")
        .collect();
    assert_eq!(selected.len(), 2);
    let mut choices = Vec::new();
    let mut identities = Vec::new();
    for b in selected {
        let taken = events(b)
            .find_map(|e| match e {
                Event::StaticChoice {
                    taken, ..
                } => Some(*taken),
                _ => None,
            })
            .unwrap();
        let grows = events(b)
            .filter(|e| matches!(e, Event::Grow { .. }))
            .count();
        assert_eq!(grows, usize::from(taken));
        choices.push(taken);
        identities.push(
            events(b)
                .find_map(|e| match e {
                    Event::Expression {
                        entity: Some(id), ..
                    } if c.entity(*id).name.as_str() == "xs" => Some(*id),
                    _ => None,
                })
                .unwrap(),
        );
        assert!(
            b.blocks
                .iter()
                .flat_map(|b| &b.successors)
                .all(|&i| i < b.blocks.len())
        );
    }
    choices.sort_unstable();
    assert_eq!(choices, [false, true]);
    assert_ne!(identities[0], identities[1]);
    assert!(!bodies(&c).any(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "unused"));
}

#[test]
fn lint_facts_off_allocate_no_sidecars() {
    let c = compile(POLY, false, false);
    assert_eq!(bodies(&c).count(), 0);
}

#[test]
fn failed_body_never_publishes_partial_events() {
    let c = compile("main :: () { n := 1; missing(); }", true, true);
    assert!(!bodies(&c).any(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main"));
}

#[test]
fn named_spread_and_deferred_arguments_keep_formal_mapping() {
    let c = compile(
        r"
        grow :: (xs: *[..] int, other: int, rest: ..int) {} @GrowsArray
        main :: () {
            xs: [..] int; rest: [2] int;
            grow(other = 3, xs = xx *xs, rest = ..rest);
        }
    ",
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    let args = events(b)
        .find_map(|e| match e {
            Event::Call {
                callee,
                args,
                ..
            } if c.pr.procs[callee.0 as usize].name.as_str() == "grow" => Some(args),
            _ => None,
        })
        .unwrap();
    assert_eq!(args.len(), 3);
    assert!(args.iter().all(|a| a.named));
    assert!(args.iter().any(|a| a.formal == 2 && a.spread));
    let region = events(b)
        .find_map(|e| match e {
            Event::Grow {
                region, ..
            } => Some(*region),
            _ => None,
        })
        .unwrap();
    assert!(args.iter().any(|a| a.formal == 0 && matches!(a.operand, LifetimeOperand::Value(v) if v == region)));
}

#[test]
fn expansions_and_cleanup_remain_in_the_actual_blocks() {
    let c = compile(
        r#"
        grow :: (xs: *[..] int) {} @GrowsArray
        marker :: () #expand { n := 1; }
        main :: () {
            xs: [..] int;
            marker();
            { defer grow(*xs); }
            #insert "grow(*xs);";
        }
    "#,
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(events(b).any(|e| matches!(
        e,
        Event::Call {
            expands: true,
            ..
        }
    )));
    assert!(events(b).any(|e| matches!(
        e,
        Event::Cleanup {
            deferred: Some(_),
            ..
        }
    )));
    assert!(events(b).any(|e| matches!(e, Event::Expansion { output, .. } if !output.is_empty())));
    assert_eq!(
        events(b)
            .filter(|e| matches!(e, Event::Grow { .. }))
            .count(),
        2
    );
}


#[test]
fn type_only_probe_does_not_emit_runtime_effects() {
    let c = compile(
        r"
        grow :: (xs: *[..] int) {} @GrowsArray
        main :: () { xs: [..] int; exists := #exists(grow(*xs)); }
    ",
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(!events(b).any(|e| matches!(e, Event::Grow { .. } | Event::Call { .. })));
}


#[test]
fn assignment_targets_and_comparisons_are_not_reads_but_their_operands_are() {
    let source = r"
        main :: () {
            p: *int; q: *int; pp := *p;
            p = q;
            p, q = q, p;
            p += 1;
            pp.* = q;
            same := p == q;
            other := p.* == q.*;
        }
    ";
    let c = compile(source, true, false);
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    for (fragment, expected) in [
        ("p = q", 1),
        ("p, q = q, p", 2),
        ("p += 1", 1),
        ("pp.* = q", 2),
        ("p == q", 0),
        ("p.* == q.*", 2),
    ] {
        let start = source.find(fragment).unwrap() as u32;
        let count = events(b).filter(|e| matches!(e, Event::Use { span, .. } if span.start >= start && span.end <= start + fragment.len() as u32)).count();
        assert_eq!(count, expected, "{fragment}");
    }
}

#[test]
fn field_identity_keeps_types_when_machine_addresses_overlap() {
    let c = compile(
        r"
        Box :: struct { values: [..] int; p: *int; }
        probe :: () {
            box: Box;
            view: [] int = box.values;
            box.p = box.values.data;
            q := box.p + 1;
        }
        main :: () { probe(); }
        ",
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    assert!(events(b).any(|e| matches!(e,
        Event::Field { base: LifetimeOperand::Place(base), result: LifetimeOperand::Place(result), base_ty, result_ty, .. }
        if base == result && base_ty != result_ty
    )));
    assert!(events(b).any(|e| matches!(e, Event::View { .. })));
    assert!(events(b).any(|e| matches!(e, Event::Offset { offset: Some(1), stride, .. } if *stride == jaic::types::TypeId::S64)));
    assert!(events(b).any(|e| matches!(e, Event::Load { .. })));
    assert!(events(b).any(|e| matches!(e, Event::Store { .. })));
}

#[test]
fn field_targets_pointer_equality_and_view_metadata_are_not_memory_reads() {
    let source = r#"
        Box :: struct { p: *int; }
        main :: () {
            box: Box; q: *int; v: [] int; s: string;
            box.p = q;
            same := box.p == null;
            value := box.p.*;
            count := v.count;
            element := v[0];
            compared := s == "x";
        }
    "#;
    let c = compile(source, true, false);
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    for (fragment, expected) in [
        ("box.p = q", 1),
        ("box.p == null", 0),
        ("box.p.*", 1),
        ("v.count", 0),
        ("v[0]", 1),
        ("s == \"x\"", 1),
    ] {
        let start = source.find(fragment).unwrap() as u32;
        let count = events(b).filter(|e| matches!(e, Event::Use { span, ty, .. } if span.start >= start && span.end <= start + fragment.len() as u32 &&
                (c.ty.types.is_pointer(*ty) || matches!(c.ty.types.kind(*ty), jaic::types::TypeKind::String | jaic::types::TypeKind::Array { kind: jaic::types::ArrayKind::View, .. })))).count();
        assert_eq!(count, expected, "{fragment}");
    }
}

#[test]
fn argument_snapshots_before_later_calls_keep_load_and_default_facts() {
    let c = compile(
        r#"
        #import "Basic";
        later :: () -> int { return 0; }
        take :: (p: *int, n: int, mode: Allocator_Mode = .RESIZE) {}
        main :: () { p: *int; take(p, later()); }
        "#,
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    let list: Vec<_> = events(b).collect();
    let (call_at, args) = list
        .iter()
        .enumerate()
        .find_map(|(i, e)| match e {
            Event::Call {
                callee,
                args,
                ..
            } if c.pr.procs[callee.0 as usize].name.as_str() == "take" => Some((i, args)),
            _ => None,
        })
        .unwrap();
    let LifetimeOperand::Value(first) = args.iter().find(|a| a.formal == 0).unwrap().operand else {
        panic!("argument value");
    };
    let load = list
        .iter()
        .position(|e| matches!(e, Event::Load { value, .. } if *value == first))
        .unwrap();
    let later = list.iter().position(|e| matches!(e, Event::Call { callee, .. } if c.pr.procs[callee.0 as usize].name.as_str() == "later")).unwrap();
    assert!(load < later && later < call_at);
    assert_eq!(
        args.iter().find(|a| a.formal == 2).unwrap().constant,
        Some(1)
    );
    assert!(b.blocks.iter().any(|b| b.normal_return));
    assert!(!b.stack_places.is_empty());
}

#[test]
fn caller_owned_cleanup_and_any_backing_storage_keep_semantic_lifetimes() {
    let c = compile(
        r#"
        #import "Basic";
        always :: () -> string {
            auto_release_temp();
            return tprint("released");
        }
        maybe :: (flag: bool) -> string {
            if flag auto_release_temp();
            return tprint("possibly released");
        }
        number :: () -> int { return 7; }
        boxes :: () {
            local := 3;
            a: Any = local;
            b: Any = number();
        }
        main :: () {}
        "#,
        true,
        false,
    );
    let exits_without_cleanup = |body: &LifetimeBody| {
        let mut seen = jaic::fxhash::HashSet::default();
        let mut queue = std::collections::VecDeque::from([0]);
        while let Some(i) = queue.pop_front() {
            if !seen.insert(i)
                || body.blocks[i]
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::Cleanup { .. }))
            {
                continue;
            }
            if body.blocks[i].normal_return {
                return true;
            }
            queue.extend(&body.blocks[i].successors);
        }
        false
    };
    let get = |name| {
        bodies(&c)
            .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == name)
            .unwrap()
    };
    assert!(!exits_without_cleanup(get("always")));
    assert!(exits_without_cleanup(get("maybe")));
    assert!(events(get("always")).any(|e| matches!(
        e,
        Event::Return {
            ty: jaic::types::TypeId::STRING,
            ..
        }
    )));
    let boxes: Vec<_> = events(get("boxes"))
        .filter_map(|e| match e {
            Event::BoxAny {
                temporary, ..
            } => Some(*temporary),
            _ => None,
        })
        .collect();
    assert_eq!(boxes, [false, true]);
}

#[test]
fn baked_temporary_strings_own_static_bytes() {
    compile(
        r#"
        #import "Basic";
        baked := tprint("baked %", 7);
        #run {
            saved := baked;
            reset_temporary_storage();
            assert(saved == "baked 7");
            assert(baked == "baked 7");
        }
        main :: () {}
        "#,
        true,
        false,
    );
}

#[test]
fn relative_pointer_syntax_has_no_selected_type_in_this_compiler() {
    // The lifetime lint cannot add a language feature: `*~T` is rejected by sema.
    let c = compile("main :: () { p: *~s32; }", true, true);
    assert!(!bodies(&c).any(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main"));
}


#[test]
fn clearing_lifetimes_releases_bodies_and_their_file_index() {
    let mut c = compile(POLY, true, false);
    let file = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "poly")
        .unwrap()
        .syntax
        .header
        .span
        .file;
    let facts = c.ed.facts_mut().unwrap();
    assert!(facts.lifetime_bodies_in(file).next().is_some());
    let refs = facts.refs.len();
    facts.clear_lifetimes();
    assert_eq!(facts.lifetime_bodies.capacity(), 0);
    assert!(facts.lifetime_bodies_in(file).next().is_none());
    assert_eq!(facts.refs.len(), refs);
    facts.clear_lifetimes();
}

#[test]
fn runtime_context_selection_and_restoration_are_recorded() {
    let c = compile(
        r#"
#import "Basic";
main :: () {
    push_context {
        local: int;
        push_context { context.logger_data = *local; }
        context.logger_data = *local;
    }
}
"#,
        true,
        false,
    );
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(events(body).any(|e| matches!(e, Event::ContextSet { .. })));
    assert!(events(body).any(|e| matches!(e, Event::ContextRead { .. })));
    assert!(events(body).any(|e| matches!(e, Event::ContextAlias { .. })));
    assert!(events(body).any(|e| matches!(e, Event::Store { ty, .. } if matches!(c.ty.types.kind(*ty), jaic::types::TypeKind::Struct(_)))));
}

#[test]
fn selected_memory_intrinsics_record_read_write_and_count_formals() {
    let c = compile(
        r#"
#import "Basic";
Fiber_Context :: struct { sp: *void; stack_low: *void; stack_high: *void; stack_dealloc: *void; }
fiber_free :: (fiber: *Fiber_Context) #intrinsic;
main :: () {
    fiber: Fiber_Context;
    fiber_free(*fiber);
    source: [8] u8;
    destination: [8] u8;
    memcpy(destination.data, source.data, 8);
    memset(destination.data, 0, 8);
}
"#,
        true,
        false,
    );
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    let memory: Vec<_> = events(body)
        .filter_map(|e| match e {
            Event::Call {
                callee,
                memory: Some(m),
                ..
            } => Some((c.pr.procs[callee.0 as usize].name.as_str(), *m)),
            _ => None,
        })
        .collect();
    assert!(memory.iter().any(|(name, m)| *name == "memcpy"
        && m.reads == 0b10
        && m.writes == 0b1
        && m.count == Some(2)));
    assert!(memory.iter().any(|(name, m)| *name == "memset"
        && m.reads == 0
        && m.writes == 0b1
        && m.count == Some(2)));
    assert!(memory.iter().any(|(name, m)| *name == "fiber_free"
        && m.reads == 0
        && m.writes == 0
        && m.count.is_none()));
}

#[test]
fn addressing_an_annotated_procedure_slot_records_its_real_storage() {
    let c = compile(
        r"
F :: (value := 1) -> int;
identity :: (value: int) -> int { return value; }
slot: F = identity;
take :: (p: *F) {}
main :: () { take(*slot); }
",
        true,
        false,
    );
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(events(body).any(|e| matches!(e, Event::Address { ty, .. } if matches!(c.ty.types.kind(*ty), jaic::types::TypeKind::Proc(_)))));
}

#[test]
fn default_images_are_recorded_as_writes_at_the_actual_initialization() {
    let c = compile("main :: () { text: string; copy := text; }", true, false);
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    let ordered: Vec<_> = events(body).collect();
    let at = ordered
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Initialize {
                    ty: jaic::types::TypeId::STRING,
                    ..
                }
            )
        })
        .unwrap();
    let first_load = ordered
        .iter()
        .position(|e| {
            matches!(
                e,
                Event::Load {
                    ty: jaic::types::TypeId::STRING,
                    ..
                }
            )
        })
        .unwrap();
    assert!(at < first_load);
}
