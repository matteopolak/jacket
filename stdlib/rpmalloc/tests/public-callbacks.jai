// Reviewed rpmalloc.md: paired callbacks, retry/decline, matching lifetime,
// and bounded mapping offsets. Native fixture uses libc.
#import "Basic";

#add_context callback_marker: s64;
#add_context callback_calls: s64;
#add_context callback_thread: u64;

RPM :: #import "rpmalloc";
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = true, RAW = false, PUBLIC_HEAPS = true, GLOBAL_CACHE = true, THREAD_CACHE = true, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = false);

probe :: #library,system "rpmalloc_callback_probe";
map_native :: (size: u64) -> *void #foreign probe "binding_rpmalloc_test_map";
release :: (memory: *void) #foreign probe "binding_rpmalloc_test_unmap";
count :: (index: u32, amount: u64) -> u64 #foreign probe "binding_rpmalloc_count";
names_raw :: () -> s32 #foreign probe "binding_rpmalloc_test_names";
send_error_raw :: () -> s32 #foreign probe "binding_rpmalloc_test_error";
config_value_raw :: (index: u32) -> u64 #foreign probe "binding_rpmalloc_test_config";

names :: () -> s32 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return names_raw();
}

send_error :: () -> s32 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return send_error_raw();
}

config_value :: (index: u32) -> u64 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return config_value_raw(index);
}

parallel :: (worker: #type () #c_call) -> s32 #foreign probe "binding_rpmalloc_parallel";
native_thread :: () -> u64 #foreign probe "binding_rpmalloc_native_thread";
context_lib :: #library,system "binding_callback_context";
context_pointer :: () -> *void #foreign context_lib "binding_callback_context_current";
handoff_reset :: () #foreign probe "binding_rpmalloc_handoff_reset";
handoff_claim :: () -> u32 #foreign probe "binding_rpmalloc_handoff_claim";
handoff_out: [4] *void;
handoff_back: [4] *void;
decline: bool;
bad_alignment: bool;
OFFSET :: 65535;
saved_offset: s64 = OFFSET;
page: [12] u8;
huge: [12] u8;

map_memory :: (size: s64, offset: *s64) -> *void {
    assert(context.callback_marker == 47 || context.callback_marker == 74);
    assert(context.callback_thread == native_thread());
    before := context_pointer();

    RPM.is_thread_initialized();
    assert(before != null && before == context_pointer());
    assert(before == cast(*void) *context);
    context.callback_calls += 1;
    count(0, 1);
    if decline return null;

    memory := map_native(cast(u64) size);
    if !memory return null;

    offset.* = saved_offset;
    count(1, 1);
    if bad_alignment return cast(*u8) memory + 8;

    return memory;
}

unmap_memory :: (address: *void, size: s64, offset: s64, full: s64) {
    assert(size > 0);

    if full {
        assert(offset == saved_offset);
        count(2, 1);
        if bad_alignment release(cast(*u8) address - 8);
        else release(address);
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

    // Public strings borrow backing bytes. Keep this storage alive through
    // finalization; live byte changes are covered by the dedicated regression.
}

worker :: () #c_call {
    push_context, defer_pop;
    context.callback_marker = 74;
    context.callback_thread = native_thread();
    RPM.thread_initialize();
    index := handoff_claim();

    assert(index < 4);
    incoming := handoff_out[index];

    assert(incoming != null && (cast(*u8) incoming)[0] == cast(u8) (81 + index));
    RPM.rpfree(incoming);
    handoff_out[index] = null;
    outgoing := RPM.rpmalloc(2048);

    assert(outgoing != null);
    (cast(*u8) outgoing)[0] = cast(u8) (91 + index);
    handoff_back[index] = outgoing;

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
    assert(count(3, 0) == 0);
    assert(names() != 0);
    assert(send_error() != 0 && count(4, 0) == 1);
    assert(config_value(0) != 3 && config_value(1) != 3 && config_value(2) > 0);
    handoff_reset();

    for 0..3 {
        handoff_out[it] = RPM.rpmalloc(3072);
        assert(handoff_out[it] != null);
        (cast(*u8) handoff_out[it])[0] = cast(u8) (81 + it);
    }

    assert(parallel(worker) == 0);

    // Joined workers finalized their own heaps before the caller frees these.
    for 0..3 {
        assert(handoff_out[it] == null && handoff_back[it] != null);
        assert((cast(*u8) handoff_back[it])[0] == cast(u8) (91 + it));
        RPM.rpfree(handoff_back[it]);
        handoff_back[it] = null;
    }

    memory := RPM.rpmalloc(8 * 1024 * 1024);

    assert(memory != null);
    (cast(*u8) memory)[0] = 93;
    RPM.rpfree(memory);
    RPM.finalize();
    assert(count(1, 0) > 0 && count(1, 0) == count(2, 0));
    // Paired offsets retain their value within the 16-bit range;
    // the adapter does not reinterpret the custom callbacks units.
    saved_offset = 7;
    setup();
    memory = RPM.rpmalloc(1024);
    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
    assert(count(1, 0) == count(2, 0));
    // Out-of-range offsets fail safely after releasing the rejected mapping.
    config: RPM.rpmalloc_config_t;
    config.memory_map = map_memory;
    config.memory_unmap = unmap_memory;
    saved_offset = 65536;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized() && count(1, 0) == count(2, 0));
    saved_offset = -1;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized() && count(1, 0) == count(2, 0));
    saved_offset = 7;
    bad_alignment = true;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized() && count(1, 0) == count(2, 0));
    bad_alignment = false;
    decline = true;
    config.memory_map = map_memory;
    config.memory_unmap = unmap_memory;
    config.map_fail_callback = retry;
    assert(RPM.initialize_config(*config) != 0);
    assert(!RPM.is_initialized());
    assert(count(3, 0) == 0); // Custom mappings never invoke OS failure retry.
    config.memory_unmap = null;
    // A missing partner selects BOTH native built-ins, without calling map.
    attempts_before := count(0, 0);

    assert(RPM.initialize_config(*config) == 0);
    assert(count(0, 0) == attempts_before);
    RPM.finalize();
    decline = false;
    assert(RPM.initialize() == 0);
    memory = RPM.rpmalloc(128);
    assert(memory != null);
    RPM.rpfree(memory);
    RPM.finalize();
    assert(context_pointer() == null);
}
