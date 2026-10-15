#import "Basic";
F :: #import "Flat_Pool";
Tiny :: #import "Flat_Pool"(DEFAULT_VIRTUAL_MEMORY_RESERVE = 8193);
Stamped :: #import "Flat_Pool"(DEFAULT_VIRTUAL_MEMORY_RESERVE = 8193, OVERWRITE_ALL_POOLS_ON_RESET = true);

#assert !#exists(F.Native);

main :: () {
    assert(type_info(F.Flat_Pool).name == "Flat_Pool");
    assert(size_of(F.Flat_Pool) == 40);
    assert(type_info(F.Flat_Pool).members[0].offset_in_bytes == 0);
    assert(type_info(F.Flat_Pool).members[1].offset_in_bytes == 8);

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS {
        page: s64;

        #if OS == .WINDOWS {
            info: W.SYSTEM_INFO;
            W.GetSystemInfo(*info);
            page = info.dwPageSize;
        } else page = P.sysconf(P._SC_PAGESIZE);

        assert(page > 0);
        extent := page * 4;
        pool: F.Flat_Pool;

        F.init(*pool, reserve = extent - 1);
        base := pool.memory_base;

        assert(base != null && pool.current_point == base && pool.first_uncommitted_page == base);
        assert(pool.address_limit - base == extent);
        first := cast(*u8) F.get(*pool, 3);

        assert(first == base && pool.first_uncommitted_page == base + page);
        first[0] = 1;
        first[1] = 2;
        first[2] = 3;
        pool.alignment = 64;
        next := cast(*u8) F.get(*pool, page + 1);

        assert((cast(u64) next & 63) == 0 && next > first);
        next[page] = 91;
        assert(next[page] == 91 && first[0] == 1 && first[2] == 3);
        committed := pool.first_uncommitted_page;
        allocator := Allocator.{ F.flat_pool_allocator_proc, *pool };
        resized := cast(*u8) allocator.proc(.RESIZE, 5, 3, first, allocator.data);

        assert(resized != first && resized[0] == 1 && resized[1] == 2 && resized[2] == 3);
        assert(first[0] == 1);
        assert(allocator.proc(.IS_THIS_YOURS, 0, 0, resized, allocator.data) != null);
        assert(allocator.proc(.IS_THIS_YOURS, 0, 0, pool.address_limit, allocator.data) == null);
        before_free := pool.current_point;

        allocator.proc(.FREE, 0, 5, resized, allocator.data);
        assert(pool.current_point == before_free);
        F.reset(*pool, true);
        assert(pool.current_point == base && pool.first_uncommitted_page == committed);
        for index: 0..before_free - base - 1 assert(base[index] == 0xcc);

        assert(allocator.proc(.IS_THIS_YOURS, 0, 0, first, allocator.data) == null);
        assert(F.get(*pool, 3) == first && first[0] == 0xcc);
        F.reset(*pool);
        assert(first[0] == 0xcc);
        F.fini(*pool);
        assert(pool.memory_base == null && pool.address_limit == null && pool.current_point == null && pool.first_uncommitted_page == null);
        F.fini(*pool);
        F.reset(*pool);
        name: string;
        flags := cast(Allocator_Caps) cast(u64) F.flat_pool_allocator_proc(.CAPS, 0, 0, *name, null);

        assert(name == F.CAPS_VERSION_STRING && flags == (.IS_THIS_YOURS | .HINT_I_AM_A_FAST_BUMP_ALLOCATOR));

        #if OS != .WINDOWS {
            // A real failed mprotect must leave the bump and commit metadata intact.
            readonly := P.open(temp_c_string(#file), P.O_RDONLY);

            assert(readonly >= 0);
            failed_mapping := P.mmap(null, cast(u64) page, P.PROT_NONE, P.MAP_SHARED, readonly, 0);

            assert(P.close(readonly) == 0);
            assert(failed_mapping != P.MAP_FAILED);
            failed_base := cast(*u8) failed_mapping;
            failed_pool := F.Flat_Pool.{ current_point = failed_base, memory_base = failed_base,
                                        first_uncommitted_page = failed_base, address_limit = failed_base + page };

            assert(F.get(*failed_pool, 1) == null);
            assert(failed_pool.current_point == failed_base && failed_pool.first_uncommitted_page == failed_base);
            F.fini(*failed_pool);
        }

        defaults: F.Flat_Pool;
        default_data := cast(*u8) F.get(*defaults, 1);

        assert(default_data != null && defaults.address_limit - defaults.memory_base == 256 * 1024 * 1024);
        default_data[0] = 17;
        F.reset(*defaults, false);
        assert(default_data[0] == 17);
        F.fini(*defaults);
        assert(F.get(*defaults, 0) != null);
        assert(defaults.current_point == defaults.memory_base && defaults.first_uncommitted_page == defaults.memory_base);
        F.fini(*defaults);
        custom: Tiny.Flat_Pool;

        Tiny.init(*custom);
        custom_data := cast(*u8) Tiny.get(*custom, 1);
        expected_extent := ((8193 + page - 1) / page) * page;

        assert(custom_data != null && custom.address_limit - custom.memory_base == expected_extent);
        custom_data[0] = 23;
        Tiny.reset(*custom, false);
        assert(custom_data[0] == 23);
        Tiny.fini(*custom);
        stamped: Stamped.Flat_Pool;
        stamp_data := cast(*u8) Stamped.get(*stamped, 16);

        assert(stamp_data != null);
        memset(stamp_data, 27, 16);
        Stamped.reset(*stamped, false);
        for i: 0..15 assert(stamp_data[i] == 0xcc);

        assert(Stamped.get(*stamped, 16) == stamp_data);
        memset(stamp_data, 31, 16);
        Stamped.reset(*stamped, true);
        for i: 0..15 assert(stamp_data[i] == 0xcc);

        Stamped.fini(*stamped);
        startup_pool: Tiny.Flat_Pool;

        Tiny.flat_pool_allocator_proc(.STARTUP, 0, 0, null, *startup_pool);
        assert(startup_pool.memory_base != null && startup_pool.address_limit - startup_pool.memory_base == expected_extent);
        startup_data := cast(*u8) Tiny.flat_pool_allocator_proc(.ALLOCATE, 1, 0, null, *startup_pool);

        assert(startup_data != null);
        startup_data[0] = 9;
        Tiny.flat_pool_allocator_proc(.SHUTDOWN, 0, 0, null, *startup_pool);
        assert(startup_pool.memory_base == null);
        F.example();

    }

    print("ok\n");
}

#scope_file
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID P :: #import "POSIX";
#if OS == .WINDOWS W :: #import "Windows";
