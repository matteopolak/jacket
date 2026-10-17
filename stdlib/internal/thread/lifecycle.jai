#scope_module
prepare_thread :: (thread: *Thread, proc: Thread_Proc, temporary_storage_size: s32, starting_storage: *Temporary_Storage, name: string, handler_address: *Init_Handler) -> bool {
    if temporary_storage_size < 0 || !proc return false;

    state := cast(*Thread_State) context.allocator.proc(.ALLOCATE, size_of(Thread_State), 0, null, context.allocator.data);
    if !state return false;

    state.* = .{};
    state.allocator = context.allocator;
    state.thread = thread;
    state.handler_address = handler_address;
    thread.proc = proc;
    thread.starting_context = context;
    thread.starting_context.context_info = type_info(#Context);
    thread.starting_context.stack_trace = null;
    #if OS != .WINDOWS thread.is_done = false;
    thread.index = next_thread_index();
    thread.starting_context.thread_index = cast(u32) thread.index;

    if starting_storage {
        thread.starting_temporary_storage = starting_storage.*;
        thread.allocator_used_for_temporary_storage = .{};
    } else {
        thread.starting_temporary_storage = .{};
        thread.starting_temporary_storage.size = temporary_storage_size;

        if temporary_storage_size > 0 {
            thread.starting_temporary_storage.data = cast(*u8) state.allocator.proc(.ALLOCATE, temporary_storage_size, 0, null, state.allocator.data);
            if !thread.starting_temporary_storage.data {
                state.allocator.proc(.FREE, 0, size_of(Thread_State), state, state.allocator.data);
                thread.index = 0;
                return false;
            }
        }

        thread.allocator_used_for_temporary_storage = state.allocator;
    }

    thread.starting_context.temporary_storage = *thread.starting_temporary_storage;
    push_context thread.starting_context state.initial_mark = get_temporary_storage_mark();

    #if SET_THREAD_NAMES && OS != .WINDOWS {
        if name.count > 0 {
            state.name = cast(*u8) state.allocator.proc(.ALLOCATE, name.count + 1, 0, null, state.allocator.data);
            if !state.name {
                if thread.allocator_used_for_temporary_storage.proc thread.allocator_used_for_temporary_storage.proc(.FREE, 0, thread.starting_temporary_storage.size, thread.starting_temporary_storage.data, thread.allocator_used_for_temporary_storage.data);
                state.allocator.proc(.FREE, 0, size_of(Thread_State), state, state.allocator.data);
                thread.index = 0;
                return false;
            }

            memcpy(state.name, name.data, name.count);
            state.name[name.count] = 0;
            state.name_size = name.count + 1;
        }

        #if SET_NAME_INSIDE_THREAD thread.c_name = state.name;
    }

    #if OS == .WINDOWS {
        thread.windows_thread = create_windows_thread(state);
        if !thread.windows_thread {
            release_thread_state(state);
            return false;
        }
    } else {
        state.alive_initialized = initialize_semaphore(*thread.is_alive_semaphore, 0);
        if state.alive_initialized state.suspended_initialized = initialize_semaphore(*thread.suspended_semaphore, 0);

        if !state.suspended_initialized {
            if cleanup_thread_semaphores(state) release_thread_state(state);
            else retain_thread_state(state);
            return false;
        }

        result := P.pthread_create(*thread.thread_handle, null, thread_entry, state);
        if result != 0 {
            context.logger("pthread creation failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            if cleanup_thread_semaphores(state) release_thread_state(state);
            else retain_thread_state(state);
            return false;
        }
    }

    state.native_created = true;
    retain_thread_state(state);

    #if OS == .WINDOWS && SET_THREAD_NAMES {
        if name.count > 0 set_windows_thread_name(thread, name);
    }

    #if OS != .WINDOWS && SET_THREAD_NAMES && !SET_NAME_INSIDE_THREAD {
        if state.name && P.pthread_setname_np(thread.thread_handle, state.name) != 0
            context.logger("thread name could not be set\n", context.logger_data, Log_Info.{ common_flags = .WARNING });
    }

    return true;
}

retain_thread_state :: (state: *Thread_State) {
    lock_registry();
    state.next = active_threads;
    active_threads = state;
    unlock_registry();
}

#scope_export
thread_start :: (thread: *Thread) {
    state := lookup_thread(thread);

    #if OS == .WINDOWS {
        if !windows_thread_check(state != null && !state.started, "thread has not been initialized or was already started") return;
        resumed := W.ResumeThread(thread.windows_thread);
        if !windows_thread_check(resumed != -1, "thread resume failed") return;
        state.started = true;
    } else {
        if !state || !state.native_created || state.started {
            sdk_ok(-1, "thread has not been initialized or was already started");
            return;
        }

        if signal_semaphore(*thread.suspended_semaphore) state.started = true;
    }
}

