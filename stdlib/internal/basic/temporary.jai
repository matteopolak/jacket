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
temp :: Allocator.{ temporary_allocator_proc, null };
temporary_allocator :: temp;

talloc :: inline (size: s64) -> *void {
    return temporary_allocator_proc(.ALLOCATE, size, 0, null, null);
}

temporary_alloc :: inline (size: s64) -> *void {
    return talloc(size);
}

talloc_string :: (count: int) -> string {
    data := cast(*u8) talloc(count);
    assert(data != null || count == 0);
    return .{ count, data };
}

copy_temporary_string :: inline (s: string) -> string {
    result := talloc_string(s.count);
    memcpy(result.data, s.data, s.count);
    return result;
}

temp_c_string :: (s: string) -> *u8 {
    result := cast(*u8) talloc(s.count + 1);

    assert(result != null);
    memcpy(result, s.data, s.count);
    result[s.count] = 0;

    return result;
}

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

    alignment := scratch_alignment(requested_size);
    if requested_size > 0x7fffffffffffffff - alignment - 16 return null;
    if mode == .RESIZE && old_memory && requested_size <= align_forward(old_size, 16) && cast(u64) old_memory % cast(u64) alignment == 0 return old_memory;

    if !ts.original_data && !ts.overflow_pages {
        ts.original_data = ts.data;
        ts.original_size = ts.size;
    }

    occupied := align_forward(cast(s64) ts.data + ts.current_page_bytes_occupied, alignment) - cast(s64) ts.data;
    padded := align_forward(requested_size, 16);

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

get_temporary_storage_mark :: inline () -> Temporary_Storage_State {
    ts := cast(*Temporary_Storage_Paged_Alloc) context.temporary_storage;
    if !ts return .{};
    return .{ ts.overflow_pages, ts.current_page_bytes_occupied, ts.total_bytes_occupied };
}

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

reset_temporary_storage :: () {
    reset_temporary_storage(context.temporary_storage);
}

#scope_file
scratch_alignment :: (size: s64) -> s64 #no_aoc {
    value := cast(u64) size;
    alignment := value & (0 - value);
    return max(cast(s64) 16, cast(s64) alignment);
}
