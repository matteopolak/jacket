use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compiler(source: &str) -> Compiler {
    compiler_cpu(source, None)
}

fn compiler_cpu(source: &str, cpu: Option<jaic::target::TargetCpu>) -> Compiler {
    compiler_target(source, None, cpu)
}

fn compiler_target(
    source: &str,
    os: Option<jaic::target::TargetOs>,
    cpu: Option<jaic::target::TargetCpu>,
) -> Compiler {
    let stdlib = std::path::PathBuf::from(jacket_config::env::STDLIB.os().unwrap());
    let mut options = Options::host();
    if let Some(os) = os {
        options.os = os;
    }
    if let Some(cpu) = cpu {
        options.cpu = cpu;
    }
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
fn temporary_epochs_contiguous_protocol_and_observation_are_distinct() {
    let c = compiler(
        r"
 Arena :: struct{data:*u8;capacity:s64;cursor:s64;}
 Mark :: struct{cursor:s64;}
 bump_fixture_allocate :: (a:*Arena,n:s64)->*u8 #no_aoc {if n<0||a.cursor<0||a.cursor>a.capacity return null;if n>a.capacity-a.cursor return null;old:=a.cursor;result:=a.data+old;a.cursor=old+n;return result;}
 bump_fixture_mark :: (a:*Arena)->Mark{return .{a.cursor};}
 bump_fixture_restore :: (a:*Arena,m:Mark){if m.cursor>=0&&m.cursor<=a.cursor a.cursor=m.cursor;}
 bump_fixture_reset :: (a:*Arena){a.cursor=0;}
 bump_fixture_counter :: (p:*s64,n:s64){p.*=n;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    assert!(
        !a.relations(id(&c, "bump_fixture_allocate"))
            .unwrap()
            .protocols
            .is_empty()
    );
    assert!(
        a.relations(id(&c, "bump_fixture_allocate"))
            .unwrap()
            .protocols
            .iter()
            .any(|p| p
                .conditions
                .iter()
                .all(|c| domain::predicate_supported(&c.predicate))),
        "successful path needs retained typed guards"
    );
    let bump_fixture_mark = a.relations(id(&c, "bump_fixture_mark")).unwrap();
    assert!(!bump_fixture_mark.observations.is_empty());
    assert!(!bump_fixture_mark.snapshots.is_empty());
    assert!(
        !a.relations(id(&c, "bump_fixture_restore"))
            .unwrap()
            .rewinds
            .is_empty()
    );
    assert!(
        !a.relations(id(&c, "bump_fixture_reset"))
            .unwrap()
            .resets
            .is_empty()
    );
    assert!(a.relations(id(&c, "bump_fixture_counter")).is_none());
}

#[test]
fn temporary_epochs_real_paged_mark_remains_observation_until_protocol_proved() {
    for overwrite in [false, true] {
        let c = compiler(&format!(
            "B :: #import \"Basic\"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY={overwrite});main :: (){{m:=B.get_temporary_storage_mark();B.set_temporary_storage_mark(m);B.reset_temporary_storage();}}"
        ));
        let a = Analysis::build(&c);
        let m = a.relations(id(&c, "get_temporary_storage_mark")).unwrap();
        assert!(
            m.observations
                .iter()
                .any(|s| s.fields.len() == 3 && s.instance.is_some())
        );
        assert!(
            m.snapshots.is_empty(),
            "paged allocation and ancestry still unsupported"
        );
        let bump_fixture_restore = a.relations(id(&c, "set_temporary_storage_mark"));
        assert!(bump_fixture_restore.is_none_or(|r| r.rewinds.is_empty() && r.incomplete));
    }
}

#[test]
fn temporary_epochs_copied_observations_preserve_instance_but_mixed_fields_do_not() {
    let c = compiler(
        r"
 Epoch_Arena :: struct{data:*u8;capacity:s64;cursor:s64;other:s64;}
 Epoch_Mark :: struct{cursor:s64;other:s64;}
 epoch_allocate :: (a:*Epoch_Arena,n:s64)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}
 epoch_copy :: (a:*Epoch_Arena)->Epoch_Mark{m:Epoch_Mark=.{a.cursor,a.other};copied:=m;return copied;}
 epoch_mixed :: (a:*Epoch_Arena,b:*Epoch_Arena)->Epoch_Mark{return .{a.cursor,b.other};}
 epoch_incoherent :: (a:*Epoch_Arena)->Epoch_Mark{m:Epoch_Mark;m.cursor=a.cursor;a.other=9;m.other=a.other;return m;}
 epoch_external :: () #foreign;
 epoch_after_callback :: (a:*Epoch_Arena)->Epoch_Mark{epoch_external();return .{a.cursor,a.other};}
 epoch_unrelated :: (n:*s64,value:s64){n.*=value;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let copied = a.relations(id(&c, "epoch_copy")).unwrap();
    assert!(copied.observations.iter().any(|m| m.fields.len() == 2));
    assert!(!copied.snapshots.is_empty());
    for name in ["epoch_mixed", "epoch_incoherent", "epoch_unrelated"] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.snapshots.is_empty()),
            "{name}"
        );
    }
    let callback = a.relations(id(&c, "epoch_after_callback")).unwrap();
    assert!(callback.incomplete);
    assert!(!callback.observations.is_empty());
    assert!(
        callback.snapshots.is_empty(),
        "opaque callback invalidates protocol capability"
    );
}

#[test]
fn temporary_epochs_later_reset_and_may_alias_header_write_do_not_preserve_protocol() {
    let c = compiler(
        r"
 Mixed_Arena :: struct{data:*u8;cursor:s64;}
 epoch_bad_reset :: (a:*Mixed_Arena,n:s64)->*u8 #no_aoc {old:=a.cursor;result:=a.data+old;a.cursor=old+n;a.cursor=0;return result;}
 epoch_bad_alias :: (a:*Mixed_Arena,b:*Mixed_Arena,n:s64)->*u8 #no_aoc {old:=a.cursor;result:=a.data+old;b.data=null;a.cursor=old+n;return result;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let bad: Vec<_> = ["epoch_bad_reset", "epoch_bad_alias"]
        .into_iter()
        .filter(|name| {
            a.relations(id(&c, name))
                .is_some_and(|r| !r.protocols.is_empty())
        })
        .collect();
    assert!(bad.is_empty(), "invalidated protocols: {bad:?}");
}

#[test]
fn temporary_epochs_overlay_projection_never_hides_asm_callback_or_overlap() {
    let mut c = compiler_cpu(
        r"
 Overlay_Epoch :: struct{raw:[16]u8;#overlay(raw) using fields:struct{data:*u8;cursor:s64;}}
 Overlay_Mark :: struct{cursor:s64;}
 epoch_overlay_allocate :: (a:*Overlay_Epoch,n:s64)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}
 epoch_overlay_mark :: (a:*Overlay_Epoch)->Overlay_Mark{return .{a.cursor};}
 epoch_overlay_write :: (a:*Overlay_Epoch,n:s64)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;a.raw[8]=0;return p;}
 epoch_overlay_asm :: (a:*Overlay_Epoch)->Overlay_Mark{#asm{nop;}return .{a.cursor};}
 epoch_overlay_pick :: (a:*Overlay_Epoch)->*Overlay_Epoch #foreign;
 epoch_overlay_callback :: (a:*Overlay_Epoch)->Overlay_Mark{return .{epoch_overlay_pick(a).cursor};}
 main :: (){}
 ",
        Some(jaic::target::TargetCpu::X64),
    );
    let a = Analysis::build(&c);
    assert!(
        !a.relations(id(&c, "epoch_overlay_mark"))
            .unwrap()
            .snapshots
            .is_empty()
    );
    for name in [
        "epoch_overlay_write",
        "epoch_overlay_asm",
        "epoch_overlay_callback",
    ] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.protocols.is_empty() && r.snapshots.is_empty()),
            "{name}"
        );
    }
    c.release_lowered_code();
    assert_eq!(a.relations, Analysis::build(&c).relations);
    let p = id(&c, "epoch_overlay_mark");
    let body =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&p)
            .unwrap();
    assert!(!body.overlay_accesses.is_empty());
    body.overlay_accesses[0].projection_event = usize::MAX;
    assert!(
        Analysis::build(&c)
            .relations(p)
            .is_none_or(|r| r.snapshots.is_empty()),
        "invalid projection certificate remains opaque"
    );
}

