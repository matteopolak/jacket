Deep_Copy :: (value: *$T, config: Deep_Copy_Config, preallocated_root: *void = null, pointer_map: *Pointer_Map = null, loc := #caller_location) -> *T {
    if !value return null;

    started := monotonic_seconds();

    defer record_elapsed(monotonic_seconds() - started);
    local_map: Pointer_Map;
    map := pointer_map;

    if !map map = *local_map;

    defer if !pointer_map H.deinit(*local_map.table);

    return cast(*T) clone_object(value, type_info(T), preallocated_root, map, config, loc);
}

#scope_file
#import "Basic";
H :: #import "Hash_Table";

clone_object :: (source: *void, info: *Type_Info, preallocated: *void, map: *Pointer_Map, config: Deep_Copy_Config, loc: Source_Code_Location) -> *void {
    if !source return null;

    existing := H.table_find_pointer(*map.table, source);
    if existing {
        assert(!preallocated || preallocated == existing.*, "conflicting Deep_Copy preallocated root at %:%", loc.fully_pathed_filename, loc.line_number);
        return existing.*;
    }

    assert(info != null && info.runtime_size >= 0);
    target := preallocated;
    if !target target = alloc(max(cast(s64) 1, info.runtime_size));

    assert(target != null, "Deep_Copy allocation failed");
    assert(target != source, "Deep_Copy requires distinct source and destination");

    if info.type == .STRUCT {
        alignment := (cast(*Type_Info_Struct) info).alignment;
        assert(alignment > 0 && cast(u64) target % cast(u64) alignment == 0, "Deep_Copy invalid reflected alignment % for %", alignment, (cast(*Type_Info_Struct) info).name);
    }

    H.table_add(*map.table, source, target);
    memcpy(target, source, info.runtime_size);
    copy_value(target, source, info, map, config, loc);

    return target;
}

copy_value :: (target: *void, source: *void, info: *Type_Info, map: *Pointer_Map, config: Deep_Copy_Config, loc: Source_Code_Location) {
    depth := record_visit();

    defer {
        #if DC_DEBUG record_leave();
    }

    #if DC_DEBUG && DEBUG_PRINT print("Deep_Copy depth % kind %\n", depth, cast(s64) info.type);

    if info.type == .STRING {
        input := (cast(*string) source).*;

        assert(input.count >= 0 && (input.count == 0 || input.data != null));
        output: string;
        if config.string_mapper_proc output = config.string_mapper_proc(input, config.string_mapper_data);
        else {
            if input.count > 0 {
                prior := H.table_find_pointer(*map.table, input.data);
                if prior output = .{ input.count, cast(*u8) prior.* };
                else {
                    output = copy_string(input);
                    H.table_add(*map.table, input.data, output.data);
                }
            }
        }

        (cast(*string) target).* = output;
    } else if info.type == .POINTER {
        pointer := cast(*Type_Info_Pointer) info;
        if config.follow_struct_pointers && pointer.pointer_to.type == .STRUCT {
            original := (cast(**void) source).*;
            (cast(**void) target).* = clone_object(original, pointer.pointer_to, null, map, config, loc);
        }
    } else if info.type == .ARRAY {
        copy_array(target, source, cast(*Type_Info_Array) info, map, config, loc);
    } else if info.type == .STRUCT {
        structure := cast(*Type_Info_Struct) info;

        assert((structure.textual_flags & .UNION) == 0, "Deep_Copy does not support overlapping unions");

        for member: structure.members {
            if (member.flags & (.CONSTANT | .IMPORTED | .OVERLAY)) != 0 continue;
            assert(member.type != null && member.offset_in_bytes >= 0 && member.offset_in_bytes <= info.runtime_size);
            assert(member.type.runtime_size >= 0 && member.type.runtime_size <= info.runtime_size - member.offset_in_bytes);
            copy_value(cast(*u8) target + member.offset_in_bytes, cast(*u8) source + member.offset_in_bytes, member.type, map, config, loc);
        }
    } else {
        assert(info.type == .INTEGER || info.type == .FLOAT || info.type == .BOOL ||
               info.type == .ENUM || info.type == .PROCEDURE || info.type == .TYPE || info.type == .VOID,
               "Deep_Copy does not support this reflected value kind: %", cast(s64) info.type);
    }
}

