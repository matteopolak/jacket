//! Conditional paths must select every actual branch at the invocation epoch.
use super::*;

#[test]
fn native_return_path_requires_a_fully_selected_closed_branch() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        C::#system_library "libc";
        native::(n:u64)->*void #foreign C "malloc";
        observed:s64;
        callback::(){observed+=1;}
        choose::(flag:bool,n:u64)->*void {
            if flag return native(n);
            callback();
            return native(n);
        }
        main::(){}
    "#,
    );
    let check = |c: &Compiler| {
        let all = Summaries::build(c);
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "choose")
                .unwrap() as u32,
        );
        let proof = all.procs[&id]
            .native_allocation
            .as_ref()
            .expect("closed selected branch");
        let apply = |flag| proof.apply_nonnull(c, &all.procs, (id, &[flag, Some(64)]), &mut 0);
        assert_eq!(apply(Some(1)).unwrap().bytes, 64);
        assert_eq!(apply(Some(0)), None, "callback branch remains Unknown");
        assert_eq!(
            apply(None),
            None,
            "an unknown branch is not an existential proof"
        );
    };
    check(&c);
    c.release_lowered_code();
    check(&c);
}

#[test]
fn native_return_path_rejects_success_assumptions_in_earlier_guards() {
    let c = super::stdlib_allocations::checked_source(
        r#"
        C::#system_library "libc";
        native::(n:u64)->*void #foreign C "malloc";
        compound_then_test::(flag:bool,n:u64)->*void {
            p:=native(n);
            if !p || flag return cast(*void)1;
            if !p return null;
            return p;
        }
        compound_direct::(flag:bool,n:u64)->*void {
            p:=native(n);
            if !p || flag return cast(*void)1;
            return p;
        }
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    for name in ["compound_then_test", "compound_direct"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let range = all.procs[&id]
            .native_allocation
            .as_ref()
            .and_then(|p| p.apply_nonnull(&c, &all.procs, (id, &[Some(0), Some(64)]), &mut 0));
        assert_eq!(
            range, None,
            "{name}: failed allocation can return nonnull sentinel"
        );
    }
}

#[test]
fn native_actual_provider_maps_the_checked_subtraction_and_prefix() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        D::#import "Default_Allocator";
        driver::()->*void{return D.allocator_proc(.ALLOCATE,64,0,null,null);}
        main::(){}
    "#,
    );
    let check = |c: &Compiler| {
        let driver = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "driver")
                .unwrap() as u32,
        );
        let body = &c.ed.facts().unwrap().lifetime_bodies[&driver];
        let (callee, inputs) = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|event| match event {
                Event::Call {
                    callee,
                    args,
                    expands: false,
                    ..
                } if args.len() == 5 => {
                    Some((*callee, args.iter().map(|a| a.constant).collect::<Vec<_>>()))
                }
                _ => None,
            })
            .unwrap();
        let all = Summaries::build(c);
        let range = all.procs[&callee]
            .native_allocation
            .as_ref()
            .unwrap()
            .apply_nonnull(c, &all.procs, (callee, &inputs), &mut 0)
            .expect("actual selected provider range");
        assert_eq!((range.bytes, range.alignment), (64, 64));
    };
    check(&c);
    c.release_lowered_code();
    check(&c);
}

#[test]
fn native_subtraction_requires_a_checked_object_offset() {
    let c = super::stdlib_allocations::checked_source(
        r#"
        C::#system_library "libc";
        native::(n:u64)->*void #foreign C "malloc";
        sub_bits::(p:*void,prefix:u64,wide:bool)->*void {
            at:=cast(u64)p+prefix;
            if wide at=at-1;
            return cast(*void)at;
        }
        add_bits::(p:*void,prefix:u64,wide:bool)->*void {
            at:=cast(u64)p+prefix;
            if wide at=at+0xffffffffffffffff;
            return cast(*void)at;
        }
        shifted_sub::(n:u64)->*void{p:=native(n);if !p return null;return sub_bits(p,16,true);}
        shifted_add::(n:u64)->*void{p:=native(n);if !p return null;return add_bits(p,16,true);}
        underflow::(n:u64)->*void{p:=native(n);if !p return null;return sub_bits(p,0,true);}
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let apply = |name: &str| {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        all.procs[&id]
            .native_allocation
            .as_ref()
            .and_then(|p| p.apply_nonnull(&c, &all.procs, (id, &[Some(136)]), &mut 0))
    };
    assert_eq!(apply("shifted_sub").unwrap().bytes, 121);
    assert_eq!(
        apply("shifted_add"),
        None,
        "Add MAX is not a subtraction certificate"
    );
    assert_eq!(
        apply("underflow"),
        None,
        "subtraction must stay in the original object"
    );
}
