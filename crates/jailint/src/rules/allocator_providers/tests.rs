use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compiler(source: &str) -> Compiler {
    let stdlib = std::path::PathBuf::from(jacket_config::env::STDLIB.os().unwrap());
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    c
}

fn id(c: &Compiler, name: &str) -> ProcId {
    ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == name && p.lit.body.is_some() && !p.is_poly)
            .unwrap() as u32,
    )
}

#[test]
fn allocator_providers_real_pool_builder_and_bucket_use_checked_pairs() {
    let mut c = compiler(
        r#"P :: #import "Pool";B :: #import "Bucket_Array";#import "Basic";main :: () {p:P.Pool;x:=P.get(*p,8);builder:String_Builder;append(*builder,"x");values:B.Bucket_Array(s64,2);B.bucket_array_add(*values,1);}"#,
    );
    let a = Analysis::build(&c);
    for name in [
        "allocate_block",
        "grow_builder",
        "find_and_occupy_empty_slot",
    ] {
        let relation = a
            .provenance(id(&c, name))
            .unwrap_or_else(|| panic!("real carrier provider missing: {name}"));
        assert!(
            relation
                .uses
                .iter()
                .any(|u| u.operation == Operation::Allocate
                    && u.provider.procedure_value != Captured::Unknown
                    && u.provider.data_value != Captured::Unknown),
            "{name}: {relation:?}"
        );
    }
    let pool = a.provenance(id(&c, "allocate_block")).unwrap();
    assert!(pool.returns.iter().any(|r| matches!(
        r.source,
        ValueSource::Callback {
            byte_offset: 8,
            ..
        }
    )));
    assert!(pool.returns.iter().any(|r| r.source == ValueSource::Null));
    let builder = a.provenance(id(&c, "grow_builder")).unwrap();
    assert!(builder.stores.iter().any(|s| s.receiver_formal == 0
        && matches!(
            s.source,
            ValueSource::Callback {
                byte_offset: 0,
                ..
            }
        )));
    let bucket = a.provenance(id(&c, "find_and_occupy_empty_slot")).unwrap();
    assert!(
        bucket.incomplete,
        "later cyclic element traversal remains unknown"
    );
    c.release_lowered_code();
    assert_eq!(a.relations, Analysis::build(&c).relations);
}

#[test]
fn allocator_providers_mixed_pair_and_field_overwrite_keep_captured_epoch() {
    let c = compiler(
        r"
 Holder :: struct {a:Allocator;b:Allocator;}
 mixed :: (x:*Holder)->*void {return x.a.proc(.ALLOCATE,8,0,null,x.b.data);}
 overwrite :: (x:*Holder,replacement:Allocator)->*void {result:=x.a.proc(.ALLOCATE,8,0,null,x.a.data);x.a=replacement;return result;}
 main :: () {}
 ",
    );
    let a = Analysis::build(&c);
    let mixed = a.provenance(id(&c, "mixed")).unwrap();
    let use_ = mixed
        .uses
        .iter()
        .find(|u| u.operation == Operation::Allocate)
        .unwrap();
    let (
        Captured::ReceiverRead {
            slot: p, ..
        },
        Captured::ReceiverRead {
            slot: d, ..
        },
    ) = (&use_.provider.procedure_value, &use_.provider.data_value)
    else {
        panic!("mixed source slots lost")
    };
    assert_eq!((p.offset, d.offset), (0, 24));
    let overwrite = a.provenance(id(&c, "overwrite")).unwrap();
    assert!(overwrite.returns.iter().any(|r| matches!(
        r.source,
        ValueSource::Callback {
            byte_offset: 0,
            ..
        }
    )));
    assert!(
        overwrite
            .uses
            .iter()
            .all(|u| matches!(u.provider.data_value, Captured::ReceiverRead { .. }))
    );
}

fn epoch_site(e: &ReadEpoch) -> &Site {
    match e {
        ReadEpoch::Load(s) | ReadEpoch::AggregateCopy(s) => s,
        ReadEpoch::ArgumentCopy {
            call, ..
        } => call,
    }
}

