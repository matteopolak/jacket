//! Exact returned-object bytes are queried at their actual load epoch.
use super::*;

#[test]
fn selected_context_constant_image_keeps_type_info_and_added_defaults() {
    let _c = stdlib_allocations::checked_source(
        r#"
        #add_context extra_value: s64 = 123;
        image: #Context : .{context_info=type_info(#Context)};
        #assert image.context_info == type_info(#Context);
        #assert image.extra_value == 123;
        main::(){allocator:=image.allocator;}
    "#,
    );
}

#[test]
fn runtime_constructor_return_preserves_the_selected_allocator_data() {
    let c = stdlib_allocations::checked_source(
        r#"
        R::#import "Runtime_Support";
        constructor_data::()->*void {
            ctx:=R.__jai_runtime_init(0,null);
            if !ctx return null;
            return ctx.allocator.data;
        }
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|body| c.proc(body.procedure).name.as_str() == "constructor_data")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let returned = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Return {
                value,
                constant: None,
                ..
            } => Some(*value),
            _ => None,
        })
        .expect("actual successful constructor DATA read");
    let ctor = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee, ..
            } if c.proc(*callee).name.as_str() == "__jai_runtime_init" => Some(*callee),
            _ => None,
        })
        .unwrap();
    let checked = crate::rules::lifetime_facts::CheckedSession::new(&c)
        .body(ctor)
        .unwrap();
    let constructor = Prepared::for_body(checked, &all);
    let ctor_body = checked.body();
    for (block, b) in ctor_body.blocks.iter().enumerate() {
        for (event, e) in b.events.iter().enumerate() {
            if matches!(e, Event::Barrier) {
                assert!(
                    ctor_body
                        .overlay_accesses
                        .iter()
                        .any(|marker| marker.block == block && marker.event == event),
                    "checked unsigned runtime count must not emit a reporting callback/cast-failure barrier"
                );
            }
        }
    }
    eprintln!(
        "actual ctor frontier: blocks={} events={} native_inventory={}",
        ctor_body.blocks.len(),
        ctor_body
            .blocks
            .iter()
            .map(|b| b.events.len())
            .sum::<usize>(),
        Resolver::for_prepared(&constructor, &all)
            .core()
            .native_reads
            .is_some()
    );
    let ctor_resolver = Resolver::for_prepared(&constructor, &all);
    let ctx = ctor_body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Return {
                value,
                constant: None,
                ..
            } => Some(*value),
            _ => None,
        })
        .unwrap();
    eprintln!(
        "actual returned ctx pointer sources={:?}",
        ctor_resolver.core().storage_versions.get(&ctx)
    );
    if let Some(jaic::sema::procs::ProcTarget::Func(function)) = c.proc(ctor).target
        && let Some(function) = c
            .out
            .program
            .funcs
            .get(function.0 as usize)
            .and_then(Option::as_ref)
    {
        for (id, block) in ctor_body.blocks.iter().enumerate() {
            if block
                .events
                .iter()
                .any(|event| matches!(event, Event::Barrier))
            {
                eprintln!(
                    "actual ctor barrier block={id} ir={:?}",
                    function.blocks[id].insts
                );
            }
        }
    }

    let mut probe_remaining = 8192;
    let probe_index = ctor_resolver.capture_index(&mut probe_remaining).unwrap();
    let mut probe_spent = 8192 - probe_remaining;
    let probe = super::super::address_arithmetic::bit_returns::captured_address_bits(
        &ctor_resolver,
        &ctor_resolver.core().storage_versions,
        &probe_index,
        ctx,
        &mut probe_spent,
    );
    eprintln!("actual returned ctx initial graph={probe:?} spent={probe_spent}");
    let mut field_work = 8192;
    let field_index = r.capture_index(&mut field_work).unwrap();
    let field_place = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Load {
                place,
                value,
                ty,
                ..
            } if *value == returned && *ty == TypeId::VOID_PTR => Some(*place),
            _ => None,
        })
        .unwrap();
    let (actual_call, field) = field_index
        .write_location(&r, field_place, TypeId::VOID_PTR, &mut field_work, 0)
        .unwrap();
    assert!(body.blocks.iter().flat_map(|b| &b.events).any(|event| matches!(event,
        Event::Call { callee, results, .. } if *callee == ctor && results.iter().any(|(_, value, _)| *value == actual_call))));
    let context_ty =
        c.ty.types
            .pointee(c.proc(ctor).sig.as_ref().unwrap().returns[0])
            .unwrap();
    let (image_place, image_source) = ctor_body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Store {
                place,
                value,
                ty,
                ..
            } if *ty == context_ty => Some((*place, *value)),
            _ => None,
        })
        .expect("actual full Context image copy");
    let images = ctor_body
        .read_only_regions
        .iter()
        .filter(|region| region.value == image_source)
        .collect::<Vec<_>>();
    assert_eq!(
        images.len(),
        1,
        "one actual selected immutable Context image"
    );
    let image = images[0];
    assert_eq!(image.ty, context_ty);
    assert!(
        field
            .offset
            .checked_add(field.width)
            .is_some_and(|end| end <= image.bytes)
    );
    assert!(
        image.leaves.iter().any(|leaf| leaf.offset == field.offset
            && leaf.width == field.width
            && leaf.ty == TypeId::VOID_PTR
            && leaf.value == jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0)),
        "the actual selected constant image must contain the requested typed null DATA leaf"
    );
    eprintln!(
        "actual ctor DATA request={field:?} selected_image=({image_place:?},{image_source:?}) frozen_bytes={} complete={}",
        image.bytes, image.complete
    );
    fn selected_field(c: &jaic::sema::Compiler, ty: TypeId, name: &str) -> (u64, TypeId) {
        let mut pending = vec![(ty, 0)];
        let mut found = None;
        while let Some((ty, offset)) = pending.pop() {
            let info = c.ty.types.struct_info(c.ty.types.as_struct(ty).unwrap());
            for field in info.fields.iter() {
                if field
                    .name
                    .is_some_and(|field_name| field_name.as_str() == name)
                {
                    assert!(found.replace((offset + field.offset, field.ty)).is_none());
                } else if field.using && c.ty.types.as_struct(field.ty).is_some() {
                    pending.push((field.ty, offset + field.offset));
                }
            }
        }
        found.unwrap()
    }
    let (allocator_offset, allocator_ty) = selected_field(&c, context_ty, "allocator");
    let (proc_offset, proc_ty) = selected_field(&c, allocator_ty, "proc");
    let (handler_offset, handler_ty) = selected_field(&c, context_ty, "assertion_failed");
    let requested = [
        (TypeId::VOID_PTR, field),
        (
            proc_ty,
            crate::rules::lifetime_facts::ByteRange {
                offset: allocator_offset + proc_offset,
                width: c.ty.types.size_of(proc_ty),
            },
        ),
        (
            handler_ty,
            crate::rules::lifetime_facts::ByteRange {
                offset: handler_offset,
                width: c.ty.types.size_of(handler_ty),
            },
        ),
    ];
    let mut remaining = 8192;
    let range = super::super::native_returns::captured_object_fields(
        &ctor_resolver,
        &all.procs,
        Some(ctx),
        &requested,
        &mut remaining,
    );
    eprintln!("actual returned ctx range={range:?} remaining={remaining}");
    assert!(
        range.is_some(),
        "the actual ctx expression must bind its checked native range before byte poststate composition"
    );
    let (_, _, values, _) = range.as_ref().unwrap();
    assert_eq!(values.len(), 3);
    for (i, value) in values.iter().enumerate() {
        assert_eq!(value.ty, requested[i].0);
        assert_eq!(value.offset, requested[i].1.offset);
        assert!(image.leaves.iter().any(|leaf| leaf == value));
        if i > 0 {
            assert!(matches!(
                value.value,
                jaic::ide::lifetimes::LifetimeDefaultValue::Procedure { .. }
            ));
        }
    }
    assert_eq!(
        range.as_ref().and_then(|(_, _, fields, _)| {
            let field = fields.first()?;
            match field.value {
                jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits) => Some((field.ty, bits)),
                _ => None,
            }
        }),
        Some((TypeId::VOID_PTR, 0)),
        "the actual returned context must preserve the selected null allocator DATA, not a default-provider class assumption"
    );
}

