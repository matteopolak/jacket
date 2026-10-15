// Invalid-domain policies diagnose and never mutate a rejected end record.
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
    bytes: [22] u8;

    for kind: 0..4 {
        context.logger = capture;
        context.assertion_failed = continue_failure;
        if kind == 0 A.zip_encode_version(.{ major = 26 });
        else if kind == 1 A.zip_encode_version(.{ major = 1, minor = 10 });
        else if kind == 2 A.zip_eocd_get_start_of_central_directory_offset(.{ 21, bytes.data });
        else if kind == 3 A.zip_eocd_set_start_of_central_directory_offset(.{ 22, bytes.data }, 0xffffffff);
        else A.zip_crc32(.{ count = -1 });

        context.logger = logger;
        context.assertion_failed = assertion;
        assert(errors == kind + 1 && assertions == kind + 1);
        for byte: bytes assert(byte == 0);
    }

    print("ok\n");
}
