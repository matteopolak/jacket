#import "Basic";
D :: #import "Debug";

main :: () {
    assert(cast(s32) D.Exit_Code.Exception == 4);

    #if OS == .WASM || OS == .ANDROID {
        assert(!D.os_is_debugger_present());
        assert(!D.is_debugger_present());
    }
}
