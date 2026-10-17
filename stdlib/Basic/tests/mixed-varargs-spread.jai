#import "Basic";
P :: #import "Process";

main :: () {
    args := string.["%s", "tail"];
    result, out, err, timed := P.run_command("/usr/bin/printf", ..args, capture_and_return_output = true);

    defer free(out);
    defer free(err);
    assert(result.type == .EXITED && result.exit_code == 0 && !timed && out == "tail" && err == "");
    print("ok\n");
}
