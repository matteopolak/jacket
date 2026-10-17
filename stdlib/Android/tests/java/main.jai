// Reviewed Android/Toolchain JAVA_HOME discovery API; native CLI is independent.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    assert(A.JAVA_ENV_NAME == "JAVA_HOME");
    success, bin, exe := A.find_java_bin_path_from_environment();

    defer free(bin);
    defer free(exe);
    home := A.tool_environment(A.JAVA_ENV_NAME);

    defer free(home);

    if home.count == 0 {
        assert(!success && bin == "" && exe == "");
    } else {
        // Whatever the host sets, the result is its java launcher under `<home>/bin`.
        assert(success);
        assert(bin.count > 4 && slice(bin, bin.count - 4, 4) == "/bin");
        assert(exe.count > bin.count && slice(exe, 0, bin.count) == bin);
    }

    print("ok\n");
}
