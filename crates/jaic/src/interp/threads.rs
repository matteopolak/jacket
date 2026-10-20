//! Threads for interpreted programs under native `jaic -interp`.
//!
//! Every Jai thread is an OS thread, but only the one holding the baton runs interpreted code,
//! so `Interp` is never touched by two threads at once. The baton moves when its holder blocks
//! (join, mutex, condition variable, semaphore, sleep), yields, has run for a while
//! (`preempt`), or stays inside one native call for longer than `NATIVE_SLICE`: a thread
//! calling C keeps the baton but lets others take it (`enter_native`), and waits for it again
//! when C returns (`leave_native`). Short C calls therefore cost no switch, and long ones
//! (`read`, `accept`, a sleep in C) no longer stop the other threads.
//!
//! Mutexes and condition variables are emulated here instead of calling libc, because a
//! thread blocked inside libc on a lock another Jai thread holds could never get it.
//!
//! A thread C started itself that calls a `#c_call` procedure (an audio render thread) is
//! adopted for the length of that call: it becomes a scheduler thread, takes the baton like any
//! other, and may block and be woken. A callback on a thread already inside a native call of
//! this interpreter (C -> Jai -> C -> Jai) runs as that thread.
//!
//! The scheduler (`Sched`) sits behind one mutex shared by all of these threads (`Shared`,
//! also the gate thunks enter through). Each thread carries its own interpreter state
//! (`ExecState`) and puts it into `Interp` while it holds the baton.
//!
//! The Win32 equivalents (`CreateThread`, critical sections, SRW locks, condition variables,
//! semaphores and events, waits on their handles) map onto the same scheduler; see `win32.rs`.
#![allow(unsafe_code)]

#[cfg(not(panic = "unwind"))]
compile_error!("native interpreter worker cancellation requires Rust panic=unwind");

mod ops;
mod pthread;
mod sched;
mod shared;
mod win32;
mod worker;

pub use self::sched::DEADLOCK_GRACE;
pub(super) use self::sched::Shared;
use self::sched::{Block, ETIMEDOUT, GThread, Worker};
use super::{Interp, Program, Res, Trap, TrapKind, Ty};

#[cfg(test)]
mod pthread_name_tests {
    use super::*;

    #[test]
    fn invalid_pthread_name_handles_never_fall_through_to_native() {
        let mut interp = Interp::default();
        for (symbol, args) in [
            ("pthread_setname_np", vec![0, 0]),
            ("pthread_getname_np", vec![0, 0, 16]),
            ("pthread_getname_np", vec![u64::MAX, 0, 16]),
        ] {
            let result = interp
                .thread_foreign(&Program::default(), symbol, &args)
                .expect("scheduler-owned pthread naming must be intercepted");
            assert!(result.is_err(), "invalid identity must not report success");
        }
    }

    #[test]
    fn inline_pthread_names_report_unsupported_without_native_fallback() {
        let mut interp = Interp::default();
        for (symbol, args) in [
            ("pthread_setname_np", vec![1, 0]),
            ("pthread_setname_np", vec![0]),
            ("pthread_getname_np", vec![1, 0, 16]),
        ] {
            let error = interp
                .inline_thread_foreign(&Program::default(), symbol, &args)
                .expect("inline scheduler must own naming dispatch")
                .expect_err("no OS pthread exists in the inline scheduler");
            assert!(error.message.contains("unsupported"), "{error:?}");
        }
    }

    #[test]
    fn wrong_signature_and_cross_target_names_are_explicit_errors() {
        let mut interp = Interp::default();
        for symbol in ["pthread_setname_np", "pthread_getname_np"] {
            assert!(
                interp
                    .thread_foreign(&Program::default(), symbol, &[])
                    .unwrap()
                    .is_err()
            );
            let program = Program {
                cross_target: true,
                ..Program::default()
            };
            let error = interp
                .thread_foreign(&program, symbol, &[1, 0, 16])
                .unwrap()
                .unwrap_err();
            assert!(error.message.contains("cross-target"));
        }
    }

    #[test]
    fn finished_or_identityless_slots_cannot_be_named() {
        let mut interp = Interp::default();
        let shared = interp.shared();
        let mut thread = GThread::new(false, None);
        thread.finished = true;
        shared.lock().threads.push(thread);
        shared.lock().threads.push(GThread::new(false, None));
        for handle in [2, 3] {
            assert!(
                interp
                    .thread_foreign(&Program::default(), "pthread_getname_np", &[handle, 0, 64])
                    .unwrap()
                    .is_err()
            );
        }
    }

    #[test]
    fn unrelated_foreigns_keep_their_existing_route() {
        let mut interp = Interp::default();
        assert!(
            interp
                .thread_foreign(&Program::default(), "unrelated_symbol", &[])
                .is_none()
        );
        assert!(
            interp
                .inline_thread_foreign(&Program::default(), "unrelated_symbol", &[])
                .is_none()
        );
    }
}