#[test]
fn requested_slot_prestates_preserve_physical_overwrites_and_call_uncertainty() {
    for (tail, overwritten) in [
        ("(cast(*Other)p).other=replacement;", true),
        ("(cast(*u32)p).*=1;", false),
        ("(cast(*u64)p).*=1;", false),
        ("unknown();", false),
    ] {
        let source = format!(
            r#"
            C::#system_library "unmodelled";
            unknown::() #foreign C "unknown";
            A::struct {{value:*void;}}
            Other::struct {{other:*void;}}
            observe::(){{}}
            slot_probe::(p:*A,original:*void,replacement:*void){{p.value=original;{tail}observe();}}
            main::(){{}}
        "#
        );
        let c = stdlib_allocations::checked_source(&source);
        let all = Summaries::build(&c);
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == "slot_probe")
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body(body.procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let writes: Vec<_> = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter_map(|event| {
                let Event::Store {
                    place,
                    value,
                    ty,
                    ..
                } = event
                else {
                    return None;
                };
                let place = r.places.existing(*place, *ty)?;
                let slot = r.physical_slot(place, c.ty.types.size_of(*ty))?;
                (slot.formal == 0 && slot.width == 8).then_some((slot, *value))
            })
            .collect();
        let at = body
            .blocks
            .iter()
            .enumerate()
            .find_map(|(block, b)| {
                b.events.iter().enumerate().find_map(|(event, e)| {
                    matches!(e,Event::Call{callee,..} if c.proc(*callee).name.as_str()=="observe")
                        .then_some(crate::rules::lifetime_movement::facts::Site {
                            block,
                            event,
                        })
                })
            })
            .unwrap();
        let sources = r.before(at, writes[0].0, &all.procs);
        use super::super::value_writes::Source;
        if overwritten {
            assert_eq!(
                sources,
                HashSet::from_iter([Source::Value(writes.last().unwrap().1)]),
                "{tail}"
            );
        } else {
            assert!(sources.contains(&Source::Value(writes[0].1)), "{tail}");
            assert!(sources.contains(&Source::Unknown), "{tail}");
        }
    }
}

