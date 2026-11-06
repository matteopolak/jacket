//! Selected typed fact, identity and executable-release parity regressions.
use super::{bodies, compile, events};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event};
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn semantic_boolean_relations_survive_executable_body_release() {
    use jaic::ide::lifetimes::LifetimeBoolean as Boolean;
    let mut c = compile(
        r"
        probe_boolean :: (a: bool, b: bool) -> bool, bool, bool {
            return !a, a && b, a || b;
        }
        probe_byte_xor :: (a: u8) -> u8 { return a ^ 1; }
        main :: () {}
    ",
        true,
        false,
    );
    c.release_lowered_code();
    let body = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_boolean")
        .unwrap();
    for kind in [0, 1, 2] {
        assert!(events(body).any(|event| matches!(event, Event::Boolean {operation,..} if matches!((kind,operation),(0,Boolean::Not(_))|(1,Boolean::And(_,_))|(2,Boolean::Or(_,_))))), "missing semantic boolean relation {kind}: {:?}",body.blocks);
    }
    for event in events(body) {
        if let Event::Boolean {
            operation: Boolean::And(left, _) | Boolean::Or(left, _),
            ..
        } = event
        {
            assert!(
                events(body)
                    .any(|event| matches!(event,Event::Branch {condition,..} if condition==left)),
                "short-circuit evaluation must retain its actual branch"
            );
        }
    }
    let bytes = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_byte_xor")
        .unwrap();
    assert!(
        !events(bytes).any(|event| matches!(event, Event::Boolean { .. })),
        "integer XOR is not a semantic bool negation"
    );
}


