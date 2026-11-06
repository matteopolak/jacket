// Sets up a mutex. It is recursive: the thread that holds it can lock it again, and has to unlock it as many times.
//
// - desired_name: a label for the mutex. Only kept in builds with `DEBUG` set.
// - desired_order: a rank for the mutex. Only kept in builds with `DEBUG` set; nothing acts on it yet. -1 for none.
init :: (using m: *Mutex, desired_name := "", desired_order: Mutex_Order = -1) {
    initialized_ok := InitializeCriticalSectionEx(*csection, 0, 0);
    if !windows_thread_check(initialized_ok != 0, "critical section initialization failed") return;

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
        order = desired_order;
    }
}

// Releases the resources of a mutex. It must be unlocked.
destroy :: (using m: *Mutex) {
    W.DeleteCriticalSection(*csection);
}

// Takes the mutex, waiting until no other thread holds it. Pair every `lock` with an `unlock`.
lock :: (using m: *Mutex) {
    W.EnterCriticalSection(*csection);
} @Locks

// Gives the mutex back. Call it from the thread that locked it.
unlock :: (using m: *Mutex) {
    W.LeaveCriticalSection(*csection);
}

// Locks the raw operating-system critical section behind a `Mutex`. Prefer `lock`.
csection_lock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    W.EnterCriticalSection(csection);
}

// Unlocks a critical section locked with `csection_lock`.
csection_unlock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    W.LeaveCriticalSection(csection);
}

// Sets up a condition variable.
//
// - desired_name: a label for the condition variable. Only kept in builds with `DEBUG` set.
init :: (using cv: *Condition_Variable, desired_name := "") {
    W.InitializeConditionVariable(*condition_variable);

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
    }
}

// Releases the resources of a condition variable. No thread may be waiting on it.
destroy :: (using cv: *Condition_Variable) {
    // Win32 condition variables have no destruction API or allocated resource.
    condition_variable = .{};
}

// Waits on a raw operating-system condition variable, giving up after `wait_time_ms` (never, when negative). Prefer
// `wait`.
cvariable_wait :: (condition_variable: *Per_OS_Condition_Variable_Type, csection: *Per_OS_Critical_Section_Type, wait_time_ms: s32 = -1) {
    timeout := cast,no_check(u32) wait_time_ms;
    if wait_time_ms < 0 timeout = 0xffffffff;
    ok := W.SleepConditionVariableCS(condition_variable, csection, timeout);
    windows_thread_check(ok != 0 || W.GetLastError() == WIN32_WAIT_TIMED_OUT, "condition variable wait failed");
}

// Wakes one waiter on a raw operating-system condition variable. Prefer `wake`.
cvariable_wake :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    W.WakeConditionVariable(condition_variable);
}

// Wakes every waiter on a raw operating-system condition variable. Prefer `wake_all`.
cvariable_wake_all :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    W.WakeAllConditionVariable(condition_variable);
}

// Waits on `c` until another thread calls `wake` or `wake_all`. It can return without either, so check the condition again afterwards.
//
// - m: the mutex the caller holds. It is released while waiting and held again when `wait` returns.
wait :: (c: *Condition_Variable, m: *Mutex) {
    cvariable_wait(*c.condition_variable, *m.csection);
}

// Wakes one thread waiting on `c`, if any. Change the shared state under the mutex first; see `wait` for an example.
wake :: (c: *Condition_Variable) {
    cvariable_wake(*c.condition_variable);
}

// Wakes every thread waiting on `c`.
wake_all :: (c: *Condition_Variable) {
    cvariable_wake_all(*c.condition_variable);
}

// Sets up a semaphore.
//
// - initial_value: how many `wait_for` calls succeed before anyone calls `signal`.
init :: (s: *Semaphore, initial_value: s32 = 0) {
    if !windows_thread_check(initial_value >= 0, "semaphore count must be nonnegative") return;
    s.event = W.CreateSemaphoreW(null, initial_value, 0x7fffffff, null);
    windows_thread_check(s.event != null, "semaphore initialization failed");
}

// Releases the resources of a semaphore.
destroy :: (s: *Semaphore) {
    closed := W.CloseHandle(s.event);
    if !windows_thread_check(closed != 0, "semaphore destruction failed") return;
    s.event = null;
}

// Adds one count to the semaphore, waking a thread blocked in `wait_for`.
signal :: (s: *Semaphore) {
    released := W.ReleaseSemaphore(s.event, 1, null);
    windows_thread_check(released != 0, "semaphore signal failed");
}

// Takes one count from the semaphore, waiting for `signal` if there is none. The result says whether it got one: `.SUCCESS`,
// `.TIMEOUT` or `.ERROR`.
//
// - milliseconds: how long to wait at most. 0 only tries; -1 waits as long as it takes.
wait_for :: (s: *Semaphore, milliseconds: s32 = -1) -> Wait_For_Result {
    timeout := cast,no_check(u32) milliseconds;
    if milliseconds < 0 timeout = 0xffffffff;

    status := W.WaitForSingleObject(s.event, timeout);
    if status == 0 return .SUCCESS;
    if status == WAIT_RESULT_TIMEOUT return .TIMEOUT;

    return .ERROR;
}

#scope_module
// Native operation failures must be diagnosed even when Basic.assert is off.
// A custom assertion handler may return; callers retain ownership on failure.
windows_thread_check :: (ok: bool, message: string, loc := #caller_location) -> bool {
    if !ok assert_helper(message, loc = loc);
    return ok;
}

#assert size_of(W.CRITICAL_SECTION) == 40;
#assert size_of(W.CONDITION_VARIABLE) == 8;

create_windows_thread :: (state: *Thread_State) -> W.HANDLE {
    return cast(W.HANDLE) begin_thread(null, 0, cast(*void) windows_thread_entry, state, 4, *state.thread.windows_thread_id);
}

monotonic_milliseconds :: () -> s64, bool #no_context {
    count := GetTickCount64();
    if count > 0x7fffffffffffffff return 0, false;
    return cast(s64) count, true;
}

windows_thread_entry :: (data: *void) -> u32 #c_call {
    state := cast(*Thread_State) data;
    thread := state.thread;

    Windows_Stack.reserve();
    Crash.attach_thread();

    push_context thread.starting_context {
        if state.handler_address && state.handler_address.* state.handler_address.*(thread);
        result := thread.proc(thread);
        return cast,no_check(u32) result;
    }
}

set_windows_thread_name :: (thread: *Thread, name: string) {
    wide := Text.utf16_path(name);
    defer free(wide);
    if !wide || SetThreadDescription(thread.windows_thread, wide) < 0
        context.logger("thread name could not be set\n", context.logger_data, Log_Info.{ common_flags = .WARNING });
}

#scope_file
WIN32_WAIT_TIMED_OUT :: 1460;
WAIT_RESULT_TIMEOUT :: 258;

Text :: #import "internal/files/windows-text";

kernel32 :: #system_library "kernel32"; @native(external)
libc :: #system_library "libc"; @native(external)
begin_thread :: (security: *void, stack_size: u32, entry: *void, argument: *void, flags: u32, id: *s32) -> u64 #foreign libc "_beginthreadex";
GetTickCount64 :: () -> u64 #foreign kernel32;
InitializeCriticalSectionEx :: (section: *W.CRITICAL_SECTION, spin_count: u32, flags: u32) -> s32 #foreign kernel32;
SetThreadDescription :: (thread: W.HANDLE, description: *u16) -> s32 #foreign kernel32;
