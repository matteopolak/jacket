// External conformance driver: run from the repository root after compiling tests/support/peer.jai.
// Example: ws-conformance python3.14 /tmp/ws-bench /tmp/ws-interop
// No scenario is skipped. HTTP/2 and other extensions are outside v1.
#import "Basic";
#import "String";
#import "Process";

main :: () {
    args := get_command_line_arguments();

    assert(args.count == 4, "expected Python executable, echo fixture executable and output directory");
    result, output, errors, timed_out := run_command(args[1], "tools/websocket_conformance/interop.py", "--binary", args[2], "--output", args[3], capture_and_return_output = true, timeout_ms = 120000);

    defer free(output);
    defer free(errors);
    assert(!timed_out && result.type == .EXITED && result.exit_code == 0, "%\n%", output, errors);
    expected := string.[
        "jai-server-ws-plain-takeover", "jai-client-ws-plain-takeover",
        "jai-server-ws-deflate-takeover", "jai-client-ws-deflate-takeover", "jai-client-ws-deflate-reset",
        "jai-server-wss-plain-takeover", "jai-client-wss-plain-takeover",
        "jai-server-wss-deflate-takeover", "jai-client-wss-deflate-takeover", "jai-client-wss-deflate-reset"
    ];
    for expected assert(contains(output, tprint("%: PASS", it)), "missing conformance result: %", it);

    print("%", output);
}
