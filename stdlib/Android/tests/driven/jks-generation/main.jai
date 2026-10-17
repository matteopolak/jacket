// Reviewed rejection table and real absent-SDK failure, never a fake keytool.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2);

    for size: s64.[-1, 512, 1536, 3072, 32768] {
        result := A.android_generate_jks_keystore("/unavailable-sdk", args[1], algorithm = .RSA, key_size = size);
        assert(!result);
    }

    for size: s64.[-1, 512, 4096] {
        result := A.android_generate_jks_keystore("/unavailable-sdk", args[1], algorithm = .DSA, key_size = size);
        assert(!result);
    }

    result := A.android_generate_jks_keystore("/unavailable-sdk", args[1]);

    assert(!result);
    print("ok\n");
}
