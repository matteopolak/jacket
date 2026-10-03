Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = true, RAW = false, PUBLIC_HEAPS = true, GLOBAL_CACHE = true, THREAD_CACHE = true, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = false);
Context :: #import "internal/bindings/callback-context";

// Reviewed live configuration fields, copied input and canonical storage.
#add_context live_marker: s64;

RPM :: #import "rpmalloc"(CONFIGURABLE = true);
Same :: #import "rpmalloc"(CONFIGURABLE = true);

probe :: #library,system "rpmalloc_live_config_probe";
native_name_raw :: (huge: u32, expected: *u8, length: u64) -> s32 #foreign probe "native_name";
native_error_raw :: () -> s32 #foreign probe "native_error";
native_size_raw :: (index: u32) -> u64 #foreign probe "native_size";
native_error_mode :: () -> s32 #foreign probe;
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

default_map: #type (size: s64, offset: *s64) -> *void;
default_unmap: #type (address: *void, size: s64, offset: s64, full: s64);
mapped_a: s64;
mapped_b: s64;
unmapped_a: s64;
unmapped_b: s64;
errors_a: s64;
errors_b: s64;
failures_a: s64;
failures_b: s64;

map_a :: (size: s64, offset: *s64) -> *void {
    check(context.live_marker == 713);
    mapped_a += 1;
    return default_map(size, offset);
}

map_b :: (size: s64, offset: *s64) -> *void {
    check(context.live_marker == 713);
    mapped_b += 1;
    return default_map(size, offset);
}

unmap_a :: (address: *void, size: s64, offset: s64, full: s64) {
    check(context.live_marker == 713);
    unmapped_a += 1;
    default_unmap(address, size, offset, full);
}

unmap_b :: (address: *void, size: s64, offset: s64, full: s64) {
    check(context.live_marker == 713);
    unmapped_b += 1;
    default_unmap(address, size, offset, full);
}

error_a :: (message: string, location: Source_Code_Location) {
    check(context.live_marker == 713 && message.count > 0 && location.line_number > 0);
    errors_a += 1;
}

error_b :: (message: string, location: Source_Code_Location) {
    check(context.live_marker == 713 && message.count > 0 && location.line_number > 0);
    errors_b += 1;
}

failure_a :: (size: s64) -> s32 {
    check(context.live_marker == 713 && size >= (1 << 62));
    failures_a += 1;
    return 0;
}

failure_b :: (size: s64) -> s32 {
    check(context.live_marker == 713 && size >= (1 << 62));
    failures_b += 1;
    return 0;
}

invalid_map :: (size: s64, offset: *s64) -> *void {
    // Must never be selected when its paired unmap is absent at initialization.
    abort();
    return null;
}

invalid_unmap :: (address: *void, size: s64, offset: s64, full: s64) {
    abort();
}

