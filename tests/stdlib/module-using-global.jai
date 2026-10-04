// A module's top-level `using global;` makes the global's members visible to importers.
#import "Basic";
#import "Using_Global";
main :: () {
    load_procs();
    assert(answer() == 42);
}