thread_is_done :: (thread: *Thread, milliseconds: s32 = 0) -> bool {
    if milliseconds < -1 {
        context.logger("invalid thread timeout\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return false;
    }

    #if OS == .WINDOWS {
        status := W.WaitForSingleObject(thread.windows_thread, cast,no_check(u32) milliseconds);
        if status == 0 return true;
        if status == 258 return false;

        context.logger("thread completion wait failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });

        return false;
    } else {
        if read_done(thread) return true;
        status := wait_for(*thread.is_alive_semaphore, milliseconds);
        if status == .ERROR context.logger("thread completion wait failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        return status == .SUCCESS && read_done(thread);
    }
}

thread_deinit :: (thread: *Thread) {
    state := lookup_thread(thread);

    #if OS == .WINDOWS {
        if !windows_thread_check(state != null && read_done(thread), "thread_deinit requires a completed thread") return;

        closed := W.CloseHandle(thread.windows_thread);
        if !windows_thread_check(closed != 0, "thread handle cleanup failed") return;

        thread.windows_thread = null;
        thread.windows_thread_id = 0;
    } else {
        if !state || (state.native_created && !read_done(thread)) {
            sdk_ok(-1, "thread_deinit requires a completed thread");
            return;
        }

        result: *void;

        if state.native_created && !state.joined {
            joined := P.pthread_join(thread.thread_handle, *result);
            if !sdk_ok(joined, "pthread join failed") return;
            state.joined = true;
        }

        if !cleanup_thread_semaphores(state) return;
    }

    lock_registry();
    previous: *Thread_State;
    cursor := active_threads;

    while cursor && cursor != state {
        previous = cursor;
        cursor = cursor.next;
    }

    assert(cursor != null);
    if previous previous.next = state.next;
    else active_threads = state.next;

    unlock_registry();
    release_thread_state(state);
    thread.index = 0;
    #if SET_THREAD_NAMES && SET_NAME_INSIDE_THREAD thread.c_name = null;
} @Releases

#scope_module
Init_Handler :: #type (thread: *Thread);

read_done :: (thread: *Thread) -> bool {
    #if OS == .WINDOWS return W.WaitForSingleObject(thread.windows_thread, 0) == 0;
    else {
        _, value := compare_and_swap(*thread.is_done, false, false);
        return value;
    }
}

#scope_module
Thread_State :: struct {
    next: *Thread_State;
    thread: *Thread;
    allocator: Allocator;
    initial_mark: Temporary_Storage_State;
    handler_address: *Init_Handler;
    started: bool;
    native_created: bool;
    joined: bool;
    alive_initialized: bool;
    suspended_initialized: bool;
    name: *u8;
    name_size: s64;
}

active_threads: *Thread_State;
registry_lock: s32;
thread_counter: s64 = 1;

lock_registry :: () {
    acquired, old := compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
    while !acquired acquired, old = compare_and_swap(*registry_lock, cast(s32) 0, cast(s32) 1);
}

unlock_registry :: () {
    changed := compare_and_swap(*registry_lock, cast(s32) 1, cast(s32) 0);
    assert(changed);
}

lookup_thread :: (thread: *Thread) -> *Thread_State {
    lock_registry();
    defer unlock_registry();
    cursor := active_threads;
    while cursor {
        if cursor.thread == thread return cursor;
        cursor = cursor.next;
    }

    return null;
}

next_thread_index :: () -> s64 {
    _, value := compare_and_swap(*thread_counter, cast(s64) 0, cast(s64) 0);

    while true {
        assert(value < 0xffffffff, "thread index exhausted");
        changed, old := compare_and_swap(*thread_counter, value, value + 1);
        if changed return value;
        value = old;
    }

    return 0;
}

release_thread_state :: (state: *Thread_State) {
    thread := state.thread;

    push_context thread.starting_context set_temporary_storage_mark(state.initial_mark);

    if thread.allocator_used_for_temporary_storage.proc {
        allocator := thread.allocator_used_for_temporary_storage;
        allocator.proc(.FREE, 0, thread.starting_temporary_storage.size, thread.starting_temporary_storage.data, allocator.data);
        thread.allocator_used_for_temporary_storage = .{};
        thread.starting_temporary_storage = .{};
    }

    allocator := state.allocator;
    if state.name allocator.proc(.FREE, 0, state.name_size, state.name, allocator.data);

    allocator.proc(.FREE, 0, size_of(Thread_State), state, allocator.data);
    thread.index = 0;
} @Releases

#if OS != .WINDOWS cleanup_thread_semaphores :: (state: *Thread_State) -> bool {
    if state.alive_initialized {
        if !destroy_semaphore(*state.thread.is_alive_semaphore) return false;
        state.alive_initialized = false;
    }

    if state.suspended_initialized {
        if !destroy_semaphore(*state.thread.suspended_semaphore) return false;
        state.suspended_initialized = false;
    }

    return true;
}

#if OS != .WINDOWS thread_entry :: (data: *void) -> *void #c_call {
    state := cast(*Thread_State) data;
    thread := state.thread;

    push_context thread.starting_context {
        status := wait_for(*thread.suspended_semaphore, -1);
        if status == .SUCCESS {
            #if SET_THREAD_NAMES && SET_NAME_INSIDE_THREAD {
                if state.name && P.pthread_setname_np(state.name) != 0
                    context.logger("thread name could not be set\n", context.logger_data, Log_Info.{ common_flags = .WARNING });
            }

            if state.handler_address && state.handler_address.* state.handler_address.*(thread);

            result := thread.proc(thread);
            completed := compare_and_swap(*thread.is_done, false, true);

            assert(completed, "thread completion was already recorded");

            signal(*thread.is_alive_semaphore);

            return cast(*void) result;
        }

        context.logger("thread start wait failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        completed := compare_and_swap(*thread.is_done, false, true);

        assert(completed, "thread completion was already recorded");

        signal(*thread.is_alive_semaphore);
    }

    return null;
}