exercise_update :: () {
    cfg := RPM.config();

    check(cfg == Same.config());
    check(cfg.memory_map != null && cfg.memory_unmap != null);
    check(cfg.error_callback == null && cfg.map_fail_callback == null);
    check(cfg.page_name.count == 0 && cfg.huge_page_name.count == 0);
    default_map = cfg.memory_map;
    default_unmap = cfg.memory_unmap;
    page := cfg.page_size;
    span := cfg.span_size;
    batch := cfg.span_map_count;

    check(page > 0 && span >= 4096 && batch > 0);
    cfg.memory_map = map_a;
    cfg.memory_unmap = unmap_a;
    // Each huge allocation needs a native mapping; the wrapper delegates to
    // the same default pair so all pre-existing mappings remain compatible.
    memory := RPM.rpmalloc(33_554_432);

    check(memory != null && mapped_a > 0);
    (cast(*u8) memory)[0] = 71;
    cfg.memory_map = map_b;
    cfg.memory_unmap = unmap_b;
    before := unmapped_b;

    RPM.rpfree(memory);
    check(unmapped_b > before);
    memory = RPM.rpmalloc(33_554_432);
    check(memory != null && mapped_b > 0);
    RPM.rpfree(memory);
    cfg.error_callback = error_a;
    check(native_error() != 0 && errors_a == 1);
    cfg.error_callback = error_b;
    check(native_error() != 0 && errors_b == 1 && errors_a == 1);
    cfg.error_callback = null;
    cfg.map_fail_callback = failure_a;
    offset: s64;

    check(default_map(1 << 62, *offset) == null && failures_a == 1);
    cfg.map_fail_callback = failure_b;
    check(default_map(1 << 62, *offset) == null && failures_b == 1 && failures_a == 1);
    cfg.map_fail_callback = null;
    check(default_map(1 << 62, *offset) == null && failures_b == 1 && failures_a == 1);
    cfg.page_name = "later-page";
    cfg.huge_page_name = "later-huge";
    check(native_name(0, cfg.page_name.data, cast(u64) cfg.page_name.count) != 0);
    check(native_name(1, cfg.huge_page_name.data, cast(u64) cfg.huge_page_name.count) != 0);
    bytes: [3] u8 = .[65, 66, 67];

    cfg.page_name = .{ data = *bytes[0], count = 3 };
    check(native_name(0, *bytes[0], 3) != 0);
    bytes[1] = 71;
    check(native_name(0, *bytes[0], 3) != 0);
    cfg.page_name = "";
    cfg.huge_page_name = "";
    check(native_name(0, null, 0) != 0 && native_name(1, null, 0) != 0);
    // Later sizing writes are live record changes, not native reconfiguration.
    cfg.page_size = 3;
    cfg.span_size = 3;
    cfg.span_map_count = 1;
    cfg.enable_huge_pages = true;
    check(native_size(0) == cast(u64) page && native_size(1) == cast(u64) span);
    check(native_size(2) == cast(u64) batch);
    mapped := default_map(page * 2, *offset);

    check(mapped != null);
    default_unmap(mapped, page * 2, offset, page * 2);
    check(cfg.page_size == 3 && cfg.span_size == 3 && cfg.enable_huge_pages);
    cfg.memory_map = default_map;
    cfg.memory_unmap = default_unmap;
}

main :: () {
    context.live_marker = 713;

    if native_error_mode() != 0 {
        check(RPM.initialize() == 0);
        RPM.config().error_callback = error_a;
        check(native_error() != 0 && errors_a == 1);
        RPM.config().error_callback = null;
        native_error();
        // Reaching this distinct abort must not count as the expected default
        // assertion; the runner also checks the native assertion diagnostic.
        abort();
    }

    check(RPM.initialize() == 0);
    exercise_update();
    RPM.finalize();
    // Copy the input, fill the entire default pair when EITHER member is absent.
    input: RPM.rpmalloc_config_t;
    input.memory_map = invalid_map;
    check(RPM.initialize_config(*input) == 0);
    check(RPM.config() != *input && RPM.config().memory_map != invalid_map);
    check(input.memory_map == invalid_map && input.memory_unmap == null);
    RPM.finalize();
    input = .{};
    input.memory_unmap = invalid_unmap;
    check(RPM.initialize_config(*input) == 0);
    check(RPM.config().memory_unmap != invalid_unmap && RPM.config().memory_map != null);
    check(input.memory_map == null && input.memory_unmap == invalid_unmap);
    RPM.finalize();
}

native_name :: (huge: u32, expected: *u8, length: u64) -> s32 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return native_name_raw(huge, expected, length);
}

native_size :: (index: u32) -> u64 {
    previous := Domain.select();
    defer Domain.restore(previous);
    return native_size_raw(index);
}

native_error :: () -> s32 {
    previous := Domain.select();

    defer Domain.restore(previous);
    caller := Context.enter(cast(*void) *context);

    defer Context.leave(caller);

    return native_error_raw();
}
