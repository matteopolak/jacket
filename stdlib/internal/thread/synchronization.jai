init :: (using m: *Mutex, desired_name := "", desired_order: Mutex_Order = -1) {
    attr: P.pthread_mutexattr_t;

    result := P.pthread_mutexattr_init(*attr);

    if !sdk_ok(result, "pthread mutex attribute initialization failed") return;

    #if OS == .MACOS {
        recursive :: P.PTHREAD_MUTEX_RECURSIVE;
    } else {
        recursive :: cast(s32) P.PTHREAD_MUTEX.RECURSIVE;
    }

    result = P.pthread_mutexattr_settype(*attr, recursive);
    if result != 0 {
        destroyed := P.pthread_mutexattr_destroy(*attr);
        sdk_ok(destroyed, "pthread mutex attribute destruction failed");
        sdk_ok(result, "pthread recursive mutex configuration failed");
        return;
    }

    result = P.pthread_mutex_init(*csection, *attr);
    destroyed := P.pthread_mutexattr_destroy(*attr);

    sdk_ok(destroyed, "pthread mutex attribute destruction failed");
    if !sdk_ok(result, "pthread mutex initialization failed") return;

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
        order = desired_order;
    }
}

destroy :: (using m: *Mutex) {
    result := P.pthread_mutex_destroy(*csection);
    if !sdk_ok(result, "pthread mutex destruction failed") return;
    #if THREAD_DEBUG initialized = false;
} @Releases

lock :: (using m: *Mutex) {
    result := P.pthread_mutex_lock(*csection);
    sdk_ok(result, "pthread mutex lock failed");
}

unlock :: (using m: *Mutex) {
    result := P.pthread_mutex_unlock(*csection);
    sdk_ok(result, "pthread mutex unlock failed");
}

init :: (using cv: *Condition_Variable, desired_name := "") {
    result := P.pthread_cond_init(*condition_variable, null);
    if !sdk_ok(result, "pthread condition initialization failed") return;

    #if THREAD_DEBUG {
        initialized = true;
        name = desired_name;
    }
}

destroy :: (using cv: *Condition_Variable) {
    result := P.pthread_cond_destroy(*condition_variable);
    if !sdk_ok(result, "pthread condition destruction failed") return;
    #if THREAD_DEBUG initialized = false;
} @Releases

csection_lock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    result := P.pthread_mutex_lock(csection);
    sdk_ok(result, "pthread critical section lock failed");
}

csection_unlock :: (csection: *Per_OS_Critical_Section_Type) #expand {
    result := P.pthread_mutex_unlock(csection);
    sdk_ok(result, "pthread critical section unlock failed");
}

cvariable_wait :: (condition_variable: *Per_OS_Condition_Variable_Type, csection: *Per_OS_Critical_Section_Type, wait_time_ms: s32 = -1) {
    result: s32;
    if wait_time_ms < 0 result = P.pthread_cond_wait(condition_variable, csection);
    else {
        deadline, ok := absolute_timeout(wait_time_ms);
        if !sdk_ok(ifx ok then 0 else -1, "clock query failed") return;
        result = P.pthread_cond_timedwait(condition_variable, csection, *deadline);
    }

    if result != P.ETIMEDOUT sdk_ok(result, "pthread condition wait failed");
}

cvariable_wake :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    result := P.pthread_cond_signal(condition_variable);
    sdk_ok(result, "pthread condition signal failed");
}

cvariable_wake_all :: (condition_variable: *Per_OS_Condition_Variable_Type) {
    result := P.pthread_cond_broadcast(condition_variable);
    sdk_ok(result, "pthread condition broadcast failed");
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
    initialize_semaphore(s, initial_value);
}

destroy :: (s: *Semaphore) {
    destroy_semaphore(s);
} @Releases

signal :: (s: *Semaphore) {
    signal_semaphore(s);
}

#scope_module
signal_semaphore :: (s: *Semaphore) -> bool {
    #if OS == .MACOS result := M.semaphore_signal(s.event);
    else result := P.sem_post(*s.semaphore);
    return sdk_ok(result, "semaphore signal failed");
}