#[test]
fn allocator_providers_named_arguments_and_copy_epochs_follow_actual_events() {
    let c = compiler(
        r#"#import "Basic";
 copy_epoch_wrapper :: (a:Allocator,mode:Allocator_Mode,bytes:s64,old_bytes:s64,previous:*void)->*void{return a.proc(mode,bytes,old_bytes,previous,a.data);}
 swap_data :: (a:*Allocator,replacement:*void)->s64 {a.data=replacement;return 8;}
 ordered :: (a:*Allocator,replacement:*void)->*void{return a.proc(.ALLOCATE,swap_data(a,replacement),0,null,a.data);}
 named :: (a:Allocator)->*void{return a.proc(allocator_data=a.data,old_memory=null,old_size=0,size=8,mode=.ALLOCATE);}
 copied :: (a:*Allocator,replacement:*void)->*void{return copy_epoch_wrapper(a.*,.ALLOCATE,swap_data(a,replacement),0,null);}
 main :: () {}
 "#,
    );
    let a = Analysis::build(&c);
    let named = a.provenance(id(&c, "named")).unwrap();
    assert!(named.uses.iter().any(|u| matches!(
        u.provider.procedure_value,
        Captured::FormalValue {
            formal: 0,
            slot: ByteSlot {
                offset: 0,
                ..
            }
        }
    ) && matches!(
        u.provider.data_value,
        Captured::FormalValue {
            formal: 0,
            slot: ByteSlot {
                offset: 8,
                ..
            }
        }
    )));
    let ordered = a.provenance(id(&c, "ordered")).unwrap();
    let use_ = ordered
        .uses
        .iter()
        .find(|u| u.operation == Operation::Allocate)
        .unwrap();
    let (
        Captured::ReceiverRead {
            epoch: p, ..
        },
        Captured::ReceiverRead {
            epoch: d, ..
        },
    ) = (&use_.provider.procedure_value, &use_.provider.data_value)
    else {
        panic!("ordered read sources lost")
    };
    let p = *epoch_site(p).chain.last().unwrap();
    let d = *epoch_site(d).chain.last().unwrap();
    let body = &c.ed.facts().unwrap().lifetime_bodies[&id(&c, "ordered")];
    assert_eq!(p.block, d.block);
    let mutation=body.blocks[p.block].events.iter().position(|e|matches!(e,jaic::ide::lifetimes::LifetimeEvent::Call{callee,..} if c.pr.procs[callee.0 as usize].name.as_str()=="swap_data")).unwrap();
    assert!(
        p.event < mutation && mutation < d.event,
        "procedure is captured before later argument effects, data after"
    );
    let copied = a.provenance(id(&c, "copied")).unwrap();
    let use_ = copied
        .uses
        .iter()
        .find(|u| u.operation == Operation::Allocate)
        .unwrap();
    let Captured::ReceiverRead {
        epoch, ..
    } = &use_.provider.data_value
    else {
        panic!("allocator argument snapshot source lost")
    };
    let site = *epoch_site(epoch).chain.last().unwrap();
    assert!(
        matches!(
            epoch,
            ReadEpoch::AggregateCopy(_) | ReadEpoch::ArgumentCopy { .. }
        ),
        "a semantic aggregate rvalue is not itself an evaluation-time copy"
    );
    assert!(matches!(
        c.ed.facts().unwrap().lifetime_bodies[&site.procedure].blocks[site.block].events
            [site.event],
        jaic::ide::lifetimes::LifetimeEvent::Store { .. }
    ));
}

#[test]
fn allocator_providers_nullable_fallback_and_saved_pair_are_guarded() {
    let c = compiler(
        r"
 fallback :: (a:*Allocator)->*void{if !a.proc a.*=context.allocator;return a.proc(.ALLOCATE,8,0,null,a.data);}
 saved :: (a:*Allocator,replacement:Allocator)->*void{chosen:=a.proc;data:=a.data;a.*=replacement;return chosen(.ALLOCATE,8,0,null,data);}
 main :: () {}
 ",
    );
    let analysis = Analysis::build(&c);
    let fallback = analysis.provenance(id(&c, "fallback")).unwrap();
    let context = fallback
        .uses
        .iter()
        .find(|u| matches!(u.provider.procedure_value, Captured::ContextRead { .. }))
        .unwrap();
    let receiver = fallback
        .uses
        .iter()
        .find(|u| matches!(u.provider.procedure_value, Captured::ReceiverRead { .. }))
        .unwrap();
    assert!(matches!(
        context.provider.data_value,
        Captured::ContextRead { .. }
    ));
    assert!(!context.guards.is_empty() && !receiver.guards.is_empty());
    assert!(
        context.guards.iter().any(|a| receiver
            .guards
            .iter()
            .any(|b| a.predicate == b.predicate && a.truth != b.truth)),
        "fallback alternatives retain the null-procedure correlation"
    );
    let saved = analysis.provenance(id(&c, "saved")).unwrap();
    assert!(
        saved.uses.iter().any(|u| u.operation == Operation::Allocate
            && matches!(
                u.provider.procedure_value,
                Captured::ReceiverRead {
                    formal: 0,
                    ..
                }
            )
            && matches!(
                u.provider.data_value,
                Captured::ReceiverRead {
                    formal: 0,
                    ..
                }
            )),
        "overwriting the receiver cannot relabel earlier captured pair bits"
    );
}

