//! Slot reuse seen from Jai code: handles of finished threads, joins and detaches.
use crate::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

/// Run `source` as a compile-time program and return the scheduler's slot count.
fn slots_after(source: &str) -> usize {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", error.render(&compiler.src.sources));
    }
    let shared = compiler.ct.interp.shared.as_ref().unwrap().clone();
    shared.lock().threads.len()
}

const POSIX: &str = r"
pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
pthread_join :: (thread:u64,result:**void)->s32 #foreign;
pthread_detach :: (thread:u64)->s32 #foreign;
sched_yield :: ()->s32 #foreign;
echo :: (argument:*void)->*void #c_call { return argument; }
spawn :: (value:u64)->u64 #no_context {
    thread:u64;
    if pthread_create(*thread,null,cast(*void)echo,cast(*void)value)!=0 return 0;
    return thread;
}
reap :: (thread:u64)->u64 #no_context {
    result:*void;
    if pthread_join(thread,*result)!=0 return 0;
    return cast(u64)result;
}
";

fn posix(body: &str) -> String {
    // Compile-time code is compiled lazily and not while a thread exists: call `reap` once
    // (on no thread) before the first thread starts.
    format!(
        "{POSIX}\ncheck :: ()->bool #no_context {{\n    if reap(0)!=0 return false;\n{body}\n}}\n#assert check();\n"
    )
}

#[test]
fn threads_made_and_joined_in_turn_reuse_one_slot() {
    let source = posix(
        "
    value:u64=1;
    while value<=300 {
        thread := spawn(value);
        if thread==0 return false;
        if reap(thread)!=value return false;
        value+=1;
    }
    return true;",
    );
    assert_eq!(
        slots_after(&source),
        2,
        "the first thread and one worker slot"
    );
}

#[test]
fn several_live_threads_joined_in_any_order_keep_their_own_results() {
    let source = posix(
        "
    round:u64=0;
    while round<40 {
        a := spawn(round*10+1);
        b := spawn(round*10+2);
        c := spawn(round*10+3);
        if reap(b)!=round*10+2 return false;
        d := spawn(round*10+4);
        if reap(d)!=round*10+4 return false;
        if reap(c)!=round*10+3 return false;
        if reap(a)!=round*10+1 return false;
        round+=1;
    }
    return true;",
    );
    assert!(
        slots_after(&source) <= 5,
        "four workers at most are alive together"
    );
}

#[test]
fn a_joined_handle_cannot_reach_the_thread_that_took_its_slot() {
    let source = posix(
        "
    first := spawn(11);
    if reap(first)!=11 return false;
    second := spawn(22);
    if second==first return false;
    // The slot is reused and the new thread has not run yet: the old handle must fail at
    // once instead of waiting for it or returning its result.
    result:*void;
    if pthread_join(first,*result)==0 return false;
    if reap(second)!=22 return false;
    return true;",
    );
    assert_eq!(slots_after(&source), 2);
}

#[test]
fn a_slot_is_not_reused_while_its_handle_is_unjoined() {
    let source = posix(
        "
    first := spawn(5);
    sched_yield();
    second := spawn(6);
    if first==second return false;
    if reap(second)!=6 return false;
    if reap(first)!=5 return false;
    return true;",
    );
    assert_eq!(slots_after(&source), 3);
}

#[test]
fn detached_threads_free_their_slots_when_they_end() {
    let source = posix(
        "
    count:u64=0;
    while count<200 {
        thread := spawn(count+1);
        if thread==0 return false;
        if pthread_detach(thread)!=0 return false;
        // Let the thread run to its end before the next one starts.
        sched_yield();
        count+=1;
    }
    return true;",
    );
    assert!(slots_after(&source) <= 3);
}

#[test]
fn a_thread_detached_after_it_ended_frees_its_slot_at_once() {
    let source = posix(
        "
    count:u64=0;
    while count<100 {
        thread := spawn(count+1);
        sched_yield();
        if pthread_detach(thread)!=0 return false;
        count+=1;
    }
    return true;",
    );
    assert!(slots_after(&source) <= 3);
}

const WIN32: &str = r"
CreateThread :: (security:*void, stack:u64, entry:*void, argument:*void, flags:u32, id:*u32)->u64 #foreign;
WaitForSingleObject :: (handle:u64, milliseconds:u32)->u32 #foreign;
GetExitCodeThread :: (handle:u64, code:*u32)->s32 #foreign;
CloseHandle :: (handle:u64)->s32 #foreign;
code_of :: (argument:*void)->u32 #c_call { return cast(u32)cast(u64)argument; }
start :: (value:u64)->u64 #no_context {
    return CreateThread(null,0,cast(*void)code_of,cast(*void)value,0,null);
}
";

#[test]
fn closed_win32_threads_reuse_their_slot_and_open_ones_keep_theirs() {
    let source = format!(
        "{WIN32}
check :: ()->bool #no_context {{
    kept := start(7);
    if WaitForSingleObject(kept,0xffffffff)!=0 return false;
    count:u64=0;
    while count<200 {{
        h := start(count+100);
        if WaitForSingleObject(h,0xffffffff)!=0 return false;
        code:u32;
        if GetExitCodeThread(h,*code)==0 return false;
        if code!=cast(u32)(count+100) return false;
        if CloseHandle(h)==0 return false;
        count+=1;
    }}
    code:u32;
    if GetExitCodeThread(kept,*code)==0 return false;
    if code!=7 return false;
    return CloseHandle(kept)!=0;
}}
#assert check();
"
    );
    assert_eq!(
        slots_after(&source),
        3,
        "the first thread, the kept one and one reused slot"
    );
}
