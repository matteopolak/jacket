// Clean closure reproducer: compiler 934a6918, original clean stdlib snapshot.
// Missing internal/compiler-api/runtime-config precedes any APK implementation.
using #import "Basic";

main :: () {
    print("ok\n");
}
