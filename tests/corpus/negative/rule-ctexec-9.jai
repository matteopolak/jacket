// rules: ctexec.9
// A failing `assert` inside #run prints its message and stops the compilation.
#import "Basic";

#run {
    limit := 3;
    assert(limit > 5, "limit is too small");
}

main :: () {}
