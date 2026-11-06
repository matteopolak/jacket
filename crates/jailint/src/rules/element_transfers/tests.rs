use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compiler(source: &str) -> Compiler {
    let stdlib = jacket_config::env::STDLIB.os().map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        std::path::PathBuf::from,
    );
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
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
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
fn element_transfers_real_unordered_carrier_and_identity_positions() {
    let mut c =
        compiler(r#"#import "Basic";main :: () {a:[]s64;array_unordered_remove_by_index(*a,0);}"#);
    let a = Analysis::build(&c);
    let transfer = a
        .transfer(id(&c, "array_unordered_remove_by_index"))
        .expect("nonempty checked real carrier proof");
    assert_eq!(transfer.receiver_formal, 0);
    assert_eq!(transfer.index_formal, 1);
    assert_eq!(transfer.count_delta, -1);
    assert!(!transfer.evidence.is_empty());
    assert_eq!(
        transfer.preconditions,
        Preconditions {
            valid_incoming_index: true,
            descriptor_disjoint_from_storage: true
        }
    );
    // Equal payloads still have distinct incoming logical identities.
    let payload = [7, 7, 7, 7];
    for (removed, want) in [(0, vec![3, 1, 2]), (1, vec![0, 3, 2]), (3, vec![0, 1, 2])] {
        let old_indices: Vec<_> = (0..3)
            .map(|new| transfer.old_index(new, removed, 4).unwrap())
            .collect();
        assert_eq!(old_indices, want);
        assert!(old_indices.iter().all(|i| payload[*i as usize] == 7));
    }
    assert_eq!(transfer.old_index(0, 0, 1), None);
    assert_eq!(transfer.old_index(0, 0, 0), None);
    assert_eq!(transfer.old_index(0, -1, 4), None);
    c.release_lowered_code();
    assert_eq!(a.transfers, Analysis::build(&c).transfers);
}

#[test]
fn element_transfers_renamed_protocol_and_unknown_counterexamples() {
    let c = compiler(
        r"
        good :: (view:*[]s64,k:s64){view.count-=1;view.data[k]=view.data[view.count];}
        Fake :: struct{count:s64;data:*s64;}
        named_like :: (x:*Fake,k:s64){x.count-=1;x.data[k]=x.data[x.count];}
        wrong_source :: (a:*[]s64,k:s64){a.count-=1;a.data[k]=a.data[0];}
        wrong_count :: (a:*[]s64,k:s64){a.count-=2;a.data[k]=a.data[a.count];}
        extra_write :: (a:*[]s64,k:s64){a.count-=1;a.data[k]=a.data[a.count];a.data[0]=99;}
        aliased_write :: (a:*[]s64,k:s64,other:*s64){a.count-=1;a.data[k]=a.data[a.count];other.*=99;}
        repeated :: (a:*[]s64,k:s64,n:s64){for 0..n {a.count-=1;a.data[k]=a.data[a.count];}}
        Item :: struct{x:s64;y:s64;}
        partial :: (a:*[]Item,k:s64){a.count-=1;a.data[k].x=a.data[a.count].x;}
        copied_alias :: (a:*[]s64,k:s64){copy:=a.*;a.count-=1;copy.data[k]=copy.data[a.count];}
        external :: (n:s64)->s64 #foreign;
        unknown :: (a:*[]s64,k:s64){a.count-=1;a.data[external(k)]=a.data[a.count];}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    assert!(a.transfer(id(&c, "good")).is_some());
    for name in [
        "named_like",
        "wrong_source",
        "wrong_count",
        "extra_write",
        "aliased_write",
        "repeated",
        "partial",
        "copied_alias",
        "unknown",
    ] {
        assert!(a.transfer(id(&c, name)).is_none(), "{name}");
    }
}

#[test]
fn element_transfers_boolean_budget_prevents_unproved_guarantee() {
    let mut source = String::from(
        r"
        external :: () #foreign;
        bounded :: (a:*[]s64,k:s64){ok:=k>=0 && k<a.count;
    ",
    );
    for _ in 0..256 {
        source.push_str("ok=!ok;\n");
    }
    source.push_str("if !ok external();a.count-=1;a.data[k]=a.data[a.count];} main :: () {}");
    let c = compiler(&source);
    assert!(Analysis::build(&c).transfer(id(&c, "bounded")).is_none());
}

#[test]
fn element_transfers_cancellation_discards_partial_relation() {
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
    let c =
        compiler("good :: (a:*[]s64,k:s64){a.count-=1;a.data[k]=a.data[a.count];} main :: () {}");
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
    jaic::interrupt::stop_after(2);
    assert!(Analysis::build(&c).transfers.is_empty());
    assert!(jaic::interrupt::disarm());
    let facts = c.ed.facts().unwrap();
    let body = &facts.lifetime_bodies[&id(&c, "good")];
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(2);
    assert!(checked::infer(&c, body, 0, 1, TypeId::S64).is_none());
    assert!(jaic::interrupt::fired());
}

#[test]
fn element_transfers_machine_overflow_cannot_prune_opaque_effect() {
    let c = compiler(
        r"
        external :: () #foreign;
        bad_s8 :: (a:*[]s64,k:s64) #no_aoc {x:s8=127;x+=1;if x<0 external();a.count-=1;a.data[k]=a.data[a.count];}
        bad_s64 :: (a:*[]s64,k:s64) #no_aoc {x:s64=0x7fff_ffff_ffff_ffff;x+=1;if x<0 external();a.count-=1;a.data[k]=a.data[a.count];}
        bad_signed_literal :: (a:*[]s64,k:s64){x:s8=-128;if x<0 external();a.count-=1;a.data[k]=a.data[a.count];}
        bad_unsigned_literal :: (a:*[]s64,k:s64){x:u8=255;if x==255 external();a.count-=1;a.data[k]=a.data[a.count];}
        safe_s8 :: (a:*[]s64,k:s64) #no_aoc {x:s8=126;x+=1;if x<0 external();a.count-=1;a.data[k]=a.data[a.count];}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    assert!(
        a.transfer(id(&c, "bad_s8")).is_none(),
        "s8 overflow must not be evaluated as i128 128"
    );
    assert!(
        a.transfer(id(&c, "bad_s64")).is_none(),
        "s64 overflow must not be evaluated as i128 2^63"
    );
    assert!(a.transfer(id(&c, "bad_signed_literal")).is_none());
    assert!(a.transfer(id(&c, "bad_unsigned_literal")).is_none());
    assert!(
        a.transfer(id(&c, "safe_s8")).is_some(),
        "nonoverflowing machine arithmetic remains supported"
    );
}

#[test]
fn element_transfers_nonzero_as_adjustment_is_not_buffer_identity() {
    let mut c = compiler(
        r"
        Item :: struct{pad:s64;#as inner:s64;}
        Same :: struct{#as inner:s64;}
        preserved :: (a:*[]s64,k:s64){p:=cast(*s64) cast(*Same) a.data;a.count-=1;p[k]=p[a.count];}
        shifted :: (a:*[]s64,k:s64){p:=cast(*s64) cast(*Item) a.data;a.count-=1;p[k]=p[a.count];}
        main :: () {}
    ",
    );
    let analysis = Analysis::build(&c);
    assert!(analysis.transfer(id(&c, "preserved")).is_some());
    assert!(analysis.transfer(id(&c, "shifted")).is_none());
    c.release_lowered_code();
    assert_eq!(analysis.transfers, Analysis::build(&c).transfers);
}

#[test]
fn element_transfers_derived_stack_addresses_do_not_hide_extra_writes() {
    let c = compiler(
        r"
        Larger :: struct {pad:s64;field:s64;}
        dynamic_alias :: (a:*[]s64,k:s64){dummy:u8;delta:=cast(s64)a.data-cast(s64)*dummy;(cast(*s64)(*dummy+delta)).*=99;a.count-=1;a.data[k]=a.data[a.count];}
        outside_actual_root :: (a:*[]s64,k:s64){dummy:u8;(cast(*Larger)*dummy).field=99;a.count-=1;a.data[k]=a.data[a.count];}
        ordinary_private :: (a:*[]s64,k:s64){dummy:Larger;dummy.field=99;a.count-=1;a.data[k]=a.data[a.count];}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    let unsupported: Vec<_> = ["dynamic_alias", "outside_actual_root"]
        .into_iter()
        .filter(|name| a.transfer(id(&c, name)).is_some())
        .collect();
    assert!(
        unsupported.is_empty(),
        "derived/OOB slot ownership: {unsupported:?}"
    );
    assert!(a.transfer(id(&c, "ordinary_private")).is_some());
}

#[test]
fn element_transfers_reject_mismatched_retained_body_key() {
    let mut c = compiler(
        r"good :: (a:*[]s64,k:s64){a.count-=1;a.data[k]=a.data[a.count];} other :: (){} main :: (){}",
    );
    let good = id(&c, "good");
    let other = id(&c, "other");
    assert!(Analysis::build(&c).transfer(good).is_some());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&good)
        .unwrap()
        .procedure = other;
    assert!(Analysis::build(&c).transfer(good).is_none());
}
