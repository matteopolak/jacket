// Reviewed rpmalloc.md: paired callbacks, retry/decline, matching lifetime,
// and signed 64-bit representable mapping offsets. Native fixture uses libc.
#import "Basic";

#add_context callback_marker: s64;
#add_context callback_calls: s64;
#add_context callback_thread: u64;

RPM :: #import "rpmalloc";

probe :: #library,system "rpmalloc_callback_probe";
map_native :: (size: u64) -> *void #foreign probe "binding_rpmalloc_test_map";
release :: (memory: *void) #foreign probe "binding_rpmalloc_test_unmap";
count :: (index: u32, amount: u64) -> u64 #foreign probe "binding_rpmalloc_count";
names :: () -> s32 #foreign probe "binding_rpmalloc_test_names";
send_error :: () -> s32 #foreign probe "binding_rpmalloc_test_error";
config_value :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_test_config";
parallel :: (worker: #type () #c_call) -> s32 #foreign probe "binding_rpmalloc_parallel";
native_thread :: () -> u64 #foreign probe "binding_rpmalloc_native_thread";
context_lib :: #library,system "binding_callback_context";
context_pointer :: () -> *void #foreign context_lib "binding_callback_context_current";
decline: bool;
OFFSET :: 0x1_0000_0007;
saved_offset: s64 = OFFSET;

map_memory :: (size: s64, offset: *s64) -> *void {
    assert(context.callback_marker == 47 || context.callback_marker == 74);
    assert(context.callback_thread == native_thread());
    before := context_pointer();

    RPM.is_thread_initialized();
    assert(before != null && before == context_pointer());
    assert(before == cast(*void) *context);
    context.callback_calls += 1;
    attempts := count(0, 1);
    if decline || attempts <= 2 return null;

    memory := map_native(cast(u64) size);
    if !memory return null;

    offset.* = saved_offset;
    count(1, 1);

    return memory;
}

unmap_memory :: (address: *void, size: s64, offset: s64, full: s64) {
    assert(size > 0);

    if full {
        assert(offset == saved_offset);
        count(2, 1);
        release(address);
    } else {
        assert(offset == 0);
    }
}

retry :: (size: s64) -> s32 {
    assert(size > 0);
    count(3, 1);
    return ifx decline then 0 else 1;
}

on_error :: (message: string, location: Source_Code_Location) {
    assert(context.callback_marker == 0);
    assert(message == "adapter test error");
    // Native errors have no Jai location; the adapter supplies its real bridge
    // call site. Exact reference provenance remains a reviewed-contract question.
    assert(location.line_number > 0);
    count(4, 1);
}

setup :: () {
    previous_callbacks := context.callback_calls;

    guarded: struct {
        before: u64 = 0x1256;
        config: RPM.rpmalloc_config_t;
        after: u64 = 0xabcd;
    };

    config := *guarded.config;
    config.memory_map = map_memory;
    config.memory_unmap = unmap_memory;
    config.map_fail_callback = retry;
    config.error_callback = on_error;
    // Reviewed nonconfigurable imports ignore all three size overrides.
    config.page_size = 3;
    config.span_size = 3;
    config.span_map_count = -3;
    page: [12] u8;
    huge: [12] u8;

    for 0..11 {
        page[it] = "binding-page".data[it];
        huge[it] = "binding-huge".data[it];
    }

    config.page_name.data = *page[0];
    config.page_name.count = 12;
    config.huge_page_name.data = *huge[0];
    config.huge_page_name.count = 12;
    assert(RPM.initialize_config(config) == 0);
    assert(context.callback_calls > previous_callbacks);
    assert(guarded.before == 0x1256 && guarded.after == 0xabcd);
    // Subsequent SDK callbacks must use the owned copy, not this stack object.
    config.memory_map = null;
    config.memory_unmap = null;
    config.map_fail_callback = null;
    config.error_callback = null;

    for 0..11 {
        page[it] = 88;
        huge[it] = 88;
    }
}

worker :: () #c_call {
    push_context, defer_pop;
    context.callback_marker = 74;
    context.callback_thread = native_thread();
    RPM.thread_initialize();

    for 0..7 {
        memory := RPM.rpmalloc(1024 * 1024);
        assert(memory != null);
        (cast(*u8) memory)[0] = 74;
        RPM.rpfree(memory);
    }

    RPM.thread_finalize(true);
}

main :: () {
    context.callback_marker = 47;
    context.callback_thread = native_thread();
    assert(context_pointer() == null);
    setup();
    assert(context_pointer() == null);
    assert(RPM.is_initialized());
    assert(count(3, 0) == 2);
    assert(names() != 0);
    assert(send_error() != 0 && count(4, 0) == 1);
    assert(config_value(0) != 3 && config_value(1) != 3 && config_value(2) > 0);
    assert(parallel(worker) == 0);
    memory := RPM.rpmalloc(8 * 1024 * 1024);

    assert(memory != null);
    (cast(*u8) memory)[0] = 93;
    RPM.rpfree(memory);
    RPM.finalize();
    assert(count(1, 0) > 0 && count(1, 0) == count(2, 0));
    // The private registry round-trips signed bits without assuming byte or
    // SDK offset units, including values the native span header cannot hold.
    saved_offset = -91;
    setup();
    memory = RPM.rpmalloc(1024);
    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
    assert(count(1, 0) == count(2, 0));
    decline = true;
    config: RPM.rpmalloc_config_t;
    config.memory_map = map_memory;
    config.memory_unmap = unmap_memory;
    config.map_fail_callback = retry;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    config.memory_unmap = null;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    decline = false;
    assert(RPM.initialize() == 0);
    memory = RPM.rpmalloc(128);
    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
    assert(context_pointer() == null);
}
