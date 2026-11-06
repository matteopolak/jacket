//! Completed immutable certificates share buffers, not dynamic resource identities.
use super::*;

#[test]
fn completed_certificate_clones_share_storage_and_last_owner_releases_it() {
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Linux;
    options.cpu = jaic::target::TargetCpu::X64;
    let c = super::stdlib_allocations::checked_source_options(
        r#"
        #import "Basic";
        C::#system_library "c";
        allocation_probe::(bytes:u64)->*void #c_call #foreign C "malloc";
        zero::(p:*s64){p.*=0;}
        driver::(header:*[..]s64)->*s64 {
            unused:=allocation_probe(64);
            zero(header.data);
            return array_add(header);
        }
        main::(){}
    "#,
        options,
    );
    let all = Summaries::build(&c);
    let frame = Rc::downgrade(all.procs.values().find_map(|s| s.frame.as_ref()).unwrap());
    let borrow = Rc::downgrade(
        all.procs
            .values()
            .find_map(|s| s.no_growth.as_ref())
            .unwrap(),
    );
    let allocation = Rc::downgrade(
        all.procs
            .values()
            .find_map(|s| s.native_allocation.as_ref())
            .unwrap(),
    );
    let bits = Rc::downgrade(
        all.procs
            .values()
            .find_map(|s| s.bit_return.as_ref())
            .unwrap(),
    );
    let copied = all.procs.clone();
    assert_eq!(
        copied, all.procs,
        "convergence retains structural value equality"
    );
    let mut seen = [false; 4];
    for (id, source) in &all.procs {
        let clone = &copied[id];
        if let Some((a, b)) = source.frame.as_ref().zip(clone.frame.as_ref()) {
            assert!(Rc::ptr_eq(a, b));
            seen[0] = true;
        }
        if let Some((a, b)) = source.no_growth.as_ref().zip(clone.no_growth.as_ref()) {
            assert!(Rc::ptr_eq(a, b));
            seen[1] = true;
        }
        if let Some((a, b)) = source
            .native_allocation
            .as_ref()
            .zip(clone.native_allocation.as_ref())
        {
            assert!(Rc::ptr_eq(a, b));
            seen[2] = true;
        }
        if let Some((a, b)) = source.bit_return.as_ref().zip(clone.bit_return.as_ref()) {
            assert!(Rc::ptr_eq(a, b));
            seen[3] = true;
        }
    }
    assert!(seen.into_iter().all(|found| found));
    drop(copied);
    assert!(
        frame.upgrade().is_some()
            && borrow.upgrade().is_some()
            && allocation.upgrade().is_some()
            && bits.upgrade().is_some()
    );
    drop(all);
    assert!(
        frame.upgrade().is_none()
            && borrow.upgrade().is_none()
            && allocation.upgrade().is_none()
            && bits.upgrade().is_none()
    );
}


#[test]
fn lock_target_copies_share_path_buffers_without_mutating_native_templates() {
    eprintln!(
        "certificate-layout summary_inline={} lock_transfer_inline={}",
        std::mem::size_of::<Summary>(),
        std::mem::size_of::<super::super::locks::Transfer>()
    );
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    let c = super::stdlib_allocations::checked_source_options(
        r#"
        K::#library,system "kernel32";
        native_enter_probe::(p:*void) #foreign K "EnterCriticalSection";
        Holder::struct{prefix:u64;storage:[8]u64;}
        wrapper::(p:*Holder){native_enter_probe(*p.storage);}
        caller::(p:*Holder){wrapper(p);}
        empty::(){}
        main::(){}
    "#,
        options,
    );
    let all = Summaries::build(&c);
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let native = all.procs[&id("native_enter_probe")].locks.as_ref().unwrap();
    let wrapper = all.procs[&id("wrapper")].locks.as_ref().unwrap();
    let caller = all.procs[&id("caller")].locks.as_ref().unwrap();
    assert!(native.target.as_ref().unwrap().steps.is_empty());
    assert!(!wrapper.target.as_ref().unwrap().steps.is_empty());
    assert_eq!(native.object.unwrap().offset, 0);
    assert_eq!(wrapper.object.unwrap().offset, 8);
    assert_eq!(caller.object.unwrap().offset, 8);
    assert!(
        all.procs[&id("empty")]
            .locks
            .as_ref()
            .unwrap()
            .target
            .is_none()
    );
    let weak = Rc::downgrade(wrapper.target.as_ref().unwrap());
    let copied = all.procs[&id("wrapper")].clone();
    assert_eq!(copied, all.procs[&id("wrapper")]);
    assert_eq!(
        copied
            .locks
            .as_ref()
            .unwrap()
            .target
            .as_ref()
            .unwrap()
            .steps
            .as_ptr(),
        wrapper.target.as_ref().unwrap().steps.as_ptr(),
        "immutable target buffers should not be copied just to overwrite a Summary"
    );
    assert!(Rc::ptr_eq(
        copied.locks.as_ref().unwrap().target.as_ref().unwrap(),
        wrapper.target.as_ref().unwrap()
    ));
    assert!(super::super::locks::Transfer::default().target.is_none());
    drop(copied);
    drop(all);
    assert!(weak.upgrade().is_none());
}
