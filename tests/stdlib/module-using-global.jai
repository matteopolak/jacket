// A module's top-level `using global;` makes the global's members visible to importers.
// rules: scope.9
#import "Basic";
#import "Using_Global";
main :: () {
    load_procs();
    assert(answer() == 42);
}
