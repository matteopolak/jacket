// Independent bounds/representation safety; restore fatal checks after each call.
#import "Basic";
A :: #import "Android/Toolchain";

errors: s64;

capture :: (message: string, data: *void, info: Log_Info) {
    if message.count > 0 && (info.common_flags & .ERROR) != 0 errors += 1;
}

continue_failure :: (location: Source_Code_Location, message: string) -> bool {
    return false;
}

main :: () {
    original_logger := context.logger;
    original_assertion := context.assertion_failed;

    for kind: 0..6 {
        input := ifx kind == 0 then "\x1f\x81" else ifx kind == 1 then "\x1f\x82\x00" else
            ifx kind == 2 then "\x82\xff" else ifx kind == 3 then "\x80" else "one";
        before := input;

        context.logger = capture;
        context.assertion_failed = continue_failure;
        if kind < 2 A.asn1_read_tag(*input);
        else if kind < 4 A.asn1_read_length(*input);
        else if kind == 4 A.read_block(*input, 4);
        else if kind == 5 A.read(u64, *input);
        else A.asn1_decode_object_identifier("\x2a\x80");

        context.logger = original_logger;
        context.assertion_failed = original_assertion;
        assert(errors == kind + 1 && input.data == before.data && input.count == before.count);
    }

    print("ok\n");
}
