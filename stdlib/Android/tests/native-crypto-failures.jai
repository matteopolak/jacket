#import "Basic";
F :: #import "File";
A :: #import "Android/Toolchain";

errors: s64;
assertions: s64;
last_location: Source_Code_Location;

capture :: (message: string, data: *void, info: Log_Info) {
    if message.count > 0 && (info.common_flags & .ERROR) != 0 errors += 1;
    if info.location.line_number > 0 last_location = info.location;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    assertions += 1;
    return false;
}

main :: () {
    private_key, loaded := F.read_entire_file(tprint("%/ec.der", FIXTURE_ROOT));

    assert(loaded);
    defer free(private_key);
    public_key, loaded_public := F.read_entire_file(tprint("%/ec.spki", FIXTURE_ROOT));

    assert(loaded_public);
    defer free(public_key);
    trailing_private := sprint("%x", private_key);
    trailing_public := sprint("%x", public_key);

    defer free(trailing_private);
    defer free(trailing_public);
    logger := context.logger;
    assertion := context.assertion_failed;

    for kind: 0..7 {
        errors = 0;
        assertions = 0;
        context.logger = capture;
        context.assertion_failed = continue_failure;
        key: *A.test_evp_pkey_st;
        if kind == 0 key = A.libressl_get_private_key("malformed");
        else if kind == 1 key = A.libressl_get_public_key("malformed");
        else if kind == 2 key = A.libressl_get_private_key(trailing_private);
        else if kind == 3 key = A.libressl_get_public_key(trailing_public);
        else if kind == 4 key = A.libressl_get_private_key("");
        else if kind == 5 A.CheckSSL(0);
        else if kind == 6 A.CheckSSL(2);
        else A.CheckSSL(-1);

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(key == null && errors > 0 && assertions == 1);
        if kind < 2 assert(errors > 1);
    }

    assert(last_location.line_number > 0);
    expected_location := Source_Code_Location.{ fully_pathed_filename = "explicit crypto caller", line_number = 1234, character_number = 56 };

    context.logger = capture;
    context.assertion_failed = continue_failure;
    A.CheckSSL(0, loc = expected_location);
    context.logger = logger;
    context.assertion_failed = assertion;
    assert(last_location.fully_pathed_filename == expected_location.fully_pathed_filename);
    assert(last_location.line_number == 1234 && last_location.character_number == 56);
    // Callback uses the supplied context, even if ambient logger differs.
    callback_context := context;
    callback_context.logger = capture;
    errors = 0;
    text := "provider error\n";
    result := A.libressl_error_callback(text.data, cast(u64) text.count, *callback_context);

    assert(result == 1 && errors == 1 && context.logger == logger);
    print("ok\n");
}
