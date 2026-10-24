// Compile-time code calling a foreign procedure of a library only the target program declares
// is an error that says so, whatever the host has installed.
#if OS != BUILD_OS || CPU != BUILD_CPU {
    #load "target-only-call.jai";
}

main :: () {}
