// Fatal reviewed errors captured only to check safe return after assertion hooks.
#import "Basic";
F :: #import "File";
A :: #import "Android/Toolchain";

errors: s64;
assertions: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if message.count > 0 && (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    root := FIXTURE_ROOT;
    store, ok := F.read_entire_file(tprint("%/valid.jks", root));

    assert(ok);
    defer free(store);
    protected, loaded_protected := F.read_entire_file(tprint("%/protected.bin", root));

    assert(loaded_protected);
    defer free(protected);
    wrapped, loaded_wrapped := F.read_entire_file(tprint("%/wrapped.der", root));

    assert(loaded_wrapped);
    defer free(wrapped);
    logger := context.logger;
    assertion := context.assertion_failed;
    negatives, loaded := F.read_entire_file(tprint("%/negative-stores.bin", root));

    assert(loaded);
    defer free(negatives);
    cursor := negatives;
    count := A.read(u32, *cursor);

    for 0..cast(s64) count - 1 {
        malformed := A.read_length_prefixed_block(*cursor);

        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        success, certificate, encrypted := A.jks_read(malformed, "password");

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(!success && certificate.count == 0 && encrypted.count == 0 && errors > 0 && assertions > 0);
    }

    negative_wrappers, loaded_wrappers := F.read_entire_file(tprint("%/negative-wrappers.bin", root));

    assert(loaded_wrappers);
    defer free(negative_wrappers);
    cursor = negative_wrappers;
    count = A.read(u32, *cursor);

    for 0..cast(s64) count - 1 {
        malformed := A.read_length_prefixed_block(*cursor);

        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        success, plaintext := A.decrypt_private_key(malformed, "password");

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(!success && plaintext.count == 0 && errors > 0 && assertions > 0);
    }

    // Every prefix is malformed, including partial integrity digests.
    for length: 0..store.count - 1 {
        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        success, certificate, encrypted := A.jks_read(.{ length, store.data }, "password");

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(!success && certificate.count == 0 && encrypted.count == 0);
        assert(errors > 0 && assertions > 0);
    }

    for length: 0..wrapped.count - 1 {
        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        success, plaintext := A.decrypt_private_key(.{ length, wrapped.data }, "password");

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(!success && plaintext.count == 0 && errors > 0 && assertions > 0);
    }

    for kind: 0..5 {
        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        success := true;
        result: string;

        if kind == 0 success, result = A.jdk_decrypt_private_key(protected, "wrong");
        else if kind == 1 success, result = A.jdk_decrypt_private_key("short", "password");
        else if kind == 2 success, result = A.decrypt_private_key(wrapped, "wrong");
        else if kind == 3 success, result = A.decrypt_private_key("\x30\x00", "password");
        else if kind == 4 {
            certificate: string;
            success, certificate, result = A.jks_read(store, "wrong");
        } else success, result = A.jdk_decrypt_private_key(protected, "\xff");

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(!success && result.count == 0 && errors > 0 && assertions > 0);
    }

    print("ok\n");
}
