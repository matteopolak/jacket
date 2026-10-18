// The system link command that `phase_timing.jai` runs for every compiler, so link time is the same work
// whichever compiler wrote the object files. `link_arguments` only builds the argument list (the unit test in
// `test_link_command.jai` checks it); `sdk_path` and `sdk_version` are what `xcrun` reported.
#import "Basic";

Link_Inputs :: struct {
    objects: [] string;
    libraries: [] string;
    output: string;
    sdk_path: string;
    sdk_version: string;
}

// The linker and its fixed arguments per OS. `{sdk_path}` and `{sdk_version}` (the minimum OS version too, so the linker has nothing to warn about) are filled in from Link_Inputs.
#if OS == .MACOS {
    LINKER :: string.["xcrun", "ld", "-arch", ARCH, "-syslibroot", "{sdk_path}", "-platform_version", "macos", "{sdk_version}", "{sdk_version}", "-lSystem"];
    #if CPU == .ARM64 ARCH :: "arm64";
    else ARCH :: "x86_64";
} else #if OS == .LINUX {
    LINKER :: string.["cc"];
} else {
    LINKER :: string.[];
}

// Empty when this OS has no system link command, in which case the compiler's own link is used.
link_arguments :: (inputs: Link_Inputs) -> [] string {
    arguments: [..] string;
    if LINKER.count == 0 return arguments;

    for LINKER {
        if it == "{sdk_path}" array_add(*arguments, inputs.sdk_path);
        else if it == "{sdk_version}" array_add(*arguments, inputs.sdk_version);
        else array_add(*arguments, it);
    }

    array_add(*arguments, "-o", inputs.output);
    array_add(*arguments, ..inputs.objects);
    array_add(*arguments, ..inputs.libraries);

    return arguments;
}