#[test]
fn allocator_providers_offsets_and_machine_width_are_conservative() {
    let c = compiler(
        r"
 shifted :: (a:Allocator)->*u8{return cast(*u8)a.proc(.ALLOCATE,64,0,null,a.data)+8;}
 before :: (a:Allocator)->*u8{return cast(*u8)a.proc(.ALLOCATE,64,0,null,a.data)-8;}
 multiplied_wrap :: (a:Allocator)->*u8{raw:=a.proc(.ALLOCATE,64,0,null,a.data);return cast(*u8)(cast(*u64)raw-0x2000000000000000);}
 composed_wrap :: (a:Allocator)->*u8{p:=cast(*u8)a.proc(.ALLOCATE,64,0,null,a.data);p=p+0x7fffffffffffffff;p=p+1;return p;}
 overflow :: (a:Allocator)->*void #no_aoc {x:s8=127;x+=1;if x<0 return a.proc(.ALLOCATE,8,0,null,a.data);return null;}
 safe :: (a:Allocator)->*void #no_aoc {x:s8=126;x+=1;if x<0 return a.proc(.ALLOCATE,8,0,null,a.data);return null;}
 signed_literal :: (a:Allocator)->*void{x:s8=-128;if x<0 return a.proc(.ALLOCATE,8,0,null,a.data);return null;}
 main :: () {}
 ",
    );
    let a = Analysis::build(&c);
    for (name, offset) in [("shifted", 8), ("before", -8)] {
        assert!(a.provenance(id(&c, name)).unwrap().returns.iter().any(
            |r| matches!(r.source,ValueSource::Callback{byte_offset,..} if byte_offset==offset)
        ));
    }
    for name in ["multiplied_wrap", "composed_wrap"] {
        let r = a.provenance(id(&c, name)).unwrap();
        assert!(r.incomplete);
        assert!(
            !r.returns
                .iter()
                .any(|r| matches!(r.source, ValueSource::Callback { .. })),
            "{name}: overflow cannot retain a false displacement"
        );
    }
    assert!(a.provenance(id(&c, "overflow")).unwrap().incomplete);
    assert!(a.provenance(id(&c, "safe")).is_none());
    assert!(a.provenance(id(&c, "signed_literal")).is_some());
}

