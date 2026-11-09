#module_parameters(
    // The address space a pool reserves when `init` is not given a size, in bytes.
    DEFAULT_VIRTUAL_MEMORY_RESERVE := 256 * 1024 * 1024,
    // Fills reset memory with `0xcc` for every pool.
    OVERWRITE_ALL_POOLS_ON_RESET := false
);

CAPS_VERSION_STRING :: "modules/Flat_Pool";

// A bump allocator over one contiguous range of reserved address space, committed page by page as it is used.
//
// `init` reserves `reserve` bytes of address space (256 MiB by default, rounded up to whole pages) without
// using memory; `get` hands out aligned slices of it from the start, committing pages when the bump point
// crosses into them. Everything stays at a fixed address, so growth never moves earlier allocations, which
// makes it suitable for data that points into itself. Running out of the reserved range fails an assert,
// so choose `reserve` for the largest use. Memory is not zeroed, nor is it ever freed piecemeal:
// the allocator interface ignores free, and a resize copies.
//
// `reset` rewinds to the start and keeps the committed pages for reuse; `fini` releases the whole range back
// to the OS. Both tell the memory debugger that everything allocated through `flat_pool_allocator_proc` is gone. A zeroed `Flat_Pool` is initialized by the first `get`. Not thread safe. Only available where the
// platform has virtual memory (macOS, Linux, Android and Windows).
//
// ```jai,skip_on(wasm)
// pool: Flat_Pool;
// init(*pool, 1024 * 1024);
// defer fini(*pool);
//
// first := cast(*int) get(*pool, size_of(int));
// first.* = 7;
// for 0..9999 get(*pool, 64); // growing never moves `first`
// assert(first.* == 7);
//
// reset(*pool);
// assert(get(*pool, size_of(int)) == first); // the same memory comes back
// ```
Flat_Pool :: struct {
    alignment: s32 = 8;
    current_point: *u8;
    memory_base: *u8;
    first_uncommitted_page: *u8;
    address_limit: *u8;
}

