// Reviewed rpmalloc.md raw profile: canonical aliases, owned config copy,
// unsigned sizes, matching allocation family and complete statistics output.
#import "Basic";
RPM :: #import "rpmalloc"(USE_RAW = true);

probe :: #library,system "rpmalloc_callback_probe";
statistics_file :: () -> *void #foreign probe "binding_rpmalloc_statistics_file";
statistics_file_size :: (file: *void) -> s64 #foreign probe "binding_rpmalloc_statistics_file_size";
statistics_file_close :: (file: *void) -> s32 #foreign probe "binding_rpmalloc_statistics_file_close";

main :: () {
    input: RPM.rpmalloc_config_t;
    input.page_name = "raw allocator";
    assert(RPM.initialize_config(*input) == 0);
    owned := RPM.config();

    assert(owned != *input && owned == RPM.rpmalloc_config());
    assert(owned.page_name.data == input.page_name.data);
    input.page_name = "changed input";
    assert(owned.page_name == "raw allocator");
    owned.page_name = "mutable storage";
    assert(RPM.config().page_name == "mutable storage");
    assert(RPM.is_initialized() && RPM.rpmalloc_is_initialized());
    RPM.set_main_thread();
    memory := RPM.rpcalloc(19, 7);

    assert(memory != null);
    bytes := cast(*u8) memory;
    for 0..132 assert(bytes[it] == 0);

    bytes[0] = 37;
    memory = RPM.rprealloc(memory, 512);
    assert(memory != null && (cast(*u8) memory)[0] == 37);
    assert(RPM.usable_size(memory) >= 512);
    assert(RPM.rpmalloc_usable_size(memory) == RPM.usable_size(memory));
    RPM.rpfree(memory);
    memory = RPM.rpaligned_calloc(64, 17, 8);
    assert(memory != null && (cast(u64) memory & 63) == 0);
    for 0..135 assert((cast(*u8) memory)[it] == 0);

    RPM.rpfree(memory);
    memory = RPM.rpmemalign(128, 321);
    assert(memory != null && (cast(u64) memory & 127) == 0);
    RPM.rpfree(memory);
    memory = cast(*void) 123;
    assert(RPM.rpposix_memalign(*memory, 3, 12) != 0);
    assert(memory == cast(*void) 123);
    assert(RPM.rpposix_memalign(*memory, 64, 77) == 0);
    assert(memory != null && (cast(u64) memory & 63) == 0);
    RPM.rpfree(memory);

    guarded: struct {
        before: u64 = 19;
        stats: RPM.rpmalloc_thread_statistics_t;
        after: u64 = 27;
    };

    RPM.thread_statistics(*guarded.stats);
    assert(guarded.before == 19 && guarded.after == 27);
    global: RPM.rpmalloc_global_statistics_t;

    RPM.global_statistics(*global);
    assert(global.mapped > 0 && global.mapped_total >= global.mapped);
    file := statistics_file();

    assert(file != null);
    RPM.dump_statistics(file);
    first_size := statistics_file_size(file);

    assert(first_size > 0);
    RPM.rpmalloc_dump_statistics(file);
    assert(statistics_file_size(file) > first_size);
    assert(statistics_file_close(file) == 0);
    RPM.rpmalloc_thread_collect();
    RPM.rpmalloc_linker_reference();
    RPM.thread_finalize(true);
    assert(!RPM.is_thread_initialized());
    RPM.thread_initialize();
    RPM.rpmalloc_thread_finalize(1);
    RPM.finalize();
    assert(!RPM.is_initialized());
}
