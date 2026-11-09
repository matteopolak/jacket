// Live public reexport/configuration storage must affect the command procedure.
#import "Basic";
#import "Extensions/Environment";
A :: #import "Android/Toolchain";
S :: #import "String";

main :: () {
    path := copy_string(output_path("portable-adb/sdk protocol fixture"));

    defer free(path);
    A.__adb_path = path;
    A.adb_be_verbose = true;
    ok, output, errors := A.adb("echo-args", "visible");

    assert(ok && output == "echo-args\nvisible\n" && errors == "");
    free(output);
    free(errors);
    A.adb_be_verbose = false;
    ok, output, errors = A.adb("echo-args", "hidden");
    assert(ok && output == "echo-args\nhidden\n" && errors == "");
    free(output);
    free(errors);
    A.__adb_path = "";
    print("ok\n");
}
