Temporary_Storage_Paged_Alloc :: struct {
    #as using base: Temporary_Storage;
    #overlay (implementation_data) using derived: struct {
        overflow_allocator: Allocator;
        overflow_pages: *Overflow_Page;
        original_data: *u8;
        original_size: s64;
        current_page_bytes_occupied: s64;
    }
}

Temporary_Storage_State :: struct {
    top_overflow_page: *Overflow_Page;
    current_page_bytes_occupied: s64;
    total_bytes_occupied: s64;
}

Overflow_Page :: struct {
    next: *Overflow_Page;
    allocator: Allocator;
    size: s64;
}

ALIGNED_OVERFLOW_PAGE_ALLOCATION :: 32;
SET_STORAGE_MARK_STAMP :: 0xda;
RESET_STORAGE_STAMP :: 0xdb;
FREE_PAGE_STAMP :: 0xdc;
// The temporary allocator: a scratch heap that is never freed piece by piece. Everything in it goes away together when
// `reset_temporary_storage` runs, which a program typically does once per frame or request. Use it for short-lived results
// so there is nothing to track or free. Set `context.allocator = temp` or use `push_allocator(temp)` to send allocations
// there.
//
// ```jai
// text := tprint("%, %", 1, 2); // allocated in temporary storage
// assert(text == "1, 2");
// reset_temporary_storage();
// ```
temp :: Allocator.{ temporary_allocator_proc, null };
// Another name for `temp`.
temporary_allocator :: temp;

// Allocates `size` bytes in temporary storage. They are not cleared, must not be freed, and last until the temporary
// storage is reset.
//
// ```jai
// bytes := cast(*u8) talloc(32);
// bytes[0] = 1;
// reset_temporary_storage();
// ```
talloc :: inline (size: s64) -> *void {
    return temporary_allocator_proc(.ALLOCATE, size, 0, null, null);
} @TempAllocates

// Another name for `talloc`.
temporary_alloc :: inline (size: s64) -> *void {
    return talloc(size);
} @TempAllocates

// A string of `count` bytes in temporary storage. It must not be freed; it lasts until the temporary storage is reset.
// The bytes are not cleared.
//
// ```jai
// s := talloc_string(3);
// memcpy(s.data, "abc".data, 3);
// assert(s == "abc");
// reset_temporary_storage();
// ```
talloc_string :: (count: int) -> string {
    data := cast(*u8) talloc(count);
    assert(data != null || count == 0);
    return .{ count, data };
} @TempAllocates

// A copy of `s` in temporary storage. It must not be freed; it lasts until the temporary storage is
// reset.
//
// ```jai
// copy := copy_temporary_string("scratch");
// assert(copy == "scratch");
// reset_temporary_storage();
// ```
copy_temporary_string :: inline (s: string) -> string {
    result := talloc_string(s.count);
    memcpy(result.data, s.data, s.count);
    return result;
} @TempAllocates

// A copy of `s` in temporary storage followed by a zero byte, for passing to C. It must not be freed; it lasts until the
// temporary storage is reset.
//
// ```jai
// c := temp_c_string("hi");
// assert(c[0] == #char "h" && c[2] == 0);
// reset_temporary_storage();
// ```
temp_c_string :: (s: string) -> *u8 {
    result := cast(*u8) talloc(s.count + 1);

    assert(result != null);
    memcpy(result, s.data, s.count);
    result[s.count] = 0;

    return result;
} @TempAllocates

temporary_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
    if mode == .CAPS {
        if old_memory {
            (cast(*string) old_memory).* = "Basic temporary storage";
        }

        return cast(*void) cast(u64) Allocator_Caps.HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE;
    }

    if mode == .FREE return null;

    if mode != .ALLOCATE && mode != .RESIZE {
        assert(mode == .STARTUP || mode == .SHUTDOWN || mode == .THREAD_START || mode == .THREAD_STOP, "unsupported temporary allocator mode");
        return null;
    }

    assert(requested_size >= 0 && old_size >= 0);
    ts := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
    if !ts {
        allocator := context.allocator;
        if allocator.proc == temporary_allocator_proc allocator = context.default_allocator;
        return allocator_call(allocator, mode, requested_size, old_size, old_memory);
    }

    alignment: s64 = TEMPORARY_ALIGNMENT;
    if requested_size > 0x7fffffffffffffff - alignment - 16 return null;
    if mode == .RESIZE && old_memory && requested_size <= align_forward(old_size, alignment) && cast(u64) old_memory % cast(u64) alignment == 0 return old_memory;

    if !ts.original_data && !ts.overflow_pages {
        ts.original_data = ts.data;
        ts.original_size = ts.size;
    }

    occupied := align_forward(cast(s64) ts.data + ts.current_page_bytes_occupied, alignment) - cast(s64) ts.data;
    padded := align_forward(requested_size, alignment);

    // The newest allocation on the current page grows where it is.
    if mode == .RESIZE && old_memory && cast(*u8) old_memory >= ts.data && cast(*u8) old_memory + align_forward(old_size, alignment) == ts.data + ts.current_page_bytes_occupied {
        start := cast(*u8) old_memory - ts.data;
        if start + padded <= ts.size {
            ts.total_bytes_occupied += start + padded - ts.current_page_bytes_occupied;
            ts.current_page_bytes_occupied = start + padded;
            ts.high_water_mark = max(ts.high_water_mark, ts.total_bytes_occupied);
            return old_memory;
        }
    }

    if occupied + padded > ts.size {
        if !add_new_page(ts, max(padded + alignment - 1, cast(s64) 4096)) return null;
        occupied = align_forward(cast(s64) ts.data, alignment) - cast(s64) ts.data;
    }

    result := ts.data + occupied;

    ts.total_bytes_occupied += occupied + padded - ts.current_page_bytes_occupied;
    ts.current_page_bytes_occupied = occupied + padded;
    ts.high_water_mark = max(ts.high_water_mark, ts.total_bytes_occupied);
    if mode == .RESIZE && old_memory memcpy(result, old_memory, min(old_size, requested_size));

    return result;
}