#[test]
fn temporary_epochs_bounds_stale_body_and_unknown_extent_remain_uncertified() {
    let mut source = String::from(
        "Budget_Arena :: struct{data:*u8;cursor:s64;}Budget_Mark :: struct{cursor:s64;}budget_allocate :: (a:*Budget_Arena,n:s64)->*u8 #no_aoc{old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}budget_probe :: (a:*Budget_Arena,flag:bool)->Budget_Mark{b:=flag;",
    );
    for _ in 0..256 {
        source.push_str("b=!b;");
    }
    source.push_str("if b return .{a.cursor};return .{};} main :: (){}");
    let mut c = compiler(&source);
    let p = id(&c, "budget_probe");
    let a = Analysis::build(&c);
    assert!(a.relations(p).is_none_or(|r| r.snapshots.is_empty()));
    let allocate = id(&c, "budget_allocate");
    assert!(!a.relations(allocate).unwrap().protocols.is_empty());
    for root in &mut c
        .ed
        .facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&allocate)
        .unwrap()
        .stack_roots
    {
        root.bytes = None;
    }
    assert!(
        Analysis::build(&c)
            .relations(allocate)
            .is_none_or(|r| r.protocols.is_empty()),
        "unknown stack extent cannot authorize private spill access"
    );
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&allocate)
        .unwrap()
        .procedure = p;
    assert!(Analysis::build(&c).relations(allocate).is_none());
}

