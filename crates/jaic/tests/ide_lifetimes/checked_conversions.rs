//! Selected compiler-generated conversions retain control and external effects.
use super::{bodies, compile, compile_with_options, events};
use jaic::ide::lifetimes::LifetimeEvent as Event;
use jaic::sema::{Compiler, Options};

#[test]
fn address_cast_widths_survive_releasing_executable_ir() {
    let mut c = compile(
        r"
        pointer_bits::(p:*void)->u64{return cast(u64)p;}
        bits_pointer::(v:u64)->*void{return cast(*void)v;}
        main::(){}
    ",
        true,
        false,
    );
    let snapshot = |c: &Compiler| {
        let mut rows = Vec::new();
        for b in bodies(c) {
            if ["pointer_bits", "bits_pointer"].contains(&c.proc(b.procedure).name.as_str()) {
                assert!(!b.address_casts.is_empty());
                for cast in &b.address_casts {
                    assert_eq!(cast.integer_bytes, 8);
                    rows.push((
                        b.procedure,
                        cast.value,
                        cast.source,
                        cast.to_pointer,
                        cast.integer_bytes,
                    ));
                }
            }
        }
        rows.sort_by_key(|row| (row.0.0, row.1.0));
        assert!(!rows.is_empty());
        rows
    };
    let before = snapshot(&c);
    c.release_lowered_code();
    assert_eq!(before, snapshot(&c));
}


#[test]
fn checked_cast_failure_retains_exact_branch_and_opaque_effect() {
    let mut options = Options::host();
    options.cast_bounds_check = 1;
    let mut c = compile_with_options(
        r"
        signed_cast_probe::(value:s64)->u64{return cast(u64)value;}
        narrow_cast_probe::(value:s64)->u32{return cast(u32)value;}
        unchecked_cast_probe::(value:s64)->u64{return cast,no_check(u64)value;}
        truncated_cast_probe::(value:s64)->u32{return cast,trunc(u32)value;}
        main::(){}
    ",
        true,
        false,
        options,
    );
    let snapshot = |c: &Compiler| {
        let mut out = Vec::new();
        for name in ["signed_cast_probe", "narrow_cast_probe"] {
            let b = bodies(c)
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
            let branch = b
                .blocks
                .iter()
                .enumerate()
                .find_map(|(at, bb)| {
                    bb.events.iter().find_map(|e| match e {
                        Event::Branch {
                            condition,
                            then_block,
                            else_block,
                            ..
                        } => Some((at, *condition, *then_block, *else_block)),
                        _ => None,
                    })
                })
                .expect("actual cast check branch");
            assert_eq!(b.blocks[branch.0].successors, vec![branch.2, branch.3]);
            assert!(
                b.blocks[branch.2]
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::Barrier)),
                "failure callback is not pure"
            );
            assert!(
                !b.blocks[branch.3]
                    .events
                    .iter()
                    .any(|e| matches!(e, Event::Barrier)),
                "passing cast does not execute the failure callback"
            );
            out.push((b.procedure, branch));
        }
        for name in ["unchecked_cast_probe", "truncated_cast_probe"] {
            let b = bodies(c)
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
            assert!(!events(b).any(|e| matches!(e, Event::Branch { .. } | Event::Barrier)));
        }
        out
    };
    let before = snapshot(&c);
    c.release_lowered_code();
    assert_eq!(before, snapshot(&c));
}

#[test]
fn disabled_and_type_only_cast_checks_do_not_fabricate_effects() {
    let mut options = Options::host();
    options.cast_bounds_check = 0;
    let mut c = compile_with_options(
        r"
        disabled_cast_probe::(value:s64)->u64{return cast(u64)value;}
        sketch_cast_probe::(value:s64,$T:type)->u64{return cast(u64)value;}
        main::(){}
    ",
        true,
        false,
        options,
    );
    let b = bodies(&c)
        .find(|b| c.proc(b.procedure).name.as_str() == "disabled_cast_probe")
        .unwrap();
    assert!(!events(b).any(|e| matches!(e, Event::Branch { .. } | Event::Barrier)));
    let count = bodies(&c).count();
    c.ide_sketch_polymorphs();
    assert_eq!(count, bodies(&c).count());
}

