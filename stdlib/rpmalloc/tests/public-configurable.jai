// Reviewed rpmalloc.md CONFIGURABLE initialization and invalid size contracts.
#import "Basic";
RPM :: #import "rpmalloc"(CONFIGURABLE = true);
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = true, RAW = false, PUBLIC_HEAPS = true, GLOBAL_CACHE = true, THREAD_CACHE = true, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = false);
Default :: #import "rpmalloc";

probe :: #library,system "rpmalloc_callback_probe";
config_value_raw :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_test_config";

config_value :: (index: u32) -> u64 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return config_value_raw(index);
}

exercise_heap :: (span_size: s64) {
    config: RPM.rpmalloc_config_t;
    config.span_size = span_size;
    config.span_map_count = 4;
    assert(RPM.initialize_config(*config) == 0);
    heap := RPM.heap_acquire();

    assert(heap != null);
    sizes: [6] s64 = .[1, 64, 512, 16384, 1_048_576, 33_554_432];

    for sizes {
        memory := RPM.heap_alloc(heap, it);

        assert(memory != null && RPM.heap_is_this_yours(heap, memory));
        (cast(*u8) memory)[0] = 61;
        (cast(*u8) memory)[it - 1] = 61;
        replacement := RPM.heap_realloc(heap, memory, it * 2, .{});

        assert(replacement != null && RPM.heap_is_this_yours(heap, replacement));
        assert((cast(*u8) replacement)[0] == 61);
        assert((cast(*u8) replacement)[it - 1] == 61);
        RPM.heap_free(heap, replacement);
        RPM.heap_free_all(heap);
    }

    RPM.heap_thread_set_current(heap);
    memory := RPM.rpmalloc(6000);

    assert(memory != null && RPM.heap_is_this_yours(heap, memory));
    RPM.rpfree(memory);
    RPM.heap_release(heap);
    RPM.finalize();
}

main :: () {
    config: RPM.rpmalloc_config_t;
    config.page_size = 3;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    config.page_size = 16384;
    config.span_size = 65536;
    config.span_map_count = 32;
    assert(RPM.initialize_config(*config) == 0);
    // Independent safety: an overlapping import must neither replace callbacks
    // nor finalize the active import's allocator domain.
    assert(Default.initialize() == 0);
    assert(Default.is_initialized() && RPM.is_initialized());
    separate := Default.rpmalloc(4096);

    assert(separate != null);
    (cast(*u8) separate)[4095] = 71;
    assert((cast(*u8) separate)[4095] == 71);
    Default.rpfree(separate);
    Default.finalize();
    assert(RPM.is_initialized() && !Default.is_initialized());
    assert(config_value(0) == 16384);
    assert(config_value(1) == 65536);
    assert(config_value(2) == 32);
    memory := RPM.rpmalloc(1024);

    assert(memory != null);
    (cast(*u8) memory)[1023] = 21;
    RPM.rpfree(memory);
    RPM.finalize();
    config.span_size = 3;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    exercise_heap(4096);
    exercise_heap(16384);
    exercise_heap(65536);
    exercise_heap(262144);
}
