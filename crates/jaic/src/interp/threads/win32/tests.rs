use super::*;
use crate::interp::NativeHost;
#[cfg(windows)]
use crate::interp::native;

#[test]
fn waiting_on_a_closed_scheduler_handle_fails_instead_of_reaching_native() {
    let mut interp = Interp::default();
    let program = Program::default();
    let closed = interp.new_object(Object::Semaphore {
        count: 0,
        max: 1,
    });
    let live = interp.new_object(Object::Semaphore {
        count: 1,
        max: 1,
    });
    let result = interp.win32_foreign(&program, "CloseHandle", &[closed]);
    assert_eq!(result.unwrap().unwrap(), vec![1]);
    for symbol in ["WaitForSingleObject", "WaitForSingleObjectEx"] {
        let result = interp.win32_foreign(&program, symbol, &[closed, INFINITE.into(), 0]);
        assert_eq!(
            result
                .expect("a closed scheduler handle is not a native one")
                .unwrap(),
            vec![WAIT_FAILED],
            "{symbol}"
        );
    }
    // Live handles are unaffected.
    let result = interp.win32_foreign(&program, "WaitForSingleObject", &[live, 0]);
    assert_eq!(result.unwrap().unwrap(), vec![WAIT_OBJECT_0]);
}

#[test]
fn closing_a_closed_scheduler_handle_fails_instead_of_reaching_native() {
    let mut interp = Interp::default();
    let program = Program::default();
    let handle = interp.new_object(Object::Semaphore {
        count: 0,
        max: 1,
    });
    let result = interp.win32_foreign(&program, "CloseHandle", &[handle]);
    assert_eq!(result.unwrap().unwrap(), vec![1]);
    let result = interp.win32_foreign(&program, "CloseHandle", &[handle]);
    assert_eq!(
        result
            .expect("a closed scheduler handle is not a native one")
            .unwrap(),
        vec![0]
    );
}

#[cfg(windows)]
#[test]
fn emulated_errors_survive_the_next_native_foreign_call() {
    let mut interp = Interp::new(Box::new(NativeHost));
    let sig = crate::ir::Sig {
        params: vec![],
        returns: vec![Ty::I32],
        conv: crate::ir::Conv::C,
        c_varargs: false,
        c_fixed: 0,
        c_abi: None,
    };
    // Seed the FFI cache with an unrelated error, as an earlier successful call can leave.
    native::set_last_error(1234);
    let result = interp
        .win32_sync_foreign(Win32Fn::CreateSemaphoreW, &[0, 0, 0, 0])
        .unwrap()
        .unwrap();
    assert_eq!(result, vec![0]);
    // Interpreter work can clobber the OS slot before GetLastError is reached.
    // SAFETY: SetLastError only writes this thread's error slot.
    unsafe { SetLastError(5678) };
    let result = native::call(GetLastError as *const () as u64, &[], &sig).unwrap();
    assert_eq!(result.first(), Some(&u64::from(ERROR_INVALID_PARAMETER)));
}

#[test]
fn managed_thread_lookup_accepts_native_handle_range_without_events() {
    let mut interp = Interp::new(Box::new(NativeHost));
    // Registry-only test: no OS call or ownership of this numeric fixture handle.
    interp.new_object_at(4, Object::Thread(0));
    assert!(
        interp.owns(4),
        "real thread handles must not depend on event creation"
    );
    interp.with_sched(|s| {
        s.objects.remove(&4);
    });
    assert!(!interp.owns(4));
}

#[cfg(windows)]
#[test]
fn sdk_thread_entrypoints_resolve_at_runtime() {
    assert!(
        compare_handles_api().is_some(),
        "Windows 10 / Server 2016 API required"
    );
    for symbol in ["SetThreadDescription", "GetThreadDescription"] {
        // Resolve from the documented original DLL as well as the modern SDK facade.
        // No static import of either description API is introduced by this test.
        for library in ["kernelbase", "kernel32"] {
            let module = native::Library::open(library, true, "").expect("system DLL must load");
            assert!(
                native::lookup(Some(&module), symbol).is_some(),
                "{library}!{symbol}"
            );
        }
    }
}

