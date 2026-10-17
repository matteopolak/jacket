Overwriting_Allocator :: struct {
    base_allocator: Allocator;
    table: H.Table(*void, u64);
    overwrite_byte: u8;
    mutex: T.Mutex;
}

get_overwriting_allocator :: (base_allocator: Allocator = .{}, overwrite_byte: u8 = 0xcd) -> Allocator {
    base := base_allocator;
    if !base.proc base = context.allocator;

    assert(base.proc != null && base.proc != overwriting_allocator_proc, "recursive overwriting allocator base");
    wrapper := cast(*Overwriting_Allocator) base.proc(.ALLOCATE, size_of(Overwriting_Allocator), 0, null, base.data);

    assert(wrapper != null, "overwriting allocator metadata allocation failed");
    wrapper.* = .{};
    wrapper.base_allocator = base;
    wrapper.table.allocator = base;
    wrapper.overwrite_byte = overwrite_byte;
    T.init(*wrapper.mutex);

    return .{ overwriting_allocator_proc, wrapper };
}

deinit_overwriting_allocator :: (a: Allocator) {
    assert(a.proc == overwriting_allocator_proc && a.data != null);
    wrapper := cast(*Overwriting_Allocator) a.data;
    base := wrapper.base_allocator;

    deinit(wrapper);
    base.proc(.FREE, 0, size_of(Overwriting_Allocator), wrapper, base.data);
} @Releases

deinit :: (ua: *Overwriting_Allocator) {
    T.lock(*ua.mutex);
    H.deinit(*ua.table);
    T.unlock(*ua.mutex);
    T.destroy(*ua.mutex);
} @Releases

overwriting_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Overwriting_Allocator";
        }

        return cast(*void) cast(u64) (Allocator_Caps.MULTIPLE_THREADS | .FREE | .HINT_I_AM_A_DEBUG_ALLOCATOR);
    }

    wrapper := cast(*Overwriting_Allocator) allocator_data;

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
        release(wrapper, old_memory, recorded);

        return fresh;
    }

    if mode == .FREE {
        if old_memory release(wrapper, old_memory, require_size(wrapper, old_memory));
        return null;
    }

    if mode == .STARTUP || mode == .SHUTDOWN || mode == .THREAD_START || mode == .THREAD_STOP {
        base := wrapper.base_allocator;
        return base.proc(mode, requested_size, old_size, old_memory, base.data);
    }

    assert(false, "unsupported overwriting allocator mode");

    return null;
}

#scope_file
#import "Basic";
H :: #import "Hash_Table";
T :: #import "Thread";

allocate :: (wrapper: *Overwriting_Allocator, bytes: s64) -> *void {
    base := wrapper.base_allocator;
    memory := base.proc(.ALLOCATE, bytes, 0, null, base.data);

    if !memory return null;

    memset(memory, wrapper.overwrite_byte, bytes);
    H.table_add(*wrapper.table, memory, cast(u64) bytes);

    return memory;
}

require_size :: (wrapper: *Overwriting_Allocator, memory: *void) -> u64 {
    found, size := H.table_find(*wrapper.table, memory);
    if !found {
        context.logger("invalid or previously freed overwriting-allocator pointer\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "invalid overwriting-allocator pointer");
    }

    return size;
}

release :: (wrapper: *Overwriting_Allocator, memory: *void, bytes: u64) {
    memset(memory, wrapper.overwrite_byte, cast(s64) bytes);
    removed := H.table_remove(*wrapper.table, memory);

    assert(removed);
    base := wrapper.base_allocator;

    base.proc(.FREE, 0, cast(s64) bytes, memory, base.data);
} @Releases
