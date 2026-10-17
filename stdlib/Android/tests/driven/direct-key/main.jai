// Reviewed direct key defaults/fixed sizes and real output paths, no user keys.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 2 || args.count == 3);

    if args.count == 3 {
        rejected := A.android_generate_signing_key_direct(args[1], "invalid-country", country_code = "TOO_LONG");
        assert(!rejected);
        print("rejected\n");
        return;
    }

    ok, ec := A.android_generate_signing_key_direct(args[1], "ec", key_size = 8192, common_name = "Test/Slash", organization_name = "Portable & Test");

    assert(ok && ec.kind == .ASN1_CERTIFICATE_AND_PRIVATE_KEY);
    rsa_ok, rsa := A.android_generate_signing_key_direct(args[1], "rsa", .RSA, key_size = 1024);

    assert(rsa_ok && rsa.kind == .ASN1_CERTIFICATE_AND_PRIVATE_KEY);
    duplicate, unused := A.android_generate_signing_key_direct(args[1], "ec");

    assert(duplicate && unused.direct.certificate_path != ec.direct.certificate_path);
    unsupported := A.android_generate_signing_key_direct(args[1], "dsa", .DSA);

    assert(!unsupported);
    print("%\n%\n%\n%\n", ec.direct.certificate_path, ec.direct.private_key_path, rsa.direct.certificate_path, rsa.direct.private_key_path);
    free(ec.direct.certificate_path);
    free(ec.direct.private_key_path);
    free(rsa.direct.certificate_path);
    free(rsa.direct.private_key_path);
    free(unused.direct.certificate_path);
    free(unused.direct.private_key_path);
}
