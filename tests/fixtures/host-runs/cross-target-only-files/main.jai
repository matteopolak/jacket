// A file only the target program loads (the host program, with the host `OS` and `CPU`, leaves it
// out) declares a native library no machine has and runs code that allocates. Compile-time code
// loads host libraries only, so the run neither tries to load the library nor fails.
#if OS != BUILD_OS || CPU != BUILD_CPU {
    #load "target-only.jai";
}

main :: () {}
