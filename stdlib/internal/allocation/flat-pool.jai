#module_parameters(DEFAULT_VIRTUAL_MEMORY_RESERVE := 256 * 1024 * 1024, OVERWRITE_ALL_POOLS_ON_RESET := false);

CAPS_VERSION_STRING :: "modules/Flat_Pool";

Flat_Pool :: struct {
    alignment: s32 = 8;
    current_point: *u8;
    memory_base: *u8;
    first_uncommitted_page: *u8;
    address_limit: *u8;
}

#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
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
    }

    reset :: (using pool: *Flat_Pool, overwrite_memory := false) {
        if !memory_base return;
        assert(current_point >= memory_base && current_point <= first_uncommitted_page, "invalid flat-pool allocated extent");
        if overwrite_memory || OVERWRITE_ALL_POOLS_ON_RESET memset(memory_base, 0xcc, current_point - memory_base);
        current_point = memory_base;
    }

    fini :: (using pool: *Flat_Pool) {
        if !memory_base return;

        assert(address_limit > memory_base, "invalid flat-pool reservation extent");
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
