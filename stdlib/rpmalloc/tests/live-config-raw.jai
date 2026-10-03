#add_context raw_live_marker: s64;

RPM :: #import "rpmalloc"(USE_RAW = true);
Domain :: #import "internal/bindings/rpmalloc-domain"(Owner = RPM.rpmalloc_config_t, STATISTICS = true, RAW = true, PUBLIC_HEAPS = false, GLOBAL_CACHE = true, THREAD_CACHE = true, ASSERTS = true, VALIDATE_ARGS = true, ADAPTIVE = false);
Context :: #import "internal/bindings/callback-context";

probe :: #library,system "rpmalloc_live_config_probe";
native_error_raw :: () -> s32 #foreign probe "raw_native_error";
libc :: #library,system "c";
abort :: () #foreign libc;

check :: (condition: bool) {
    if !condition abort();
}

map_default: #type (size: u64, offset: *u64) -> *void;
unmap_default: #type (address: *void, size: u64, offset: u64, full: u64);
maps: u64;
unmaps: u64;
errors: u64;

map_live :: (size: u64, offset: *u64) -> *void {
    check(context.raw_live_marker == 713);
    maps += 1;
    return map_default(size, offset);
}

unmap_live :: (address: *void, size: u64, offset: u64, full: u64) {
    check(context.raw_live_marker == 713);
    unmaps += 1;
    unmap_default(address, size, offset, full);
}

error_live :: (message: string) {
    check(context.raw_live_marker == 713 && message.count > 0);
    errors += 1;
}

main :: () {
    context.raw_live_marker = 713;
    check(RPM.rpmalloc_initialize() == 0);
    cfg := RPM.rpmalloc_config();

    check(cfg.memory_map != null && cfg.memory_unmap != null);
    check(cfg.error_callback == null && cfg.map_fail_callback == null);
    check(cfg.page_name.count == 0 && cfg.huge_page_name.count == 0);
    map_default = cfg.memory_map;
    unmap_default = cfg.memory_unmap;
    cfg.memory_map = map_live;
    cfg.memory_unmap = unmap_live;
    memory := RPM.rpmalloc(33_554_432);

    check(memory != null && maps > 0);
    (cast(*u8) memory)[0] = 71;
    RPM.rpfree(memory);
    check(unmaps > 0);
    cfg.error_callback = error_live;
    previous := Domain.select();
    caller := Context.enter(cast(*void) *context);

    check(native_error_raw() != 0 && errors == 1);
    Context.leave(caller);
    Domain.restore(previous);
    cfg.error_callback = null;
    cfg.memory_map = map_default;
    cfg.memory_unmap = unmap_default;
    RPM.rpmalloc_finalize();
}