#[test]
fn actual_constructor_size_call_keeps_its_checked_effect_frame() {
    let c = stdlib_allocations::checked_source(
        r#"R::#import "Runtime_Support"; main::(){ctx:=R.__jai_runtime_init(0,null);}"#,
    );
    let mut all = Summaries::build(&c);
    let constructor =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "__jai_runtime_init")
            .unwrap();
    let callees: Vec<_> = constructor
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| match e {
            Event::Call {
                callee, ..
            } if c.proc(*callee).name.as_str() == "temporary_storage_size" => Some(*callee),
            _ => None,
        })
        .collect();
    assert_eq!(
        callees.len(),
        1,
        "actual selected runtime configuration call"
    );
    assert!(
        c.ed.facts().unwrap().lifetime_bodies[&callees[0]]
            .read_only_regions
            .iter()
            .all(|region| region.leaves.is_empty() && !region.complete),
        "the early runtime configuration extent must not freeze its later-finalized number"
    );
    let frame = Rc::clone(
        all.procs[&callees[0]]
            .frame
            .as_ref()
            .expect("checked scalar-return body still needs a complete effect frame"),
    );
    assert!(
        frame
            .typed()
            .is_some_and(|f| f.writes.is_empty() && f.reads.is_empty())
    );
    let intrinsic = c.ed.facts().unwrap().lifetime_bodies[&callees[0]]
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Call {
                callee,
                expands: true,
                ..
            } => Some(*callee),
            _ => None,
        })
        .unwrap();
    assert!(frame.compatible(&all.procs));
    all.procs.get_mut(&intrinsic).unwrap().trusted =
        Some(Box::new(super::super::contracts::Trusted::Opaque));
    assert!(
        frame.current(&c),
        "the retained bodies themselves did not change"
    );
    assert!(
        !frame.compatible(&all.procs),
        "current expanded declaration promise is still required"
    );
    all.procs.remove(&intrinsic);
    assert!(
        !frame.compatible(&all.procs),
        "missing current declaration rejects"
    );
}

