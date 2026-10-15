// Requires actual Java SDK. Use only a new runner-owned destination, never a user store.
using #import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 3);
    ok, key := A.android_generate_jks_keystore(args[1], args[2], common_name = "JKS, Test", organization_name = "Portable & Test");

    assert(ok && key.kind == .JKS);
    assert(key.jks.keystore_path == args[2]);
    assert(key.jks.alias == "jaiiskewl" && key.jks.password == "password");
    free(key.jks.keystore_path);
    free(key.jks.alias);
    free(key.jks.password);
    print("ok\n");
}
