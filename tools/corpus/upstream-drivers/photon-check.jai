// Type-checks DavidColson/Photon's editor for Windows. Its build.jai runs the Windows-only
// resource tools (icon and manifest editing) and injects the build type as a string, so the sweep
// copies this driver next to source/main.jai instead (case `photon-check` in
// tools/upstream-cases.json).
BuildType :: enum u8 {
    DEBUG;
    OPTIMIZED_DEBUG;
    SHIPPING;
}

BUILD_TYPE :: BuildType.DEBUG;

#load "main.jai";
