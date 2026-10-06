// rules: ops.16
// Found by fuzzing: `~` and `-` on a `Type` value made a garbage type id, and printing it crashed
// the interpreter.
#import "Basic";

main :: () {
    T := int;
    x := ~T;
    print("%\n", x);
}
