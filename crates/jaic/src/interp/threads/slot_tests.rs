//! Slot reuse seen from Jai code: handles of finished threads, joins and detaches.
//!
//! The POSIX tests are a macro so the inline scheduler (`threads_inline_tests.rs`) runs the
//! same ones.
use crate::sema::{Compiler, Options, ProgramSource, VirtualFs};
use std::rc::Rc;

/// Run the `check :: () -> bool` in `source` at compile time and return the scheduler's slot
/// count.
fn slots_after(source: &str) -> usize {
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    let source = format!("{source}\n#assert check();\n");
    if let Err(error) = compiler.compile_sources(&[ProgramSource::String(source)]) {
        panic!("{}", error.render(&compiler.src.sources));
    }
    let shared = compiler.ct.interp.shared.as_ref().unwrap().clone();
    shared.lock().threads.len()
}

pub(in crate::interp) const POSIX: &str = r"
pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
pthread_join :: (thread:u64,result:**void)->s32 #foreign;
pthread_detach :: (thread:u64)->s32 #foreign;
pthread_mutex_init :: (mutex:*void,attributes:*void)->s32 #foreign;
pthread_mutex_lock :: (mutex:*void)->s32 #foreign;
pthread_mutex_unlock :: (mutex:*void)->s32 #foreign;
sched_yield :: ()->s32 #foreign;
echo :: (argument:*void)->*void #c_call { return argument; }
locker :: (argument:*void)->*void #c_call { pthread_mutex_lock(argument); return argument; }
unlocker :: (argument:*void)->*void #c_call { return cast(*void)cast(u64)pthread_mutex_unlock(argument); }
spawn_on :: (entry:*void,value:u64)->u64 #no_context {
    thread:u64;
    if pthread_create(*thread,null,entry,cast(*void)value)!=0 return 0;
    return thread;
}
spawn :: (value:u64)->u64 #no_context { return spawn_on(cast(*void)echo,value); }
reap :: (thread:u64)->u64 #no_context {
    result:*void;
    if pthread_join(thread,*result)!=0 return 0;
    return cast(u64)result;
}
";

/// A test program around `body`, a `check` body that returns a bool.
pub(in crate::interp) fn posix(body: &str) -> String {
    posix_warmed(&[], body)
}

/// Like `posix`, with each of `entries` (thread entry points defined in the program) run once
/// on a thread of its own first. Compile-time code is compiled lazily and not while a thread
/// exists, so every entry a test starts later must already have run once, alone. `reap` is
/// always warmed. An entry that writes a global leaves it changed: reset it at the start of
/// `body`.
pub(in crate::interp) fn posix_warmed(entries: &[&str], body: &str) -> String {
    let warm: String = entries
        .iter()
        .map(|entry| format!("    reap(spawn_on(cast(*void){entry},0));\n"))
        .collect();
    format!(
        "{POSIX}\ncheck :: ()->bool #no_context {{\n    if reap(0)!=0 return false;\n{warm}{body}\n}}\n"
    )
}