#[cfg(test)]
mod worker_lifetime_tests {
    #[cfg(windows)]
    use super::sched::CANCELLED_EXIT_CODE;
    use super::*;
    use crate::interp::{Host, ir, native};
    use crate::sema::{Compiler, Options, ProgramSource, VirtualFs};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    #[test]
    fn compiler_keeps_its_owned_state_alive_through_interpreter_teardown() {
        struct LifetimeHost {
            source: std::rc::Weak<str>,
            alive: Rc<std::cell::Cell<bool>>,
        }
        impl Host for LifetimeHost {
            fn write(&mut self, _: &[u8], _: bool) {
                panic!("teardown-only observer must not execute");
            }
            fn foreign(
                &mut self,
                _: &str,
                _: &[u64],
                _: &ir::Sig,
            ) -> Option<Result<Vec<u64>, String>> {
                panic!("teardown-only observer must not execute");
            }
            fn native_linking(&self) -> bool {
                panic!("teardown-only observer must not execute");
            }
        }
        impl Drop for LifetimeHost {
            fn drop(&mut self) {
                self.alive.set(self.source.upgrade().is_some());
            }
        }
        let mut compiler = Compiler::new(Options::host(), Rc::new(VirtualFs::default()));
        let source: Rc<str> = "main :: () {}".into();
        let weak = Rc::downgrade(&source);
        compiler.sources.add("lifetime-order.jai", source);
        let alive = Rc::new(std::cell::Cell::new(false));
        compiler.ct.interp.host = Box::new(LifetimeHost {
            source: weak,
            alive: alive.clone(),
        });
        drop(compiler);
        assert!(
            alive.get(),
            "compiler state must outlive interpreter shutdown and worker joins"
        );
    }

    fn cancelled_worker_is_reaped(suspended: bool) {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        let source = format!(
            r"
CreateThread :: (security:*void, stack:u64, entry:*void, argument:*void, flags:u32, id:*u32)->u64 #foreign;
Sleep :: (milliseconds:u32) #foreign;
worker :: (argument:*void)->u32 #c_call {{ Sleep(0xffff_ffff); return 17; }}
#run {{ CreateThread(null,0,cast(*void)worker,null,{},null); Sleep(5); }}
",
            if suspended {
                4
            } else {
                0
            }
        );
        compiler
            .compile_sources(&[ProgramSource::String(source)])
            .unwrap();
        let shared = compiler.ct.interp.shared.as_ref().unwrap().clone();
        assert!(
            Arc::strong_count(&shared) > 1,
            "worker must exist before teardown"
        );
        #[cfg(windows)]
        let alias = shared.lock().threads[1]
            .native_handle
            .as_ref()
            .unwrap()
            .try_clone()
            .unwrap();
        drop(compiler);
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            assert_eq!(
                win32::native_exit_code(alias.as_raw_handle() as u64)
                    .unwrap()
                    .1,
                CANCELLED_EXIT_CODE
            );
        }
        assert_eq!(
            Arc::strong_count(&shared),
            1,
            "teardown must join workers, not park them with their shared state"
        );
    }

    #[test]
    fn shutdown_releases_a_callback_lease_waiting_for_the_baton() {
        // No interpreter dereference occurs: root slot retains the baton throughout.
        let shared = Shared::new(0);
        let callback_shared = shared.clone();
        let join = std::thread::spawn(move || {
            native::Gate::run(&*callback_shared, 0, &mut |_| {
                panic!("cancelled callback must not execute")
            })
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !shared.lock().threads.iter().any(|thread| thread.adopted) {
            if Instant::now() >= deadline {
                shared.close();
                assert!(join.join().expect("callback must stop normally").is_err());
                panic!("callback must request its lease");
            }
            std::thread::yield_now();
        }
        shared.close();
        assert!(join.join().unwrap().is_err());
        assert!(
            shared
                .lock()
                .threads
                .iter()
                .filter(|thread| thread.adopted)
                .all(|thread| thread.finished)
        );
        assert_eq!(Arc::strong_count(&shared), 1);
    }

    #[test]
    fn teardown_reaps_a_worker_returning_from_a_finite_native_call() {
        let mut options = Options::host();
        options.runtime_support = false;
        let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
        let (foreign, call) = if cfg!(windows) {
            (
                "pause :: (milliseconds:u32,alertable:s32)->u32 #foreign \"SleepEx\";",
                "pause(50,0);",
            )
        } else {
            (
                "pause :: (fds:*void,count:u64,timeout:s32)->s32 #foreign \"poll\";",
                "pause(null,0,50);",
            )
        };
        let source = format!(
            r"
CreateThread :: (security:*void, stack:u64, entry:*void, argument:*void, flags:u32, id:*u32)->u64 #foreign;
Sleep :: (milliseconds:u32) #foreign;
{foreign}
worker :: (argument:*void)->u32 #c_call {{ {call} return 17; }}
#run {{ CreateThread(null,0,cast(*void)worker,null,0,null); Sleep(5); }}
"
        );
        compiler
            .compile_sources(&[ProgramSource::String(source)])
            .unwrap();
        let shared = compiler.ct.interp.shared.as_ref().unwrap().clone();
        drop(compiler);
        assert_eq!(
            Arc::strong_count(&shared),
            1,
            "native call must return, cancel safely and join"
        );
    }

    #[test]
    fn teardown_reaps_a_never_started_suspended_worker() {
        cancelled_worker_is_reaped(true);
    }

    #[test]
    fn teardown_reaps_a_worker_blocked_in_interpreted_sleep() {
        cancelled_worker_is_reaped(false);
    }
}
