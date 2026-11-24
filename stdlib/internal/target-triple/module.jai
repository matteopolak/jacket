// The one place that turns an OS and CPU into a clang target triple. Everything else about a
// target (pointer width, Apple-ness, multiarch directory) is read back from clang.

// The triple clang builds for when the caller names none. "" when `os` and `cpu` have no default:
// the caller must pass a triple itself.
default_target_triple :: (os: Operating_System_Tag, cpu: CPU_Tag) -> string {
    arm := cpu == .ARM64;
    if cpu != .ARM64 && cpu != .X64 return "";

    if os == .MACOS return ifx arm then "arm64-apple-darwin" else "x86_64-apple-darwin";
    if os == .IOS return ifx arm then "arm64-apple-ios" else "x86_64-apple-ios";
    if os == .WINDOWS return ifx arm then "aarch64-pc-windows-msvc" else "x86_64-pc-windows-msvc";
    if os == .LINUX return ifx arm then "aarch64-unknown-linux-gnu" else "x86_64-unknown-linux-gnu";
    if os == .ANDROID return ifx arm then "aarch64-linux-android" else "x86_64-linux-android";

    return "";
}

// The triple `arguments` ask for with `-target <triple>`, `--target=<triple>` or `-target=<triple>`
// (the last one wins, as in clang), or "" when they name none.
target_triple_from_arguments :: (arguments: [] string) -> string {
    result: string;
    index := 0;

    while index < arguments.count {
        argument := arguments[index];
        index += 1;

        if argument == "-target" {
            if index < arguments.count {
                result = arguments[index];
                index += 1;
            }
        } else if begins_with(argument, "--target=") result = slice(argument, 9, argument.count - 9);
        else if begins_with(argument, "-target=") result = slice(argument, 8, argument.count - 8);
    }

    return result;
}

// The triple to give clang: the one in `arguments` when there is one, else the default for `os` and `cpu`.
target_triple :: (os: Operating_System_Tag, cpu: CPU_Tag, arguments: [] string) -> string {
    explicit := target_triple_from_arguments(arguments);
    if explicit return explicit;

    return default_target_triple(os, cpu);
}

#scope_file
#import "String";
