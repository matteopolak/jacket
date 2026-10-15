// Reviewed Android/Toolchain JAVA_HOME discovery API; native CLI is independent.
#import "Basic";
A :: #import "Android/Toolchain";

main :: () {
    assert(A.JAVA_ENV_NAME == "JAVA_HOME");
    success, bin, exe := A.find_java_bin_path_from_environment();

    defer free(bin);
    defer free(exe);
    arguments := get_command_line_arguments();

    defer array_free(arguments);

    if arguments.count < 2 || arguments[1] == "missing" {
        assert(!success && bin == "" && exe == "");
    } else {
        assert(success);
        assert(bin == arguments[1]);
        assert(exe == arguments[2]);
    }

    print("ok\n");
}