/// The tests run against each scheduler: `$slots_after` runs a source under it and returns how
/// many thread slots it made. `posix` must be in scope where this is used.
macro_rules! posix_slot_tests {
    ($slots_after:path) => {
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
                $slots_after(&source),
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
                $slots_after(&source) <= 5,
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
            assert_eq!($slots_after(&source), 2);
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
            assert_eq!($slots_after(&source), 3);
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
            assert!($slots_after(&source) <= 3);
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
            assert!($slots_after(&source) <= 3);
        }

        #[test]
        fn a_detached_handle_is_dead() {
            let source = posix(
                "
    thread := spawn(3);
    if pthread_detach(thread)!=0 return false;
    if pthread_detach(thread)==0 return false;
    result:*void;
    if pthread_join(thread,*result)==0 return false;
    sched_yield();
    if pthread_detach(thread)==0 return false;
    return true;",
            );
            assert_eq!($slots_after(&source), 2);
        }

        #[test]
        fn detaching_a_thread_wakes_a_blocked_joiner_with_an_error() {
            // The target waits for a lock main holds, so it cannot end first.
            let source = posix_warmed(
                &["joiner"],
                "
    if pthread_mutex_init(*gate,null)!=0 return false;
    helper:u64;
    result:*void;
    // Warming ran the joiner once: undo what it did.
    join_status=-1;
    if pthread_mutex_lock(*gate)!=0 return false;
    pthread_create(*target,null,cast(*void)blocked,null);
    sched_yield();
    pthread_create(*helper,null,cast(*void)joiner,null);
    sched_yield();
    if pthread_detach(target)!=0 return false;
    turns:u64=0;
    while join_status==-1 && turns<20 {
        sched_yield();
        turns+=1;
    }
    // The joiner got EINVAL while the thread was still running.
    if join_status==-1 || join_status==0 return false;
    // Joining it now is refused at once as well.
    if pthread_join(target,*result)==0 return false;
    if pthread_mutex_unlock(*gate)!=0 return false;
    if pthread_join(helper,*result)!=0 return false;
    return true;",
            ) + "
gate:[8]u64;
target:u64;
join_status:s32=-1;
blocked :: (argument:*void)->*void #c_call { pthread_mutex_lock(*gate); pthread_mutex_unlock(*gate); return argument; }
joiner :: (argument:*void)->*void #c_call {
    result:*void;
    join_status = pthread_join(target,*result);
    return argument;
}
";
            $slots_after(&source);
        }

        #[test]
        fn a_lock_held_by_a_finished_thread_stays_held_among_many_mutexes() {
            // A thread that ends holding a lock leaves it held, and threads that take its
            // slot never own it, however many other mutexes there are.
            let source = posix(
                "
    locks:[512]u64;
    index:u64=0;
    while index<512 {
        if pthread_mutex_init(*locks[index],null)!=0 return false;
        index+=1;
    }
    // A thread locks the last mutex and ends without unlocking it.
    last := cast(u64)*locks[511];
    if reap(spawn_on(cast(*void)locker,last))!=last return false;
    // The thread that takes its slot does not own the lock.
    if reap(spawn_on(cast(*void)unlocker,last))==0 return false;
    count:u64=0;
    while count<100 {
        thread := spawn(count);
        if reap(thread)!=count return false;
        count+=1;
    }
    if pthread_mutex_unlock(*locks[511])==0 return false;
    // The main thread's own locks still work beside them.
    if pthread_mutex_lock(*locks[0])!=0 return false;
    if pthread_mutex_unlock(*locks[0])!=0 return false;
    return true;",
            );
            assert_eq!($slots_after(&source), 2);
        }
    };
}

pub(in crate::interp) use posix_slot_tests;

posix_slot_tests!(slots_after);

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
"
    );
    assert_eq!(
        slots_after(&source),
        3,
        "the first thread, the kept one and one reused slot"
    );
}

/// A kernel-backed thread handle (`CreateThread` on Windows) is closed once, which frees its
/// slot, and a second `CloseHandle` on it fails the way the kernel reports a dead handle.
#[cfg(windows)]
#[test]
fn closing_a_closed_kernel_thread_handle_fails_with_invalid_handle() {
    let source = format!(
        "{WIN32}
GetLastError :: ()->u32 #foreign;
check :: ()->bool #no_context {{
    h := start(9);
    if WaitForSingleObject(h,0xffffffff)!=0 return false;
    if CloseHandle(h)==0 return false;
    if CloseHandle(h)!=0 return false;
    // ERROR_INVALID_HANDLE
    return GetLastError()==6;
}}
"
    );
    assert_eq!(
        slots_after(&source),
        2,
        "the first thread and one worker slot"
    );
}

/// Closing a kernel-backed handle whose thread may still be running frees the slot once the
/// thread ends, so later threads reuse it.
#[cfg(windows)]
#[test]
fn closing_a_running_kernel_thread_handle_frees_its_slot_when_it_ends() {
    let source = format!(
        "{WIN32}
check :: ()->bool #no_context {{
    count:u64=0;
    while count<50 {{
        h := start(count);
        if CloseHandle(h)==0 return false;
        count+=1;
    }}
    return true;
}}
"
    );
    assert!(slots_after(&source) <= 4);
}