#[test]
fn temporary_epochs_cancellation_never_publishes_partial_certificates() {
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
        "Epoch_Cancel :: struct{data:*u8;cursor:s64;}epoch_cancel :: (a:*Epoch_Cancel,n:s64)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}main :: (){}",
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
    for count in [1, 16, 256] {
        jaic::interrupt::arm();
        jaic::interrupt::stop_after(count);
        assert!(Analysis::build(&c).relations.is_empty());
        assert!(jaic::interrupt::disarm());
    }
}

#[test]
fn temporary_epochs_unknown_aggregate_write_cannot_disappear_from_transitions() {
    let c = compiler(
        r"
 Header_Epoch :: struct{data:*u8;capacity:s64;cursor:s64;}
 Header_Mark :: struct{cursor:s64;}
 header_epoch_allocate :: (a:*Header_Epoch,n:s64)->*u8 #no_aoc{old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}
 header_epoch_mark :: (a:*Header_Epoch)->Header_Mark{return .{a.cursor};}
 header_epoch_bad :: (a:*Header_Epoch,n:s64,bytes:[24]u8)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;(cast(*[24]u8)a).* = bytes;return p;}
 header_epoch_bad_reset :: (a:*Header_Epoch,bytes:[24]u8){a.cursor=0;(cast(*[24]u8)a).* = bytes;}
 header_epoch_bad_restore :: (a:*Header_Epoch,m:Header_Mark,bytes:[24]u8){a.cursor=m.cursor;(cast(*[24]u8)a).* = bytes;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let bad: Vec<_> = [
        "header_epoch_bad",
        "header_epoch_bad_reset",
        "header_epoch_bad_restore",
    ]
    .into_iter()
    .filter(|name| {
        a.relations(id(&c, name)).is_some_and(|r| {
            !r.protocols.is_empty() || !r.rewinds.is_empty() || !r.resets.is_empty()
        })
    })
    .collect();
    assert!(
        bad.is_empty(),
        "untracked writes preserved transitions: {bad:?}"
    );
}

#[test]
fn temporary_epochs_budget_exhaustion_never_publishes_complete_protocol_rows() {
    let c = compiler(
        "Cutoff_Epoch :: struct{data:*u8;cursor:s64;}cutoff_epoch :: (a:*Cutoff_Epoch,n:s64)->*u8 #no_aoc{old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}main :: (){}",
    );
    assert!(
        !Analysis::build(&c)
            .relations(id(&c, "cutoff_epoch"))
            .unwrap()
            .protocols
            .is_empty()
    );
    for limit in [0, 1, 16] {
        let a = checked::build_budgeted(&c, limit);
        assert!(
            a.relations.values().all(|r| r.incomplete),
            "unprocessed row published complete at limit{limit}"
        );
    }
}

#[test]
fn temporary_epochs_separation_obligations_protect_the_whole_protocol_header() {
    let c = compiler(
        "Frame_Epoch :: struct{data:*u8;capacity:s64;cursor:s64;}Frame_Mark :: struct{cursor:s64;}frame_epoch_allocate :: (a:*Frame_Epoch,n:s64)->*u8 #no_aoc{old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}frame_epoch_mark :: (a:*Frame_Epoch)->Frame_Mark{return .{a.cursor};}main :: (){}",
    );
    let a = Analysis::build(&c);
    let p = &a
        .relations(id(&c, "frame_epoch_allocate"))
        .unwrap()
        .protocols[0];
    let size = c.ty.types.size_of(p.instance.view_ty);
    let allocation=p.obligations.iter().any(|o|matches!(o,Obligation::HeaderDataSeparation{state,..} if state.offset==0&&state.width>=size));
    let snapshot=a.relations(id(&c,"frame_epoch_mark")).unwrap().snapshots.iter().all(|m|m.obligations.iter().any(|o|matches!(o,Obligation::OutputHeaderSeparation{state,..} if state.offset==0&&state.width>=size)));
    assert!(
        allocation && snapshot,
        "whole header separation missing: allocation{allocation} snapshot{snapshot}"
    );
}

#[test]
fn temporary_epochs_cursor_must_not_overwrite_the_data_pointer_bytes() {
    let c = compiler(
        "Aliased_Epoch :: struct{data:*u8;#overlay(data) using aliased:struct{cursor:s64;}}aliased_epoch :: (a:*Aliased_Epoch,n:s64)->*u8 #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}main :: (){}",
    );
    let a = Analysis::build(&c);
    assert!(
        a.relations(id(&c, "aliased_epoch"))
            .is_none_or(|r| r.protocols.is_empty()),
        "cursor store changes protocol data pointer"
    );
}

#[test]
fn temporary_epochs_other_borrowed_outputs_cannot_bypass_state_frames() {
    let c = compiler(
        r"
 Output_Epoch :: struct{data:*u8;capacity:s64;cursor:s64;}
 Output_Mark :: struct{cursor:s64;}
 output_epoch_allocate :: (a:*Output_Epoch,n:s64)->*u8 #no_aoc{old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}
 output_epoch_mark :: (a:*Output_Epoch)->Output_Mark{return .{a.cursor};}
 output_epoch_bad_allocate :: (a:*Output_Epoch,n:s64)->(*u8,Output_Epoch) #no_aoc {old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p,a.*;}
 output_epoch_bad_reset :: (a:*Output_Epoch)->Output_Epoch{a.cursor=0;return a.*;}
 output_epoch_bad_restore :: (a:*Output_Epoch,m:Output_Mark)->Output_Epoch{a.cursor=m.cursor;return a.*;}
 output_epoch_bad_mark :: (a:*Output_Epoch)->(Output_Mark,Output_Epoch){return .{a.cursor},a.*;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let bad: Vec<_> = [
        "output_epoch_bad_allocate",
        "output_epoch_bad_reset",
        "output_epoch_bad_restore",
        "output_epoch_bad_mark",
    ]
    .into_iter()
    .filter(|name| {
        a.relations(id(&c, name)).is_some_and(|r| {
            !r.protocols.is_empty()
                || !r.resets.is_empty()
                || !r.rewinds.is_empty()
                || !r.snapshots.is_empty()
        })
    })
    .collect();
    assert!(
        bad.is_empty(),
        "unframed output preserved templates: {bad:?}"
    );
}

#[test]
fn temporary_epochs_unsigned_reinterpretation_does_not_become_signed_length_proof() {
    let c = compiler(
        "Cast_Epoch :: struct{data:*u8;cursor:s64;}cast_epoch :: (a:*Cast_Epoch,raw:u64)->*u8 #no_aoc {n:=cast(s64)raw;old:=a.cursor;p:=a.data+old;a.cursor=old+n;return p;}main :: (){}",
    );
    let a = Analysis::build(&c);
    assert!(
        a.relations(id(&c, "cast_epoch"))
            .is_none_or(|r| r.protocols.is_empty()),
        "source unsigned bits cannot silently become a signed nonnegative length witness"
    );
}

#[test]
fn temporary_epochs_whole_mark_forwarding_preserves_opaque_origin_only() {
    let c = compiler(
        r"
 Forward_Mark :: struct{page:*u8;cursor:s64;}
 whole_forward :: (m:Forward_Mark)->Forward_Mark{return m;}
 copied_forward :: (m:Forward_Mark)->Forward_Mark{copy:=m;return copy;}
 self_forward :: (m:Forward_Mark)->Forward_Mark{local:=m;local=local;return local;}
 saved_forward :: (m:Forward_Mark)->Forward_Mark{local:=m;saved:=local;local.cursor=7;return saved;}
 partial_forward :: (m:Forward_Mark)->Forward_Mark{local:=m;local.cursor=7;return local;}
 rebuilt_forward :: (m:Forward_Mark)->Forward_Mark{return .{m.page,m.cursor};}
 mixed_forward :: (a:Forward_Mark,b:Forward_Mark)->Forward_Mark{return .{a.page,b.cursor};}
 unknown_forward :: (m:Forward_Mark,other:*Forward_Mark)->Forward_Mark{local:=m;local=other.*;return local;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    for name in [
        "whole_forward",
        "copied_forward",
        "self_forward",
        "saved_forward",
    ] {
        let r = a.relations(id(&c, name)).unwrap();
        assert!(
            r.forwards
                .iter()
                .any(|f| f.source_formal == 0 && f.fields.len() == 2),
            "{name}"
        );
        assert!(r.incomplete);
    }
    for name in [
        "partial_forward",
        "rebuilt_forward",
        "mixed_forward",
        "unknown_forward",
    ] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.forwards.is_empty()),
            "{name} cannot synthesize an origin"
        );
    }
}

#[test]
fn temporary_epochs_forwarding_requires_actual_argument_snapshot_and_source_state_frame() {
    let c = compiler(
        r"
 Epoch_Forward_Mark :: struct{page:*u8;cursor:s64;}
 named_forward :: (ignore:s64,value:Epoch_Forward_Mark)->Epoch_Forward_Mark{return value;}
 late_mutate_forward :: (p:*Epoch_Forward_Mark)->s64{p.cursor=99;return 0;}
 late_caller_forward :: (m:Epoch_Forward_Mark)->Epoch_Forward_Mark{return named_forward(value=m,ignore=late_mutate_forward(*m));}
 remember_forward :: (p:*Epoch_Forward_Mark) #foreign;
 callback_forward :: (m:Epoch_Forward_Mark)->Epoch_Forward_Mark{remember_forward(*m);return m;}
 extra_output_forward :: (m:Epoch_Forward_Mark)->(Epoch_Forward_Mark,Epoch_Forward_Mark){return m,m;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let row = a.relations(id(&c, "named_forward")).unwrap();
    let f = row.forwards.iter().find(|f| f.source_formal == 1).unwrap();
    assert!(f.obligations.iter().any(|o| matches!(
        o,
        Obligation::CallerArgumentSnapshot {
            formal: 1,
            ..
        }
    )));
    assert!(f.obligations.iter().any(|o| matches!(
        o,
        Obligation::ForwardedOutputHeaderSeparation {
            result: 0,
            source_formal: 1
        }
    )));
    for name in [
        "late_caller_forward",
        "callback_forward",
        "extra_output_forward",
    ] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.forwards.is_empty()),
            "{name} remains unsupported rather than rereading the mutable argument"
        );
    }
}

#[test]
fn temporary_epochs_forwarding_copy_certificate_and_guarded_alternatives_are_retained() {
    let mut c = compiler(
        r"
 Certified_Forward_Mark :: struct{page:*u8;cursor:s64;}
 certified_forward :: (m:Certified_Forward_Mark)->Certified_Forward_Mark{return m;}
 choose_forward :: (left:Certified_Forward_Mark,right:Certified_Forward_Mark,selector:s64)->Certified_Forward_Mark{if selector<0 return left;return right;}
 alias_saved_forward :: (m:Certified_Forward_Mark)->Certified_Forward_Mark{local:=m;saved:=local;p:=*local;p.cursor=9;return saved;}
 bytes_forward :: (m:Certified_Forward_Mark,bytes:[16]u8)->Certified_Forward_Mark{local:=m;(cast(*[16]u8)*local).* = bytes;return local;}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    let choice = a.relations(id(&c, "choose_forward")).unwrap();
    assert!(
        choice
            .forwards
            .iter()
            .any(|f| f.source_formal == 0 && !f.conditions.is_empty())
    );
    assert!(
        choice
            .forwards
            .iter()
            .any(|f| f.source_formal == 1 && !f.conditions.is_empty())
    );
    assert!(
        !a.relations(id(&c, "alias_saved_forward"))
            .unwrap()
            .forwards
            .is_empty()
    );
    assert!(
        a.relations(id(&c, "bytes_forward"))
            .is_none_or(|r| r.forwards.is_empty())
    );
    c.release_lowered_code();
    assert_eq!(a.relations, Analysis::build(&c).relations);
    let p = id(&c, "certified_forward");
    let f = &a.relations(p).unwrap().forwards[0];
    let copy = f.entry_copy_site.chain.last().unwrap();
    let body =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&p)
            .unwrap();
    let event = &mut body.blocks[copy.block].events[copy.event];
    let jaic::ide::lifetimes::LifetimeEvent::Store {
        span, ..
    } = event
    else {
        panic!("actual entry-copy Store required")
    };
    *event = jaic::ide::lifetimes::LifetimeEvent::StatementEnd {
        span: *span,
    };
    assert!(
        Analysis::build(&c)
            .relations(p)
            .is_none_or(|r| r.forwards.is_empty()),
        "owned root alone cannot fabricate entry copy provenance"
    );
}

#[test]
fn temporary_epochs_real_mark_transport_uses_checked_payload_on_host_and_windows() {
    for target in [None, Some(jaic::target::TargetOs::Windows)] {
        for overwrite in [false, true] {
            let source = format!(
                "B :: #import \"Basic\"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY={overwrite});real_mark_transport :: (m:B.Temporary_Storage_State)->B.Temporary_Storage_State{{copy:=m;return copy;}}main :: (){{m:=B.get_temporary_storage_mark();n:=real_mark_transport(m);B.set_temporary_storage_mark(n);}}"
            );
            let mut c = compiler_target(
                &source,
                target,
                target.map(|_| jaic::target::TargetCpu::X64),
            );
            let p = id(&c, "real_mark_transport");
            let a = Analysis::build(&c);
            let r = a.relations(p).unwrap();
            assert!(
                r.forwards
                    .iter()
                    .any(|f| f.source_formal == 0 && f.fields.len() == 3)
            );
            assert!(r.incomplete);
            c.release_lowered_code();
            assert_eq!(a.relations, Analysis::build(&c).relations);
        }
    }
}

#[test]
fn temporary_epochs_real_initial_page_loop_bypass_keeps_header_writes() {
    for target in [None, Some(jaic::target::TargetOs::Windows)] {
        for overwrite in [false, true] {
            let source = format!(
                "B :: #import \"Basic\"()(TEMP_ALLOCATOR_OVERWRITE_MEMORY={overwrite});main :: (){{m:=B.get_temporary_storage_mark();B.set_temporary_storage_mark(m);B.reset_temporary_storage();}}"
            );
            let mut c = compiler_target(
                &source,
                target,
                target.map(|_| jaic::target::TargetCpu::X64),
            );
            let p = id(&c, "free_pages_down_to");
            let a = Analysis::build(&c);
            let row = a.relations(p).unwrap();
            let b = &row.loop_bypasses[0];
            assert!(
                b.hypotheses
                    .iter()
                    .any(|h| matches!(h, LoopHypothesis::EntryPointerZero { .. }))
            );
            assert!(
                b.hypotheses
                    .iter()
                    .any(|h| matches!(h, LoopHypothesis::NullArgumentSnapshot { .. }))
            );
            assert!(!b.component.is_empty() && !b.excluded_body_edges.is_empty());
            assert!(b.paths.iter().all(|p| !p.writes.is_empty()));
            assert!(
                b.paths.iter().any(|p| p.writes.len() >= 4),
                "lazy original-data initialization must remain visible"
            );
            assert!(
                row.rewinds.is_empty() && row.resets.is_empty() && row.incomplete,
                "bypass is not rewind/free/identity"
            );
            c.release_lowered_code();
            assert_eq!(a.relations, Analysis::build(&c).relations);
        }
    }
}

#[test]
fn temporary_epochs_loop_bypass_does_not_skip_feasible_writes_calls_or_aliases() {
    let c = compiler(
        r"
 Bypass_Node :: struct{next:*Bypass_Node;}
 Bypass_Header :: struct{link:*Bypass_Node;original:*u8;data:*u8;}
 bypass_touch :: (p:*Bypass_Node) #foreign;
 bypass_good :: (h:*Bypass_Header,stop:*Bypass_Node){if !h.original&&!h.link h.original=h.data;while h.link!=stop{top:=h.link;h.link=top.next;bypass_touch(top);}h.data=h.original;}
 bypass_write :: (h:*Bypass_Header,stop:*Bypass_Node,replacement:*Bypass_Node){h.link=replacement;while h.link!=stop{top:=h.link;h.link=top.next;bypass_touch(top);}}
 bypass_branch :: (h:*Bypass_Header,stop:*Bypass_Node,replacement:*Bypass_Node,selector:s64){if selector>0 h.link=replacement;while h.link!=stop{top:=h.link;h.link=top.next;bypass_touch(top);}}
 bypass_call :: (h:*Bypass_Header,stop:*Bypass_Node){bypass_touch(h.link);while h.link!=stop{top:=h.link;h.link=top.next;bypass_touch(top);}}
 Bypass_Alias :: struct{link:*Bypass_Node;#overlay(link) using view:struct{original:*u8;}data:*u8;}
 bypass_alias :: (h:*Bypass_Alias,stop:*Bypass_Node){if !h.original&&!h.link h.original=h.data;while h.link!=stop{top:=h.link;h.link=top.next;bypass_touch(top);}}
 main :: (){}
 ",
    );
    let a = Analysis::build(&c);
    assert!(
        !a.relations(id(&c, "bypass_good"))
            .unwrap()
            .loop_bypasses
            .is_empty()
    );
    for name in [
        "bypass_write",
        "bypass_branch",
        "bypass_call",
        "bypass_alias",
    ] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.loop_bypasses.is_empty()),
            "{name} cannot certify one successful arm alone"
        );
    }
}

