// Rejected architecture: adding source does not authorize undeclared callees.
C :: #import "Compiler";

exercise :: () {
    C.add_build_string("continuation :: () {}", -1);
    continuation();
}

#run,stallable exercise();

main :: () {}