#[test]
fn scalar_read_expansion_keeps_independent_contracts_and_opaque_intrinsics() {
    let c = stdlib_allocations::checked_source(
        r#"
        #import "Basic";
        __jaic_runtime_temporary_storage_size::()->u32 #intrinsic #no_context; @ResetsTemporaryStorage
        black_box::(value:s64)->s64 #intrinsic #no_context;
        noted_reader::()->s64 #no_context{return __jaic_runtime_temporary_storage_size();}
        opaque_reader::()->s64 #no_context{return black_box(1);}
        probe::(a:*[..]s64)->*s64{noted_reader();opaque_reader();return array_add(a);}
        main::(){}
        "#,
    );
    let all = Summaries::build(&c);
    for name in ["noted_reader", "opaque_reader"] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        assert!(all.procs[&body.procedure].frame.is_none(), "{name}");
        assert!(
            body.blocks.iter().flat_map(|b| &b.events).any(|e| matches!(
                e,
                Event::Call {
                    expands: false,
                    ..
                }
            )),
            "{name}: ordinary call boundary retained"
        );
    }
}

#[test]
fn bound_byte_coordinates_use_actual_stride_and_exact_address_identity() {
    let c = stdlib_allocations::checked_source(
        r#"
        byte_header::(p:*u8)->*u8{return p-16;}
        word_header::(p:*u64)->*u64{return p-2;}
        huge_header::(p:*u64)->*u64{return p-0x7fff_ffff_ffff_ffff;}
        different_header::(p:*u64,q:*u64)->*u64{return q-2;}
        main::(){}
        "#,
    );
    let all = Summaries::build(&c);
    for (name, expected) in [
        ("byte_header", Some(-16)),
        ("word_header", Some(-16)),
        ("huge_header", None),
        ("different_header", None),
    ] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body(body.procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let root = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|e| match e {
                Event::Parameter {
                    formal: 0,
                    incoming,
                    ..
                } => Some(*incoming),
                _ => None,
            })
            .unwrap();
        let returned = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|e| match e {
                Event::Return {
                    value,
                    ty,
                    ..
                } => Some((*value, c.ty.types.pointee(*ty).unwrap())),
                _ => None,
            })
            .unwrap();
        let mut remaining = 8192;
        let index = r.capture_index(&mut remaining).unwrap();
        let found = index.relative_write(
            &r,
            (&r.core().storage_versions, Some(root)),
            returned,
            &mut remaining,
            0,
        );
        assert_eq!(found.map(|(_, offset, _)| offset), expected, "{name}");
        assert!(
            index
                .write_location(&r, returned.0, returned.1, &mut remaining, 0)
                .is_none(),
            "legacy nonzero offsets remain unsupported: {name}"
        );
    }
}

