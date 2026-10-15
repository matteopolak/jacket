// Default-import ADB assertions vary; verification is a separate enabled instance.
#import "Basic"(ENABLE_ASSERT = true);
A :: #import "Android/Toolchain";
S :: #import "String";

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
    path := sprint("%/../../../target/portable-adb/sdk protocol fixture", S.path_strip_filename(#file));
    failing := sprint("%/../../../target/portable-adb/start failure SDK/platform-tools/adb", S.path_strip_filename(#file));

    defer free(path);
    defer free(failing);
    A.__adb_path = path;
    context.logger = capture;
    info, valid := A.adb_version();

    assert(valid && info.protocol_version == "1.0.41");
    A.__adb_path = failing;
    original_assertion := context.assertion_failed;

    context.assertion_failed = continue_failure;
    completed, output, diagnostic, code := A.adb("version");

    context.assertion_failed = original_assertion;
    assert(!completed && code == 0 && output == "" && diagnostic == "" && errors == 1);
    args := get_command_line_arguments();

    defer array_free(args);
    disabled := args.count > 1 && args[1] == "disabled";

    assert(assertions == (ifx disabled then 0 else 1));
    A.__adb_path = "";
    print("ok\n");
}
