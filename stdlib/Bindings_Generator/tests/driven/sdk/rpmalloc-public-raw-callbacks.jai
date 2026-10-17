// Reviewed rpmalloc.md raw config callback signatures and shallow string copy.
#import "Basic";
RPM :: #import "rpmalloc"(USE_RAW = true);

#add_context raw_callback_marker: s64;
probe :: #library,system "rpmalloc_callback_probe";
map_native :: (size: u64) -> *void #foreign probe "binding_rpmalloc_test_map";
release :: (memory: *void) #foreign probe "binding_rpmalloc_test_unmap";
mapped: s64;
unmapped: s64;
errors: s64;
failures: s64;
decline: bool;

map_memory :: (size: u64, offset: *u64) -> *void {
    assert(context.raw_callback_marker == 61);
    if decline return null;

    memory := map_native(size);
    if memory {
        offset.* = 65535;
        mapped += 1;
    }

    return memory;
}

unmap_memory :: (address: *void, size: u64, offset: u64, full: u64) {
    assert(size > 0);

    if full {
        assert(offset == 65535);
        release(address);
        unmapped += 1;
    } else assert(offset == 0);
}

error :: (message: string) {
    assert(message == "adapter test error");
    errors += 1;
}

failure :: (size: u64) -> s32 {
    assert(size > 0);
    failures += 1;
    return 0;
}

main :: () {
    context.raw_callback_marker = 61;
    input: RPM.rpmalloc_config_t;
    input.memory_map = map_memory;
    input.memory_unmap = unmap_memory;
    input.map_fail_callback = failure;
    input.error_callback = error;
    assert(RPM.initialize_config(*input) == 0);
    owned := RPM.config();

    assert(owned != *input && owned.memory_map == map_memory);
    input.memory_map = null;
    assert(owned.memory_map == map_memory);
    assert(owned.error_callback == error);
    memory := RPM.rpmalloc(8 * 1024 * 1024);

    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
    assert(mapped > 0 && mapped == unmapped && failures == 0);
    input.memory_map = map_memory;
    decline = true;
    assert(RPM.initialize_config(*input) != 0);
    assert(!RPM.is_initialized() && failures == 0);
    input.memory_unmap = null;
    assert(RPM.initialize_config(*input) == 0);
    memory = RPM.rpmalloc(37);
    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
}
