init :: (using m: *Mutex, desired_name := "", desired_order: Mutex_Order = -1) {
    initialized_ok := InitializeCriticalSectionEx(*csection, 0, 0);
    if !windows_thread_check(initialized_ok != 0, "critical section initialization failed") return;

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
        order = desired_order;
    }
}

destroy :: (using m: *Mutex) {
    W.DeleteCriticalSection(*csection);
}

lock :: (using m: *Mutex) {
    W.EnterCriticalSection(*csection);
}

unlock :: (using m: *Mutex) {
    W.LeaveCriticalSection(*csection);
}

csection_lock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    W.EnterCriticalSection(csection);
}

csection_unlock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    W.LeaveCriticalSection(csection);
}

init :: (using cv: *Condition_Variable, desired_name := "") {
    W.InitializeConditionVariable(*condition_variable);

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
    }
}

destroy :: (using cv: *Condition_Variable) {
    // Win32 condition variables have no destruction API or allocated resource.
    condition_variable = .{};
}

cvariable_wait :: (condition_variable: *Per_OS_Condition_Variable_Type, csection: *Per_OS_Critical_Section_Type, wait_time_ms: s32 = -1) {
    timeout := cast,no_check(u32) wait_time_ms;
    if wait_time_ms < 0 timeout = 0xffffffff;
    ok := W.SleepConditionVariableCS(condition_variable, csection, timeout);
    windows_thread_check(ok != 0 || W.GetLastError() == 1460, "condition variable wait failed");
}

cvariable_wake :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    W.WakeConditionVariable(condition_variable);
}

cvariable_wake_all :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    W.WakeAllConditionVariable(condition_variable);
}

wait :: (c: *Condition_Variable, m: *Mutex) {
    cvariable_wait(*c.condition_variable, *m.csection);
}

wake :: (c: *Condition_Variable) {
    cvariable_wake(*c.condition_variable);
}

wake_all :: (c: *Condition_Variable) {
    cvariable_wake_all(*c.condition_variable);
}

init :: (s: *Semaphore, initial_value: s32 = 0) {
    if !windows_thread_check(initial_value >= 0, "semaphore count must be nonnegative") return;
    s.event = W.CreateSemaphoreW(null, initial_value, 0x7fffffff, null);
    windows_thread_check(s.event != null, "semaphore initialization failed");
}

destroy :: (s: *Semaphore) {
    closed := W.CloseHandle(s.event);
    if !windows_thread_check(closed != 0, "semaphore destruction failed") return;
    s.event = null;
}

signal :: (s: *Semaphore) {
    released := W.ReleaseSemaphore(s.event, 1, null);
    windows_thread_check(released != 0, "semaphore signal failed");
}

wait_for :: (s: *Semaphore, milliseconds: s32 = -1) -> Wait_For_Result {
    timeout := cast,no_check(u32) milliseconds;
    if milliseconds < 0 timeout = 0xffffffff;

    status := W.WaitForSingleObject(s.event, timeout);
    if status == 0 return .SUCCESS;
    if status == 258 return .TIMEOUT;

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

    push_context thread.starting_context {
        if state.handler_address && state.handler_address.* state.handler_address.*(thread);
        result := thread.proc(thread);
        return cast,no_check(u32) result;
    }
}

set_windows_thread_name :: (thread: *Thread, name: string) {
    wide := Text.utf16_path(name);
    defer free(wide);
    if !wide || W.SetThreadDescription(thread.windows_thread, wide) < 0
        context.logger("thread name could not be set\n", context.logger_data, Log_Info.{ common_flags = .WARNING });
}

#scope_file
Text :: #import "internal/files/windows-text";

kernel32 :: #system_library "kernel32";
libc :: #system_library "libc";
begin_thread :: (security: *void, stack_size: u32, entry: *void, argument: *void, flags: u32, id: *s32) -> u64 #foreign libc "_beginthreadex";
GetTickCount64 :: () -> u64 #foreign kernel32;
InitializeCriticalSectionEx :: (section: *W.CRITICAL_SECTION, spin_count: u32, flags: u32) -> s32 #foreign kernel32;