#scope_export
wait_for :: (s: *Semaphore, milliseconds: s32 = -1) -> Wait_For_Result #no_context {
    #if OS == .MACOS {
        deadline: s64;

        if milliseconds >= 0 {
            now, ok := monotonic_milliseconds();
            if !ok return .ERROR;
            deadline = now + milliseconds;
        }

        while true {
            result: M.kern_return_t;
            if milliseconds < 0 result = M.semaphore_wait(s.event);
            else {
                now, ok := monotonic_milliseconds();
                if !ok return .ERROR;

                remaining := deadline - now;
                if remaining < 0 remaining = 0;

                interval: M.mach_timespec_t;
                interval.tv_sec = cast(u32) (remaining / 1000);
                interval.tv_nsec = cast(s32) ((remaining % 1000) * 1000000);
                result = M.semaphore_timedwait(s.event, interval);
            }

            if result == 0 return .SUCCESS;
            if result == M.KERN_OPERATION_TIMED_OUT return .TIMEOUT;
            if result != M.KERN_ABORTED return .ERROR;
        }
    } else {
        deadline: P.timespec;

        if milliseconds > 0 {
            value, ok := absolute_timeout(milliseconds);
            if !ok return .ERROR;
            deadline = value;
        }

        while true {
            result: s32;
            if milliseconds < 0 result = P.sem_wait(*s.semaphore);
            else if milliseconds == 0 result = P.sem_trywait(*s.semaphore);
            else result = P.sem_timedwait(*s.semaphore, *deadline);
            if result == 0 return .SUCCESS;

            code := P.errno();
            if code == P.EAGAIN || code == P.ETIMEDOUT return .TIMEOUT;
            if code != P.EINTR return .ERROR;
        }
    }

    return .ERROR;
}

#scope_module
// The result is evaluated by the caller before optional assertion expansion.
// Void public APIs log failures even when assertions are disabled.
sdk_ok :: (result: s32, message: string) -> bool {
    if result == 0 return true;
    context.logger(message, context.logger_data, Log_Info.{ common_flags = .ERROR });
    assert(false, message);
    return false;
}

initialize_semaphore :: (s: *Semaphore, initial_value: s32) -> bool {
    if initial_value < 0 return sdk_ok(-1, "semaphore initial count is negative");

    #if OS == .MACOS {
        owner := mach_task_self_;
        event: M.semaphore_t;
        result := M.semaphore_create(owner, *event, M.SYNC_POLICY_FIFO, initial_value);

        if !sdk_ok(result, "Mach semaphore initialization failed") return false;

        s.owner = owner;
        s.event = event;
    } else {
        result := P.sem_init(*s.semaphore, 0, cast(u32) initial_value);
        if !sdk_ok(result, "semaphore initialization failed") return false;
    }

    return true;
}

destroy_semaphore :: (s: *Semaphore) -> bool {
    #if OS == .MACOS {
        result := M.semaphore_destroy(s.owner, s.event);
        if !sdk_ok(result, "Mach semaphore destruction failed") return false;
        s.event = 0;
    } else {
        result := P.sem_destroy(*s.semaphore);
        if !sdk_ok(result, "semaphore destruction failed") return false;
    }

    return true;
} @Releases

absolute_timeout :: (milliseconds: s32) -> P.timespec, bool #no_context {
    value: P.timespec;
    if P.clock_gettime(P.clockid_t.REALTIME, *value) != 0 return value, false;

    value.tv_sec += milliseconds / 1000;
    value.tv_nsec += (milliseconds % 1000) * 1000000;
    if value.tv_nsec >= 1000000000 {
        value.tv_nsec -= 1000000000;
        value.tv_sec += 1;
    }

    return value, true;
}

monotonic_milliseconds :: () -> s64, bool #no_context {
    value: P.timespec;
    if P.clock_gettime(P.clockid_t.MONOTONIC, *value) != 0 return 0, false;
    return value.tv_sec * 1000 + value.tv_nsec / 1000000, true;
}

#scope_file
#if OS == .MACOS {
    libc :: #system_library "libc";
    mach_task_self_: M.task_t #elsewhere libc;
}
