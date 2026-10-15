// Safe structural diagnostics for incomplete/wrong certificate fields.
#import "Basic";
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

    for input, index: string.["", "\x05\x00", "\x30\x00", "\x30\x02\x05\x00", "\x30\x02\x30\x00", "\x30\x07\x30\x05\xa0\x03\x02\x01\x03"] {
        context.logger = capture;
        context.assertion_failed = continue_failure;
        value := A.x509_get_subject_public_key_info(input);

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(value.value_data.count == 0 && value.content.count == 0);
        assert(errors == index + 1 && assertions == index + 1);
    }

    print("ok\n");
}
