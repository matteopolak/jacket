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
    assert!(events(b).any(|e| matches!(e, Event::Copy { .. })));
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