#[test]
fn actual_call_context_uses_the_captured_constructor_result() {
    let mut c = stdlib_allocations::checked_source(
        r#"
        R::#import "Runtime_Support";
        consume::(){}
        caller::() #c_call {
            ctx:=R.__jai_runtime_init(0,null);
            if !ctx return;
            push_context ctx.* consume();
        }
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|body| c.proc(body.procedure).name.as_str() == "caller")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let row = body
        .call_contexts
        .iter()
        .find(|row| {
            matches!(&body.blocks[row.block].events[row.event],
        Event::Call{callee,..} if c.proc(*callee).name.as_str()=="consume")
        })
        .unwrap();
    let jaic::ide::lifetimes::LifetimeContextArgument::Value(value) = row.argument else {
        panic!("actual context operand")
    };
    let expected = body
        .blocks
        .iter()
        .flat_map(|block| &block.events)
        .find_map(|event| match event {
            Event::Call {
                callee,
                results,
                ..
            } if c.proc(*callee).name.as_str() == "__jai_runtime_init" => {
                results.first().map(|result| result.1)
            }
            _ => None,
        })
        .unwrap();
    let mut remaining = 8192;
    let index = r.capture_index(&mut remaining).unwrap();
    assert_eq!(
        index
            .pointer_leaf(&r, value, &mut remaining, 0)
            .map(|leaf| leaf.0),
        Some(expected),
        "actual Call context must follow the real selector Store/Load epoch, not a default selector class"
    );
    assert!(
        body.context_input.is_none(),
        "C-call has no incoming context assumption"
    );
    let procedure = body.procedure;
    drop(index);
    drop(r);
    drop(prepared);
    let malformed =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&procedure)
            .unwrap();
    for event in malformed
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.events)
    {
        if let Event::ContextRead {
            place,
            value: loaded,
            ..
        } = event
            && *loaded == value
        {
            *place = expected; // The returned heap object is not the compiler selector cell.
        }
    }
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut remaining = 8192;
    let index = r.capture_index(&mut remaining).unwrap();
    assert!(
        index.pointer_leaf(&r, value, &mut remaining, 0).is_none(),
        "malformed selector address cannot become a load/proven context alias"
    );
}

#[test]
fn actual_generated_entry_carries_initial_append_context_requests() {
    let c = stdlib_allocations::checked_source_with_contracts(
        r#"
        #import "Basic";
        main::(){items:[..]int;p:=array_add(*items);items.count=items.allocated;array_add(*items,1);p.*=2;}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let facts = c.ed.facts().unwrap();
    let edge = facts
        .lifetime_generated_entries
        .keys()
        .find_map(|id| c.lifetime_generated_entry(*id))
        .unwrap();
    assert_eq!(
        edge.context,
        jaic::ide::lifetimes::LifetimeContextTransfer::ForwardHidden
    );
    let caller = facts
        .lifetime_bodies
        .values()
        .find(|body| {
            body.blocks.iter().flat_map(|block| &block.events)
        .any(|event|matches!(event,Event::Call{callee,expands:false,..} if *callee==edge.wrapper))
        })
        .unwrap();
    let requests = all.procs[&caller.procedure]
        .context_reads
        .as_ref()
        .expect("actual selected generated call must carry demanded bytes");
    assert_eq!(
        requests.fields.len(),
        3,
        "allocator proc/DATA and assertion handler, not default-provider certainty"
    );
    assert!(caller.context_types.contains(&requests.ty));
}


#[test]
fn actual_constructor_publishes_its_demanded_returned_slots() {
    let c = stdlib_allocations::checked_source_with_contracts(
        r#"
        #import "Basic";
        main::(){items:[..]int;p:=array_add(*items);items.count=items.allocated;array_add(*items,1);p.*=2;}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|body| c.proc(body.procedure).name.as_str() == "__jai_runtime_init")
            .unwrap();
    let slots = all.procs[&body.procedure]
        .returned_slots
        .as_ref()
        .expect("actual Call to ContextSet demand must be published by the constructor itself");
    assert_eq!(slots.fields.len(), 3);
    assert!(
        slots.lifetime.is_some(),
        "actual native allocation must succeed and survive to the returned Context"
    );
    let proof = slots.lifetime.as_ref().unwrap();
    let mut remaining = 8192;
    assert!(proof.current(&c, &all.procs, &mut remaining));
    let mut changed = all.procs.clone();
    changed.get_mut(&body.procedure).unwrap().trusted =
        Some(Box::new(super::super::contracts::Trusted::Opaque));
    assert!(
        !proof.current(&c, &changed, &mut remaining),
        "a retained returned-object witness cannot bypass a new owner contract"
    );

    assert!(slots.fields.iter().any(|leaf| matches!(
        leaf.value,
        jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(0)
    )));
    assert_eq!(
        slots
            .fields
            .iter()
            .filter(|leaf| matches!(
                leaf.value,
                jaic::ide::lifetimes::LifetimeDefaultValue::Procedure { .. }
            ))
            .count(),
        2
    );
}

#[test]
fn actual_constructor_constant_reaches_the_guarded_context_field_read() {
    let mut c = stdlib_allocations::checked_source_with_contracts(
        r#"
        #import "Basic";
        R::#import "Runtime_Support";
        caller::() #c_call {
            ctx:=R.__jai_runtime_init(0,null);
            if !ctx return;
            push_context ctx.* { selected:=context.allocator.proc; }
        }
        main::(){caller();items:[..]int;array_add(*items);}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|body| c.proc(body.procedure).name.as_str() == "caller")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let (value, ty) = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Load {
                value,
                ty,
                ..
            } if matches!(c.ty.types.kind(*ty), jaic::types::TypeKind::Proc(_)) => {
                Some((*value, *ty))
            }
            _ => None,
        })
        .unwrap();
    let mut remaining = 8192;
    let captured=r.returned_constant(&all.procs,value,&mut remaining)
        .expect("actual constructor Call, guarded result, ContextSet and field Load must share the installed typed value");
    assert_eq!(captured.0, ty);
    assert!(matches!(
        captured.1,
        jaic::ide::lifetimes::LifetimeDefaultValue::Procedure { .. }
    ));
    let procedure = body.procedure;
    let ctor = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee, ..
            } if all
                .procs
                .get(callee)
                .is_some_and(|s| s.returned_slots.is_some()) =>
            {
                Some(*callee)
            }
            _ => None,
        })
        .unwrap();
    drop(r);
    drop(prepared);
    let mut signature = (**c.pr.procs[ctor.0 as usize].sig.as_ref().unwrap()).clone();
    signature.returns[0] = TypeId::VOID_PTR;
    c.pr.procs[ctor.0 as usize].sig = Some(std::sync::Arc::new(signature));
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    assert!(
        r.returned_constant(&all.procs, value, &mut remaining)
            .is_none(),
        "retained call result must agree with current selected return ABI"
    );
}

