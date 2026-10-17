#if OS == .WINDOWS {
    #assert CPU == .X64, "the Windows crash handler supports x64 only";

    #load "../internal/runtime/windows-crash-types.jai";
    #load "../internal/runtime/windows-crash-handler.jai";
} else {
    #assert OS == .MACOS || OS == .LINUX, "native crash diagnostics support Windows x64, Linux and macOS";

    #load "../internal/runtime/crash-types.jai";
    #load "../internal/runtime/crash-handler.jai";
}
