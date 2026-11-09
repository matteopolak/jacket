//! The inline scheduler's thread slots, seen from Jai code. The POSIX tests are the ones the
//! native scheduler runs (`threads/slot_tests.rs`).
use super::SandboxHost;
use super::threads::slot_tests::{posix, posix_slot_tests, posix_warmed};
use crate::ir::{FuncId, Linkage};
use crate::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

/// Run the `check :: () -> bool` in `source` as a program on the inline scheduler (compile-time
/// code cannot suspend, so it is not run there) and return its slot count.
fn slots_after(source: &str) -> usize {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler.ct.interp.host = Box::new(SandboxHost::default());
    let source = format!(
        "{source}\n#program_export \"probe\" probe :: () -> s64 #c_call {{ if check() return 0; return 1; }}\n"
    );
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&compiler.src.sources));
    }
    let probe = compiler
        .out
        .program
        .funcs
        .iter()
        .position(|f| {
            f.as_ref()
                .is_some_and(|f| f.linkage == Linkage::Export("probe".into()))
        })
        .expect("the probe is exported");
    compiler.ct.interp.compile_time = false;
    let result = compiler
        .ct
        .interp
        .call(&compiler.out.program, FuncId(probe as u32), &[]);
    assert_eq!(result.unwrap(), vec![0], "check() failed");
    compiler
        .ct
        .interp
        .isched
        .as_ref()
        .expect("the program made threads")
        .slot_count()
}

posix_slot_tests!(slots_after);

#[test]
fn threads_taken_with_tryjoin_reuse_one_slot() {
    let source = posix(
        "
    count:u64=0;
    while count<50 {
        thread := spawn(count+1);
        result:*void;
        while pthread_tryjoin_np(thread,*result)!=0 { }
        if cast(u64)result!=count+1 return false;
        // The thread was joined: its handle is dead.
        if pthread_tryjoin_np(thread,*result)==0 return false;
        count+=1;
    }
    return true;",
    )
    .replace(
        "sched_yield :: ()->s32 #foreign;",
        "sched_yield :: ()->s32 #foreign;\npthread_tryjoin_np :: (thread:u64,result:**void)->s32 #foreign;",
    );
    assert_eq!(slots_after(&source), 2);
}

#[test]
fn a_second_joiner_of_one_thread_fails_instead_of_waiting_for_a_later_thread() {
    // Two threads wait for the same one. The first to run takes its result and frees the
    // slot; the other must not go on waiting for whatever thread takes the slot next.
    let source = r"
pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
pthread_join :: (thread:u64,result:**void)->s32 #foreign;
sched_yield :: ()->s32 #foreign;
target:u64;
other:u64;
helper_status:s32=-1;
quick :: (argument:*void)->*void #c_call { return argument; }
slow :: (argument:*void)->*void #c_call { sched_yield(); return argument; }
joiner :: (argument:*void)->*void #c_call {
    result:*void;
    helper_status = pthread_join(target,*result);
    // The slot of `target` is free now, and this thread takes it.
    pthread_create(*other,null,cast(*void)quick,cast(*void)6);
    return null;
}
check :: ()->bool #no_context {
    result:*void;
    if pthread_join(0,*result)==0 return false;
    pthread_create(*target,null,cast(*void)slow,cast(*void)5);
    helper:u64;
    pthread_create(*helper,null,cast(*void)joiner,null);
    sched_yield();
    // Both wait for `target`.
    main_status := pthread_join(target,*result);
    if pthread_join(helper,*result)!=0 return false;
    if (main_status==0)==(helper_status==0) return false;
    if pthread_join(other,*result)!=0 return false;
    return cast(u64)result==6;
}
";
    assert!(slots_after(source) <= 4);
}
