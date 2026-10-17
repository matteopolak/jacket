// Independent diagnosed unsupported inputs, with fatal result checks restored.
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

    for text, index: string.["\x01\x01\x01", "\x22\x01\x01", "\x03\x01\x01", "\x17\x01Z", "\x17\x0d25010203040xZ", "\x02\x01\xff", "\x05\x01x"] {
        context.logger = capture;
        context.assertion_failed = continue_failure;
        A.asn1_print(text);
        context.logger = logger;
        context.assertion_failed = assertion;
        assert(errors == index + 1 && assertions == index + 1);
    }

    print("ok\n");
}
