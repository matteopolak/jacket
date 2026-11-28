// Prints the signal or exception, the faulting address and a native stack trace to standard
// error when the program crashes, then lets the crash end the program as it would have. The
// runtime calls [init] when `ENABLE_BACKTRACE_ON_CRASH` is set; threads started later call
// `attach_thread` so a stack overflow on them is reported too.

#if OS == .WINDOWS {
    #assert CPU == .X64 || CPU == .ARM64, "the Windows crash handler requires x64 or ARM64";

    #load "../internal/runtime/windows-crash-types.jai";
    #load "../internal/runtime/windows-crash-handler.jai";
} else {
    #assert OS == .MACOS || OS == .LINUX, "native crash diagnostics support Windows x64/arm64, Linux and macOS";

    #load "../internal/runtime/crash-types.jai";
    #load "../internal/runtime/crash-handler.jai";
}