#[test]
fn noncast_check_and_trap_reports_retain_opaque_effects() {
    let c = compile(
        r"
        atomic_store::(pointer:*$T,new:T) #intrinsic;
        atomic_check_probe::(p:*u64){atomic_store(p,7);}
        missing_return_probe::()->s64{}
        main::(){}
    ",
        true,
        false,
    );
    for name in ["atomic_check_probe", "missing_return_probe"] {
        let b = bodies(&c)
            .find(|b| c.proc(b.procedure).name.as_str() == name)
            .unwrap();
        assert!(
            events(b).any(|e| matches!(e, Event::Barrier)),
            "selected report path: {name}"
        );
    }
}


#[test]
fn checked_integer_and_records_selected_type_and_survives_release() {
    let mut c = compile(r"
        unsigned_and::(a:u64,b:u64)->u64{return a&b;}
        signed_and::(a:s64,b:s64)->s64{return a&b;}
        main::(){a:u64;b:u64;present:=#exists(a&b);}
    ", true, false);
    let snapshot=|c:&Compiler| {
        let mut rows=Vec::new();
        for (name,expected) in [("unsigned_and",jaic::types::TypeId::U64),("signed_and",jaic::types::TypeId::S64)] {
            let body=bodies(c).find(|b|c.proc(b.procedure).name.as_str()==name).unwrap();
            let ands:Vec<_>=events(body).filter_map(|e|match e {
                Event::IntegerBinary{ty,value,left,right,op:jaic::ir::BinOp::And,..}=>Some((*ty,*value,*left,*right)),
                _=>None,
            }).collect();
            assert_eq!(ands.len(),1,"{name}");
            assert_eq!(ands[0].0,expected);
            rows.push((body.procedure,ands));
        }
        let main=bodies(c).find(|b|c.proc(b.procedure).name.as_str()=="main").unwrap();
        assert!(!events(main).any(|e|matches!(e,Event::IntegerBinary{op:jaic::ir::BinOp::And,..})),"type-only AND is not executed");
        rows
    };
    let before=snapshot(&c);
    c.release_lowered_code();
    assert_eq!(snapshot(&c),before);
}


#[test]
fn arithmetic_overflow_callbacks_retain_the_actual_failure_edge() {
    for (runtime,mode) in [(true,0),(true,1),(true,2),(false,1)] {
        let mut options=Options::host();
        options.runtime_support=runtime;
        options.arithmetic_overflow_check=mode;
        let mut c=compile_with_options(r"
            unsigned_add::(a:u64,b:u64)->u64{return a+b;}
            signed_add::(a:s64,b:s64)->s64{return a+b;}
            unchecked_add::(a:u64,b:u64)->u64 #no_aoc{return a+b;}
            main::(){a:u64;b:u64;present:=#exists(a+b);}
        ",true,false,options);
        let snapshot=|c:&Compiler| {
            let mut rows=Vec::new();
            for name in ["unsigned_add","signed_add"] {
                let body=bodies(c).find(|b|c.proc(b.procedure).name.as_str()==name).unwrap();
                let branches:Vec<_>=body.blocks.iter().enumerate().flat_map(|(block,b)|b.events.iter().filter_map(move |e|match e {
                    Event::Branch{condition,then_block,else_block,..}=>Some((block,*condition,*then_block,*else_block)),_=>None,
                })).collect();
                assert_eq!(branches.len(),usize::from(runtime&&mode!=0),"{name} mode{mode} runtime{runtime}");
                if let Some(&(at,_,fail,pass))=branches.first() {
                    assert_eq!(body.blocks[at].successors,vec![fail,pass]);
                    assert!(body.blocks[fail].events.iter().any(|e|matches!(e,Event::Barrier)));
                    assert!(!body.blocks[pass].events.iter().any(|e|matches!(e,Event::Barrier)));
                    assert!(body.blocks[fail].successors.contains(&pass),"preserve the actual emitted continuation");
                }
                rows.push((body.procedure,branches));
            }
            for name in ["main","unchecked_add"] {
                let body=bodies(c).find(|b|c.proc(b.procedure).name.as_str()==name).unwrap();
                assert!(!events(body).any(|e|matches!(e,Event::Branch{..}|Event::Barrier)),"{name}");
            }
            rows
        };
        let before=snapshot(&c);
        c.release_lowered_code();
        assert_eq!(snapshot(&c),before);
    }
}