copy_array :: (target: *void, source: *void, info: *Type_Info_Array, map: *Pointer_Map, config: Deep_Copy_Config, loc: Source_Code_Location) {
    assert(info.array_type != .FIXED, "Deep_Copy does not support fixed arrays");
    original := cast(*Array_View_64) source;
    output := cast(*Array_View_64) target;
    capacity := original.count;

    assert(capacity >= 0);

    if info.array_type == .RESIZABLE {
        dynamic_source := cast(*Resizable_Array) source;
        dynamic_target := cast(*Resizable_Array) target;

        capacity = dynamic_source.allocated;
        assert(capacity >= original.count);
        dynamic_target.allocator = context.allocator;
    }

    element_size := info.element_type.runtime_size;

    assert(element_size > 0 && capacity <= 0x7fffffffffffffff / element_size);

    if capacity == 0 {
        output.data = null;
        return;
    }

    assert(original.data != null);
    prior := H.table_find_pointer(*map.table, original.data);
    if prior {
        output.data = cast(*u8) prior.*;
        return;
    }

    bytes := capacity * element_size;
    storage := cast(*u8) alloc(bytes);

    assert(storage != null, "Deep_Copy array allocation failed");
    H.table_add(*map.table, original.data, storage);
    output.data = storage;
    memcpy(storage, original.data, bytes);

    for index: 0..original.count - 1 {
        offset := index * element_size;
        copy_value(storage + offset, original.data + offset, info.element_type, map, config, loc);
    }
}

statistics_lock: s32;

lock_statistics :: () {
    acquired := compare_and_swap(*statistics_lock, cast(s32) 0, cast(s32) 1);
    while !acquired acquired = compare_and_swap(*statistics_lock, cast(s32) 0, cast(s32) 1);
}

unlock_statistics :: () {
    released := compare_and_swap(*statistics_lock, cast(s32) 1, cast(s32) 0);
    assert(released);
}

record_visit :: () -> int {
    lock_statistics();
    defer unlock_statistics();
    helper_count += 1;

    #if DC_DEBUG {
        indent += 1;
        return indent;
    } else return 0;
}

#if DC_DEBUG record_leave :: () {
    lock_statistics();
    defer unlock_statistics();
    indent -= 1;
}

record_elapsed :: (seconds: float64) {
    assert(seconds >= 0, "Deep_Copy timer moved backwards");
    lock_statistics();
    defer unlock_statistics();
    time_total += seconds;
}

timer_failure :: () {
    // A failed clock cannot yield a valid duration, including with optional
    // assertions disabled or a returning user assertion handler.
    context.assertion_failed(#caller_location, "Deep_Copy monotonic timer failed");
    abort_timer_failure();
}

Timer_Libc :: #system_library "libc";
abort_timer_failure :: () #foreign Timer_Libc "abort";

#if OS == .WINDOWS {
    W :: #import "Windows";

    monotonic_seconds :: () -> float64 {
        ticks: W.LARGE_INTEGER;
        frequency: W.LARGE_INTEGER;

        counter_status := W.QueryPerformanceCounter(*ticks);
        frequency_status := W.QueryPerformanceFrequency(*frequency);

        if counter_status == 0 || frequency_status == 0 || frequency.QuadPart <= 0 timer_failure();

        return cast(float64) ticks.QuadPart / cast(float64) frequency.QuadPart;
    }
} else {
    #assert OS == .MACOS || OS == .LINUX, "Deep_Copy timer backend is unavailable on this target";

    P :: #import "POSIX";

    monotonic_seconds :: () -> float64 {
        now: P.timespec;
        status := P.clock_gettime(P.clockid_t.MONOTONIC, *now);
        if status != 0 timer_failure();
        return cast(float64) now.tv_sec + cast(float64) now.tv_nsec / 1000000000.0;
    }
}
