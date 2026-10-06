// A failed assert inside a `#run` that initializes a constant fails the compile, and the
// constant is not evaluated a second time with assertions silenced.
#import "Basic";

probe :: () -> bool {
    assert(false, "deliberately false");
    return true;
}
OK :: #run probe();

main :: () { print("main ran %\n", OK); }
