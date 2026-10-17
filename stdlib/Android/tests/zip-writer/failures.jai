using #import "Basic";
A :: #import "Android/Toolchain";

errors: s64;
assertions: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    logger := context.logger;
    assertion := context.assertion_failed;

    context.logger = capture;
    context.assertion_failed = continue_failure;
    w: A.Zip_Writer;
    unsupported := A.zip_add_file_from_memory(*w, "abc", "bad", .BZIP2);
    initial_errors := errors;
    initial_assertions := assertions;
    bad_alignment := A.zip_add_file_from_memory(*w, "abc", "bad", alignment = 16384);
    insufficient_padding := A.zip_add_file_from_memory(*w, "abc", "bad", alignment = 4096);

    A.zip_init_writer(*w, max_alignment = -1);
    negative := A.zip_add_file_from_memory(*w, .{ count = -1 }, "bad");
    oversized := A.zip_add_file_from_memory(*w, .{ count = 0x100000000 }, "bad");
    null_data := A.zip_add_file_from_memory(*w, .{ count = 1 }, "bad");
    oversized_name := A.zip_add_file_from_memory(*w, "abc", .{ count = 0x10000 });

    context.logger = logger;
    context.assertion_failed = assertion;
    assert(!unsupported && initial_errors == 1 && initial_assertions == 0);
    assert(!bad_alignment && !insufficient_padding && !negative && !oversized && !null_data && !oversized_name);
    assert(errors == 8 && assertions == 7);
    assert(w.zip_file_infos.count == 0);
    assert(builder_string_length(*w.builder) == 0);
    print("ok\n");
}