#[cfg(windows)]
#[test]
fn suspend_only_aliases_are_identified_without_query_rights() {
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn DuplicateHandle(
            source_process: usize,
            source: usize,
            target_process: usize,
            target: *mut usize,
            access: u32,
            inherit: i32,
            flags: u32,
        ) -> i32;
    }
    let (done, wait) = std::sync::mpsc::channel::<()>();
    let worker = OwnedHandle::from(std::thread::spawn(move || {
        // Closing the channel stops this real OS worker, including assertion unwind.
        for () in wait {}
    }));
    let raw = worker.as_raw_handle() as usize;
    let mut alias = 0;
    // SAFETY: -1 is our current-process pseudo handle, raw is owned, alias is local.
    let duplicated =
        unsafe { DuplicateHandle(usize::MAX, raw, usize::MAX, &raw mut alias, 2, 0, 0) };
    assert_ne!(duplicated, 0);
    // SAFETY: DuplicateHandle transferred one native thread handle to alias.
    let alias = unsafe { OwnedHandle::from_raw_handle(alias as *mut _) };
    let mut interp = Interp::new(Box::new(NativeHost));
    let stop_worker = done; // Drop before interp on a failed assertion; join cannot hang.
    let shared = interp.shared();
    let mut thread = GThread::new(false, None);
    // SAFETY: the full-rights private handle supports GetThreadId.
    thread.native_id = unsafe { GetThreadId(raw) };
    thread.worker = Some(worker);
    shared.lock().add(thread);
    let alias_raw = alias.as_raw_handle() as u64;
    // SAFETY: SetLastError/GetLastError operate on this OS thread only.
    unsafe { SetLastError(1234) };
    assert_eq!(interp.is_unmanaged_thread_alias(alias_raw), Ok(true));
    // SAFETY: as above, this thread's last-error value.
    let last_error = unsafe { GetLastError() };
    assert_eq!(last_error, 1234);
    drop(alias);
    // SAFETY: as above.
    unsafe { SetLastError(5678) };
    assert_eq!(interp.is_unmanaged_thread_alias(alias_raw), Ok(false));
    // SAFETY: as above.
    let last_error = unsafe { GetLastError() };
    assert_eq!(last_error, 5678);
    drop(stop_worker);
    drop(interp);
}

#[cfg(windows)]
#[test]
fn interpreter_teardown_closes_its_owned_thread_handle() {
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    let mut interp = Interp::new(Box::new(NativeHost));
    let shared = interp.shared();
    let handle = OwnedHandle::from(std::thread::spawn(|| {}));
    let raw = handle.as_raw_handle() as usize;
    shared.lock().threads[0].native_handle = Some(handle);
    // SAFETY: raw borrows the live OwnedHandle stored in shared state.
    assert_eq!(unsafe { WaitForSingleObject(raw, 5000) }, 0);
    // SAFETY: the same live handle supports the standard protection flag.
    assert_ne!(unsafe { SetHandleInformation(raw, 2, 2) }, 0);
    drop(interp);
    // Keep shared alive: cleanup cannot rely on Shared::drop, because a parked
    // scheduler thread or a native callback may still own an Arc to it.
    assert!(shared.lock().threads[0].native_handle.is_none());
    // SAFETY: WaitForSingleObject accepts invalid handles and reports WAIT_FAILED.
    assert_eq!(unsafe { WaitForSingleObject(raw, 0) }, u32::MAX);
}

#[cfg(windows)]
#[test]
fn joining_an_ended_worker_handle_returns() {
    use std::os::windows::io::OwnedHandle;
    let worker = OwnedHandle::from(std::thread::spawn(|| {}));
    join_worker(worker);
}

#[cfg(windows)]
#[test]
fn joining_a_running_worker_handle_waits_for_it() {
    use std::os::windows::io::OwnedHandle;
    use std::sync::atomic::{AtomicBool, Ordering};
    let done = std::sync::Arc::new(AtomicBool::new(false));
    let flag = done.clone();
    let worker = OwnedHandle::from(std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        flag.store(true, Ordering::SeqCst);
    }));
    join_worker(worker);
    assert!(
        done.load(Ordering::SeqCst),
        "join returned before the thread ended"
    );
}

#[cfg(windows)]
#[test]
fn releasing_a_protected_thread_handle_closes_it() {
    use std::os::windows::io::{AsRawHandle, OwnedHandle};
    let handle = OwnedHandle::from(std::thread::spawn(|| {}));
    let raw = handle.as_raw_handle() as usize;
    // SAFETY: raw borrows the live OwnedHandle; waiting and protecting need no other state.
    assert_eq!(unsafe { WaitForSingleObject(raw, 5000) }, 0);
    // SAFETY: as above; protect-from-close would make a plain drop fail.
    assert_ne!(unsafe { SetHandleInformation(raw, 2, 2) }, 0);
    release_thread_handle(handle);
    // SAFETY: WaitForSingleObject accepts invalid handles and reports WAIT_FAILED.
    assert_eq!(unsafe { WaitForSingleObject(raw, 0) }, u32::MAX);
}