#[test]
fn temporary_epochs_loop_bypass_rejects_other_accesses_cycles_and_exhaustion() {
    let c = compiler(
        r"
 Boundary_Node::struct{next:*Boundary_Node;}
 Boundary_Header::struct{head:*Boundary_Node;data:*u8;}
 boundary_touch::(p:*Boundary_Node)#foreign;
 boundary_other::(h:*Boundary_Header,stop:*Boundary_Node,other:*u8){x:=other.*;while h.head!=stop{top:=h.head;h.head=top.next;boundary_touch(top);}h.data=cast(*u8)x;}
 boundary_cycle::(h:*Boundary_Header,stop:*Boundary_Node,flag:s64){while flag>0{}while h.head!=stop{top:=h.head;h.head=top.next;boundary_touch(top);}}
 boundary_opaque::(h:*Boundary_Header,stop:*Boundary_Node){#asm {nop;}while h.head!=stop{top:=h.head;h.head=top.next;boundary_touch(top);}}
 boundary_good::(h:*Boundary_Header,stop:*Boundary_Node){while h.head!=stop{top:=h.head;h.head=top.next;boundary_touch(top);}h.data=null;}
 main::(){}
 ",
    );
    let a = Analysis::build(&c);
    assert!(
        !a.relations(id(&c, "boundary_good"))
            .unwrap()
            .loop_bypasses
            .is_empty()
    );
    for name in ["boundary_other", "boundary_cycle", "boundary_opaque"] {
        assert!(
            a.relations(id(&c, name))
                .is_none_or(|r| r.loop_bypasses.is_empty()),
            "{name}"
        );
    }
    assert!(checked::build_budgeted(&c, 0).relations.is_empty());
    assert!(checked::build_budgeted(&c, 100).relations.is_empty());
}
