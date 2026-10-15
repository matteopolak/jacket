// Reviewed rpmalloc.md CONFIGURABLE initialization and invalid size contracts.
#import "Basic";
RPM :: #import "rpmalloc"(CONFIGURABLE = true);
Default :: #import "rpmalloc";

probe :: #library,system "rpmalloc_callback_probe";
config_value :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_test_config";

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
    assert(Default.initialize() != 0);
    assert(!Default.is_initialized());
    Default.finalize();
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
}