#[test]
fn allocator_providers_aliases_and_unsupported_private_writes_do_not_keep_stale_bits() {
    let c = compiler(
        r"
 Larger :: struct{pad:s64;value:*void;}
 alias :: (a:*Allocator,b:*Allocator,data:*void)->*void{chosen:=a.proc;old:=a.data;b.data=data;return chosen(.ALLOCATE,8,0,null,a.data);}
 unsupported_write :: (a:Allocator)->*void{dummy:u8;raw:=a.proc(.ALLOCATE,8,0,null,a.data);(cast(*Larger)*dummy).value=null;return raw;}
 aggregate_write :: (a:Allocator,bytes:[32]u8)->*void{raw:=a.proc(.ALLOCATE,8,0,null,a.data);tiny:u8;(cast(*[32]u8)*tiny).* = bytes;return raw;}
 aggregate_mixed :: (a:Allocator,bytes:[32]u8,flag:bool)->*void{raw:=a.proc(.ALLOCATE,8,0,null,a.data);tiny:u8;if flag {(cast(*[32]u8)*tiny).* = bytes;}return raw;}
 aggregate_init :: (a:Allocator)->*void{raw:=a.proc(.ALLOCATE,8,0,null,a.data);tiny:u8;(cast(*[32]u8)*tiny).* = .{};return raw;}
 stored_unsupported_write :: (a:Allocator)->*void{dummy:u8;raw:=a.proc(.ALLOCATE,8,0,null,a.data);p:=cast(**void)(*dummy+8);p.*=null;return raw;}
 unsupported_copy :: (a:Allocator,src:*Allocator)->*void{local:=a;dummy:u8;delta:=cast(s64)src-cast(s64)*dummy;local=(cast(*Allocator)(*dummy+delta)).*;return local.proc(.ALLOCATE,8,0,null,local.data);}
 remember :: (p:*Allocator) #foreign;
 mutate_saved :: () #foreign;
 escaped :: (a:Allocator)->*void{local:=a;remember(*local);local=a;mutate_saved();return local.proc(.ALLOCATE,8,0,null,local.data);}
 captured :: (a:Allocator)->*void{local:=a;remember(*local);local=a;chosen:=local.proc;data:=local.data;mutate_saved();return chosen(.ALLOCATE,8,0,null,data);}
 repeated :: (a:Allocator,n:s64)->*void{raw:*void;for 0..n raw=a.proc(.ALLOCATE,8,0,null,a.data);return raw;}
 recurse :: (a:Allocator,n:s64)->*void{if n>0 return recurse(a,n-1);return a.proc(.ALLOCATE,8,0,null,a.data);}
 main :: () {}
 ",
    );
    let a = Analysis::build(&c);
    let alias = a.provenance(id(&c, "alias")).unwrap();
    assert!(
        alias.uses.iter().any(|u| matches!(
            u.provider.data_value,
            Captured::ReceiverRead {
                formal: 0,
                ..
            }
        )),
        "a and b may alias: no stale field or unproved substitution of data"
    );
    let bad = a.provenance(id(&c, "unsupported_write")).unwrap();
    assert!(bad.incomplete);
    assert!(
        !bad.returns
            .iter()
            .any(|r| matches!(r.source, ValueSource::Callback { .. })),
        "unsupported private projection may overwrite another private result slot"
    );
    for name in ["aggregate_mixed", "aggregate_init", "aggregate_write"] {
        let r = a.provenance(id(&c, name)).unwrap();
        assert!(r.incomplete, "{name} extent not certified");
        assert!(
            r.returns.iter().any(|r| r.source == ValueSource::Unknown),
            "{name} must preserve unknown output arm"
        );
        if name != "aggregate_mixed" {
            assert!(
                !r.returns
                    .iter()
                    .any(|r| matches!(r.source, ValueSource::Callback { .. })),
                "{name} must not preserve a stale callback source"
            );
        }
    }
    let stored = a.provenance(id(&c, "stored_unsupported_write")).unwrap();
    assert!(stored.incomplete);
    assert!(
        !stored
            .returns
            .iter()
            .any(|r| matches!(r.source, ValueSource::Callback { .. }))
    );
    for name in ["unsupported_copy", "escaped"] {
        let r = a.provenance(id(&c, name)).unwrap();
        assert!(r.incomplete);
        assert!(
            r.uses
                .iter()
                .all(|u| u.provider.procedure_value == Captured::Unknown
                    || u.provider.data_value == Captured::Unknown),
            "{name}"
        );
    }
    assert!(
        a.provenance(id(&c, "captured"))
            .unwrap()
            .uses
            .iter()
            .any(|u| matches!(
                u.provider.procedure_value,
                Captured::FormalValue {
                    formal: 0,
                    ..
                }
            ) && matches!(
                u.provider.data_value,
                Captured::FormalValue {
                    formal: 0,
                    ..
                }
            ))
    );
    assert!(a.provenance(id(&c, "repeated")).is_none());
    let recurse = a.provenance(id(&c, "recurse")).unwrap();
    assert!(recurse.incomplete);
    assert!(
        recurse
            .returns
            .iter()
            .any(|r| r.source == ValueSource::Unknown)
    );
}

#[test]
fn allocator_providers_structural_bounds_and_stale_body_remain_unknown() {
    let mut source = String::from("bounded :: (a:Allocator,flag:bool)->*void{ok:=flag;");
    for _ in 0..256 {
        source.push_str("ok=!ok;\n");
    }
    source.push_str("if ok return a.proc(.ALLOCATE,8,0,null,a.data);return null;}main :: () {}");
    let mut c = compiler(&source);
    let r = Analysis::build(&c)
        .provenance(id(&c, "bounded"))
        .cloned()
        .unwrap();
    assert!(r.incomplete);
    let id = id(&c, "bounded");
    let p = &mut c.pr.procs[id.0 as usize];
    p.lit = std::sync::Arc::new((*p.lit).clone());
    assert!(Analysis::build(&c).provenance(id).is_none());
}