get_size_including_overflow_page_header :: (data_size: s64) -> s64 {
    return align_forward(size_of(Overflow_Page), 32) + data_size;
}

add_new_page :: (ts: *Temporary_Storage_Paged_Alloc, minimum_size: s64) -> bool {
    allocator := ts.overflow_allocator;
    if !allocator.proc || allocator.proc == temporary_allocator_proc {
        allocator = context.allocator;
        if allocator.proc == temporary_allocator_proc allocator = context.default_allocator;
        ts.overflow_allocator = allocator;
    }

    page := cast(*Overflow_Page) allocator_call(allocator, .ALLOCATE, get_size_including_overflow_page_header(minimum_size), 0, null);
    if !page return false;

    page.next = ts.overflow_pages;
    page.allocator = allocator;
    page.size = minimum_size;
    ts.overflow_pages = page;
    ts.data = cast(*u8) page + align_forward(size_of(Overflow_Page), 32);
    ts.size = minimum_size;
    ts.current_page_bytes_occupied = 0;

    return true;
}

free_pages_down_to :: (ts: *Temporary_Storage_Paged_Alloc, page: *Overflow_Page) {
    if !ts.original_data && !ts.overflow_pages {
        ts.original_data = ts.data;
        ts.original_size = ts.size;
    }

    while ts.overflow_pages != page {
        top := ts.overflow_pages;

        assert(top != null, "invalid temporary storage mark");
        ts.overflow_pages = top.next;
        allocator := top.allocator;

        #if TEMP_ALLOCATOR_OVERWRITE_MEMORY memset(cast(*u8) top + align_forward(size_of(Overflow_Page), 32), FREE_PAGE_STAMP, top.size);
        allocator_call(allocator, .FREE, 0, get_size_including_overflow_page_header(top.size), top);
    }

    if page {
        ts.data = cast(*u8) page + align_forward(size_of(Overflow_Page), 32);
        ts.size = page.size;
    } else {
        ts.data = ts.original_data;
        ts.size = ts.original_size;
    }
}

// The current position in temporary storage. Pass it to `set_temporary_storage_mark` to free everything allocated after
// this point while keeping what came before.
get_temporary_storage_mark :: inline () -> Temporary_Storage_State {
    ts := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
    if !ts return .{};
    return .{ ts.overflow_pages, ts.current_page_bytes_occupied, ts.total_bytes_occupied };
}

// Rewinds temporary storage to `state`, a value from `get_temporary_storage_mark`. Everything allocated since then is gone.
//
// ```jai
// mark := get_temporary_storage_mark();
// scratch := tprint("%", 12345);
// assert(scratch == "12345");
// set_temporary_storage_mark(mark);
// ```
set_temporary_storage_mark :: inline (state: Temporary_Storage_State, loc := #caller_location) {
    ts := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
    if !ts return;

    #if TEMP_ALLOCATOR_OVERWRITE_MEMORY {
        assert(state.current_page_bytes_occupied >= 0);
        end := ifx ts.overflow_pages == state.top_overflow_page then ts.current_page_bytes_occupied else -1;

        free_pages_down_to(ts, state.top_overflow_page);
        if end < 0 end = ts.size;

        assert(state.current_page_bytes_occupied <= end && end <= ts.size);
        memset(ts.data + state.current_page_bytes_occupied, SET_STORAGE_MARK_STAMP, end - state.current_page_bytes_occupied);
    } else {
        free_pages_down_to(ts, state.top_overflow_page);
    }

    ts.current_page_bytes_occupied = state.current_page_bytes_occupied;
    ts.total_bytes_occupied = state.total_bytes_occupied;
    ts.last_set_mark_location = loc;
}

// Rewinds temporary storage to where it is now when the enclosing scope ends. Put it at the top of a procedure that makes
// temporary strings so its callers don't pile them up.
//
// ```jai
// shout :: (s: string) -> int {
//     auto_release_temp();
//     upper := tprint("%!", s);
//     return upper.count;
// }
// assert(shout("hey") == 4);
// ```
auto_release_temp :: (loc := #caller_location) #expand {
    mark := get_temporary_storage_mark();
    `defer set_temporary_storage_mark(mark, loc);
}

reset_temporary_storage :: (_ts: *Temporary_Storage) {
    if !_ts return;

    ts := cast(*Temporary_Storage_Paged_Alloc) _ts;

    #if TEMP_ALLOCATOR_OVERWRITE_MEMORY {
        end := ifx ts.overflow_pages == null then ts.current_page_bytes_occupied else -1;
        free_pages_down_to(ts, null);
        if end < 0 end = ts.size;
        memset(ts.data, RESET_STORAGE_STAMP, end);
    } else {
        free_pages_down_to(ts, null);
    }

    ts.current_page_bytes_occupied = 0;
    ts.total_bytes_occupied = 0;
}

// Frees everything in temporary storage at once. Any pointer or string taken from it is invalid afterwards.
//
// ```jai
// a := tprint("one");
// b := tprint("two");
// assert(a == "one" && b == "two");
// reset_temporary_storage();
// ```
reset_temporary_storage :: () {
    reset_temporary_storage(context.temporary_storage);
}

#scope_file
TEMPORARY_ALIGNMENT :: 8;