#[test]
fn selected_projection_and_dynamic_element_delta_survive_body_release() {
    let mut c = compile(
        r"
        probe_elements :: (p: *[..] s64, index: int, delta: int) -> *s64, s64 {
            count := p.count;
            chosen := *p.data[index];
            advanced := p.data + delta;
            backward := p.data - delta;
            return chosen, count;
        }
        main :: () {}
    ",
        true,
        false,
    );
    c.release_lowered_code();
    let body = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_elements")
        .unwrap();
    let projections: Vec<_> = events(body)
        .filter_map(|event| {
            if let Event::FieldProjection {
                base,
                base_ty,
                result_ty,
                offset,
                ..
            } = event
            {
                Some((*base, *base_ty, *result_ty, *offset))
            } else {
                None
            }
        })
        .collect();
    assert!(
        projections.len() >= 4,
        "selected inline projection facts missing: {:?}",
        body.blocks
    );
    let formal_slot = events(body)
        .find_map(|event| {
            if let Event::Parameter {
                formal: 0,
                place,
                ..
            } = event
            {
                Some(*place)
            } else {
                None
            }
        })
        .unwrap();
    assert!(
        projections
            .iter()
            .all(|(base, ty, _, _)| *base != formal_slot
                && matches!(
                    c.ty.types.kind(*ty),
                    jaic::types::TypeKind::Array {
                        kind: jaic::types::ArrayKind::Resizable,
                        ..
                    }
                )),
        "projection base must be the captured descriptor address, not its pointer variable"
    );
    let deltas: Vec<_> = events(body)
        .filter_map(|event| {
            if let Event::Offset {
                offset,
                subscript,
                ..
            } = event
            {
                Some((*offset, *subscript))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(deltas.len(), 3, "{deltas:?}");
    assert_eq!(
        deltas
            .iter()
            .filter(|(_, subscript)| subscript.is_some())
            .count(),
        2,
        "indexing/addition preserve their selected dynamic delta; subtraction is unknown until signed negation is represented: {deltas:?}"
    );
    assert!(
        deltas.iter().all(|(offset, _)| offset.is_none()),
        "these are dynamic element offsets"
    );
}


#[test]
fn checked_arithmetic_width_and_shifted_pointer_cast_are_not_erased() {
    let mut c = compile(
        r"
        Item :: struct {pad:s64; #as inner:s64;}
        probe_shift :: (p:*Item) -> *s64 {return cast(*s64)p;}
        probe_width :: (a:s8) -> s8 {return a+1;}
        main :: () {}
    ",
        true,
        false,
    );
    c.release_lowered_code();
    let shift = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_shift")
        .unwrap();
    let (base, result) = events(shift)
        .find_map(|event| {
            if let Event::FieldProjection {
                base,
                result,
                offset: 8,
                ..
            } = event
            {
                Some((*base, *result))
            } else {
                None
            }
        })
        .expect("the checked #as member displacement must be retained");
    assert!(
        !events(shift).any(
            |event| matches!(event,Event::Copy {value,source} if *value==result && *source==base)
        ),
        "different addresses are not a semantic Copy"
    );
    let width = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_width")
        .unwrap();
    assert!(events(width).any(|event|matches!(event,Event::IntegerBinary {ty,..} if c.ty.types.int_info(*ty)==Some((8,true)))),"integer facts retain checked operation width/sign");
}


#[test]
fn selected_copy_semantics_and_literal_projection_survive_body_release() {
    let mut c = compile(
        r"
        lookalike_copy :: (destination:*void,source:*void,count:int) {}
        probe_copy :: (destination:*void,source:*void,count:int) {
            memcpy(destination,source,count);
            memset(destination,0,count);
            lookalike_copy(destination,source,count);
        }
        probe_literal :: (data:*u8,count:int)->string {return .{count,data};}
        main :: () {}
    ",
        true,
        false,
    );
    c.release_lowered_code();
    let copy = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_copy")
        .unwrap();
    let copies: Vec<_> = events(copy)
        .filter_map(|event| {
            if let Event::Call {
                memory: Some(memory),
                ..
            } = event
            {
                memory.copy()
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        copies.len(),
        1,
        "read/write masks alone are not byte equality: {:?}",
        copy.blocks
    );
    assert_eq!(
        (
            copies[0].destination_formal,
            copies[0].source_formal,
            copies[0].count_formal
        ),
        (0, 1, 2)
    );
    assert!(
        !copies[0].requires_disjoint,
        "selected intrinsic has memmove semantics"
    );
    assert!(events(copy).any(|event|matches!(event,Event::Call {callee,memory:None,..} if c.proc(*callee).name.as_str()=="lookalike_copy")),"ordinary lookalike has no primitive copy fact");
    let literal = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe_literal")
        .unwrap();
    assert!(events(literal).any(|event|matches!(event,Event::FieldProjection {base_ty,result_ty,offset:8,..} if *base_ty==jaic::types::TypeId::STRING && c.ty.types.pointee(*result_ty)==Some(jaic::types::TypeId::U8))),"literal pointer-slot projection must be selected while checking: {:?}",literal.blocks);
}

#[test]
fn stack_roots_retain_original_extents_without_derived_addresses() {
    let mut c = compile(
        r"
        Large :: struct { pad: u64; field: u64; }
        probe :: (index: s64) {
            bytes: [16] u8;
            (cast(*u8)*bytes + index).* = 1;
            small: u8;
            (cast(*Large)*small).field = 0;
        }
    ",
        true,
        false,
    );
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    assert!(body.stack_roots.iter().any(|root| root.bytes == Some(16)));
    assert!(body.stack_roots.iter().any(|root| root.bytes == Some(1)));
    let roots: Vec<_> = body.stack_roots.iter().map(|root| root.value).collect();
    for event in events(body) {
        match event {
            Event::Offset {
                value,
                source,
                ..
            } if value != source => assert!(!roots.contains(value)),
            Event::FieldProjection {
                base,
                result,
                offset,
                ..
            } if base != result && *offset > 0 => assert!(!roots.contains(result)),
            _ => {}
        }
    }
    assert!(
        body.slot_aliases
            .iter()
            .all(|(alias, canonical)| !roots.contains(alias) && roots.contains(canonical))
    );
    let expected = body.stack_roots.clone();
    c.release_lowered_code();
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    assert_eq!(body.stack_roots, expected);
}

#[test]
fn allocator_protocol_retains_selected_nominal_tags_roles_and_release_parity() {
    let mut c = compile(
        r"
        probe :: (a:Allocator) {
            a.proc(allocator_data=a.data,old_memory=null,mode=.ALLOCATE,old_size=0,size=8);
        }
    ",
        true,
        false,
    );
    let selected = |c: &Compiler| {
        let body = bodies(c)
            .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
            .unwrap();
        events(body)
            .find_map(|event| match event {
                Event::IndirectCall {
                    allocator: true,
                    allocator_protocol: Some(protocol),
                    args,
                    ..
                } => {
                    assert_eq!(
                        args.iter()
                            .find(|arg| arg.formal == protocol.operation_formal)
                            .unwrap()
                            .constant,
                        Some(protocol.allocate_value)
                    );
                    assert_eq!(protocol.operation_formal, 0);
                    assert_eq!(protocol.size_formal, 1);
                    assert_eq!(protocol.old_size_formal, 2);
                    assert_eq!(protocol.old_memory_formal, 3);
                    assert_eq!(
                        args.iter()
                            .find(|arg| arg.formal == protocol.size_formal)
                            .unwrap()
                            .constant,
                        Some(8)
                    );
                    assert_eq!(
                        args.iter()
                            .find(|arg| arg.formal == protocol.old_size_formal)
                            .unwrap()
                            .constant,
                        Some(0)
                    );
                    assert!(
                        c.ty.types.is_pointer(
                            args.iter()
                                .find(|arg| arg.formal == protocol.old_memory_formal)
                                .unwrap()
                                .ty
                        )
                    );
                    assert_eq!(protocol.data_formal, 4);
                    assert_eq!(protocol.result_position, 0);
                    Some(**protocol)
                }
                _ => None,
            })
            .expect("selected allocator ABI must carry its checked invocation roles")
    };
    let expected = selected(&c);
    assert_eq!(
        (
            expected.allocate_value,
            expected.resize_value,
            expected.free_value
        ),
        (0, 1, 2)
    );
    c.release_lowered_code();
    assert_eq!(selected(&c), expected);
}

#[test]
fn allocator_protocol_rejects_ambiguous_abis_and_preserves_changed_tags() {
    use jaic::sema::FileSystem;
    use std::path::{Path, PathBuf};
    struct PreludeFs {
        allocation: PathBuf,
        bytes: Vec<u8>,
    }
    impl FileSystem for PreludeFs {
        fn read(&self, path: &Path) -> Option<Vec<u8>> {
            if NativeFs.canonical(path) == self.allocation {
                Some(self.bytes.clone())
            } else {
                NativeFs.read(path)
            }
        }
        fn is_file(&self, path: &Path) -> bool {
            NativeFs.is_file(path)
        }
        fn is_dir(&self, path: &Path) -> bool {
            NativeFs.is_dir(path)
        }
        fn canonical(&self, path: &Path) -> PathBuf {
            NativeFs.canonical(path)
        }
    }
    let stdlib = test_support::repo::root().join("stdlib");
    let allocation = stdlib.join("prelude/allocation.jai");
    let original = std::fs::read_to_string(&allocation).unwrap();
    let changed = original
        .replace("ALLOCATE :: 0;", "ALLOCATE :: 101;")
        .replace("RESIZE :: 1;", "RESIZE :: 103;")
        .replace("FREE :: 2;", "FREE :: 107;");
    let negative = original
        .replace("ALLOCATE :: 0;", "ALLOCATE :: -11;")
        .replace("RESIZE :: 1;", "RESIZE :: -13;")
        .replace("FREE :: 2;", "FREE :: -17;");
    let cases = vec![
        (changed, Some((101, 103, 107))),
        (
            negative.clone(),
            Some(((-11i64) as u64, (-13i64) as u64, (-17i64) as u64)),
        ),
        (
            negative.replace("Allocator_Mode :: enum {", "Allocator_Mode :: enum s8 {"),
            Some((245, 243, 239)),
        ),
        (original.replace("RESIZE :: 1;", "RESIZE :: 0;"), None),
        (
            original.replace("allocator_data: *void", "user_data: *void"),
            None,
        ),
        (
            original.replace(") -> *void;", ") -> *void #no_context;"),
            None,
        ),
        (original.replace(") -> *void;", ") -> *void #c_call;"), None),
        (
            original.replace(
                "Allocator_Mode :: enum {",
                "Allocator_Mode :: enum_flags u32 {",
            ),
            None,
        ),
        (
            format!(
                "Other_Mode :: enum {{ALLOCATE::0;RESIZE::1;FREE::2;}}\n{}",
                original.replace("mode: Allocator_Mode", "mode: Other_Mode")
            ),
            None,
        ),
    ];
    for (index, (allocation_source, expected)) in cases.into_iter().enumerate() {
        assert_ne!(
            allocation_source, original,
            "fixture {index} must change the selected prelude"
        );
        let fs = PreludeFs {
            allocation: NativeFs.canonical(&allocation),
            bytes: allocation_source.into_bytes(),
        };
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.runtime_support = false;
        options.set_stdlib(&fs, &stdlib);
        let mut c = Compiler::new(options, Rc::new(fs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
        facts.lint = true;
        c.ed.record(facts);
        c.begin_sources(&[ProgramSource::String(
            r"
            probe :: (a:Allocator) {a.proc(.ALLOCATE,8,0,null,a.data);}
            force_mode :: ()->Allocator_Mode {return .ALLOCATE;}
            main :: () {}
        "
            .into(),
        )])
        .unwrap();
        c.finish_program().unwrap();
        let diagnostics = c.ide_check_all();
        assert!(diagnostics.is_empty(), "fixture {index}: {diagnostics:?}");
        let body = bodies(&c)
            .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
            .unwrap();
        let protocols: Vec<_> = events(body)
            .filter_map(|event| match event {
                Event::IndirectCall {
                    allocator: true,
                    allocator_protocol,
                    ..
                } => Some(allocator_protocol.as_deref()),
                _ => None,
            })
            .collect();
        assert_eq!(protocols.len(), 1, "fixture {index}");
        let actual = protocols[0].map(|p| (p.allocate_value, p.resize_value, p.free_value));
        assert_eq!(actual, expected, "fixture {index}");
    }
}

#[test]
fn allocator_protocol_checks_actual_lexical_roles_not_only_equal_type() {
    let c = compile(
        r"
        Other_Callback :: #type (mode:Allocator_Mode,size:s64,old_size:s64,allocator_data:*void,old_memory:*void)->*void;
        probe :: (callback:Other_Callback) {callback(.ALLOCATE,8,0,null,null);}
    ",
        true,
        false,
    );
    let body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    let mut checked = false;
    for event in events(body) {
        if let Event::IndirectCall {
            allocator,
            allocator_protocol,
            ..
        } = event
        {
            assert!(
                *allocator,
                "fixture must share the old canonical structural type"
            );
            assert!(
                allocator_protocol.is_none(),
                "equal pointer parameter types do not certify swapped lexical data roles"
            );
            checked = true;
        }
    }
    assert!(checked);
}

#[test]
fn overlay_selection_barriers_keep_exact_checked_projection_origins() {
    let mut c = compile(
        r"
        Overlay :: struct {value:s64; #overlay(value) alternate:s64;}
        probe :: (p:*Overlay)->s64 { return p.alternate; }
        Indirect :: struct {using link:*Overlay; stamp:s64; #overlay(stamp) duplicate:s64;}
        probe_indirect :: (p:*Indirect)->s64 { return p.alternate; }
        probe_write :: (p:*Overlay) { p.alternate = 9; }
        choose :: (p:*Overlay)->*Overlay { p.value = 1; return p; }
        probe_side_effect :: (p:*Overlay)->s64 { return choose(p).alternate; }
        probe_asm :: () { #asm {nop;} }
        main :: () {}
    ",
        true,
        false,
    );
    let body = bodies(&c)
        .find(|body| c.pr.procs[body.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    assert_eq!(body.overlay_accesses.len(), 1);
    for access in &body.overlay_accesses {
        assert!(matches!(
            body.blocks[access.block].events[access.event],
            Event::Barrier
        ));
        assert!(
            matches!(body.blocks[access.projection_block].events[access.projection_event], Event::FieldProjection{result,base_ty,..} if result == access.result && base_ty == access.descriptor_ty)
        );
    }
    for name in ["probe_indirect", "probe_asm"] {
        let other = bodies(&c)
            .find(|body| c.pr.procs[body.procedure.0 as usize].name.as_str() == name)
            .unwrap();
        assert!(
            other.overlay_accesses.is_empty(),
            "{name}: unsupported projections and asm remain opaque"
        );
        assert!(
            events(other).any(|event| matches!(event, Event::Barrier)),
            "{name}: original barrier retained"
        );
    }
    for name in ["probe_write", "probe_side_effect"] {
        let other = bodies(&c)
            .find(|body| c.pr.procs[body.procedure.0 as usize].name.as_str() == name)
            .unwrap();
        assert_eq!(other.overlay_accesses.len(), 1);
        let access = other.overlay_accesses[0];
        if name == "probe_write" {
            assert!(
                other.blocks[access.block].events[access.event + 1..]
                    .iter()
                    .any(|event| matches!(
                        event,
                        Event::Store {
                            constant: Some(9),
                            ..
                        }
                    )),
                "selection metadata must not erase the overlapping write"
            );
        } else {
            assert!(other.blocks[access.projection_block].events[..access.projection_event].iter().any(|event| matches!(event,Event::Call{callee,..} if c.pr.procs[callee.0 as usize].name.as_str() == "choose")),"base expression call precedes the exact projection");
        }
    }
    let before = body.overlay_accesses.clone();
    let procedure = body.procedure;
    c.release_lowered_code();
    assert_eq!(
        c.ed.facts().unwrap().lifetime_bodies[&procedure].overlay_accesses,
        before
    );
}

#[test]
fn selected_default_images_keep_typed_values_at_the_write_epoch() {
    use jaic::ide::lifetimes::LifetimeDefaultValue;
    let mut c = compile(
        r"
        choose :: (value:s64)->s64 {return value;}
        replacement :: (value:s64)->s64 {return value+1;}
        Inner :: struct {tag:u8=13;}
        Marker :: #type,distinct s16;
        Hooks :: struct {callback:(value:s64)->s64=choose; marker:Marker=cast(Marker)-7; nested:[2]Inner;}
        #add_context selected_default_hook:Hooks=.{callback=replacement};
        probe :: () {hooks:Hooks; hooks.callback(2);}
        probe_context :: () #c_call {push_context {context.selected_default_hook.callback=choose;}}
        main :: () {}
    ",
        true,
        false,
    );
    let choose =
        c.pr.procs
            .iter()
            .position(|procedure| procedure.name.as_str() == "choose")
            .unwrap();
    let body = bodies(&c)
        .find(|body| c.pr.procs[body.procedure.0 as usize].name.as_str() == "probe")
        .unwrap();
    let images: Vec<_> = events(body)
        .filter_map(|event| match event {
            Event::DefaultImage {
                leaves,
                complete,
                ..
            } => Some((leaves.clone(), *complete)),
            _ => None,
        })
        .collect();
    assert!(
        images.iter().any(|(leaves, complete)| *complete
            && leaves.iter().any(|leaf| matches!(leaf.value,LifetimeDefaultValue::Procedure {procedure,function} if procedure==jaic::sema::ProcId(choose as u32) && matches!(c.pr.procs[choose].target,Some(jaic::sema::procs::ProcTarget::Func(actual)) if actual==function)))
            && leaves.iter().any(|leaf| leaf.width == 2
                && matches!(c.ty.types.kind(leaf.ty), jaic::types::TypeKind::Distinct(_))
                && leaf.value == LifetimeDefaultValue::Scalar(65529))
            && leaves
                .iter()
                .filter(|leaf| leaf.value == LifetimeDefaultValue::Scalar(13))
                .count()
                == 2),
        "selected checked callback and nested default bytes must survive"
    );
    for block in &body.blocks {
        for (ordinal, event) in block.events.iter().enumerate() {
            if let Event::DefaultImage {
                place,
                ty,
                ..
            } = event
            {
                assert!(
                    ordinal > 0
                        && matches!(block.events[ordinal-1],Event::Initialize{place:prior,ty:prior_ty,..} if prior==*place && prior_ty==*ty),
                    "values belong to this exact default write"
                );
            }
        }
    }
    let replacement =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "replacement")
            .unwrap();
    let context_body = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "probe_context")
        .unwrap();
    let context_type = c.ty.context_type.unwrap();
    assert!(events(context_body).any(|event|matches!(event,Event::DefaultImage{ty,leaves,..} if *ty==context_type && leaves.iter().any(|leaf|matches!(leaf.value,LifetimeDefaultValue::Procedure {procedure,function} if procedure==jaic::sema::ProcId(replacement as u32) && matches!(c.pr.procs[replacement].target,Some(jaic::sema::procs::ProcTarget::Func(actual)) if actual==function))))),"fresh selected context uses its actual overridden callback default");
    assert!(
        events(context_body).any(|event| matches!(event, Event::Store { .. })),
        "later context overwrite remains a separate effect"
    );
    assert!(
        !events(body).any(|event| matches!(event,Event::DefaultImage{ty,..} if *ty==context_type)),
        "ordinary incoming context is not default initialized by this body"
    );
    let procedure = body.procedure;
    c.release_lowered_code();
    let after: Vec<_> = events(&c.ed.facts().unwrap().lifetime_bodies[&procedure])
        .filter_map(|event| match event {
            Event::DefaultImage {
                leaves,
                complete,
                ..
            } => Some((leaves.clone(), *complete)),
            _ => None,
        })
        .collect();
    assert_eq!(after, images);
}

#[test]
fn selected_bounded_writes_are_not_inferred_from_memory_masks() {
    let mut c = compile(
        r#"#import "Basic";
        C :: #system_library "c";
        lookalike_fill :: (p:*void,value:s32,count:u64)->*void #foreign C "memset";
        byte_lookup :: (dest:*u8,table:*u8,indices:*u8,width:s64) #intrinsic;
        Fiber_Context :: struct {sp:*void;stack_low:*void;stack_high:*void;stack_dealloc:*void;}
        fiber_free :: (fiber:*Fiber_Context) #intrinsic;
        probe_fill :: (destination:*u8,source:*u8,count:s64) {
            memset(destination,1,count);
            memset(count=0,dest=null,value=0);
            memcpy(destination,source,count);
            memcmp(destination,source,count);
            byte_lookup(destination,source,source,16);
            atomic_store(cast(*s64)destination,0);
            fiber:Fiber_Context;fiber_free(*fiber);
            lookalike_fill(destination,1,cast(u64)count);
        }
        main :: (){}"#,
        true,
        false,
    );
    let collect = |c: &Compiler| {
        let body = bodies(c)
            .find(|b| c.proc(b.procedure).name.as_str() == "probe_fill")
            .unwrap();
        events(body)
            .filter_map(|event| {
                let Event::Call {
                    callee,
                    memory,
                    args,
                    ..
                } = event
                else {
                    return None;
                };
                Some((*callee, *memory, args.clone()))
            })
            .collect::<Vec<_>>()
    };
    let before = collect(&c);
    let writes: Vec<_> = before
        .iter()
        .filter_map(|(callee, memory, args)| {
            memory
                .and_then(|m| m.bounded_write())
                .map(|write| (*callee, write, args))
        })
        .collect();
    assert_eq!(
        writes.len(),
        2,
        "only actual memset calls establish a closed write range"
    );
    for (callee, write, _) in &writes {
        assert_eq!(c.proc(*callee).name.as_str(), "memset");
        assert_eq!((write.destination_formal, write.count_formal), (0, 2));
    }
    assert!(
        writes.iter().any(|(_, write, args)| args
            .iter()
            .any(|a| a.formal == usize::from(write.count_formal) && a.constant == Some(0))),
        "named zero-count call retains actual formal binding"
    );
    for (callee, memory, _) in &before {
        let name = c.proc(*callee).name.as_str();
        if name != "memset" {
            assert!(memory.is_none_or(|m| m.bounded_write().is_none()), "{name}");
        }
    }
    assert!(
        before.iter().any(
            |(callee, memory, _)| c.proc(*callee).name.as_str() == "lookalike_fill"
                && memory.is_none()
        )
    );
    eprintln!(
        "lifetime-layout memory={} event={} body={}",
        std::mem::size_of::<jaic::ide::lifetimes::LifetimeMemoryEffects>(),
        std::mem::size_of::<Event>(),
        std::mem::size_of::<LifetimeBody>()
    );
    c.release_lowered_code();
    let after = collect(&c);
    let identity = |(callee, memory, _): &(
        jaic::sema::ProcId,
        Option<jaic::ide::lifetimes::LifetimeMemoryEffects>,
        Vec<jaic::ide::lifetimes::LifetimeArgument>,
    )| (*callee, *memory);
    assert_eq!(
        before.iter().map(identity).collect::<Vec<_>>(),
        after.iter().map(identity).collect::<Vec<_>>()
    );
}

/// Select the fixture edge even when `Runtime_Support` also generates a wrapper.
/// For example, each context-matrix case has exactly one local `wrapper`.
fn generated_fixture_entry(c: &Compiler) -> jaic::sema::ProcId {
    let ids: Vec<_> =
        c.ed.facts()
            .unwrap()
            .lifetime_generated_entries
            .keys()
            .copied()
            .filter(|id| {
                let p = &c.pr.procs[id.0 as usize];
                p.name.as_str() == "wrapper"
                    && c.src
                        .files
                        .iter()
                        .any(|f| f.id == p.span.file && Some(f.module) == c.ns.main_module)
            })
            .collect();
    assert_eq!(ids.len(), 1);
    ids[0]
}

#[test]
fn generated_entry_edges_record_only_completed_selected_calls() {
    use jaic::ide::lifetimes::LifetimeContextTransfer as Transfer;
    for (wrapper_flags, main_flags, expected) in [
        ("", "", Transfer::ForwardHidden),
        ("#c_call", "", Transfer::Null),
        ("", "#c_call", Transfer::Absent),
        ("#c_call", "#c_call", Transfer::Absent),
    ] {
        let call = if wrapper_flags.is_empty() && !main_flags.is_empty() {
            "push_context { wrapper(); }"
        } else {
            "wrapper();"
        };
        let source = format!(
            "wrapper :: () {wrapper_flags} #entry_point; main :: () {main_flags} {{ {call} }}"
        );
        let mut c = compile(&source, true, false);
        let id = generated_fixture_entry(&c);
        let edge = c.lifetime_generated_entry(id).unwrap();
        assert_eq!(edge.context, expected);
        assert_eq!(c.pr.procs[edge.target.0 as usize].name.as_str(), "main");
        let identity = (edge.wrapper_func, edge.target_func, edge.context);
        c.release_lowered_code();
        let edge = c.lifetime_generated_entry(id).unwrap();
        assert_eq!(
            (edge.wrapper_func, edge.target_func, edge.context),
            identity
        );
        c.ed.facts_mut().unwrap().clear_lifetimes();
        assert!(c.lifetime_generated_entry(id).is_none());
        assert_eq!(
            c.ed.facts().unwrap().lifetime_generated_entries.capacity(),
            0
        );
        assert!(
            compile(&source, false, false)
                .ed
                .facts()
                .unwrap()
                .lifetime_generated_entries
                .is_empty()
        );
    }
}

#[test]
fn generated_entry_edges_reject_stale_identities_and_failed_regeneration() {
    use jaic::sema::procs::ProcTarget;
    let mut c = compile(
        "wrapper :: () #entry_point; alternate :: (){} bad :: (value:s64){} ordinary :: () #foreign; main :: () {wrapper();alternate();ordinary();}",
        true,
        false,
    );
    let id = generated_fixture_entry(&c);
    let edge = c.lifetime_generated_entry(id).unwrap().clone();
    assert!(c.ed.facts().unwrap().lifetime_generated_entry_bytes() > 0);
    for procedure in [id, edge.target] {
        let old = c.pr.procs[procedure.0 as usize].target;
        c.pr.procs[procedure.0 as usize].target =
            Some(ProcTarget::Func(jaic::ir::FuncId(u32::MAX)));
        assert!(c.lifetime_generated_entry(id).is_none());
        c.pr.procs[procedure.0 as usize].target = old;
        let syntax = c.pr.procs[procedure.0 as usize].lit.clone();
        c.pr.procs[procedure.0 as usize].lit = std::sync::Arc::new((*syntax).clone());
        assert!(c.lifetime_generated_entry(id).is_none());
        c.pr.procs[procedure.0 as usize].lit = syntax;
    }
    c.ed.facts_mut()
        .unwrap()
        .lifetime_generated_entries
        .get_mut(&id)
        .unwrap()
        .wrapper = edge.target;
    assert!(c.lifetime_generated_entry(id).is_none());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_generated_entries
        .insert(id, edge);
    assert!(c.lifetime_generated_entry(id).is_some());
    let original = c.pr.procs[id.0 as usize].sig.clone().unwrap();
    let mut changed = (*original).clone();
    changed.has_context = !changed.has_context;
    c.pr.procs[id.0 as usize].sig = Some(std::sync::Arc::new(changed));
    assert!(c.lifetime_generated_entry(id).is_none());
    c.pr.procs[id.0 as usize].sig = Some(original);
    let target = c.lifetime_generated_entry(id).unwrap().target;
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&target)
        .unwrap()
        .procedure = id;
    assert!(c.lifetime_generated_entry(id).is_none());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&target)
        .unwrap()
        .procedure = target;
    let ordinary =
        c.pr.procs
            .iter()
            .enumerate()
            .find(|(_, p)| p.name.as_str() == "ordinary")
            .unwrap()
            .0;
    assert!(
        c.lifetime_generated_entry(jaic::sema::ProcId(ordinary as u32))
            .is_none()
    );
    c.options.entry_point = "alternate".into();
    c.pr.procs[id.0 as usize].target = None;
    let span = c.pr.procs[id.0 as usize].span;
    c.proc_func(id, span).unwrap();
    let new_edge = c.lifetime_generated_entry(id).unwrap();
    assert_eq!(
        c.pr.procs[new_edge.target.0 as usize].name.as_str(),
        "alternate"
    );
    c.options.entry_point = "bad".into();
    c.pr.procs[id.0 as usize].target = None;
    assert!(c.proc_func(id, span).is_err());
    assert!(
        !c.ed
            .facts()
            .unwrap()
            .lifetime_generated_entries
            .contains_key(&id)
    );
    c.options.entry_point = "alternate".into();
    c.pr.procs[id.0 as usize].target = None;
    c.proc_func(id, span).unwrap();
    assert!(c.lifetime_generated_entry(id).is_some());
    c.options.entry_point = "missing".into();
    c.pr.procs[id.0 as usize].target = None;
    let _ = c.proc_func(id, span);
    assert!(
        !c.ed
            .facts()
            .unwrap()
            .lifetime_generated_entries
            .contains_key(&id)
    );
    assert!(c.lifetime_generated_entry(id).is_none());
}

#[test]
fn selected_zero_write_is_separate_from_typed_leaf_coverage() {
    let mut c = compile(
        r"Nonzero :: struct {tag:s64=7;} zero_descriptor :: () {values:[..]s64;} copied_default :: () {value:Nonzero;} main :: (){}",
        true,
        false,
    );
    let images = |c: &Compiler| {
        bodies(c)
            .filter(|body| {
                matches!(
                    c.proc(body.procedure).name.as_str(),
                    "zero_descriptor" | "copied_default"
                )
            })
            .flat_map(|body| {
                events(body).filter_map(move |event| match event {
                    Event::DefaultImage {
                        ty,
                        bytes,
                        zeroed,
                        complete,
                        ..
                    } => Some((body.procedure, *ty, *bytes, *zeroed, *complete)),
                    _ => None,
                })
            })
            .collect::<Vec<_>>()
    };
    let before = images(&c);
    assert!(
        before
            .iter()
            .any(|(_, ty, bytes, zeroed, complete)| matches!(
                c.ty.types.kind(*ty),
                jaic::types::TypeKind::Array {
                    kind: jaic::types::ArrayKind::Resizable,
                    ..
                }
            ) && *bytes == c.ty.types.size_of(*ty)
                && *zeroed
                && !complete)
    );
    assert!(before.iter().any(
        |(procedure, _, _, zeroed, _)| c.proc(*procedure).name.as_str() == "copied_default"
            && !zeroed
    ));
    c.release_lowered_code();
    assert_eq!(before, images(&c));
    eprintln!(
        "default-zero-fact Event={} Body={}",
        std::mem::size_of::<Event>(),
        std::mem::size_of::<LifetimeBody>()
    );
}

#[test]
fn immutable_aggregate_copy_sources_retain_exact_typed_extents() {
    let mut c = compile(
        r"
        Item::struct {value:s64;}
        Pair::struct {left:s64;right:s64;}
        G:Item;
        immutable_source::(p:*Item){p.*=Item.{};}
        immutable_pair::(p:*Pair){p.*=Pair.{};}
        mutable_source::(p:*Item){p.*=G;}
        borrowed_source::(p:*Item,q:*Item){p.*=q.*;}
        main::(){}
    ",
        true,
        false,
    );
    let collect = |c: &Compiler| {
        bodies(c)
            .map(|b| (b.procedure, b.read_only_regions.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    for (name, width) in [("immutable_source", 8), ("immutable_pair", 16)] {
        let body = bodies(&c)
            .find(|b| c.proc(b.procedure).name.as_str() == name)
            .unwrap();
        assert!(
            body.read_only_regions.iter().any(|r| r.bytes == width
                && r.complete
                && !r.leaves.is_empty()
                && r.leaves.iter().all(
                    |leaf| leaf.value == jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0)
                )
                && events(body).any(
                    |e| matches!(e,Event::Store{value,ty,..} if *value==r.value && *ty==r.ty)
                )),
            "{name}: {:?}",
            body.read_only_regions
        );
        assert!(
            body.vector_bytes()
                >= body.read_only_regions.capacity()
                    * std::mem::size_of::<jaic::ide::lifetimes::LifetimeReadOnlyRegion>()
        );
    }
    for name in ["mutable_source", "borrowed_source"] {
        let body = bodies(&c)
            .find(|b| c.proc(b.procedure).name.as_str() == name)
            .unwrap();
        assert!(body.read_only_regions.is_empty(), "{name}");
    }
    let before = collect(&c);
    c.release_lowered_code();
    assert_eq!(before, collect(&c));
    eprintln!(
        "event={} body={} immutable_region={}",
        std::mem::size_of::<Event>(),
        std::mem::size_of::<LifetimeBody>(),
        std::mem::size_of::<jaic::ide::lifetimes::LifetimeReadOnlyRegion>()
    );
}

#[test]
fn file_body_selection_retains_the_original_map_key_certificate() {
    let mut c = compile("probe::(p:*s64){p.*=1;}main::(){}", true, false);
    let body = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "probe")
        .unwrap();
    let (id, file) = (body.procedure, body.syntax.header.span.file);
    assert!(
        c.ed.facts()
            .unwrap()
            .lifetime_bodies_in(file)
            .any(|b| b.procedure == id)
    );
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&id)
        .unwrap()
        .procedure = jaic::sema::ProcId(u32::MAX);
    assert!(
        !c.ed
            .facts()
            .unwrap()
            .lifetime_bodies_in(file)
            .any(|b| b.procedure == jaic::sema::ProcId(u32::MAX)),
        "the file index must not erase the original map-key check"
    );
    c.release_lowered_code();
    assert!(
        !c.ed
            .facts()
            .unwrap()
            .lifetime_bodies_in(file)
            .any(|b| b.procedure == jaic::sema::ProcId(u32::MAX))
    );
    c.ed.facts_mut().unwrap().lifetime_bodies.remove(&id);
    assert!(
        !c.ed
            .facts()
            .unwrap()
            .lifetime_bodies_in(file)
            .any(|b| b.procedure == id)
    );
    assert!(
        c.ed.facts()
            .unwrap()
            .lifetime_bodies_in(jaic::source::FileId(u32::MAX))
            .next()
            .is_none()
    );
}
