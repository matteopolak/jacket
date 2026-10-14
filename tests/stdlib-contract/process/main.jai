// Public token splitting and command-line quoting contracts.
#import "Basic";
Process :: #import "Process";

main :: () {
    pieces := Process.break_command_into_strings(" maple   \"red tree\" cedar ");
    assert(pieces.count == 3);
    assert(pieces[0] == "maple" && pieces[1] == "red tree" && pieces[2] == "cedar");

    args: [] string = .["plain", "two words", "say\"hi"];
    quoted := Process.get_quoted_command_string(args);
    assert(quoted == "plain \"two words\" \"say\\\"hi\"");

    #if OS_IS_UNIX {
        result, stdout, stderr, timed_out := Process.run_command(
            "/bin/sh", "-c", "printf '%s' \"$1\"; printf 'warn' >&2",
            "sh", "silver maple", capture_and_return_output=true, timeout_ms=2000);
        assert(!timed_out && result.type == .EXITED && result.exit_code == 0);
        assert(stdout == "silver maple" && stderr == "warn");
    }

    print("ok\n");
}