#[test]
fn allocator_providers_cancellation_discards_partial_observations() {
    const CHILD: jacket_config::env::Var = jacket_config::env::LIFETIME_CANCELLATION_CHILD;
    if !CHILD.is_set() {
        let name = std::thread::current().name().unwrap().to_owned();
        assert!(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &name, "--test-threads=1"])
                .env(CHILD.name, "1")
                .status()
                .unwrap()
                .success()
        );
        return;
    }
    let c = compiler(
        "probe :: (a:Allocator)->*void{return a.proc(.ALLOCATE,8,0,null,a.data);}main :: () {}",
    );
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
            jaic::interrupt::stop_after(0);
        }
    }
    let _reset = Reset;
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(3);
    assert!(Analysis::build(&c).relations.is_empty());
    assert!(jaic::interrupt::disarm());
    let mut solver = checked::Solver::new(&c);
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(6);
    assert!(solver.summary(id(&c, "probe")).is_none());
    assert!(jaic::interrupt::disarm());
}

#[test]
fn allocator_providers_selected_tags_and_nominal_roles_are_not_guessed() {
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
    let stdlib = PathBuf::from(jacket_config::env::STDLIB.os().unwrap());
    let allocation = stdlib.join("prelude/allocation.jai");
    let original = std::fs::read_to_string(&allocation).unwrap();
    let changed = original
        .replace("ALLOCATE :: 0;", "ALLOCATE :: 101;")
        .replace("RESIZE :: 1;", "RESIZE :: 103;")
        .replace("FREE :: 2;", "FREE :: 107;");
    let negative = original
        .replace("ALLOCATE :: 0;", "ALLOCATE :: -11;")
        .replace("RESIZE :: 1;", "RESIZE :: -13;")
        .replace("FREE :: 2;", "FREE :: -17;")
        .replace("Allocator_Mode :: enum {", "Allocator_Mode :: enum s8 {");
    for (text, supported) in [
        (changed, true),
        (negative, true),
        (
            original.replace("allocator_data: *void", "user_data: *void"),
            false,
        ),
    ] {
        let fs = PreludeFs {
            allocation: NativeFs.canonical(&allocation),
            bytes: text.into_bytes(),
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
  allocated :: (a:Allocator)->*void{return a.proc(.ALLOCATE,8,0,null,a.data);}
  resized :: (a:Allocator)->*void{return a.proc(.RESIZE,8,0,null,a.data);}
  freed :: (a:Allocator)->*void{return a.proc(.FREE,0,0,null,a.data);}
  caps :: (a:Allocator)->*void{return a.proc(.CAPS,0,0,null,a.data);}
  main :: () {}
  "
            .into(),
        )])
        .unwrap();
        c.finish_program().unwrap();
        assert!(c.ide_check_all().is_empty());
        let a = Analysis::build(&c);
        for (name, operation) in [
            ("allocated", Operation::Allocate),
            ("resized", Operation::Resize),
            ("freed", Operation::Free),
        ] {
            let relation = a.provenance(id(&c, name));
            if supported {
                assert!(
                    relation
                        .unwrap()
                        .uses
                        .iter()
                        .any(|u| u.operation == operation),
                    "{name}"
                );
            } else {
                assert!(
                    relation.is_none(),
                    "canonical role failure must remain unknown"
                );
            }
        }
        if supported {
            assert!(
                a.provenance(id(&c, "caps"))
                    .unwrap()
                    .uses
                    .iter()
                    .all(|u| matches!(
                        u.operation,
                        Operation::OtherConstant {
                            bits: 10,
                            ..
                        }
                    ))
            );
        }
        c.release_lowered_code();
        assert_eq!(a.relations, Analysis::build(&c).relations);
    }
    let c = compiler(
        r"Other_Callback :: #type(mode:Allocator_Mode,size:s64,old_size:s64,allocator_data:*void,old_memory:*void)->*void;fake :: (callback:Other_Callback)->*void{return callback(.ALLOCATE,8,0,null,null);}main :: () {}",
    );
    assert!(
        Analysis::build(&c).provenance(id(&c, "fake")).is_none(),
        "equal pointer types do not certify swapped lexical data roles"
    );
}

#[test]
fn allocator_providers_reject_mismatched_retained_body_key() {
    let mut c = compiler(
        "probe :: (a:Allocator)->*void{return a.proc(.ALLOCATE,8,0,null,a.data);}other :: (){} main :: (){}",
    );
    let p = id(&c, "probe");
    let other = id(&c, "other");
    assert!(Analysis::build(&c).provenance(p).is_some());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&p)
        .unwrap()
        .procedure = other;
    assert!(Analysis::build(&c).provenance(p).is_none());
}
