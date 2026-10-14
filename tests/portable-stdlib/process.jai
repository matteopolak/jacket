#import "Basic";
P :: #import "Process";

main :: () {
    result, out, err, timed := P.run_command("/bin/sh", "-c", "printf copper; printf blue >&2; exit 7", capture_and_return_output = true, timeout_ms = 2000);

    assert(!timed && result.type == .EXITED && result.exit_code == 7);
    assert(out == "copper" && err == "blue");
    free(out);
    free(err);
    result, out, err, timed = P.run_command("/bin/sleep", "2", capture_and_return_output = true, timeout_ms = 25);
    assert(timed && result.type == .SIGNALED && result.signal == 9);
    free(out);
    free(err);
    result, out, err, timed = P.run_command("/does-not-exist-portable-test", capture_and_return_output = true);
    assert(!timed && result.type == .FAILED_TO_LAUNCH && out == "" && err == "");
    result, out, err, timed = P.run_command("/bin/sh", "-c", "i=0; while [ $i -lt 4096 ]; do printf 0123456789abcdef; printf fedcba9876543210 >&2; i=$((i+1)); done", capture_and_return_output = true, timeout_ms = 10000);
    assert(!timed && result.type == .EXITED && result.exit_code == 0);
    assert(out.count == 65536 && err.count == 65536 && out[0] == 48 && err[0] == 102);
    free(out);
    free(err);
    print("ok\n");
}
