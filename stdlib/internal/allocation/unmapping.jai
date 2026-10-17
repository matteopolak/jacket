Unmapping_Allocator :: struct {
    table: H.Table(*void, u64);
    mutex: T.Mutex;
}

get_unmapping_allocator :: () -> Allocator {
    owner := context.allocator;
    allocation := cast(*Wrapper_Allocation) owner.proc(.ALLOCATE, size_of(Wrapper_Allocation), 0, null, owner.data);

    assert(allocation != null, "unmapping allocator metadata allocation failed");
    allocation.* = .{};
    allocation.owner = owner;
    wrapper := *allocation.wrapper;
    wrapper.table.allocator = owner;
    T.init(*wrapper.mutex);

    return .{ unmapping_allocator_proc, wrapper };
}

deinit_unmapping_allocator :: (a: Allocator) {
    assert(a.proc == unmapping_allocator_proc && a.data != null);
    wrapper := cast(*Unmapping_Allocator) a.data;
    offset := type_info(Wrapper_Allocation).members[1].offset_in_bytes;
    allocation := cast(*Wrapper_Allocation) (cast(*u8) wrapper - offset);
    owner := allocation.owner;

    deinit(wrapper);
    owner.proc(.FREE, 0, size_of(Wrapper_Allocation), allocation, owner.data);
} @Releases

deinit :: (ua: *Unmapping_Allocator) {
    T.lock(*ua.mutex);
    H.deinit(*ua.table);
    T.unlock(*ua.mutex);
    T.destroy(*ua.mutex);
} @Releases

unmapping_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Unmapping_Allocator";
        }

        return cast(*void) cast(u64) (Allocator_Caps.MULTIPLE_THREADS | .FREE | .HINT_I_AM_A_DEBUG_ALLOCATOR);
    }

    wrapper := cast(*Unmapping_Allocator) allocator_data;

    assert(wrapper != null);
    T.lock(*wrapper.mutex);
    defer T.unlock(*wrapper.mutex);

    if mode == .ALLOCATE {
        assert(requested_size >= 0);
        return allocate(wrapper, requested_size);
    }

    if mode == .RESIZE {
        assert(requested_size >= 0 && old_size >= 0);
        if !old_memory return allocate(wrapper, requested_size);

        recorded := require_size(wrapper, old_memory);

        assert(cast(u64) old_size <= recorded, "resize source extent exceeds its recorded allocation");
        fresh := allocate(wrapper, requested_size);
        if !fresh return null;

        memcpy(fresh, old_memory, min(old_size, requested_size));
        protect_old(wrapper, old_memory, recorded);

        return fresh;
    }

    if mode == .FREE {
        if old_memory protect_old(wrapper, old_memory, require_size(wrapper, old_memory));
        return null;
    }

    if mode == .STARTUP || mode == .SHUTDOWN || mode == .THREAD_START || mode == .THREAD_STOP return null;

    assert(false, "unsupported unmapping allocator mode");

    return null;
}

#scope_file
#import "Basic";
H :: #import "Hash_Table";
T :: #import "Thread";
VM :: #import "internal/allocation/virtual-memory";

Wrapper_Allocation :: struct {
    owner: Allocator;
    wrapper: Unmapping_Allocator;
}

#if OS == .WINDOWS {
    W :: #import "Windows";
    S :: #import "System";

    OS_Error_Code :: W.OS_Error_Code;

    last_error :: () -> OS_Error_Code {
        return W.GetLastError();
    }

    allocation_overflow :: () -> OS_Error_Code {
        return cast(OS_Error_Code) 8;
    } // ERROR_NOT_ENOUGH_MEMORY.

    error_description :: (code: OS_Error_Code) -> string {
        return S.get_error_string(code);
    }
} else {
    P :: #import "POSIX";

    OS_Error_Code :: P.OS_Error_Code;
    libc :: #system_library "libc";
    c_strerror :: (error: s32) -> *u8 #foreign libc "strerror";

    last_error :: () -> OS_Error_Code {
        return P.errno();
    }

    allocation_overflow :: () -> OS_Error_Code {
        P.set_errno(P.ENOMEM);
        return P.ENOMEM;
    }

    error_description :: (code: OS_Error_Code) -> string {
        description := c_strerror(cast(s32) code);
        if description return to_string(description);
        return "";
    }
}

report_failure :: (code: OS_Error_Code) {
    context.logger("unmapping allocator OS failure: ", context.logger_data, Log_Info.{ common_flags = .ERROR });
    description := error_description(code);
    if description.count > 0 context.logger(description, context.logger_data, Log_Info.{ common_flags = .ERROR });
}

page_size :: () -> s64 {
    size := VM.page_size();
    assert(size > 0, "OS page-size query failed");
    return size;
}

effective_size :: (requested: s64) -> s64 {
    #if OS == .WINDOWS return max(requested, 8);
    else return requested;
}

writable_extent :: (requested: s64, page: s64) -> s64, bool {
    if requested > 0x7fffffffffffffff - 2 * page return 0, false;
    return ((requested + page - 1) / page) * page, true;
}

allocate :: (wrapper: *Unmapping_Allocator, bytes: s64) -> *void {
    page := page_size();
    if page <= 0 return null;

    effective := effective_size(bytes);
    writable, valid := writable_extent(effective, page);

    if !valid {
        report_failure(allocation_overflow());
        return null;
    }

    total := writable + page;
    base := VM.reserve(cast(u64) total);

    if !base {
        report_failure(last_error());
        return null;
    }

    if writable > 0 && !VM.commit(base, cast(u64) writable) {
        error := last_error();

        if !VM.release(base, cast(u64) total) report_failure(last_error());

        report_failure(error);
        #if OS != .WINDOWS P.set_errno(error);

        return null;
    }

    memory := base + writable - effective;

    H.table_add(*wrapper.table, memory, cast(u64) bytes);

    return memory;
}

require_size :: (wrapper: *Unmapping_Allocator, memory: *void) -> u64 {
    found, size := H.table_find(*wrapper.table, memory);
    if !found {
        context.logger("invalid or previously freed unmapping-allocator pointer\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "invalid unmapping-allocator pointer");
    }

    return size;
}

protect_old :: (wrapper: *Unmapping_Allocator, memory: *void, bytes: u64) {
    page := page_size();
    if page <= 0 return;

    effective := effective_size(cast(s64) bytes);
    writable, valid := writable_extent(effective, page);

    assert(valid);
    if !valid return;

    base := cast(*u8) memory - writable + effective;
    if writable > 0 && !VM.read_only(base, cast(u64) writable) {
        report_failure(last_error());
        assert(false, "freed mapping could not be protected");
        return;
    }

    removed := H.table_remove(*wrapper.table, memory);

    assert(removed);
}
