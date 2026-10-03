#import "Basic";
D :: #import "Debug";
N :: #import "Debug"(USE_GRAPHICS = false);
State :: #import "internal/debug-state";

main :: () {
    Setter :: #type (interactive: bool);
    setter: Setter = D.set_report_mode;
    previous := State.interactive_reporting;

    defer D.set_report_mode(previous);

    D.set_report_mode(interactive = true);
    assert(State.interactive_reporting);
    N.set_report_mode(interactive = false);
    assert(!State.interactive_reporting);
    setter(true);
    assert(State.interactive_reporting);
    N.set_report_mode(false);
    assert(!State.interactive_reporting);

    #if OS == .WASM {
        assert(!D.os_is_debugger_present());
        assert(!N.os_is_debugger_present());
    }

    print("ok\n");
}