#[test]
fn actual_generated_entry_receives_the_preserved_context_constants() {
    let c = stdlib_allocations::checked_source_with_contracts(
        r#"
        #import "Basic";
        main::(){items:[..]int;p:=array_add(*items);items.count=items.allocated;array_add(*items,1);p.*=2;}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let facts = c.ed.facts().unwrap();
    let edge = facts
        .lifetime_generated_entries
        .keys()
        .find_map(|id| c.lifetime_generated_entry(*id))
        .unwrap();
    let (body, at) = facts
        .lifetime_bodies
        .values()
        .find_map(|body| {
            body.blocks.iter().enumerate().find_map(|(block, b)| {
                b.events.iter().enumerate().find_map(|(event, e)| {
                    matches!(e,Event::Call{callee,expands:false,..} if *callee==edge.wrapper)
                        .then_some((
                            body,
                            crate::rules::lifetime_movement::facts::Site {
                                block,
                                event,
                            },
                        ))
                })
            })
        })
        .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut remaining = 8192;
    let fields = r.context_constants(&all.procs, at, &mut remaining).expect(
        "the real generated entry must receive the preserved constructor proc/DATA/handler values",
    );
    assert_eq!(fields.len(), 3);
    let frame_call = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee,
                args,
                results,
                expands: false,
                ..
            } if args.is_empty()
                && results.is_empty()
                && all.procs.get(callee).is_some_and(|s| {
                    s.frame.as_ref().is_some_and(|f| {
                        f.condition.is_none()
                            && f.typed()
                                .is_some_and(|t| t.reads.is_empty() && t.writes.is_empty())
                    })
                }) =>
            {
                Some(*callee)
            }
            _ => None,
        })
        .unwrap();
    let mut changed = all.procs.clone();
    changed.get_mut(&frame_call).unwrap().trusted =
        Some(Box::new(super::super::contracts::Trusted::Opaque));
    let mut remaining = 8192;
    assert!(
        r.context_constants(&changed, at, &mut remaining).is_none(),
        "current independent call effects must invalidate watched context bytes"
    );
}
