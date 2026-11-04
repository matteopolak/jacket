// Web needs native worker threads. Keep the loaded tests at their original indentation:
// their here-strings check the exact whitespace of rendered HTML.
#if OS != .WASM {
    #load "support/unit-suite.jai";
}
