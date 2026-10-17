// Real private C SDK backend; this does not import or validate the unfinished
// public rpmalloc facade and never casts its Jai callback/heap/config records.
#import "internal/bindings/rpmalloc-sdk";
#load "generated/rpmalloc-sdk-layout.jai";
#load "../../../../internal/bindings/rpmalloc-checked/module.jai";

probe :: #library,system "rpmalloc_sdk_probe";
sdk_size :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_size";
sdk_offset :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_offset";
sdk_field_size :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_field_size";
libc :: #library,system "c";
abort :: () #foreign libc;
tmpfile :: () -> *void #foreign libc;
fflush :: (stream: *void) -> s32 #foreign libc;
ferror :: (stream: *void) -> s32 #foreign libc;
fgetc :: (stream: *void) -> s32 #foreign libc;
fclose :: (stream: *void) -> s32 #foreign libc;
#if OS == .WINDOWS C_Long :: s32;
else C_Long :: s64;
fseek :: (stream: *void, offset: C_Long, origin: s32) -> s32 #foreign libc;

check :: (value: bool) {
    if !value abort();
}

bytes_are :: (memory: *void, count: int, value: u8) {
    bytes := cast(*u8) memory;
    index: int;

    while index < count {
        check(bytes[index] == value);
        index += 1;
    }
}

fill :: (memory: *void, count: int, value: u8) {
    bytes := cast(*u8) memory;
    index: int;

    while index < count {
        bytes[index] = value;
        index += 1;
    }
}

main :: () {
    sdk_layout_checks();
    config: rpmalloc_config_t;
    config.span_size = 65536;
    config.span_map_count = 64;
    check(rpmalloc_initialize_config(*config) == 0);
    rpmalloc_set_main_thread();
    check(rpmalloc_config().span_size == 65536);
    check(rpmalloc_is_thread_initialized() != 0);
    rpmalloc_thread_initialize();

    memory := rpmalloc(64);

    check(memory != null && rpmalloc_usable_size(memory) >= 64);
    fill(memory, 64, 0x31);
    grown := rprealloc(memory, 4096);

    check(grown != null && rpmalloc_usable_size(grown) >= 4096);
    bytes_are(grown, 64, 0x31);
    rpfree(grown);

    zeroed := rpcalloc(16, 8);

    check(zeroed != null);
    bytes_are(zeroed, 128, 0);
    rpfree(zeroed);

    aligned := rpaligned_alloc(64, 128);

    check(aligned != null && cast(u64) aligned % 64 == 0);
    fill(aligned, 128, 0x42);
    realigned := rpaligned_realloc(aligned, 64, 4096, 128, 0);

    check(realigned != null && cast(u64) realigned % 64 == 0);
    bytes_are(realigned, 128, 0x42);
    rpfree(realigned);

    aligned_zeroed := rpaligned_calloc(128, 16, 8);

    check(aligned_zeroed != null && cast(u64) aligned_zeroed % 128 == 0);
    bytes_are(aligned_zeroed, 128, 0);
    rpfree(aligned_zeroed);
    aligned = rpmemalign(256, 128);
    check(aligned != null && cast(u64) aligned % 256 == 0);
    rpfree(aligned);
    check(rpposix_memalign(*aligned, 128, 128) == 0);
    check(aligned != null && cast(u64) aligned % 128 == 0);
    rpfree(aligned);
    aligned = cast(*void) 1;
    check(posix_memalign(*aligned, 3, 128) == 22);
    check(aligned == cast(*void) 1);
    check(posix_memalign(*aligned, 0, 128) == 22);
    check(posix_memalign(null, 128, 128) == 22);
    check(posix_memalign(*aligned, rpmalloc_config().span_size, 128) == 22);
    check(aligned == cast(*void) 1);
    check(posix_memalign(*aligned, 128, 0xffffffffffffffff) != 0);
    check(aligned == cast(*void) 1);
    check(posix_memalign(*aligned, 128, 128) == 0);
    check(aligned != null && cast(u64) aligned % 128 == 0);
    rpfree(aligned);

    heap := rpmalloc_heap_acquire();

    check(heap != null);
    memory = rpmalloc_heap_alloc(heap, 64);
    check(memory != null && rpmalloc_get_heap_for_ptr(memory) == heap);
    fill(memory, 64, 0x53);
    grown = rpmalloc_heap_realloc(heap, memory, 4096, 0);
    check(grown != null);
    bytes_are(grown, 64, 0x53);
    rpmalloc_heap_free(heap, grown);
    zeroed = rpmalloc_heap_calloc(heap, 8, 16);
    check(zeroed != null);
    bytes_are(zeroed, 128, 0);
    aligned = rpmalloc_heap_aligned_alloc(heap, 128, 128);
    check(aligned != null && cast(u64) aligned % 128 == 0);
    fill(aligned, 128, 0x64);
    realigned = rpmalloc_heap_aligned_realloc(heap, aligned, 128, 4096, 0);
    check(realigned != null && cast(u64) realigned % 128 == 0);
    bytes_are(realigned, 128, 0x64);
    aligned_zeroed = rpmalloc_heap_aligned_calloc(heap, 128, 8, 16);
    check(aligned_zeroed != null && cast(u64) aligned_zeroed % 128 == 0);
    bytes_are(aligned_zeroed, 128, 0);
    rpmalloc_heap_free_all(heap);
    rpmalloc_heap_release(heap);

    current := rpmalloc_heap_acquire();

    check(current != null);
    rpmalloc_heap_thread_set_current(current);
    memory = rpmalloc(64);
    check(memory != null && rpmalloc_get_heap_for_ptr(memory) == current);
    rpfree(memory);
    rpmalloc_thread_collect();

    statistics: struct {
        before: u64 = 0x1234567887654321;
        value: rpmalloc_global_statistics_t;
        after: u64 = 0x89abcdef76543210;
    }

    rpmalloc_global_statistics(*statistics.value);
    check(statistics.value.mapped_peak > 0 && statistics.value.mapped_total >= statistics.value.mapped);
    check(statistics.before == 0x1234567887654321 && statistics.after == 0x89abcdef76543210);

    thread_statistics: struct {
        before: u64 = 0x1234567887654321;
        value: rpmalloc_thread_statistics_t;
        after: u64 = 0x89abcdef76543210;
    }

    rpmalloc_thread_statistics(*thread_statistics.value);
    allocations, frees: u64;

    for thread_statistics.value.size_use {
        check(it.alloc_current <= it.alloc_total && it.alloc_peak >= it.alloc_current);
        allocations += it.alloc_total;
        frees += it.free_total;
    }

    check(allocations > 0 && frees > 0);
    check(thread_statistics.before == 0x1234567887654321 && thread_statistics.after == 0x89abcdef76543210);
    report := tmpfile();

    check(report != null);
    rpmalloc_dump_statistics(report);
    check(fflush(report) == 0 && ferror(report) == 0);
    check(fseek(report, 0, 0) == 0);
    check(fgetc(report) >= 0 && ferror(report) == 0);
    check(fclose(report) == 0);
    rpmalloc_thread_finalize(1);
    check(rpmalloc_is_thread_initialized() == 0);
    rpmalloc_finalize();
    check(rpmalloc_initialize() == 0);
    memory = rpmalloc(64);
    check(memory != null);
    rpfree(memory);
    rpmalloc_linker_reference();
    rpmalloc_finalize();
}