#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
    // Reserves `reserve` bytes of address space for the pool (256 MiB by default) and sets it up. No memory is used until `get`
    // touches pages.
    //
    // ```jai,skip_on(wasm)
    // pool: Flat_Pool;
    // init(*pool, 1024 * 1024);
    // defer fini(*pool);
    // assert(get(*pool, 16) != null);
    // ```
    init :: (using pool: *Flat_Pool, reserve := DEFAULT_VIRTUAL_MEMORY_RESERVE) {
        assert(memory_base == null, "flat pool must be finalized before initialization");
        assert(reserve > 0, "flat-pool reservation must be positive");
        page := Native.page_size();

        assert(page > 0, "OS page-size query failed");
        assert(reserve <= 0x7fffffffffffffff - (page - 1), "flat-pool reservation rounding overflow");
        extent := ((reserve + page - 1) / page) * page;
        memory := Native.reserve(cast(u64) extent);

        assert(memory != null, "flat-pool address reservation failed");
        if !memory return;

        base := cast(*u8) memory;

        memory_base = base;
        current_point = base;
        first_uncommitted_page = base;
        address_limit = base + extent;
    }

    // Returns `nbytes` bytes from the pool, aligned to its alignment. The memory is not cleared. Running out of the reserved
    // range fails an assert.
    //
    // ```jai,skip_on(wasm)
    // pool: Flat_Pool;
    // defer fini(*pool);
    // a := cast(*int) get(*pool, size_of(int));
    // a.* = 3;
    // assert(a.* == 3);
    // ```
    get :: (using pool: *Flat_Pool, nbytes: s64) -> *void {
        assert(nbytes >= 0 && alignment > 0 && (alignment & (alignment - 1)) == 0, "invalid flat-pool request/alignment");
        if !memory_base init(pool);

        assert(memory_base != null && current_point >= memory_base && current_point <= address_limit, "uninitialized or invalid flat pool");
        cursor := cast(u64) current_point;
        mask := cast(u64) alignment - 1;

        assert(cursor <= 0xffffffffffffffff - mask, "flat-pool alignment overflow");
        aligned := (cursor + mask) & ~mask;
        limit := cast(u64) address_limit;

        assert(aligned <= limit && cast(u64) nbytes <= limit - aligned, "flat-pool reservation exhausted");
        end := aligned + cast(u64) nbytes;
        committed := cast(u64) first_uncommitted_page;

        assert(committed >= cast(u64) memory_base && committed <= limit, "invalid flat-pool committed extent");

        if end > committed {
            page := Native.page_size();

            assert(page > 0, "OS page-size query failed");
            page_size := cast(u64) page;

            assert(end <= 0xffffffffffffffff - (page_size - 1), "flat-pool page rounding overflow");
            commit_end := ((end + page_size - 1) / page_size) * page_size;

            assert(commit_end <= limit, "flat-pool reservation must end on a page boundary");
            if !Native.commit(first_uncommitted_page, commit_end - committed) return null;

            first_uncommitted_page = cast(*u8) commit_end;
        }

        current_point = cast(*u8) end;

        return cast(*void) aligned;
    } @BorrowsElement(0,unknown,0)

    // Rewinds the pool to the start, keeping its committed pages for reuse. Pointers from `get` are invalid afterwards.
    //
    // ```jai,skip_on(wasm)
    // pool: Flat_Pool;
    // defer fini(*pool);
    // first := get(*pool, 32);
    // reset(*pool);
    // assert(get(*pool, 32) == first);
    // ```
    reset :: (using pool: *Flat_Pool, overwrite_memory := false) {
        if !memory_base return;

        assert(current_point >= memory_base && current_point <= first_uncommitted_page, "invalid flat-pool allocated extent");
        release_all_allocations(Allocator.{ flat_pool_allocator_proc, pool });
        if overwrite_memory || OVERWRITE_ALL_POOLS_ON_RESET memset(memory_base, 0xcc, current_point - memory_base);

        current_point = memory_base;
    } @ResetsElements

    // Gives the pool's whole address range back to the system.
    fini :: (using pool: *Flat_Pool) {
        if !memory_base return;

        assert(address_limit > memory_base, "invalid flat-pool reservation extent");
        release_all_allocations(Allocator.{ flat_pool_allocator_proc, pool });
        released := Native.release(memory_base, cast(u64) (address_limit - memory_base));
        if !released {
            context.logger("flat-pool reservation release failed\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
            assert(false, "flat-pool reservation release failed");
            return;
        }

        current_point = null;
        memory_base = null;
        first_uncommitted_page = null;
        address_limit = null;
    }

    // The `Allocator_Proc` for a flat pool: use it as `Allocator.{ flat_pool_allocator_proc, *pool }`. Allocate and resize take
    // memory from the pool, free does nothing.
    //
    // ```jai,skip_on(wasm)
    // pool: Flat_Pool;
    // defer fini(*pool);
    // context.allocator = Allocator.{ flat_pool_allocator_proc, *pool };
    // numbers := NewArray(4, int);
    // assert(numbers.count == 4);
    // ```
    flat_pool_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void {
        if mode == .CAPS {
            if old_memory {
                (cast(*string) old_memory).* = CAPS_VERSION_STRING;
            }

            return cast(*void) cast(u64) (Allocator_Caps.IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR);
        }

        if mode == .FREE return null;

        pool := cast(*Flat_Pool) allocator_data;

        assert(pool != null, "flat-pool allocator requires instance data");
        if mode == .ALLOCATE return get(pool, requested_size);

        if mode == .RESIZE {
            assert(old_size >= 0 && requested_size >= 0, "negative flat-pool resize extent");
            result := get(pool, requested_size);
            if result && old_memory memcpy(result, old_memory, min(old_size, requested_size));
            return result;
        }

        if mode == .IS_THIS_YOURS {
            address := cast(*u8) old_memory;
            if pool.memory_base && address >= pool.memory_base && address < pool.current_point return cast(*void) 1;
            return null;
        }

        if mode == .SHUTDOWN {
            fini(pool);
            return null;
        }

        if mode == .STARTUP {
            if !pool.memory_base init(pool);
            return null;
        }

        assert(false, "unsupported Flat_Pool allocator mode");

        return null;
    }

    // A small demonstration: initialize, allocate, reset with overwrite, finalize. It exists as a smoke test and is not part of the API.
    example :: () {
        pool: Flat_Pool;

        init(*pool);
        defer fini(*pool);
        data := cast(*u8) get(*pool, 32);

        assert(data != null, "flat-pool example commit failed");
        memset(data, 0, 32);
        reset(*pool, true);
        assert(data[0] == 0xcc);
    }
}

#scope_file
#import "Basic";
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS Native :: #import "internal/allocation/virtual-memory";
